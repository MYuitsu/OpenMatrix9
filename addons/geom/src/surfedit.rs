// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact surface edits by knot operations: split at an isocurve (Rhino 5 Split with the
//! Isocurve option) and move the seam of a closed surface (SrfSeam). The shapes are
//! unchanged; only the parameterisation is cut or turned.
use crate::exact::{Nc, clamp_to};
use crate::extend::{assemble, lines};
use om9_addon_abi::session::Geo;

fn normalised(curves: Vec<Nc>) -> Vec<Nc> {
    curves
        .into_iter()
        .map(|mut c| {
            let (a, b) = (c.knots[0], *c.knots.last().unwrap());
            c.knots.iter_mut().for_each(|v| *v = (*v - a) / (b - a));
            c
        })
        .collect()
}

/// Split `g` at the isocurve `t` (normalised) of direction `dir` (0: u = t): two surfaces.
pub fn split_at(g: &Geo, dir: usize, t: f64) -> Result<(Geo, Geo), String> {
    if !(1e-9..=1.0 - 1e-9).contains(&t) {
        return Err("the isocurve is on the surface's edge".into());
    }
    let (curves, degree, _, knots) = lines(g, dir).ok_or("not a NURBS surface")?;
    let curves = normalised(curves);
    let first: Option<Vec<Nc>> = curves.iter().map(|c| clamp_to(c, 0.0, t)).collect();
    let second: Option<Vec<Nc>> = curves.iter().map(|c| clamp_to(c, t, 1.0)).collect();
    let (first, second) = (first.ok_or("could not split")?, second.ok_or("could not split")?);
    Ok((assemble(&first, dir, degree, knots.clone()), assemble(&second, dir, degree, knots)))
}

/// Move the seam of a surface closed in direction `dir` to `t` (normalised).
pub fn move_seam(g: &Geo, dir: usize, t: f64) -> Result<Geo, String> {
    let (curves, degree, _, knots) = lines(g, dir).ok_or("not a NURBS surface")?;
    let curves = normalised(curves);
    if !curves.iter().all(|c| c.closed(1e-6)) {
        return Err("the surface is not closed in that direction".into());
    }
    let moved: Vec<Nc> = curves.iter().map(|c| c.with_seam(t)).collect();
    Ok(assemble(&moved, dir, degree, knots))
}


/// Rhino 5 TweenSurfaces (MatchMethod=None, made exact): the surfaces turned to run the same
/// way (corners matched), made compatible (same degrees and knots — shapes unchanged), and
/// `count` surfaces between them by moving each control point (and weight) evenly.
pub fn tween(a: &Geo, b: &Geo, count: usize) -> Result<Vec<Geo>, String> {
    use crate::merge::{of_geo, reverse, to_geo};
    use crate::network::conform_v;
    let sa = of_geo(a).ok_or("not a NURBS surface")?;
    let sb = of_geo(b).ok_or("not a NURBS surface")?;
    let corners = |s: &crate::network::Srf| {
        let (n, m) = (s.rows.len() - 1, s.rows[0].len() - 1);
        [s.rows[0][0], s.rows[n][0], s.rows[0][m], s.rows[n][m]].map(crate::exact::euclid)
    };
    let ca = corners(&sa);
    // The way of the second surface (swap u/v, flip u, flip v) nearest the first's corners.
    let mut best: Option<(f64, crate::network::Srf)> = None;
    for swap in [false, true] {
        for fu in [false, true] {
            for fv in [false, true] {
                let mut t = if swap { sb.flipped() } else { sb.clone() };
                if fu {
                    t = reverse(&t, 0);
                }
                if fv {
                    t = reverse(&t, 1);
                }
                let cb = corners(&t);
                let d: f64 = ca.iter().zip(&cb).map(|(p, q)| om9_addon_abi::session::dist(*p, *q)).sum();
                if best.as_ref().is_none_or(|x| d < x.0) {
                    best = Some((d, t));
                }
            }
        }
    }
    let mut both = [sa, best.unwrap().1];
    conform_v(&mut both);
    let mut fl = [both[0].flipped(), both[1].flipped()];
    conform_v(&mut fl);
    let [x, y] = fl.map(|s| s.flipped());
    let mut out = Vec::new();
    for k in 1..=count.max(1) {
        let t = k as f64 / (count.max(1) + 1) as f64;
        let rows = x
            .rows
            .iter()
            .zip(&y.rows)
            .map(|(ra, rb)| {
                ra.iter()
                    .zip(rb)
                    .map(|(p, q)| {
                        let (pe, qe) = (crate::exact::euclid(*p), crate::exact::euclid(*q));
                        let w = p[3] + (q[3] - p[3]) * t;
                        let e: [f64; 3] = std::array::from_fn(|i| pe[i] + (qe[i] - pe[i]) * t);
                        [e[0] * w, e[1] * w, e[2] * w, w]
                    })
                    .collect()
            })
            .collect();
        out.push(to_geo(&crate::network::Srf { rows, degree: x.degree, knots: x.knots.clone() }));
    }
    Ok(out)
}

