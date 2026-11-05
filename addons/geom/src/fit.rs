// SPDX-License-Identifier: LGPL-2.1-or-later
//! Fitting NURBS through points: curves through points at given parameters and surfaces
//! through a grid of points (the rows share their parameters, so the surface is one
//! tensor-product patch), plus sampling helpers for surfaces.
use crate::nurbs::Srf;
use crate::{basis, find_span};
use om9_addon_abi::session::{Geo, Vec3, dist};

/// Chord-length parameters in [0, 1] of a polyline.
pub fn chord_params(points: &[Vec3]) -> Vec<f64> {
    let mut s = vec![0.0];
    for w in points.windows(2) {
        s.push(s.last().unwrap() + dist(w[0], w[1]));
    }
    let total = *s.last().unwrap();
    if total < 1e-15 {
        let n = points.len().max(2) - 1;
        return (0..points.len()).map(|i| i as f64 / n as f64).collect();
    }
    s.iter().map(|v| v / total).collect()
}

/// Knots by averaging the parameters (clamped, domain [0, 1]).
pub fn averaged_knots(params: &[f64], p: usize) -> Vec<f64> {
    let n = params.len();
    let mut k = vec![0.0; p + 1];
    for j in 1..n - p {
        k.push(params[j..j + p].iter().sum::<f64>() / p as f64);
    }
    k.extend(std::iter::repeat_n(1.0, p + 1));
    k
}

/// Solve the interpolation matrix once and apply it to many right-hand sides (each a list of
/// points): control points so that the curve passes through `rhs[r][k]` at `params[k]`.
fn solve(params: &[f64], p: usize, knots: &[f64], rhs: &[Vec<Vec3>]) -> Vec<Vec<Vec3>> {
    let n = params.len();
    let mut a = vec![vec![0.0; n]; n];
    for (k, &u) in params.iter().enumerate() {
        let span = find_span(n, p, u, knots);
        let b = basis(span, u, p, knots);
        for (i, v) in b.iter().enumerate() {
            a[k][span - p + i] = *v;
        }
    }
    // Gaussian elimination with partial pivoting, all right-hand sides at once.
    let mut x: Vec<Vec<Vec3>> = rhs.to_vec();
    let m = rhs.len();
    for col in 0..n {
        let piv = (col..n)
            .max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))
            .unwrap();
        a.swap(col, piv);
        for r in x.iter_mut() {
            r.swap(col, piv);
        }
        let d = a[col][col];
        if d.abs() < 1e-14 {
            continue;
        }
        for row in col + 1..n {
            let f = a[row][col] / d;
            if f == 0.0 {
                continue;
            }
            for c in col..n {
                a[row][c] -= f * a[col][c];
            }
            for r in 0..m {
                for c in 0..3 {
                    x[r][row][c] -= f * x[r][col][c];
                }
            }
        }
    }
    for col in (0..n).rev() {
        let d = a[col][col];
        for r in 0..m {
            let mut v = x[r][col];
            for c in col + 1..n {
                for k in 0..3 {
                    v[k] -= a[col][c] * x[r][c][k];
                }
            }
            for k in 0..3 {
                v[k] /= if d.abs() < 1e-14 { 1.0 } else { d };
            }
            x[r][col] = v;
        }
    }
    x
}

/// A NURBS surface through a grid of points: `rows[i][j]` (i along u, j along v), degree up
/// to `degree` in each direction. All rows must have the same length.
pub fn surface_through(rows: &[Vec<Vec3>], degree: [usize; 2]) -> Option<Geo> {
    let nu = rows.len();
    let nv = rows.first()?.len();
    if nu < 2 || nv < 2 || rows.iter().any(|r| r.len() != nv) {
        return None;
    }
    let pu = degree[0].min(nu - 1).max(1);
    let pv = degree[1].min(nv - 1).max(1);
    // Shared parameters: the average of every row's (column's) chord parameters.
    let mut tv = vec![0.0; nv];
    for r in rows {
        for (a, b) in tv.iter_mut().zip(chord_params(r)) {
            *a += b / nu as f64;
        }
    }
    let cols: Vec<Vec<Vec3>> = (0..nv)
        .map(|j| rows.iter().map(|r| r[j]).collect())
        .collect();
    let mut su = vec![0.0; nu];
    for c in &cols {
        for (a, b) in su.iter_mut().zip(chord_params(c)) {
            *a += b / nv as f64;
        }
    }
    let kv = averaged_knots(&tv, pv);
    let ku = averaged_knots(&su, pu);
    // Along v for every row, then along u for every column of the result.
    let r1 = solve(&tv, pv, &kv, rows);
    let cols1: Vec<Vec<Vec3>> = (0..nv).map(|j| r1.iter().map(|r| r[j]).collect()).collect();
    let c2 = solve(&su, pu, &ku, &cols1);
    let mut points = Vec::with_capacity(nu * nv);
    for i in 0..nu {
        for c in c2.iter() {
            points.push(c[i]);
        }
    }
    Some(Geo::Surface {
        degree: [pu, pv],
        size: [nu, nv],
        points,
        weights: vec![1.0; nu * nv],
        knots: [ku, kv],
    })
}

