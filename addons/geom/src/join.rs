// SPDX-License-Identifier: LGPL-2.1-or-later
//! Joining curves into one exact NURBS curve, for the `Join` options of Fillet, Chamfer,
//! Blend, ArcBlend, Match and Offset `Cap`: each piece is split into rational Bézier spans,
//! raised to a common degree and laid end to end (knots repeated `degree` times at the joins,
//! so the shape is unchanged).
use crate::{Piece, composite_quadratic};
use om9_addon_abi::session::{Geo, Vec3, add, cross, dist, norm, scale, sub};

fn unit3(v: Vec3) -> Option<Vec3> {
    let l = norm(v);
    (l > 1e-15).then(|| scale(v, 1.0 / l))
}

/// Homogeneous control point `[w·x, w·y, w·z, w]`.
type H = [f64; 4];

fn h(p: Vec3, w: f64) -> H {
    [p[0] * w, p[1] * w, p[2] * w, w]
}
fn lerp(a: H, b: H, t: f64) -> H {
    std::array::from_fn(|i| a[i] * (1.0 - t) + b[i] * t)
}

/// Insert the knot `u` once (Boehm) into a clamped curve of degree `p`.
fn insert(p: usize, kv: &mut Vec<f64>, pw: &mut Vec<H>, u: f64) {
    let n = pw.len();
    let mut k = p;
    while k + 1 < n && kv[k + 1] <= u {
        k += 1;
    }
    let mut out = Vec::with_capacity(n + 1);
    for i in 0..=n {
        out.push(if i + p <= k {
            pw[i]
        } else if i > k {
            pw[i - 1]
        } else {
            let d = kv[i + p] - kv[i];
            let a = if d.abs() < 1e-300 {
                0.0
            } else {
                (u - kv[i]) / d
            };
            lerp(pw[i - 1], pw[i], a)
        });
    }
    kv.insert(k + 1, u);
    *pw = out;
}

fn homogeneous(points: &[Vec3], weights: &[f64]) -> Vec<H> {
    points
        .iter()
        .enumerate()
        .map(|(i, &q)| h(q, weights.get(i).copied().unwrap_or(1.0)))
        .collect()
}

fn nurbs_of(p: usize, kv: Vec<f64>, pw: &[H]) -> Geo {
    let weights: Vec<f64> = pw.iter().map(|q| q[3]).collect();
    let rational = weights.iter().any(|w| (w - 1.0).abs() > 1e-12);
    Geo::Nurbs {
        degree: p,
        points: pw
            .iter()
            .map(|q| [q[0] / q[3], q[1] / q[3], q[2] / q[3]])
            .collect(),
        weights: if rational { weights } else { Vec::new() },
        knots: kv,
    }
}

/// Split a clamped (rational) NURBS into Bézier spans by knot insertion.
fn bezier_spans(degree: usize, points: &[Vec3], weights: &[f64], knots: &[f64]) -> Vec<Vec<H>> {
    let p = degree;
    let mut kv = knots.to_vec();
    let mut pw = homogeneous(points, weights);
    let (lo, hi) = (kv[p], kv[pw.len()]);
    // Interior knots to full multiplicity p.
    let mut interior: Vec<f64> = kv
        .iter()
        .copied()
        .filter(|&k| k > lo + 1e-12 && k < hi - 1e-12)
        .collect();
    interior.dedup_by(|a, b| (*a - *b).abs() < 1e-12);
    for u in interior {
        let have = kv.iter().filter(|&&k| (k - u).abs() < 1e-12).count();
        for _ in have..p {
            insert(p, &mut kv, &mut pw, u);
        }
    }
    pw.windows(p + 1).step_by(p).map(|w| w.to_vec()).collect()
}

/// Split a NURBS at the knot-domain parameter `u` (strictly inside): (before, after).
fn split_nurbs(p: usize, points: &[Vec3], weights: &[f64], knots: &[f64], u: f64) -> (Geo, Geo) {
    let mut kv = knots.to_vec();
    let mut pw = homogeneous(points, weights);
    let have = kv.iter().filter(|&&k| (k - u).abs() < 1e-12).count();
    for _ in have..p {
        insert(p, &mut kv, &mut pw, u);
    }
    // Snap the inserted knots to exactly u.
    for k in kv.iter_mut() {
        if (*k - u).abs() < 1e-12 {
            *k = u;
        }
    }
    let r = kv.iter().position(|&k| k == u).unwrap_or(p);
    let mut left_k: Vec<f64> = kv[..r].to_vec();
    left_k.extend(std::iter::repeat_n(u, p + 1));
    let mut right_k: Vec<f64> = vec![u; p + 1];
    right_k.extend_from_slice(&kv[r + p..]);
    (
        nurbs_of(p, left_k, &pw[..r]),
        nurbs_of(p, right_k, &pw[r - 1..]),
    )
}