/// Rhino 5 ShrinkTrimmedSrf: the part of `g` over the parameter box [u0, u1] × [v0, v1]
/// (normalised): full-multiplicity knots there, the outer control points dropped; the
/// surface itself does not change.
pub fn shrink(g: &Geo, u: [f64; 2], v: [f64; 2]) -> Result<Geo, String> {
    let (curves, degree, _, knots) = lines(g, 0).ok_or("not a NURBS surface")?;
    let cut: Option<Vec<Nc>> = normalised(curves).iter().map(|c| clamp_to(c, u[0], u[1])).collect();
    let g = assemble(&cut.ok_or("could not shrink")?, 0, degree, knots);
    let (curves, degree, _, knots) = lines(&g, 1).ok_or("not a NURBS surface")?;
    let cut: Option<Vec<Nc>> = normalised(curves).iter().map(|c| clamp_to(c, v[0], v[1])).collect();
    Ok(assemble(&cut.ok_or("could not shrink")?, 1, degree, knots))
}


/// Degree ≥ 3 and spans no wider than half the reach in each direction (shape unchanged).
fn refine_for(g: &Geo, reach: [f64; 2]) -> Result<Geo, String> {
    let mut g = g.clone();
    for dir in 0..2 {
        let (curves, degree, _, knots) = lines(&g, dir).ok_or("not a NURBS surface")?;
        let mut curves = normalised(curves);
        let p = curves[0].degree;
        if p < 3 {
            curves = curves.iter().map(|c| c.elevate(3 - p)).collect();
        }
        let width = (reach[dir] / 2.0).clamp(1.0 / 64.0, 1.0);
        let mut ks: Vec<f64> = curves[0].knots.clone();
        ks.dedup_by(|x, y| (*x - *y).abs() < 1e-12);
        let mut extra = Vec::new();
        for w in ks.windows(2) {
            let n = ((w[1] - w[0]) / width).ceil() as usize;
            for k in 1..n {
                extra.push(w[0] + (w[1] - w[0]) * k as f64 / n as f64);
            }
        }
        for c in curves.iter_mut() {
            for x in &extra {
                c.insert(*x);
            }
        }
        let mut deg = degree;
        deg[dir] = curves[0].degree;
        g = assemble(&curves, dir, deg, knots);
    }
    Ok(g)
}

