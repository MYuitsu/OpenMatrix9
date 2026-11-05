// SPDX-License-Identifier: LGPL-2.1-or-later
//! Curves taken from meshes: plane sections, mesh–mesh intersections, feature edges, naked
//! borders, silhouettes; closest points on a mesh; chaining loose segments into polylines.
use om9_addon_abi::session::{SelectedMesh, Vec3, add, cross, dist, dot, norm, scale, sub};

/// Join segments that share end points (within `tol`) into polylines; closed chains repeat
/// no point and are flagged.
pub fn chain(segments: &[[Vec3; 2]], tol: f64) -> Vec<(Vec<Vec3>, bool)> {
    let key = |p: Vec3| -> [i64; 3] {
        let k = 1.0 / tol.max(1e-12);
        [
            (p[0] * k).round() as i64,
            (p[1] * k).round() as i64,
            (p[2] * k).round() as i64,
        ]
    };
    let mut ends: std::collections::HashMap<[i64; 3], Vec<usize>> = Default::default();
    for (i, s) in segments.iter().enumerate() {
        ends.entry(key(s[0])).or_default().push(i);
        ends.entry(key(s[1])).or_default().push(i);
    }
    let mut used = vec![false; segments.len()];
    let mut out = Vec::new();
    for start in 0..segments.len() {
        if used[start] {
            continue;
        }
        used[start] = true;
        let mut line = vec![segments[start][0], segments[start][1]];
        // Grow forward, then backward.
        for forward in [true, false] {
            loop {
                let tip = if forward {
                    *line.last().unwrap()
                } else {
                    line[0]
                };
                let next = ends
                    .get(&key(tip))
                    .and_then(|c| c.iter().copied().find(|&j| !used[j]));
                let Some(j) = next else { break };
                used[j] = true;
                let s = segments[j];
                let other = if key(s[0]) == key(tip) { s[1] } else { s[0] };
                if forward {
                    line.push(other);
                } else {
                    line.insert(0, other);
                }
            }
        }
        let closed = line.len() > 3 && dist(line[0], *line.last().unwrap()) <= tol * 2.0;
        if closed {
            line.pop();
        }
        line.dedup_by(|a, b| dist(*a, *b) < tol * 0.1);
        if line.len() >= 2 {
            out.push((line, closed));
        }
    }
    out
}

fn tri(m: &SelectedMesh, t: usize) -> [Vec3; 3] {
    let i = m.triangles[t];
    [
        m.vertices[i[0] as usize],
        m.vertices[i[1] as usize],
        m.vertices[i[2] as usize],
    ]
}

/// Segments where the plane (point `o`, normal `n`) cuts the triangles.
pub fn plane_segments(m: &SelectedMesh, o: Vec3, n: Vec3) -> Vec<[Vec3; 2]> {
    let mut out = Vec::new();
    for t in 0..m.triangles.len() {
        let p = tri(m, t);
        let d: Vec<f64> = p.iter().map(|&q| dot(sub(q, o), n)).collect();
        let mut pts = Vec::new();
        for k in 0..3 {
            let (a, b) = (k, (k + 1) % 3);
            if (d[a] > 0.0) != (d[b] > 0.0) {
                let s = d[a] / (d[a] - d[b]);
                pts.push(add(p[a], scale(sub(p[b], p[a]), s)));
            }
        }
        if pts.len() == 2 {
            out.push([pts[0], pts[1]]);
        }
    }
    out
}

/// Section polylines of a mesh by a plane.
pub fn section(m: &SelectedMesh, o: Vec3, n: Vec3) -> Vec<(Vec<Vec3>, bool)> {
    chain(&plane_segments(m, o, n), 1e-7)
}