/// The part of an exact curve between the normalized parameters `t0 < t1` (as
/// [`crate::eval_geo`] counts them). Closed curves wrap when `t1 < t0`.
pub fn sub_geo(g: &Geo, t0: f64, t1: f64) -> Option<Geo> {
    let closed = match g {
        Geo::Circle { .. } => true,
        Geo::Polyline { closed, .. } => *closed,
        Geo::Nurbs { points, .. } => points.len() > 2 && dist(points[0], *points.last()?) < 1e-9,
        _ => false,
    };
    let (t0, t1) = (t0.clamp(0.0, 1.0), t1.clamp(0.0, 1.0));
    if t1 < t0 {
        if !closed {
            return sub_geo(g, t1, t0).and_then(|r| reverse(&r));
        }
        if let Geo::Circle {
            center,
            x,
            y,
            radius,
        } = g
        {
            let tau = std::f64::consts::TAU;
            return Some(turned(
                *center,
                *x,
                *y,
                *radius,
                t0 * tau,
                (t1 + 1.0 - t0) * tau,
            ));
        }
        let a = sub_geo(g, t0, 1.0);
        let b = sub_geo(g, 0.0, t1);
        return match (a, b) {
            (Some(a), Some(b)) => join(&[a, b], 1e-6),
            (a, b) => a.or(b),
        };
    }
    if t1 - t0 < 1e-12 {
        return None;
    }
    let at = |t: f64| crate::eval_geo(g, t);
    Some(match g {
        Geo::Line { .. } => Geo::Line {
            start: at(t0)?,
            end: at(t1)?,
        },
        Geo::Arc {
            center,
            x,
            y,
            radius,
            angle,
        } => turned(*center, *x, *y, *radius, t0 * angle, (t1 - t0) * angle),
        Geo::Circle {
            center,
            x,
            y,
            radius,
        } => {
            let tau = std::f64::consts::TAU;
            turned(*center, *x, *y, *radius, t0 * tau, (t1 - t0).min(1.0) * tau)
        }
        Geo::Polyline { points, closed } => {
            let mut pts = points.clone();
            if *closed {
                pts.push(points[0]);
            }
            let segs = (pts.len() - 1) as f64;
            let mut out = vec![at(t0)?];
            for (i, &q) in pts.iter().enumerate() {
                let ti = i as f64 / segs;
                if ti > t0 + 1e-12 && ti < t1 - 1e-12 {
                    out.push(q);
                }
            }
            out.push(at(t1)?);
            if out.len() == 2 {
                Geo::Line {
                    start: out[0],
                    end: out[1],
                }
            } else {
                Geo::Polyline {
                    points: out,
                    closed: false,
                }
            }
        }
        Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        } => {
            let (a, b) = (knots[*degree], knots[points.len()]);
            let (u0, u1) = (a + (b - a) * t0, a + (b - a) * t1);
            let mut cur = g.clone();
            if u1 < b - 1e-12 {
                cur = split_nurbs(*degree, points, weights, knots, u1).0;
            }
            if u0 > a + 1e-12 {
                let Geo::Nurbs {
                    degree,
                    points,
                    weights,
                    knots,
                } = &cur
                else {
                    return None;
                };
                cur = split_nurbs(*degree, points, weights, knots, u0).1;
            }
            cur
        }
        _ => return None,
    })
}

/// The arc of the circle (`center`, `x`, `y`, `r`) from angle `from` over `sweep`.
fn turned(center: Vec3, x: Vec3, y: Vec3, r: f64, from: f64, sweep: f64) -> Geo {
    let (c, s) = (from.cos(), from.sin());
    let nx = add(scale(x, c), scale(y, s));
    let ny = add(scale(x, -s), scale(y, c));
    if sweep >= std::f64::consts::TAU - 1e-12 {
        return Geo::Circle {
            center,
            x: nx,
            y: ny,
            radius: r,
        };
    }
    Geo::Arc {
        center,
        x: nx,
        y: ny,
        radius: r,
        angle: sweep,
    }
}

