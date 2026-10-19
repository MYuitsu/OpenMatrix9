//! WHERE REGIONS MEET: the corners and the edges of the faces a mesh was split into, before any curve is fitted to
//! them.
//!
//! A side of a triangle whose neighbour lies in another region - or that has no neighbour - is a side of a border, and
//! the region on its left walks it along the triangle's winding. A region's border sides link into loops by turning
//! about the corner each ends in, through the region's own triangles. Where the region across changes along a loop,
//! the loop passes a corner of the solid; a loop along which it never changes is one closed edge, a washer's round
//! edge. Every corner of a border side is a corner of triangles of both regions, so it lies on both surfaces within the
//! tolerance: a false corner can stand no farther from a true one than two tangent surfaces stay within the tolerance
//! of each other, about sqrt(2 x tolerance x radius) - 0.09 mm on a bar rounded by 2 mm.

use crate::{Prepared, Region, Surface, NO_NEIGHBOUR};
use nalgebra::{DMatrix, DVector, Vector3};
use std::collections::HashMap;

/// No region: across a border the mesh leaves open.
pub const OPEN: u32 = u32::MAX;

/// An edge between two faces.
#[derive(Clone, Debug)]
pub struct Edge {
    /// The face on the left of the edge as its points run, seen from outside the solid, and the face on its right, or
    /// `OPEN`.
    pub faces: [u32; 2],
    /// The corners it runs between, or `None` for a closed edge.
    pub ends: Option<[usize; 2]>,
    /// The mesh corners it runs through, in order; where it has corners, the first and the last point are theirs.
    pub points: Vec<[f64; 3]>,
}

/// The corners and the edges of the faces, and how each face is bounded.
#[derive(Clone, Debug, Default)]
pub struct Boundaries {
    pub vertices: Vec<[f64; 3]>,
    /// The mesh corner each corner of the solid was found at.
    pub corners: Vec<u32>,
    pub edges: Vec<Edge>,
    /// For every region, its loops; a loop is its edges in order, each with whether the loop walks it as its points
    /// run. The region lies on the left of every loop seen from outside: its outer loop turns anticlockwise, a hole's
    /// clockwise.
    pub loops: Vec<Vec<Vec<(usize, bool)>>>,
}

