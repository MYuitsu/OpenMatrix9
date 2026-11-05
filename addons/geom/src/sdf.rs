// SPDX-License-Identifier: LGPL-2.1-or-later
//! Signed distance grids of closed meshes and their level surfaces (marching tetrahedra), for
//! offsets that stay clean where a vertex offset would fold: rounded edges (Fillet Edge as a
//! morphological opening), shells, thick offsets of a whole object.
use crate::mesh::Mesh;
use om9_addon_abi::session::{Vec3, add, cross, dot, scale, sub};

/// Values on a regular grid (x fastest).
pub struct Grid {
    pub origin: Vec3,
    pub cell: f64,
    pub dims: [usize; 3],
    pub values: Vec<f64>,
}

impl Grid {
    fn at(&self, i: usize, j: usize, k: usize) -> f64 {
        self.values[(k * self.dims[1] + j) * self.dims[0] + i]
    }
    fn point(&self, i: usize, j: usize, k: usize) -> Vec3 {
        add(
            self.origin,
            [i as f64 * self.cell, j as f64 * self.cell, k as f64 * self.cell],
        )
    }
}

/// Closest point on triangle `abc` to `p` (Ericson, Real-Time Collision Detection 5.1.5).
pub fn closest_on_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let ap = sub(p, a);
    let d1 = dot(ab, ap);
    let d2 = dot(ac, ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = sub(p, b);
    let d3 = dot(ab, bp);
    let d4 = dot(ac, bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return add(a, scale(ab, d1 / (d1 - d3)));
    }
    let cp = sub(p, c);
    let d5 = dot(ab, cp);
    let d6 = dot(ac, cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return add(a, scale(ac, d2 / (d2 - d6)));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        return add(b, scale(sub(c, b), (d4 - d3) / ((d4 - d3) + (d5 - d6))));
    }
    let denom = 1.0 / (va + vb + vc);
    add(a, add(scale(ab, vb * denom), scale(ac, vc * denom)))
}

/// Signed distance grid of a closed mesh (negative inside), exact within `band` of the
/// surface and clamped to ±`band` farther away. Grids are capped near 6 M points: the cell
/// grows if needed.
pub fn from_mesh(m: &Mesh, cell: f64, band: f64) -> Grid {
    let b = m.bbox();
    let pad = band + 2.0 * cell;
    let lo = sub(b[0], [pad; 3]);
    let hi = add(b[1], [pad; 3]);
    let mut cell = cell.max(1e-4);
    let mut dims = [0usize; 3];
    loop {
        for i in 0..3 {
            dims[i] = ((hi[i] - lo[i]) / cell).ceil() as usize + 1;
        }
        if dims[0] * dims[1] * dims[2] <= 6_000_000 {
            break;
        }
        cell *= 1.25;
    }
    let n = dims[0] * dims[1] * dims[2];
    let mut dist = vec![band; n];
    let idx = |i: usize, j: usize, k: usize| (k * dims[1] + j) * dims[0] + i;
    let tri = |t: &[u32; 3]| {
        (
            m.vertices[t[0] as usize],
            m.vertices[t[1] as usize],
            m.vertices[t[2] as usize],
        )
    };
    // Unsigned distance within the band, triangle by triangle.
    let reach = band + cell;
    for t in &m.triangles {
        let (a, bb, c) = tri(t);
        let mut r0 = [0usize; 3];
        let mut r1 = [0usize; 3];
        for i in 0..3 {
            let mn = a[i].min(bb[i]).min(c[i]) - reach;
            let mx = a[i].max(bb[i]).max(c[i]) + reach;
            r0[i] = (((mn - lo[i]) / cell).floor().max(0.0)) as usize;
            r1[i] = ((((mx - lo[i]) / cell).ceil()) as usize).min(dims[i] - 1);
        }
        for k in r0[2]..=r1[2] {
            for j in r0[1]..=r1[1] {
                for i in r0[0]..=r1[0] {
                    let p = add(lo, [i as f64 * cell, j as f64 * cell, k as f64 * cell]);
                    let q = closest_on_triangle(p, a, bb, c);
                    let d = om9_addon_abi::session::dist(p, q);
                    let e = &mut dist[idx(i, j, k)];
                    if d < *e {
                        *e = d;
                    }
                }
            }
        }
    }
    // Sign: parity of crossings along +x for every (y, z) row, triangles bucketed by row.
    let mut rows: Vec<Vec<u32>> = vec![Vec::new(); dims[1] * dims[2]];
    for (ti, t) in m.triangles.iter().enumerate() {
        let (a, bb, c) = tri(t);
        let j0 = (((a[1].min(bb[1]).min(c[1]) - lo[1]) / cell).floor().max(0.0)) as usize;
        let j1 = (((a[1].max(bb[1]).max(c[1]) - lo[1]) / cell).ceil() as usize).min(dims[1] - 1);
        let k0 = (((a[2].min(bb[2]).min(c[2]) - lo[2]) / cell).floor().max(0.0)) as usize;
        let k1 = (((a[2].max(bb[2]).max(c[2]) - lo[2]) / cell).ceil() as usize).min(dims[2] - 1);
        for k in k0..=k1 {
            for j in j0..=j1 {
                rows[k * dims[1] + j].push(ti as u32);
            }
        }
    }
    for k in 0..dims[2] {
        for j in 0..dims[1] {
            // Rays slightly off the grid lines so they do not graze shared edges.
            let y = lo[1] + j as f64 * cell + cell * 1.234e-4;
            let z = lo[2] + k as f64 * cell + cell * 2.345e-4;
            let mut xs: Vec<f64> = Vec::new();
            for &ti in &rows[k * dims[1] + j] {
                let (a, bb, c) = tri(&m.triangles[ti as usize]);
                // Intersection of the line (·, y, z) with the triangle in the yz projection.
                let (u0, v0) = (bb[1] - a[1], bb[2] - a[2]);
                let (u1, v1) = (c[1] - a[1], c[2] - a[2]);
                let det = u0 * v1 - u1 * v0;
                if det.abs() < 1e-18 {
                    continue;
                }
                let (py, pz) = (y - a[1], z - a[2]);
                let s = (py * v1 - u1 * pz) / det;
                let t = (u0 * pz - py * v0) / det;
                if s >= 0.0 && t >= 0.0 && s + t <= 1.0 {
                    xs.push(a[0] + s * (bb[0] - a[0]) + t * (c[0] - a[0]));
                }
            }
            xs.sort_by(f64::total_cmp);
            let mut h = 0;
            for i in 0..dims[0] {
                let x = lo[0] + i as f64 * cell;
                while h < xs.len() && xs[h] < x {
                    h += 1;
                }
                if h % 2 == 1 {
                    let e = &mut dist[idx(i, j, k)];
                    *e = -*e;
                }
            }
        }
    }
    Grid {
        origin: lo,
        cell,
        dims,
        values: dist,
    }
}