/// How [`extend_geo`] continues a curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extension {
    /// The curve's own shape: lines stay lines, arcs arcs, NURBS continue their last span.
    Natural,
    /// A straight tangent line, joined.
    Line,
    /// A tangent arc with the end's curvature, joined.
    Arc,
    /// The tangent arc merged into one curve.
    Smooth,
}

/// Extend an open curve at its end (`at_end`) or start by `length` (≥ 0).
pub fn extend_geo(g: &Geo, at_end: bool, length: f64, how: Extension) -> Option<Geo> {
    if length <= 1e-12 {
        return Some(g.clone());
    }
    if !at_end {
        return extend_geo(&reverse(g)?, true, length, how).and_then(|r| reverse(&r));
    }
    let e = |t: f64| crate::eval_geo(g, t);
    let end = e(1.0)?;
    // End tangent and curvature from the exact curve.
    let hh = 1e-4;
    let (p0, p1) = (e(1.0 - 2.0 * hh)?, e(1.0 - hh)?);
    let d1 = sub(end, p1);
    let tan = unit3(d1)?;
    let d0 = sub(p1, p0);
    let turn = cross(d0, d1);
    let ds = (norm(d0) + norm(d1)) / 2.0;
    let kappa = if ds > 1e-15 {
        norm(turn) / (norm(d0) * norm(d1)).max(1e-300) / ds
    } else {
        0.0
    };
    let line = || Geo::Line {
        start: end,
        end: add(end, scale(tan, length)),
    };
    let arc = || -> Geo {
        if kappa * length < 1e-6 {
            return line();
        }
        let r = 1.0 / kappa;
        let nrm = unit3(turn).unwrap_or([0.0, 0.0, 1.0]);
        let inward = cross(nrm, tan);
        let center = add(end, scale(inward, r));
        let x = scale(sub(end, center), 1.0 / r);
        let angle = (length / r).min(std::f64::consts::TAU - 1e-6);
        Geo::Arc {
            center,
            x,
            y: tan,
            radius: r,
            angle,
        }
    };
    Some(match (how, g) {
        (Extension::Line, _) | (Extension::Natural, Geo::Line { .. }) => match g {
            Geo::Line { start, .. } => Geo::Line {
                start: *start,
                end: add(end, scale(tan, length)),
            },
            _ => join(&[g.clone(), line()], 1e-6)?,
        },
        (
            Extension::Natural,
            Geo::Polyline {
                points,
                closed: false,
            },
        ) => {
            let mut p = points.clone();
            *p.last_mut()? = add(end, scale(tan, length));
            Geo::Polyline {
                points: p,
                closed: false,
            }
        }
        (
            Extension::Natural,
            Geo::Arc {
                center,
                x,
                y,
                radius,
                angle,
            },
        ) => Geo::Arc {
            center: *center,
            x: *x,
            y: *y,
            radius: *radius,
            angle: (angle + length / radius).min(std::f64::consts::TAU),
        },
        (Extension::Natural, Geo::Nurbs { .. }) => extend_span(g, length)?,
        (Extension::Arc, _) => join(&[g.clone(), arc()], 1e-6)?,
        _ => join(&[g.clone(), arc()], 1e-6)?,
    })
}