/// The corners and edges of the faces `found` splits `p` into.
pub fn boundaries(p: &Prepared, found: &[Region]) -> Boundaries {
    let n = p.mesh.tris.len();
    let tris = &p.mesh.tris;
    let mut owner = vec![OPEN; n];
    for (r, region) in found.iter().enumerate() {
        for &t in &region.tris {
            owner[t as usize] = r as u32;
        }
    }
    let across = |(t, k): (usize, usize)| -> u32 {
        let u = p.neighbours[t][k];
        if u == NO_NEIGHBOUR {
            OPEN
        } else {
            owner[u as usize]
        }
    };
    let border = |s: (usize, usize)| across(s) != owner[s.0];
    let from = |(t, k): (usize, usize)| tris[t][k];
    let to = |(t, k): (usize, usize)| tris[t][(k + 1) % 3];
    // The side across side `s`, running the other way; `None` where the mesh has no single neighbour.
    let twin = |s: (usize, usize)| -> Option<(usize, usize)> {
        let u = p.neighbours[s.0][s.1];
        if u == NO_NEIGHBOUR {
            return None;
        }
        let u = u as usize;
        (0..3).find(|&j| tris[u][j] == to(s) && tris[u][(j + 1) % 3] == from(s)).map(|j| (u, j))
    };
    // THE NEXT SIDE OF THE SAME BORDER after `s`: turn about the corner it ends in, through the region's own triangles,
    // until a side of the border starts there
    let next = |s: (usize, usize)| -> (usize, usize) {
        let mut at = (s.0, (s.1 + 1) % 3);
        for _ in 0..n {
            if border(at) {
                break;
            }
            let Some((u, j)) = twin(at) else { break };
            at = (u, (j + 1) % 3);
        }
        at
    };

    let mut seen = vec![false; 3 * n];
    let mut walks: Vec<Vec<Vec<(usize, usize)>>> = vec![Vec::new(); found.len()];
    for t in 0..n {
        if owner[t] == OPEN {
            continue;
        }
        for k in 0..3 {
            if seen[3 * t + k] || !border((t, k)) {
                continue;
            }
            let mut sides = Vec::new();
            let mut s = (t, k);
            while !seen[3 * s.0 + s.1] {
                seen[3 * s.0 + s.1] = true;
                sides.push(s);
                s = next(s);
            }
            walks[owner[t] as usize].push(sides);
        }
    }

    // THE CORNERS: where the region across changes along a loop, and every region meeting there
    let mut vertex_at: HashMap<u32, usize> = HashMap::new();
    let mut meeting: Vec<Vec<u32>> = Vec::new();
    for (r, loops) in walks.iter().enumerate() {
        for sides in loops {
            let m = sides.len();
            for i in 0..m {
                let (before, here) = (across(sides[(i + m - 1) % m]), across(sides[i]));
                if before == here {
                    continue;
                }
                let v = *vertex_at.entry(from(sides[i])).or_insert_with(|| {
                    meeting.push(Vec::new());
                    meeting.len() - 1
                });
                for f in [r as u32, before, here] {
                    if f != OPEN && !meeting[v].contains(&f) {
                        meeting[v].push(f);
                    }
                }
            }
        }
    }
    let corner = |c: u32| {
        let q = p.mesh.verts[c as usize];
        [q.x, q.y, q.z]
    };
    let mut vertices = vec![[0.0; 3]; meeting.len()];
    let mut corners = vec![0u32; meeting.len()];
    for (&c, &v) in &vertex_at {
        let on: Vec<&Surface> = meeting[v].iter().filter_map(|&f| found[f as usize].surface.as_ref()).collect();
        vertices[v] = place(corner(c), &on);
        corners[v] = c;
    }

    // THE EDGES: a loop cut at every corner it passes; the same run walked back by the region across is the same edge
    let mut edge_of: HashMap<(usize, usize), (usize, bool)> = HashMap::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut loops = vec![Vec::new(); found.len()];
    for (r, region_walks) in walks.iter().enumerate() {
        for sides in region_walks {
            let m = sides.len();
            let cuts: Vec<usize> = (0..m).filter(|&i| vertex_at.contains_key(&from(sides[i]))).collect();
            let runs: Vec<Vec<(usize, usize)>> = if cuts.is_empty() {
                vec![sides.clone()]
            } else {
                (0..cuts.len())
                    .map(|c| {
                        let (a, z) = (cuts[c], cuts[(c + 1) % cuts.len()]);
                        let len = match (z + m - a) % m {
                            0 => m,
                            l => l,
                        };
                        (0..len).map(|j| sides[(a + j) % m]).collect()
                    })
                    .collect()
            };
            let mut walk = Vec::new();
            for run in runs {
                if let Some(&(e, forward)) = twin(run[0]).and_then(|tw| edge_of.get(&tw)) {
                    walk.push((e, !forward));
                    continue;
                }
                let e = edges.len();
                let ends = (!cuts.is_empty()).then(|| [vertex_at[&from(run[0])], vertex_at[&to(run[run.len() - 1])]]);
                let mut points: Vec<[f64; 3]> = run.iter().map(|&s| corner(from(s))).collect();
                if let Some([a, z]) = ends {
                    points[0] = vertices[a];
                    points.push(vertices[z]);
                }
                edges.push(Edge { faces: [r as u32, across(run[0])], ends, points });
                for &s in &run {
                    edge_of.insert(s, (e, true));
                }
                walk.push((e, true));
            }
            loops[r].push(walk);
        }
    }
    Boundaries { vertices, corners, edges, loops }
}

/// How far `v` lies from `s`, outside positive: the one of the two sides the normal the fit found points to.
fn signed(s: &Surface, v: &Vector3<f64>) -> f64 {
    match s {
        Surface::Plane { point, normal } => (v - Vector3::from(*point)).dot(&Vector3::from(*normal)),
        Surface::Cylinder { point, axis, radius } => {
            let d = v - Vector3::from(*point);
            let a = Vector3::from(*axis);
            (d - a * d.dot(&a)).norm() - radius
        }
        Surface::Sphere { center, radius } => (v - Vector3::from(*center)).norm() - radius,
        Surface::Cone { apex, axis, half_angle } => {
            let d = v - Vector3::from(*apex);
            let a = Vector3::from(*axis);
            let z = d.dot(&a);
            (d - a * z).norm() * half_angle.cos() - z * half_angle.sin()
        }
        Surface::Torus { center, axis, major, minor } => {
            let d = v - Vector3::from(*center);
            let a = Vector3::from(*axis);
            let z = d.dot(&a);
            ((d - a * z).norm() - major).hypot(z) - minor
        }
        // a free form pulls a corner nowhere: its border is the mesh's
        Surface::Free { .. } | Surface::Spline(_) => 0.0,
        Surface::Helix { .. } | Surface::RoundHelix { .. } | Surface::Coil { .. } => s.signed_off(v),
    }
}