/// The level surface `value = iso` of a grid as a closed, welded mesh facing +value.
pub fn level(g: &Grid, iso: f64) -> Mesh {
    // Kuhn subdivision of a cube into 6 tetrahedra (corner bits: x=1, y=2, z=4).
    const TETS: [[usize; 4]; 6] = [
        [0, 1, 3, 7],
        [0, 1, 5, 7],
        [0, 2, 3, 7],
        [0, 2, 6, 7],
        [0, 4, 5, 7],
        [0, 4, 6, 7],
    ];
    let [nx, ny, nz] = g.dims;
    let gid = |i: usize, j: usize, k: usize| (k * ny + j) * nx + i;
    let mut m = Mesh::default();
    let mut edge_vertex = std::collections::HashMap::<(usize, usize), u32>::new();
    let mut vertex = |a: (usize, [usize; 3]), b: (usize, [usize; 3]), m: &mut Mesh| -> u32 {
        let key = (a.0.min(b.0), a.0.max(b.0));
        *edge_vertex.entry(key).or_insert_with(|| {
            let (va, vb) = (g.at(a.1[0], a.1[1], a.1[2]), g.at(b.1[0], b.1[1], b.1[2]));
            let t = ((iso - va) / (vb - va)).clamp(0.0, 1.0);
            let pa = g.point(a.1[0], a.1[1], a.1[2]);
            let pb = g.point(b.1[0], b.1[1], b.1[2]);
            m.vertices.push(add(pa, scale(sub(pb, pa), t)));
            (m.vertices.len() - 1) as u32
        })
    };
    for k in 0..nz - 1 {
        for j in 0..ny - 1 {
            for i in 0..nx - 1 {
                let corner = |c: usize| -> (usize, [usize; 3]) {
                    let p = [i + (c & 1), j + ((c >> 1) & 1), k + ((c >> 2) & 1)];
                    (gid(p[0], p[1], p[2]), p)
                };
                for tet in TETS {
                    let v: Vec<(usize, [usize; 3])> = tet.iter().map(|&c| corner(c)).collect();
                    let inside: Vec<bool> = v.iter().map(|c| g.at(c.1[0], c.1[1], c.1[2]) < iso).collect();
                    let ins: Vec<usize> = (0..4).filter(|&q| inside[q]).collect();
                    let outs: Vec<usize> = (0..4).filter(|&q| !inside[q]).collect();
                    let pos = |q: usize| g.point(v[q].1[0], v[q].1[1], v[q].1[2]);
                    let mid = |p: usize, q: usize| scale(add(pos(p), pos(q)), 0.5);
                    // Orientation is decided on undistorted proxies (tet corners / edge
                    // midpoints), so slivers near grid points never flip.
                    let push = |tri: [u32; 3], proxy: [Vec3; 3], toward_out: Vec3, m: &mut Mesh| {
                        let n = cross(sub(proxy[1], proxy[0]), sub(proxy[2], proxy[0]));
                        if dot(n, toward_out) >= 0.0 {
                            m.triangles.push(tri);
                        } else {
                            m.triangles.push([tri[0], tri[2], tri[1]]);
                        }
                    };
                    let centroid = |ids: &[usize]| -> Vec3 {
                        let s = ids.iter().fold([0.0; 3], |acc, &q| add(acc, pos(q)));
                        scale(s, 1.0 / ids.len() as f64)
                    };
                    match ins.len() {
                        1 | 3 => {
                            let (lone, others, lone_inside) = if ins.len() == 1 {
                                (ins[0], outs.clone(), true)
                            } else {
                                (outs[0], ins.clone(), false)
                            };
                            let a = vertex(v[lone], v[others[0]], &mut m);
                            let b = vertex(v[lone], v[others[1]], &mut m);
                            let c = vertex(v[lone], v[others[2]], &mut m);
                            let proxy = [mid(lone, others[0]), mid(lone, others[1]), mid(lone, others[2])];
                            let away = sub(centroid(&others), pos(lone));
                            let toward_out = if lone_inside { away } else { scale(away, -1.0) };
                            push([a, b, c], proxy, toward_out, &mut m);
                        }
                        2 => {
                            let (p0, p1) = (ins[0], ins[1]);
                            let (q0, q1) = (outs[0], outs[1]);
                            let a = vertex(v[p0], v[q0], &mut m);
                            let b = vertex(v[p0], v[q1], &mut m);
                            let c = vertex(v[p1], v[q1], &mut m);
                            let d = vertex(v[p1], v[q0], &mut m);
                            let (pa, pb, pc, pd) = (mid(p0, q0), mid(p0, q1), mid(p1, q1), mid(p1, q0));
                            let toward_out = sub(centroid(&outs), centroid(&ins));
                            // Both halves of the quad share the quad's orientation.
                            let n = cross(sub(pb, pa), sub(pc, pa));
                            if dot(n, toward_out) >= 0.0 {
                                m.triangles.push([a, b, c]);
                                m.triangles.push([a, c, d]);
                            } else {
                                m.triangles.push([a, c, b]);
                                m.triangles.push([a, d, c]);
                            }
                            let _ = (pd, push);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    m
}

/// Offset a closed mesh by `d` (outward > 0) on a grid of `cell`.
pub fn offset(m: &Mesh, d: f64, cell: f64) -> Mesh {
    let g = from_mesh(m, cell, d.abs() + 3.0 * cell);
    level(&g, d)
}

/// Round the convex edges of a closed mesh with radius `r` (erode then grow back).
pub fn round_edges(m: &Mesh, r: f64, cell: f64) -> Mesh {
    let eroded = offset(m, -r, cell);
    if eroded.triangles.is_empty() {
        return m.clone();
    }
    offset(&eroded, r, cell)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{cuboid, sphere};
    use std::f64::consts::PI;

    #[test]
    fn offsets_and_rounds() {
        let s = sphere(2.0, 48);
        let g = from_mesh(&s, 0.1, 0.5);
        let c = g.dims.map(|d| d / 2);
        assert!(g.at(c[0], c[1], c[2]) < 0.0, "inside is negative");
        let grown = offset(&s, 0.5, 0.15);
        assert!(grown.is_closed(), "level surfaces are closed");
        let exact = 4.0 / 3.0 * PI * 2.5f64.powi(3);
        assert!((grown.volume() / exact - 1.0).abs() < 0.03, "{} vs {exact}", grown.volume());
        let b = cuboid([0.0; 3], [4.0, 4.0, 4.0]);
        let r = round_edges(&b, 0.8, 0.16);
        assert!(r.is_closed());
        // Rounded box: 64 minus the edge and corner material removed by an r = 0.8 rounding.
        let removed = (4.0 - PI) * 0.64 * (12.0 * (4.0 - 1.6)) / 4.0 * 1.0 + 8.0 * (0.512 - 4.0 / 3.0 * PI * 0.512 / 8.0);
        let expected = 64.0 - removed;
        assert!((r.volume() / expected - 1.0).abs() < 0.03, "{} vs {expected}", r.volume());
    }
}