/// Edges whose two triangles meet at more than `angle` degrees, and naked edges.
pub fn feature_edges(m: &SelectedMesh, angle: f64, naked_only: bool) -> Vec<[Vec3; 2]> {
    let mut by_edge = std::collections::HashMap::<(u32, u32), Vec<usize>>::new();
    for (i, t) in m.triangles.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            by_edge.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let normal = |t: usize| {
        let p = tri(m, t);
        let n = cross(sub(p[1], p[0]), sub(p[2], p[0]));
        scale(n, 1.0 / norm(n).max(1e-300))
    };
    let cos = angle.to_radians().cos();
    let mut out = Vec::new();
    for ((a, b), ts) in &by_edge {
        let keep = if ts.len() == 1 {
            true
        } else if naked_only {
            false
        } else {
            ts.len() > 2 || dot(normal(ts[0]), normal(ts[1])) < cos
        };
        if keep {
            out.push([m.vertices[*a as usize], m.vertices[*b as usize]]);
        }
    }
    out
}

/// Edges where the mesh turns from facing `view` to facing away (the outline seen along it).
pub fn silhouette(m: &SelectedMesh, view: Vec3) -> Vec<[Vec3; 2]> {
    let mut by_edge = std::collections::HashMap::<(u32, u32), Vec<usize>>::new();
    for (i, t) in m.triangles.iter().enumerate() {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            by_edge.entry((a.min(b), a.max(b))).or_default().push(i);
        }
    }
    let facing = |t: usize| {
        let p = tri(m, t);
        dot(cross(sub(p[1], p[0]), sub(p[2], p[0])), view) > 0.0
    };
    by_edge
        .iter()
        .filter(|(_, ts)| ts.len() == 1 || (ts.len() == 2 && facing(ts[0]) != facing(ts[1])))
        .map(|((a, b), _)| [m.vertices[*a as usize], m.vertices[*b as usize]])
        .collect()
}

/// Segment where two triangles cross, if any.
fn tri_tri(a: [Vec3; 3], b: [Vec3; 3]) -> Option<[Vec3; 2]> {
    // Points where the edges of one triangle pierce the other.
    let pierce = |p: [Vec3; 3], q: [Vec3; 3], out: &mut Vec<Vec3>| {
        let n = cross(sub(q[1], q[0]), sub(q[2], q[0]));
        if norm(n) < 1e-18 {
            return;
        }
        for k in 0..3 {
            let (s, e) = (p[k], p[(k + 1) % 3]);
            let ds = dot(sub(s, q[0]), n);
            let de = dot(sub(e, q[0]), n);
            if (ds > 0.0) == (de > 0.0) || ds == de {
                continue;
            }
            let x = add(s, scale(sub(e, s), ds / (ds - de)));
            // Inside q?
            let inside = (0..3).all(|j| {
                let (u, v) = (q[j], q[(j + 1) % 3]);
                dot(cross(sub(v, u), sub(x, u)), n) >= -1e-12
            });
            if inside {
                out.push(x);
            }
        }
    };
    let mut pts = Vec::new();
    pierce(a, b, &mut pts);
    pierce(b, a, &mut pts);
    pts.dedup_by(|x, y| dist(*x, *y) < 1e-12);
    if pts.len() < 2 {
        return None;
    }
    // The two farthest points of the crossing.
    let mut best = (0.0, 0, 1);
    for i in 0..pts.len() {
        for j in i + 1..pts.len() {
            let d = dist(pts[i], pts[j]);
            if d > best.0 {
                best = (d, i, j);
            }
        }
    }
    (best.0 > 1e-12).then(|| [pts[best.1], pts[best.2]])
}