/// The point nearest `start` that lies on every surface of `on` as nearly as they allow. Three planes meet in one point,
/// and it is found to the last digit; where the surfaces are tangent the point may slide along them, and `settle` keeps
/// it by `start`.
pub(crate) fn place(start: [f64; 3], on: &[&Surface]) -> [f64; 3] {
    if on.is_empty() {
        return start;
    }
    let x = settle(&start, |x| on.iter().map(|s| signed(s, &Vector3::new(x[0], x[1], x[2]))).collect());
    [x[0], x[1], x[2]]
}

/// The numbers nearest `start` that bring every residual to zero as nearly as they allow: damped least squares, the
/// steps taken by central differences.
///
/// ON A LEASH OF FOUR TIMES THE WORST RESIDUAL AT `start`. Faces meeting at 30 deg or steeper - the sharp edge - move a
/// corner by at most 1/sin 30 x sqrt 3 = 3.5 times its distance; farther than that only faces meeting at a graze lead
/// it, and there the exact meeting point says nothing the mesh does. Measured before the leash: a corner of
/// `cube_gears` went 90 mm off its mesh corner, of `seam_test_object` 201 mm, and 22 174 of its corners farther than
/// ten tolerances.
pub(crate) fn settle(start: &[f64], residuals: impl Fn(&[f64]) -> Vec<f64>) -> Vec<f64> {
    let n = start.len();
    let cost = |x: &[f64]| residuals(x).iter().map(|r| r * r).sum::<f64>();
    let origin = DVector::from_column_slice(start);
    let leash = 4.0 * residuals(start).iter().fold(0.0f64, |m, r| m.max(r.abs()));
    let mut x = origin.clone();
    let mut c = cost(x.as_slice());
    let mut damping = 1e-6;
    for _ in 0..50 {
        if c < 1e-26 {
            break;
        }
        let h = 1e-7 * (1.0 + x.norm());
        let r = DVector::from_vec(residuals(x.as_slice()));
        let mut j = DMatrix::zeros(r.len(), n);
        for k in 0..n {
            let (mut up, mut down) = (x.clone(), x.clone());
            up[k] += h;
            down[k] -= h;
            let d = (DVector::from_vec(residuals(up.as_slice())) - DVector::from_vec(residuals(down.as_slice()))) / (2.0 * h);
            j.set_column(k, &d);
        }
        let jtj = j.transpose() * &j;
        let jtr = j.transpose() * &r;
        let scale = jtj.trace().max(1e-30);
        let mut better = false;
        for _ in 0..12 {
            if let Some(step) = (&jtj + DMatrix::identity(n, n) * (damping * scale)).lu().solve(&(-&jtr)) {
                let y = &x + step;
                let cy = cost(y.as_slice());
                if cy < c && (&y - &origin).norm() <= leash {
                    (x, c, better) = (y, cy, true);
                    damping = (damping * 0.1).max(1e-12);
                    break;
                }
            }
            damping *= 10.0;
        }
        if !better {
            break;
        }
    }
    x.as_slice().to_vec()
}

#[cfg(test)]
mod tests {
    use super::place;
    use crate::Surface;

    /// A CORNER OF FACES THAT MEET AT A GRAZE STAYS BY ITS MESH CORNER. Two planes 0.0001 rad apart and a third across
    /// them meet 10 mm away from a corner that lies on the first and the third exactly and 0.001 off the second: that
    /// far point is where the least squares lead, yet nothing in the mesh says the corner is there.
    #[test]
    fn a_corner_of_grazing_faces_stays_by_its_mesh_corner() {
        let tilt: f64 = 1e-4;
        let flat = Surface::Plane { point: [0.0, 0.0, 0.0], normal: [0.0, 0.0, 1.0] };
        let (s, c) = (tilt.sin(), tilt.cos());
        let grazing = Surface::Plane { point: [10.0, 0.0, 0.0], normal: [-s, 0.0, c] };
        let across = Surface::Plane { point: [0.0, 0.0, 0.0], normal: [0.0, 1.0, 0.0] };
        let at = place([0.0, 0.0, 0.0], &[&flat, &grazing, &across]);
        let moved = (at[0].powi(2) + at[1].powi(2) + at[2].powi(2)).sqrt();
        assert!(moved < 0.01, "the corner went {moved} mm off to {at:?}");
    }
}