/// Continue the last Bézier span of a NURBS past its end (de Casteljau extrapolation) until
/// the added part is `length` long.
fn extend_span(g: &Geo, length: f64) -> Option<Geo> {
    let Geo::Nurbs {
        degree,
        points,
        weights,
        knots,
    } = g
    else {
        return None;
    };
    let p = *degree;
    let n = points.len();
    let (a, b) = (knots[p], knots[n]);
    let last = knots[..n]
        .iter()
        .rev()
        .copied()
        .find(|&k| k < b - 1e-12)
        .unwrap_or(a)
        .max(a);
    // Split the last span off (so it is a Bézier span), keep the rest as is.
    let (head, tail) = if last > a + 1e-12 {
        let (h0, t0) = split_nurbs(p, points, weights, knots, last);
        (Some(h0), t0)
    } else {
        (None, g.clone())
    };
    let Geo::Nurbs {
        points: tp,
        weights: tw,
        ..
    } = &tail
    else {
        return None;
    };
    let bz = homogeneous(tp, tw);
    let grow = |s: f64| -> Vec<H> {
        // Left polygon of de Casteljau at s (> 1): the span over [0, s].
        let mut pts = bz.clone();
        let mut left = vec![pts[0]];
        for r in 1..=p {
            for i in 0..=p - r {
                pts[i] = lerp(pts[i], pts[i + 1], s);
            }
            left.push(pts[0]);
        }
        left
    };
    let span_len = |hs: &[H], from: f64| -> f64 {
        let g = nurbs_of(p, [vec![0.0; p + 1], vec![1.0; p + 1]].concat(), hs);
        let m = 200;
        let pts: Vec<Vec3> = (0..=m)
            .filter_map(|i| crate::eval_geo(&g, from + (1.0 - from) * i as f64 / m as f64))
            .collect();
        pts.windows(2).map(|w| dist(w[0], w[1])).sum()
    };
    // Find s with the added length (part of [0, s] beyond 1/s) equal to `length`.
    let (mut lo, mut hi) = (1.0, 2.0);
    while span_len(&grow(hi), 1.0 / hi) < length && hi < 1e6 {
        lo = hi;
        hi *= 2.0;
    }
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        if span_len(&grow(mid), 1.0 / mid) < length {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let ext = grow((lo + hi) / 2.0);
    if ext.iter().any(|q| q[3] <= 1e-9) {
        return None;
    }
    let tail = nurbs_of(p, [vec![0.0; p + 1], vec![1.0; p + 1]].concat(), &ext);
    match head {
        Some(h0) => join(&[h0, tail], 1e-6),
        None => Some(tail),
    }
}

/// Raise a Bézier span by one degree.
fn elevate(b: &[H]) -> Vec<H> {
    let p = b.len() - 1;
    let mut out = vec![b[0]];
    for i in 1..=p {
        let a = i as f64 / (p + 1) as f64;
        out.push(lerp(b[i], b[i - 1], a));
    }
    out.push(b[p]);
    out
}

/// The spans of a curve, in order.
fn spans(g: &Geo) -> Option<Vec<Vec<H>>> {
    Some(match g {
        Geo::Line { start, end } => vec![vec![h(*start, 1.0), h(*end, 1.0)]],
        Geo::Polyline { points, closed } => {
            let mut p = points.clone();
            if *closed && p.len() > 2 && dist(p[0], *p.last()?) > 1e-9 {
                p.push(p[0]);
            }
            p.windows(2)
                .map(|w| vec![h(w[0], 1.0), h(w[1], 1.0)])
                .collect()
        }
        Geo::Circle {
            center,
            x,
            y,
            radius,
        } => arc_spans(*center, *x, *y, *radius, std::f64::consts::TAU)?,
        Geo::Arc {
            center,
            x,
            y,
            radius,
            angle,
        } => arc_spans(*center, *x, *y, *radius, *angle)?,
        Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        } => bezier_spans(*degree, points, weights, knots),
        _ => return None,
    })
}

fn arc_spans(c: Vec3, x: Vec3, y: Vec3, r: f64, angle: f64) -> Option<Vec<Vec<H>>> {
    let k = (angle.abs() / std::f64::consts::FRAC_PI_2 - 1e-9)
        .ceil()
        .max(1.0) as usize;
    let at = |t: f64| add(c, add(scale(x, r * t.cos()), scale(y, r * t.sin())));
    let pieces: Vec<Piece> = (0..k)
        .map(|i| Piece::Arc {
            center: c,
            start: at(angle * i as f64 / k as f64),
            end: at(angle * (i + 1) as f64 / k as f64),
        })
        .collect();
    spans(&composite_quadratic(&pieces)?)
}

fn start_of(s: &[Vec<H>]) -> Vec3 {
    let p = s[0][0];
    [p[0] / p[3], p[1] / p[3], p[2] / p[3]]
}
fn end_of(s: &[Vec<H>]) -> Vec3 {
    let p = *s.last().unwrap().last().unwrap();
    [p[0] / p[3], p[1] / p[3], p[2] / p[3]]
}
fn reversed(s: Vec<Vec<H>>) -> Vec<Vec<H>> {
    s.into_iter()
        .rev()
        .map(|mut b| {
            b.reverse();
            b
        })
        .collect()
}