/// A surface whose control points are the grid itself (clamped uniform knots): smooth, inside
/// the grid's hull, touching its corners and edges' ends.
pub fn surface_on_grid(rows: &[Vec<Vec3>], degree: [usize; 2]) -> Option<Geo> {
    let nu = rows.len();
    let nv = rows.first()?.len();
    if nu < 2 || nv < 2 || rows.iter().any(|r| r.len() != nv) {
        return None;
    }
    let pu = degree[0].min(nu - 1).max(1);
    let pv = degree[1].min(nv - 1).max(1);
    let k = |n: usize, p: usize| -> Vec<f64> {
        let kk = crate::clamped_uniform(n, p);
        let last = *kk.last().unwrap();
        kk.iter().map(|v| v / last).collect()
    };
    Some(Geo::Surface {
        degree: [pu, pv],
        size: [nu, nv],
        points: rows.iter().flatten().copied().collect(),
        weights: vec![1.0; nu * nv],
        knots: [k(nu, pu), k(nv, pv)],
    })
}

/// `nu` × `nv` points of a surface on a regular (s, t) grid.
pub fn grid(s: &Srf, nu: usize, nv: usize) -> Vec<Vec<Vec3>> {
    (0..nu)
        .map(|i| {
            let a = i as f64 / (nu - 1).max(1) as f64;
            (0..nv)
                .map(|j| s.at_st(a, j as f64 / (nv - 1).max(1) as f64))
                .collect()
        })
        .collect()
}

/// Rows of a surface on a sub-domain `[s0, s1] × [t0, t1]`.
pub fn sub_grid(
    s: &Srf,
    s_range: [f64; 2],
    t_range: [f64; 2],
    nu: usize,
    nv: usize,
) -> Vec<Vec<Vec3>> {
    (0..nu)
        .map(|i| {
            let a = s_range[0] + (s_range[1] - s_range[0]) * i as f64 / (nu - 1).max(1) as f64;
            (0..nv)
                .map(|j| {
                    s.at_st(
                        a,
                        t_range[0] + (t_range[1] - t_range[0]) * j as f64 / (nv - 1).max(1) as f64,
                    )
                })
                .collect()
        })
        .collect()
}

/// A triangle mesh of a grid of points (rows along u), for previews and mesh results.
pub fn grid_mesh(rows: &[Vec<Vec3>]) -> crate::mesh::Mesh {
    let nu = rows.len();
    let nv = rows.first().map_or(0, Vec::len);
    let mut m = crate::mesh::Mesh::default();
    for r in rows {
        m.vertices.extend_from_slice(r);
    }
    for i in 0..nu.saturating_sub(1) {
        for j in 0..nv.saturating_sub(1) {
            let a = (i * nv + j) as u32;
            let b = a + nv as u32;
            m.triangles.push([a, b, b + 1]);
            m.triangles.push([a, b + 1, a + 1]);
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;
    use om9_addon_abi::session::norm;

    #[test]
    fn surfaces_pass_through_their_grid() {
        // A cylinder patch: rows along the axis, columns around.
        let rows: Vec<Vec<Vec3>> = (0..6)
            .map(|i| {
                (0..9)
                    .map(|j| {
                        let a = std::f64::consts::PI * j as f64 / 8.0;
                        [5.0 * a.cos(), 5.0 * a.sin(), i as f64]
                    })
                    .collect()
            })
            .collect();
        let g = surface_through(&rows, [3, 3]).unwrap();
        let s = Srf::of(&g).unwrap();
        // The grid points lie on the surface (closest parameters reach them).
        for r in &rows {
            for p in r {
                let (a, b) = s.closest_st(*p);
                assert!(dist(s.at_st(a, b), *p) < 1e-3, "{p:?}");
            }
        }
        // In between it stays on the cylinder.
        for i in 0..=10 {
            for j in 0..=10 {
                let q = s.at_st(i as f64 / 10.0, j as f64 / 10.0);
                assert!((norm([q[0], q[1], 0.0]) - 5.0).abs() < 0.01, "{q:?}");
            }
        }
        let g2 = surface_on_grid(&rows, [3, 3]).unwrap();
        let s2 = Srf::of(&g2).unwrap();
        assert!(dist(s2.at_st(0.0, 0.0), rows[0][0]) < 1e-12);
        assert!(dist(s2.at_st(1.0, 1.0), rows[5][8]) < 1e-12);
        assert_eq!(grid_mesh(&rows).triangles.len(), 5 * 8 * 2);
    }
}