/// Rhino 5 SoftEditSrf: move the surface point at (a, b) by `mv`; the control points around
/// it move with a smooth fall-off over `reach` (parameter distance in u and v), scaled so the
/// point lands exactly; `fix_edges` keeps the boundary control points still.
pub fn soft_edit(g: &Geo, a: f64, b: f64, mv: [f64; 3], reach: [f64; 2], fix_edges: bool) -> Result<Geo, String> {
    // A coarse surface (a plane of four points, say) has no control points near the base
    // point: refine it first — degree 3 and spans no wider than the fall-off — so the edit
    // stays local. The shape is unchanged by this.
    let g = &refine_for(g, reach)?;
    let Geo::Surface { degree, size, points, weights, knots } = g else { return Err("not a NURBS surface".into()) };
    let n01 = |k: &[f64]| {
        let (x, y) = (k[0], *k.last().unwrap());
        k.iter().map(|v| (v - x) / (y - x)).collect::<Vec<f64>>()
    };
    let (ku, kv) = (n01(&knots[0]), n01(&knots[1]));
    let (p, q) = (degree[0], degree[1]);
    let greville = |k: &[f64], d: usize, i: usize| (1..=d).map(|r| k[i + r]).sum::<f64>() / d.max(1) as f64;
    let w = |k: usize| weights.get(k).copied().unwrap_or(1.0);
    let smooth = |x: f64| {
        let x = x.clamp(0.0, 1.0);
        x * x * (3.0 - 2.0 * x)
    };
    let fall = |i: usize, j: usize| -> f64 {
        if fix_edges && (i == 0 || j == 0 || i + 1 == size[0] || j + 1 == size[1]) {
            return 0.0;
        }
        let du = (greville(&ku, p, i) - a) / reach[0].max(1e-9);
        let dv = (greville(&kv, q, j) - b) / reach[1].max(1e-9);
        smooth(1.0 - (du * du + dv * dv).sqrt())
    };
    // Rational basis at (a, b): the moved point is Σ R_ij·f_ij·k·mv; k makes it exactly mv.
    let su = crate::find_span(size[0], p, a.clamp(0.0, 1.0), &ku);
    let sv = crate::find_span(size[1], q, b.clamp(0.0, 1.0), &kv);
    let (bu, bv) = (crate::basis(su, a.clamp(0.0, 1.0), p, &ku), crate::basis(sv, b.clamp(0.0, 1.0), q, &kv));
    let mut den = 0.0;
    let mut num = 0.0;
    for (x, nu) in bu.iter().enumerate() {
        for (y, nv) in bv.iter().enumerate() {
            let (i, j) = (su - p + x, sv - q + y);
            let r = nu * nv * w(i * size[1] + j);
            den += r;
            num += r * fall(i, j);
        }
    }
    if num < 1e-9 * den {
        return Err("no control point can move there (widen the distances or turn FixEdges off)".into());
    }
    let k = den / num;
    let moved: Vec<[f64; 3]> = points
        .iter()
        .enumerate()
        .map(|(idx, pt)| {
            let f = fall(idx / size[1], idx % size[1]) * k;
            [pt[0] + mv[0] * f, pt[1] + mv[1] * f, pt[2] + mv[2] * f]
        })
        .collect();
    Ok(Geo::Surface { degree: *degree, size: *size, points: moved, weights: weights.clone(), knots: knots.clone() })
}