/// Join curves whose ends meet within `tol` into one curve (in chain order, reversing pieces
/// as needed). `None` when they do not form one chain.
pub fn join(curves: &[Geo], tol: f64) -> Option<Geo> {
    let mut rest: Vec<Vec<Vec<H>>> = curves.iter().map(spans).collect::<Option<_>>()?;
    if rest.is_empty() {
        return None;
    }
    let mut chain = rest.remove(0);
    while !rest.is_empty() {
        let (a, b) = (start_of(&chain), end_of(&chain));
        let mut hit = None;
        for (i, s) in rest.iter().enumerate() {
            let (c, d) = (start_of(s), end_of(s));
            if dist(b, c) <= tol {
                hit = Some((i, false, true));
            } else if dist(b, d) <= tol {
                hit = Some((i, true, true));
            } else if dist(a, d) <= tol {
                hit = Some((i, false, false));
            } else if dist(a, c) <= tol {
                hit = Some((i, true, false));
            }
            if hit.is_some() {
                break;
            }
        }
        let (i, rev, at_end) = hit?;
        let mut s = rest.remove(i);
        if rev {
            s = reversed(s);
        }
        if at_end {
            chain.extend(s);
        } else {
            s.extend(chain);
            chain = s;
        }
    }
    let q = chain.iter().map(|b| b.len() - 1).max()?;
    let closed = dist(start_of(&chain), end_of(&chain)) <= tol;
    if q == 1 {
        let mut points: Vec<Vec3> = chain
            .iter()
            .map(|b| [b[0][0] / b[0][3], b[0][1] / b[0][3], b[0][2] / b[0][3]])
            .collect();
        if !closed {
            points.push(end_of(&chain));
        }
        return Some(Geo::Polyline { points, closed });
    }
    let mut pw: Vec<H> = Vec::new();
    let mut knots = vec![0.0; q + 1];
    for (k, b) in chain.iter().enumerate() {
        let mut b = b.clone();
        while b.len() - 1 < q {
            b = elevate(&b);
        }
        if k == 0 {
            pw.push(b[0]);
        }
        pw.extend_from_slice(&b[1..]);
        if k + 1 < chain.len() {
            knots.extend(std::iter::repeat_n((k + 1) as f64, q));
        }
    }
    knots.extend(std::iter::repeat_n(chain.len() as f64, q + 1));
    Some(nurbs_of(q, knots, &pw))
}

/// The curve reversed (same shape, opposite direction).
pub fn reverse(g: &Geo) -> Option<Geo> {
    Some(match g {
        Geo::Line { start, end } => Geo::Line {
            start: *end,
            end: *start,
        },
        Geo::Polyline { points, closed } => {
            let mut p = points.clone();
            p.reverse();
            Geo::Polyline {
                points: p,
                closed: *closed,
            }
        }
        Geo::Nurbs {
            degree,
            points,
            weights,
            knots,
        } => {
            let (a, b) = (knots[0], *knots.last()?);
            let mut k: Vec<f64> = knots.iter().map(|&u| a + b - u).collect();
            k.reverse();
            let mut p = points.clone();
            p.reverse();
            let mut w = weights.clone();
            w.reverse();
            Geo::Nurbs {
                degree: *degree,
                points: p,
                weights: w,
                knots: k,
            }
        }
        Geo::Arc {
            center,
            x,
            y,
            radius,
            angle,
        } => {
            let (c, s) = (angle.cos(), angle.sin());
            Geo::Arc {
                center: *center,
                x: add(scale(*x, c), scale(*y, s)),
                y: sub(scale(*x, s), scale(*y, c)),
                radius: *radius,
                angle: *angle,
            }
        }
        Geo::Circle {
            center,
            x,
            y,
            radius,
        } => Geo::Circle {
            center: *center,
            x: *x,
            y: scale(*y, -1.0),
            radius: *radius,
        },
        other => join(std::slice::from_ref(other), 1e-9).and_then(|g| reverse(&g))?,
    })
}

