// SPDX-License-Identifier: LGPL-2.1-or-later
//! Trim a surface along a curve lying on it, keeping the side of a picked point (Rhino's
//! Trim=Yes for FilletSrf / ChamferSrf).
//!
//! The kept face is bounded by the curve and, when the curve runs edge to edge, by the
//! pieces of the surface's own boundary (exact isoparametric sub-curves) on the kept side;
//! a closed curve inside the surface is the whole boundary (the inside kept) or a hole (the
//! outside kept). The result is a `Geo::Trimmed` (exact face in OM9's B-Rep kernel) with a
//! display mesh for builds without it.
#![allow(clippy::needless_range_loop, clippy::type_complexity)]
use crate::exact::{Nc, clamp_to};
use crate::mesh::Mesh;
use crate::nurbs::Srf;
use om9_addon_abi::session::{Geo, Vec3, dist};

fn iso_edges(g: &Geo) -> Option<[Nc; 4]> {
    let Geo::Surface { degree, size, points, weights, knots } = g else { return None };
    let w = |k: usize| weights.get(k).copied().unwrap_or(1.0);
    let h = |k: usize| {
        let p = points[k];
        [p[0] * w(k), p[1] * w(k), p[2] * w(k), w(k)]
    };
    let n01 = |k: &[f64]| {
        let (a, b) = (k[0], *k.last().unwrap());
        k.iter().map(|v| (v - a) / (b - a)).collect::<Vec<f64>>()
    };
    let (ku, kv) = (n01(&knots[0]), n01(&knots[1]));
    let (nu, nv) = (size[0], size[1]);
    Some([
        // v = 0 (u runs), u = 1 (v runs), v = 1 (u runs), u = 0 (v runs).
        Nc { degree: degree[0], points: (0..nu).map(|i| h(i * nv)).collect(), knots: ku.clone() },
        Nc { degree: degree[1], points: (0..nv).map(|j| h((nu - 1) * nv + j)).collect(), knots: kv.clone() },
        Nc { degree: degree[0], points: (0..nu).map(|i| h(i * nv + nv - 1)).collect(), knots: ku },
        Nc { degree: degree[1], points: (0..nv).map(h).collect(), knots: kv },
    ])
}

/// Place on the boundary, counter-clockwise in (u, v): 0..4.
fn perimeter(u: f64, v: f64) -> Option<f64> {
    let e = 1e-6;
    if v <= e {
        Some(u.clamp(0.0, 1.0))
    } else if u >= 1.0 - e {
        Some(1.0 + v.clamp(0.0, 1.0))
    } else if v >= 1.0 - e {
        Some(2.0 + (1.0 - u).clamp(0.0, 1.0))
    } else if u <= e {
        Some(3.0 + (1.0 - v).clamp(0.0, 1.0))
    } else {
        None
    }
}

fn uv_at(t: f64) -> (f64, f64) {
    let t = t.rem_euclid(4.0);
    match t as usize {
        0 => (t, 0.0),
        1 => (1.0, t - 1.0),
        2 => (1.0 - (t - 2.0), 1.0),
        _ => (0.0, 1.0 - (t - 3.0)),
    }
}

/// The boundary from `ta` counter-clockwise to `tb` (wrapping), as exact curve pieces.
fn ccw(edges: &[Nc; 4], ta: f64, tb: f64) -> Option<Vec<Nc>> {
    let mut end = tb;
    while end <= ta + 1e-12 {
        end += 4.0;
    }
    let mut out = Vec::new();
    let mut t = ta;
    while t < end - 1e-9 {
        let k = (t.floor() as i64).rem_euclid(4) as usize;
        let base = t.floor();
        let stop = (base + 1.0).min(end);
        let (a, b) = (t - base, stop - base);
        if b - a > 1e-9 {
            let piece = match k {
                0 | 1 => clamp_to(&edges[k], a, b)?,
                _ => clamp_to(&edges[k], 1.0 - b, 1.0 - a)?.reversed(),
            };
            out.push(piece);
        }
        t = stop;
    }
    Some(out)
}

fn inside(p: (f64, f64), poly: &[(f64, f64)]) -> bool {
    let mut c = false;
    for i in 0..poly.len() {
        let (a, b) = (poly[i], poly[(i + 1) % poly.len()]);
        if (a.1 > p.1) != (b.1 > p.1) && p.0 < a.0 + (b.0 - a.0) * (p.1 - a.1) / (b.1 - a.1) {
            c = !c;
        }
    }
    c
}