/// Rhino 5 SetSurfaceTangent: the surface leaves its edge (`dir`: the parameter that ends
/// there, `at_end`) along `d` — the second row of control points turned onto that direction
/// (lengths kept). Curves across the edge are refined first (shape unchanged) so the far
/// side does not move.
pub fn set_tangent(g: &Geo, dir: usize, at_end: bool, d: [f64; 3]) -> Result<Geo, String> {
    use om9_addon_abi::session::{add, dot, norm, scale, sub, unit};
    if norm(d) < 1e-12 {
        return Err("the direction has no length".into());
    }
    let d = unit(d);
    let (curves, degree, _, knots) = lines(g, dir).ok_or("not a NURBS surface")?;
    let mut curves = normalised(curves);
    if at_end {
        curves = curves.iter().map(Nc::reversed).collect();
    }
    // Enough control points that row 1 is not near the far edge.
    let p = curves[0].degree;
    for _ in 0..6 {
        let first = curves[0].knots.iter().copied().find(|v| *v > 1e-12).unwrap_or(1.0);
        if curves[0].points.len() >= p + 3 && first < 1.0 {
            break;
        }
        for c in curves.iter_mut() {
            c.insert(first / 2.0);
        }
    }
    for c in curves.iter_mut() {
        let (p0, p1) = (crate::exact::euclid(c.points[0]), crate::exact::euclid(c.points[1]));
        let old = sub(p1, p0);
        let len = norm(old);
        let dd = if dot(d, old) < 0.0 { scale(d, -1.0) } else { d };
        let q = add(p0, scale(dd, len));
        let w = c.points[1][3];
        c.points[1] = [q[0] * w, q[1] * w, q[2] * w, w];
    }
    if at_end {
        curves = curves.iter().map(Nc::reversed).collect();
    }
    let mut deg = degree;
    deg[dir] = curves[0].degree;
    Ok(assemble(&curves, dir, deg, knots))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact::{self, H};
    use crate::nurbs::Srf;
    use om9_addon_abi::session::dist;

    fn cylinder() -> Geo {
        let ring = exact::arc([0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 3.0, std::f64::consts::TAU);
        let rows: Vec<Vec<H>> = ring.points.iter().map(|c| vec![*c, [c[0], c[1], 4.0 * c[3], c[3]]]).collect();
        exact::surface(&rows, [2, 1], [ring.knots.clone(), vec![0.0, 0.0, 1.0, 1.0]])
    }

    #[test]
    fn split_and_seam_keep_the_shape_exactly() {
        let g = cylinder();
        let s = Srf::of(&g).unwrap();
        let (a, b) = split_at(&g, 0, 0.3).unwrap();
        let (fa, fb) = (Srf::of(&a).unwrap(), Srf::of(&b).unwrap());
        // The pieces meet at the isocurve and are on the cylinder.
        for j in 0..=4 {
            let v = j as f64 / 4.0;
            assert!(dist(fa.at_st(1.0, v), s.at_st(0.3, v)) < 1e-9);
            assert!(dist(fb.at_st(0.0, v), s.at_st(0.3, v)) < 1e-9);
        }
        for f in [&fa, &fb] {
            for i in 0..=20 {
                let p = f.at_st(i as f64 / 20.0, 0.5);
                assert!((p[0].hypot(p[1]) - 3.0).abs() < 1e-9);
            }
        }
        // Split along v (height) too.
        let (lo, hi) = split_at(&g, 1, 0.25).unwrap();
        assert!((Srf::of(&lo).unwrap().at_st(0.4, 1.0)[2] - 1.0).abs() < 1e-9);
        assert!((Srf::of(&hi).unwrap().at_st(0.4, 0.0)[2] - 1.0).abs() < 1e-9);
        // Seam moved: the new start is the old point at 0.3; still the cylinder.
        let m = move_seam(&g, 0, 0.3).unwrap();
        let fm = Srf::of(&m).unwrap();
        assert!(dist(fm.at_st(0.0, 0.5), s.at_st(0.3, 0.5)) < 1e-9);
        for i in 0..=40 {
            let p = fm.at_st(i as f64 / 40.0, 0.5);
            assert!((p[0].hypot(p[1]) - 3.0).abs() < 1e-9);
        }
        // Not closed across v.
        assert!(move_seam(&g, 1, 0.5).is_err());
    }

    #[test]
    fn tween_shrink_and_soft_edit() {
        // Tween a flat square and a cylinder strip of other structure: the halfway surface's
        // corners are halfway between the corners.
        let flat = Geo::Surface {
            degree: [1, 1],
            size: [2, 2],
            points: vec![[0.0, 0.0, 0.0], [0.0, 4.0, 0.0], [6.0, 0.0, 0.0], [6.0, 4.0, 0.0]],
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        };
        let raised = match &flat {
            Geo::Surface { degree, size, points, weights, knots } => Geo::Surface {
                degree: *degree,
                size: *size,
                // The same corners but listed the other way round in v: the tween must match them.
                points: vec![points[1], points[0], points[3], points[2]].iter().map(|p| [p[0], p[1], p[2] + 2.0]).collect(),
                weights: weights.clone(),
                knots: knots.clone(),
            },
            _ => unreachable!(),
        };
        let t = tween(&flat, &raised, 1).unwrap();
        let f = Srf::of(&t[0]).unwrap();
        for (a, b) in [(0.0, 0.0), (1.0, 1.0), (0.3, 0.6)] {
            assert!((f.at_st(a, b)[2] - 1.0).abs() < 1e-12);
        }
        // A tween of a cylinder with itself is the cylinder (exact).
        let c = cylinder();
        let t = tween(&c, &c, 2).unwrap();
        for g in &t {
            let f = Srf::of(g).unwrap();
            for i in 0..=20 {
                let p = f.at_st(i as f64 / 20.0, 0.5);
                assert!((p[0].hypot(p[1]) - 3.0).abs() < 1e-9);
            }
        }
        // Shrink: the sub-surface is exactly the original part.
        let sh = shrink(&c, [0.2, 0.7], [0.1, 0.9]).unwrap();
        let (fs, fc) = (Srf::of(&sh).unwrap(), Srf::of(&c).unwrap());
        assert!(dist(fs.at_st(0.0, 0.0), fc.at_st(0.2, 0.1)) < 1e-9);
        assert!(dist(fs.at_st(1.0, 1.0), fc.at_st(0.7, 0.9)) < 1e-9);
        // Soft edit: the picked point lands exactly; far away nothing moves.
        let k = vec![0.0, 0.0, 0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 1.0, 1.0];
        let rows: Vec<Vec<H>> = (0..7).map(|i| (0..7).map(|j| [i as f64, j as f64, 0.0, 1.0]).collect()).collect();
        let sheet = exact::surface(&rows, [3, 3], [k.clone(), k]);
        let s0 = Srf::of(&sheet).unwrap();
        let before = s0.at_st(0.5, 0.5);
        let e = soft_edit(&sheet, 0.5, 0.5, [0.0, 0.0, 1.0], [0.3, 0.3], true).unwrap();
        let s1 = Srf::of(&e).unwrap();
        assert!(dist(s1.at_st(0.5, 0.5), [before[0], before[1], before[2] + 1.0]) < 1e-12);
        // FixEdges: every edge stays where it was.
        for k in 0..=10 {
            let t = k as f64 / 10.0;
            for (a, b) in [(t, 0.0), (t, 1.0), (0.0, t), (1.0, t)] {
                assert!(dist(s1.at_st(a, b), s0.at_st(a, b)) < 1e-12, "edge moved at {a},{b}");
            }
        }
    }

    #[test]
    fn set_tangent_turns_the_edge_direction_only() {
        // A flat 4 × 4 sheet: make it leave its u = 0 edge going up at 45°.
        let rows: Vec<Vec<H>> = (0..4).map(|i| (0..4).map(|j| [i as f64 * 4.0 / 3.0, j as f64 * 4.0 / 3.0, 0.0, 1.0]).collect()).collect();
        let k = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];
        let sheet = exact::surface(&rows, [3, 3], [k.clone(), k]);
        let out = set_tangent(&sheet, 0, false, [1.0, 0.0, 1.0]).unwrap();
        let (s0, s1) = (Srf::of(&sheet).unwrap(), Srf::of(&out).unwrap());
        for j in 0..=8 {
            let v = j as f64 / 8.0;
            let (a, b) = (s1.at_st(0.0, v), s1.at_st(1e-5, v));
            assert!(dist(a, s0.at_st(0.0, v)) < 1e-12, "the edge stays");
            let dz = (b[2] - a[2]) / (b[0] - a[0]);
            assert!((dz - 1.0).abs() < 1e-3, "45° at the edge: {dz}");
            assert!(dist(s1.at_st(1.0, v), s0.at_st(1.0, v)) < 1e-12, "the far edge stays");
        }
    }
}