/// Intersection polylines of two meshes.
pub fn mesh_intersection(a: &SelectedMesh, b: &SelectedMesh) -> Vec<(Vec<Vec3>, bool)> {
    let bbox = |m: &SelectedMesh, t: usize| {
        let p = tri(m, t);
        let mut lo = p[0];
        let mut hi = p[0];
        for q in &p[1..] {
            for i in 0..3 {
                lo[i] = lo[i].min(q[i]);
                hi[i] = hi[i].max(q[i]);
            }
        }
        (lo, hi)
    };
    // Bucket b's triangles on a coarse grid.
    let (mut lo, mut hi) = ([f64::MAX; 3], [f64::MIN; 3]);
    for v in &b.vertices {
        for i in 0..3 {
            lo[i] = lo[i].min(v[i]);
            hi[i] = hi[i].max(v[i]);
        }
    }
    let cells = ((b.triangles.len() as f64).cbrt().ceil() as usize).clamp(1, 64);
    let size: Vec<f64> = (0..3)
        .map(|i| ((hi[i] - lo[i]) / cells as f64).max(1e-9))
        .collect();
    let cell =
        |p: Vec3, i: usize| (((p[i] - lo[i]) / size[i]).floor().max(0.0) as usize).min(cells - 1);
    let mut grid = vec![Vec::new(); cells * cells * cells];
    for t in 0..b.triangles.len() {
        let (l, h) = bbox(b, t);
        for x in cell(l, 0)..=cell(h, 0) {
            for y in cell(l, 1)..=cell(h, 1) {
                for z in cell(l, 2)..=cell(h, 2) {
                    grid[(x * cells + y) * cells + z].push(t);
                }
            }
        }
    }
    let mut segs = Vec::new();
    for ta in 0..a.triangles.len() {
        let (l, h) = bbox(a, ta);
        if (0..3).any(|i| h[i] < lo[i] || l[i] > hi[i]) {
            continue;
        }
        let mut seen = std::collections::HashSet::new();
        for x in cell(l, 0)..=cell(h, 0) {
            for y in cell(l, 1)..=cell(h, 1) {
                for z in cell(l, 2)..=cell(h, 2) {
                    for &tb in &grid[(x * cells + y) * cells + z] {
                        if seen.insert(tb)
                            && let Some(s) = tri_tri(tri(a, ta), tri(b, tb))
                        {
                            segs.push(s);
                        }
                    }
                }
            }
        }
    }
    chain(&segs, 1e-6)
}

/// The point of the mesh nearest to `p`.
pub fn closest_on_mesh(m: &SelectedMesh, p: Vec3) -> Option<Vec3> {
    (0..m.triangles.len())
        .map(|t| {
            let q = tri(m, t);
            crate::sdf::closest_on_triangle(p, q[0], q[1], q[2])
        })
        .min_by(|a, b| dist(*a, p).total_cmp(&dist(*b, p)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{cuboid, sphere};

    fn sel(m: crate::mesh::Mesh) -> SelectedMesh {
        SelectedMesh {
            vertices: m.vertices,
            triangles: m.triangles,
        }
    }

    #[test]
    fn sections_edges_and_intersections() {
        let s = sel(sphere(2.0, 48));
        let loops = section(&s, [0.0; 3], [0.0, 0.0, 1.0]);
        assert_eq!(loops.len(), 1);
        assert!(loops[0].1, "the equator is closed");
        for p in &loops[0].0 {
            assert!((norm(*p) - 2.0).abs() < 0.01);
        }
        let b = sel(cuboid([0.0; 3], [1.0, 1.0, 1.0]));
        let edges = feature_edges(&b, 30.0, false);
        assert_eq!(
            edges.len(),
            12,
            "a box has 12 sharp edges (diagonals are flat)"
        );
        assert_eq!(chain(&edges, 1e-9).len() >= 3, true);
        let sil = silhouette(&s, [0.0, 0.0, 1.0]);
        let ring = chain(&sil, 1e-9);
        assert!(
            ring.iter().any(|r| r.1),
            "the outline of a ball seen from above is closed"
        );
        let a = sel(cuboid([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]));
        let c = sel(cuboid([0.0, 0.0, 0.0], [2.0, 2.0, 2.0]));
        let x = mesh_intersection(&a, &c);
        let total: f64 = x
            .iter()
            .map(|(l, closed)| {
                let mut s: f64 = l.windows(2).map(|w| dist(w[0], w[1])).sum();
                if *closed {
                    s += dist(l[0], *l.last().unwrap());
                }
                s
            })
            .sum();
        assert!(
            (total - 6.0).abs() < 1e-6,
            "two unit boxes cross along 6 mm of edges: {total}"
        );
        assert_eq!(
            closest_on_mesh(&a, [5.0, 0.0, 0.0]).unwrap(),
            [1.0, 0.0, 0.0]
        );
    }
}