/// One side of a surface cut along a curve: its trim loops (exact) and, in (u, v), its
/// outline and holes (for choosing a side and drawing it).
struct Side {
    loops: Vec<Vec<Nc>>,
    outline: Vec<(f64, f64)>,
    holes: Vec<Vec<(f64, f64)>>,
}

/// Both sides of `g` cut along `curve` (lying on it, edge to edge or closed inside).
fn sides(s: &Srf, edges: &[Nc; 4], curve: &Nc) -> Result<[Side; 2], String> {
    let n = 64;
    let cuv: Vec<(f64, f64)> = (0..=n).map(|i| s.closest_st(curve.eval(i as f64 / n as f64))).collect();
    let closed = dist(curve.start(), curve.end()) < 1e-6;
    if closed {
        let rect = vec![(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];
        let boundary = ccw(edges, 0.0, 4.0).ok_or("could not trim")?;
        return Ok([
            Side { loops: vec![vec![curve.clone()]], outline: cuv.clone(), holes: vec![] },
            Side { loops: vec![boundary, vec![curve.clone()]], outline: rect, holes: vec![cuv] },
        ]);
    }
    let (p0, p1) = (cuv[0], cuv[n]);
    let (Some(t0), Some(t1)) = (perimeter(p0.0, p0.1), perimeter(p1.0, p1.1)) else {
        return Err("the trim curve does not run from edge to edge".into());
    };
    // Exact places: project the curve's ends on the boundary curves they are on.
    let exact_t = |t: f64, at: Vec3| -> f64 {
        let k = (t.floor() as usize).min(3);
        let u = edges[k].closest(at);
        k as f64 + if k < 2 { u } else { 1.0 - u }
    };
    let (t0, t1) = (exact_t(t0, curve.start()), exact_t(t1, curve.end()));
    // Two ways back along the boundary: counter-clockwise from t1 to t0, or the other.
    let way = |forward: bool| -> Option<Side> {
        let pieces = if forward {
            ccw(edges, t1, t0)?
        } else {
            ccw(edges, t0, t1)?.into_iter().rev().map(|c| c.reversed()).collect()
        };
        let mut poly = cuv.clone();
        let steps = 32;
        let mut end = if forward { t0 } else { t1 };
        let start = if forward { t1 } else { t0 };
        while end <= start + 1e-12 {
            end += 4.0;
        }
        let mut path: Vec<(f64, f64)> = (0..=steps).map(|i| uv_at(start + (end - start) * i as f64 / steps as f64)).collect();
        if !forward {
            path.reverse();
        }
        poly.extend(path);
        let mut lp = vec![curve.clone()];
        lp.extend(pieces);
        Some(Side { loops: vec![lp], outline: poly, holes: vec![] })
    };
    Ok([way(true).ok_or("could not trim")?, way(false).ok_or("could not trim")?])
}

/// The trimmed face of one side, with its display mesh.
fn face(g: &Geo, s: &Srf, side: &Side) -> Geo {
    let k = 40;
    let mut vertices = Vec::new();
    for i in 0..=k {
        for j in 0..=k {
            vertices.push(s.at_st(i as f64 / k as f64, j as f64 / k as f64));
        }
    }
    let at = |i: usize, j: usize| (i * (k + 1) + j) as u32;
    let mut triangles = Vec::new();
    for i in 0..k {
        for j in 0..k {
            let c = ((i as f64 + 0.5) / k as f64, (j as f64 + 0.5) / k as f64);
            if inside(c, &side.outline) && !side.holes.iter().any(|h| inside(c, h)) {
                triangles.push([at(i, j), at(i + 1, j), at(i + 1, j + 1)]);
                triangles.push([at(i, j), at(i + 1, j + 1), at(i, j + 1)]);
            }
        }
    }
    let display = Mesh { vertices, triangles }.geo("Trimmed surface", None, None);
    Geo::Trimmed {
        surface: Box::new(g.clone()),
        loops: side.loops.iter().map(|l| l.iter().map(Nc::to_geo).collect()).collect(),
        display: Some(Box::new(display)),
    }
}

/// Trim `g` along `curve` (on it), keeping the side where `keep` is.
pub fn trim_keeping(g: &Geo, curve: &Nc, keep: Vec3) -> Result<Geo, String> {
    let s = Srf::of(g).ok_or("not a NURBS surface")?;
    let edges = iso_edges(g).ok_or("not a NURBS surface")?;
    let both = sides(&s, &edges, curve)?;
    let kuv = s.closest_st(keep);
    let pick = |x: &Side| inside(kuv, &x.outline) && !x.holes.iter().any(|h| inside(kuv, h));
    let side = if pick(&both[0]) { &both[0] } else { &both[1] };
    Ok(face(g, &s, side))
}

/// Rhino's Split: `g` cut along `curve` into its two sides, the side where `keep` is first.
pub fn split_keeping(g: &Geo, curve: &Nc, keep: Vec3) -> Result<[Geo; 2], String> {
    let s = Srf::of(g).ok_or("not a NURBS surface")?;
    let edges = iso_edges(g).ok_or("not a NURBS surface")?;
    let both = sides(&s, &edges, curve)?;
    let kuv = s.closest_st(keep);
    let pick = |x: &Side| inside(kuv, &x.outline) && !x.holes.iter().any(|h| inside(kuv, h));
    let (a, b) = if pick(&both[0]) { (&both[0], &both[1]) } else { (&both[1], &both[0]) };
    Ok([face(g, &s, a), face(g, &s, b)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_line_across_a_plane_keeps_the_picked_side() {
        let g = Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: vec![[0.0, 0.0, 0.0], [0.0, 4.0, 0.0], [10.0, 0.0, 0.0], [10.0, 4.0, 0.0]],
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        };
        let cut = Nc::from_geo(&Geo::Line { start: [3.0, 0.0, 0.0], end: [3.0, 4.0, 0.0] }).unwrap();
        for (keep, xs) in [([6.0, 2.0, 0.0], (3.0, 10.0)), ([1.0, 2.0, 0.0], (0.0, 3.0))] {
            let t = trim_keeping(&g, &cut, keep).unwrap();
            let Geo::Trimmed { loops, .. } = &t else { panic!() };
            assert_eq!(loops.len(), 1);
            // The loop's corners span the kept x range.
            let mut lo = f64::MAX;
            let mut hi = f64::MIN;
            for c in &loops[0] {
                let n = Nc::from_geo(c).unwrap();
                for k in 0..=8 {
                    let p = n.eval(k as f64 / 8.0);
                    lo = lo.min(p[0]);
                    hi = hi.max(p[0]);
                }
            }
            assert!((lo - xs.0).abs() < 1e-9 && (hi - xs.1).abs() < 1e-9, "{lo} {hi}");
            // Closed chain.
            let ncs: Vec<Nc> = loops[0].iter().map(|c| Nc::from_geo(c).unwrap()).collect();
            for w in 0..ncs.len() {
                let next = &ncs[(w + 1) % ncs.len()];
                assert!(dist(ncs[w].end(), next.start()) < 1e-9, "chained {:?}", ncs.iter().map(|c| (c.start(), c.end())).collect::<Vec<_>>());
            }
        }
    }

    #[test]
    fn split_gives_both_sides() {
        let g = Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: vec![[0.0, 0.0, 0.0], [0.0, 4.0, 0.0], [10.0, 0.0, 0.0], [10.0, 4.0, 0.0]],
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        };
        let cut = Nc::from_geo(&Geo::Line { start: [3.0, 0.0, 0.0], end: [3.0, 4.0, 0.0] }).unwrap();
        let [a, b] = split_keeping(&g, &cut, [1.0, 2.0, 0.0]).unwrap();
        let span = |t: &Geo| {
            let Geo::Trimmed { loops, .. } = t else { panic!() };
            let (mut lo, mut hi) = (f64::MAX, f64::MIN);
            for c in &loops[0] {
                let n = Nc::from_geo(c).unwrap();
                for k in 0..=8 {
                    let p = n.eval(k as f64 / 8.0);
                    lo = lo.min(p[0]);
                    hi = hi.max(p[0]);
                }
            }
            (lo, hi)
        };
        let (sa, sb) = (span(&a), span(&b));
        assert!((sa.0 - 0.0).abs() < 1e-9 && (sa.1 - 3.0).abs() < 1e-9, "{sa:?}");
        assert!((sb.0 - 3.0).abs() < 1e-9 && (sb.1 - 10.0).abs() < 1e-9, "{sb:?}");
    }
}