/// The exact edge curve of an untrimmed surface: `dir` 0 the edge where u is fixed (u = 0, or
/// u = 1 with `at_end`), 1 where v is fixed.
pub fn edge_curve(g: &Geo, dir: usize, at_end: bool) -> Option<crate::exact::Nc> {
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
    let (nu, nv) = (size[0], size[1]);
    Some(if dir == 0 {
        let i = if at_end { nu - 1 } else { 0 };
        crate::exact::Nc { degree: degree[1], points: (0..nv).map(|j| h(i * nv + j)).collect(), knots: n01(&knots[1]) }
    } else {
        let j = if at_end { nv - 1 } else { 0 };
        crate::exact::Nc { degree: degree[0], points: (0..nu).map(|i| h(i * nv + j)).collect(), knots: n01(&knots[0]) }
    })
}

/// Rhino's SplitAtTangents: `g` cut along direction `dir` (0: at u values, 1: at v values)
/// wherever it has a crease — a knot of full multiplicity where the surface turns sharply
/// across it (the kinks of a polycurve section or profile). Exact pieces, in order.
pub fn split_at_kinks(g: &Geo, dir: usize) -> Vec<Geo> {
    let Some(s) = crate::nurbs::Srf::of(g) else { return vec![g.clone()] };
    let Geo::Surface { degree, knots, .. } = g else { return vec![g.clone()] };
    let k = &knots[dir];
    let (a, b) = (k[0], *k.last().unwrap());
    let p = degree[dir];
    // Interior knots of multiplicity ≥ degree, normalised.
    let mut cands: Vec<f64> = Vec::new();
    let mut i = p + 1;
    while i < k.len() - p - 1 {
        let v = k[i];
        let mult = k.iter().filter(|x| (**x - v).abs() < 1e-12).count();
        if mult >= p && v > a + 1e-12 && v < b - 1e-12 {
            cands.push((v - a) / (b - a));
        }
        i += mult.max(1);
    }
    let at = |t: f64, c: f64| if dir == 0 { s.at_st(t, c) } else { s.at_st(c, t) };
    let creased = |t: f64| {
        let h = 1e-7;
        (0..=6).any(|j| {
            let c = j as f64 / 6.0;
            let (p0, pl, pr) = (at(t, c), at(t - h, c), at(t + h, c));
            let (dl, dr) = (om9_addon_abi::session::sub(p0, pl), om9_addon_abi::session::sub(pr, p0));
            let (nl, nr) = (om9_addon_abi::session::norm(dl), om9_addon_abi::session::norm(dr));
            nl > 1e-14 && nr > 1e-14 && om9_addon_abi::session::dot(dl, dr) / (nl * nr) < (1e-3f64).cos()
        })
    };
    let kinks: Vec<f64> = cands.into_iter().filter(|t| creased(*t)).collect();
    if kinks.is_empty() {
        return vec![g.clone()];
    }
    // Cut from the last kink back, so the earlier parameters stay where they are.
    let mut rest = g.clone();
    let mut out = Vec::new();
    let mut hi = 1.0;
    for t in kinks.iter().rev() {
        let local = t / hi;
        match split_at(&rest, dir, local) {
            Ok((first, second)) => {
                out.push(second);
                rest = first;
                hi = *t;
            }
            Err(_) => continue,
        }
    }
    out.push(rest);
    out.reverse();
    out
}