/// Total length of a curve (dense samples).
pub fn length(g: &Geo) -> f64 {
    let n = 512;
    let pts: Vec<Vec3> = (0..=n)
        .filter_map(|i| crate::eval_geo(g, i as f64 / n as f64))
        .collect();
    pts.windows(2).map(|w| norm(sub(w[1], w[0]))).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval_geo;

    fn on(g: &Geo, p: Vec3) -> f64 {
        (0..=40000)
            .map(|i| dist(eval_geo(g, i as f64 / 40000.0).unwrap(), p))
            .fold(f64::INFINITY, f64::min)
    }

    #[test]
    fn line_arc_nurbs_join_exactly() {
        let line = Geo::Line {
            start: [0.0; 3],
            end: [10.0, 0.0, 0.0],
        };
        // Quarter circle from (10,0) to (15,5) around (10,5): reversed on purpose.
        let arc = Geo::Arc {
            center: [10.0, 5.0, 0.0],
            x: [1.0, 0.0, 0.0],
            y: [0.0, -1.0, 0.0],
            radius: 5.0,
            angle: std::f64::consts::FRAC_PI_2,
        };
        let cubic = Geo::Nurbs {
            degree: 3,
            points: vec![
                [15.0, 5.0, 0.0],
                [15.0, 8.0, 0.0],
                [12.0, 10.0, 0.0],
                [10.0, 12.0, 0.0],
            ],
            weights: vec![],
            knots: vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
        };
        let j = join(&[line.clone(), cubic.clone(), arc.clone()], 1e-6).unwrap();
        let Geo::Nurbs { degree, .. } = &j else {
            panic!("{j:?}")
        };
        assert_eq!(*degree, 3);
        assert!(dist(eval_geo(&j, 0.0).unwrap(), [0.0; 3]) < 1e-9);
        assert!(dist(eval_geo(&j, 1.0).unwrap(), [10.0, 12.0, 0.0]) < 1e-9);
        // Every sample of each piece lies on the joined curve.
        for g in [&line, &arc, &cubic] {
            for i in 0..=20 {
                let p = eval_geo(g, i as f64 / 20.0).unwrap();
                assert!(on(&j, p) < 2e-3, "{p:?}");
            }
        }
        // The arc part stays a true arc (rational, radius 5).
        for i in 0..=50 {
            let p = eval_geo(&j, i as f64 / 50.0).unwrap();
            if p[0] > 10.0 + 1e-6 && p[1] < 5.0 - 1e-6 {
                assert!((dist(p, [10.0, 5.0, 0.0]) - 5.0).abs() < 1e-9, "{p:?}");
            }
        }
        assert!(
            (length(&j) - (10.0 + 5.0 * std::f64::consts::FRAC_PI_2 + length(&cubic))).abs() < 1e-2
        );
    }

    #[test]
    fn lines_join_to_a_closed_polyline_and_gaps_refuse() {
        let l = |a: Vec3, b: Vec3| Geo::Line { start: a, end: b };
        let j = join(
            &[
                l([0.0; 3], [1.0, 0.0, 0.0]),
                l([1.0, 1.0, 0.0], [1.0, 0.0, 0.0]),
                l([1.0, 1.0, 0.0], [0.0; 3]),
            ],
            1e-9,
        )
        .unwrap();
        assert!(
            matches!(&j, Geo::Polyline { points, closed: true } if points.len() == 3),
            "{j:?}"
        );
        assert!(
            join(
                &[
                    l([0.0; 3], [1.0, 0.0, 0.0]),
                    l([2.0, 0.0, 0.0], [3.0, 0.0, 0.0])
                ],
                1e-6
            )
            .is_none()
        );
    }

    #[test]
    fn reverse_keeps_the_shape() {
        let g = Geo::Nurbs {
            degree: 2,
            points: vec![[0.0; 3], [1.0, 2.0, 0.0], [3.0, 0.0, 0.0], [4.0, 1.0, 0.0]],
            weights: vec![],
            knots: vec![0.0, 0.0, 0.0, 0.3, 1.0, 1.0, 1.0],
        };
        let r = reverse(&g).unwrap();
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            assert!(dist(eval_geo(&g, t).unwrap(), eval_geo(&r, 1.0 - t).unwrap()) < 1e-9);
        }
    }

    fn cubic() -> Geo {
        Geo::Nurbs {
            degree: 3,
            points: vec![
                [0.0; 3],
                [1.0, 2.0, 0.0],
                [3.0, 2.0, 0.0],
                [4.0, 0.0, 0.0],
                [6.0, -1.0, 0.0],
            ],
            weights: vec![],
            knots: vec![0.0, 0.0, 0.0, 0.0, 0.4, 1.0, 1.0, 1.0, 1.0],
        }
    }

    #[test]
    fn sub_curves_follow_the_original() {
        let g = cubic();
        let s = sub_geo(&g, 0.2, 0.7).unwrap();
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let p = eval_geo(&s, t).unwrap();
            assert!(dist(p, eval_geo(&g, 0.2 + 0.5 * t).unwrap()) < 1e-9, "{t}");
        }
        let arc = Geo::Arc {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            angle: 3.0,
        };
        let a = sub_geo(&arc, 0.5, 1.0).unwrap();
        assert!(dist(eval_geo(&a, 0.0).unwrap(), eval_geo(&arc, 0.5).unwrap()) < 1e-12);
        assert!(dist(eval_geo(&a, 1.0).unwrap(), eval_geo(&arc, 1.0).unwrap()) < 1e-12);
        let circle = Geo::Circle {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 1.0,
        };
        // Wrapping across the seam: from 3/4 to 1/4 turn is a half circle through (1, 0).
        let w = sub_geo(&circle, 0.75, 0.25).unwrap();
        assert!(
            dist(eval_geo(&w, 0.5).unwrap(), [1.0, 0.0, 0.0]) < 1e-9,
            "{w:?}"
        );
        let poly = Geo::Polyline {
            points: vec![[0.0; 3], [2.0, 0.0, 0.0], [2.0, 2.0, 0.0]],
            closed: false,
        };
        let p = sub_geo(&poly, 0.25, 0.75).unwrap();
        assert!(
            matches!(&p, Geo::Polyline { points, .. } if points.len() == 3),
            "{p:?}"
        );
        // Reversed parameters on an open curve give the reversed piece.
        let r = sub_geo(&g, 0.7, 0.2).unwrap();
        assert!(dist(eval_geo(&r, 0.0).unwrap(), eval_geo(&g, 0.7).unwrap()) < 1e-9);
    }

    #[test]
    fn natural_extension_keeps_lines_arcs_and_spans() {
        let l = Geo::Line {
            start: [0.0; 3],
            end: [1.0, 0.0, 0.0],
        };
        let e = extend_geo(&l, true, 2.0, Extension::Natural).unwrap();
        assert_eq!(
            e,
            Geo::Line {
                start: [0.0; 3],
                end: [3.0, 0.0, 0.0]
            }
        );
        let e = extend_geo(&l, false, 1.0, Extension::Natural).unwrap();
        assert!(
            dist(eval_geo(&e, 0.0).unwrap(), [-1.0, 0.0, 0.0]) < 1e-12,
            "{e:?}"
        );
        let arc = Geo::Arc {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            angle: 1.0,
        };
        let Geo::Arc { angle, .. } = extend_geo(&arc, true, 1.0, Extension::Natural).unwrap()
        else {
            panic!()
        };
        assert!((angle - 1.5).abs() < 1e-12);
        let g = cubic();
        let e = extend_geo(&g, true, 1.5, Extension::Natural).unwrap();
        assert!(
            (length(&e) - length(&g) - 1.5).abs() < 2e-2,
            "{} {}",
            length(&e),
            length(&g)
        );
        // The original curve is untouched inside.
        for i in 0..=20 {
            let p = eval_geo(&g, i as f64 / 20.0).unwrap();
            assert!(on(&e, p) < 1e-3);
        }
        // Line and arc extensions: tangent at the joint, the right length.
        for how in [Extension::Line, Extension::Arc, Extension::Smooth] {
            let e = extend_geo(&g, true, 1.0, how).unwrap();
            assert!((length(&e) - length(&g) - 1.0).abs() < 2e-2, "{how:?}");
        }
        // The arc extension of an arc continues the same circle.
        let e = extend_geo(&arc, true, 1.0, Extension::Arc).unwrap();
        let p = eval_geo(&e, 1.0).unwrap();
        assert!((norm(p) - 2.0).abs() < 1e-3, "{p:?}");
    }
}