#[cfg(test)]
mod kink_tests {
    use super::*;

    /// A loft-like surface over an L-shaped section (a kink at its middle): two pieces, each
    /// flat.
    #[test]
    fn splits_at_the_crease_only() {
        // Section: (0,0,0) → (2,0,0) → (2,2,0), degree 1; extruded 3 in z (degree 1).
        let g = Geo::Surface {
            degree: [1, 1],
            size: [2, 3],
            points: vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [2.0, 2.0, 0.0], [0.0, 0.0, 3.0], [2.0, 0.0, 3.0], [2.0, 2.0, 3.0]],
            weights: vec![1.0; 6],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 0.5, 1.0, 1.0]],
        };
        let parts = split_at_kinks(&g, 1);
        assert_eq!(parts.len(), 2);
        let a = crate::nurbs::Srf::of(&parts[0]).unwrap();
        let b = crate::nurbs::Srf::of(&parts[1]).unwrap();
        assert!((0..=4).all(|i| a.at_st(0.5, i as f64 / 4.0)[1].abs() < 1e-12), "first leg y = 0");
        assert!((0..=4).all(|i| (b.at_st(0.5, i as f64 / 4.0)[0] - 2.0).abs() < 1e-12), "second leg x = 2");
        // A smooth surface stays whole.
        let flat = Geo::Surface {
            degree: [1, 1],
            size: [2, 3],
            points: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [0.0, 0.0, 3.0], [1.0, 0.0, 3.0], [2.0, 0.0, 3.0]],
            weights: vec![1.0; 6],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 0.5, 1.0, 1.0]],
        };
        assert_eq!(split_at_kinks(&flat, 1).len(), 1);
    }
}
