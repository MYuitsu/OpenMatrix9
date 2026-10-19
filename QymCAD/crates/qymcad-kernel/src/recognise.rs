//! THE BODY: the faces found on a mesh, bounded by the loops of their edges, handed to the kernel to build and sew.
//!
//! A region no face could be built on - and one whose face took a wrong piece of its surface - is left as mesh, as the
//! plan has it: its triangles go in as flat faces, its edges run along their sides, and the body closes all the same.

use crate::{BoundedFace, EdgeCurve, FaceEdge, FaceSurface, Shape};
use qymcad_meshfit::{Boundaries, Curve, Prepared, Region, Surface, Tolerance, OPEN};

/// A body built from the faces found on a mesh.
pub struct Recognised {
    pub shape: Shape,
    /// For every region, the area its face took of its surface; `None` where the region was left as mesh.
    pub areas: Vec<Option<f64>>,
    /// How many sides no second face met in the sewing: none means the shell closed.
    pub free_edges: usize,
    /// Where those sides are, up to 256 of them: the middle of each.
    pub free_at: Vec<[f64; 3]>,
    /// How many triangles of the regions left as mesh made no face: each is a hole of its own size.
    pub loose_dropped: usize,
}

/// The body the faces of `found` make; `None` where the kernel built nothing, its words in
/// `qymcad_kernel::last_kernel_refusal`.
///
/// Built twice where it has to be: the first body says how much of its surface every face took, and a face that took a
/// piece outside half to twice its region's triangles - or none at all - is taken back, its region left as mesh, and
/// the body built again. Measured on `cube_gears`: such faces are crumbs of one or two triangles fitted by spheres of
/// 4 676 mm on a part of 30, and they took pieces 10^5 to 10^8 times their own.
pub fn recognise(p: &Prepared, found: &[Region], b: &Boundaries, curves: &[Curve], tol: &Tolerance) -> Option<Recognised> {
    recognise_until(p, found, b, curves, tol, &|| false)
}

/// The same, asking `stop` before every round of building: `None` once it says yes - a long recognition is broken off
/// where it stands rather than built to the end for nobody.
pub fn recognise_until(p: &Prepared, found: &[Region], b: &Boundaries, curves: &[Curve], tol: &Tolerance, stop: &dyn Fn() -> bool) -> Option<Recognised> {
    // THE FACES ARE KEPT FOR THE ROUNDS, and let go however the recognition ends
    struct Kept;
    impl Drop for Kept {
        fn drop(&mut self) {
            Shape::keep_faces(false);
        }
    }
    Shape::keep_faces(true);
    let _kept = Kept;
    let mut kept = found.to_vec();
    // the rounds ask only the area each face took: the faces are built, not sewn, and the body is made once after them
    let mut traced = curves.to_vec();
    let mut areas = describe(p, &kept, b, &traced).areas(tol)?;
    // TAKEN BACK UNTIL NOTHING IS LEFT WRONG. A region left as mesh changes the edges of its neighbours, and a
    // neighbour's face can fail in its turn; with one round only, those failures stayed as holes - measured on
    // `cube_gears`, 5 to 40 sides of every gear met no second face and the body came back a shell. On a smooth handle the
    // fifth round was the first to find nothing wrong; after three, a face that failed in the last one stood unbuilt, a
    // hole, and the whole body was built a second time around it.
    let sag = std::cell::OnceCell::new();
    for _ in 0..6 {
        let crumbs: Vec<usize> = (0..kept.len())
            .filter(|&r| kept[r].surface.is_some())
            .filter(|&r| match areas[r] {
                None => true,
                Some(took) => !(0.9..=1.1).contains(&(took / area(p, &kept[r]).max(1e-300))),
            })
            .collect();
        if crumbs.is_empty() {
            break;
        }
        // A PRIMITIVE THAT MADE NO FACE GETS A FREE FORM over its own triangles before it is left as mesh: measured on
        // a smooth handle, 176 regions (107 strips of cylinders) failed on their primitives and went back as 13 000
        // loose triangles; a region that fails on a free form too is mesh.
        for &r in &crumbs {
            kept[r].surface = match kept[r].surface {
                Some(Surface::Spline(_)) | None => None,
                Some(_) => qymcad_meshfit::free_form_of(p, &kept[r].tris, tol, *sag.get_or_init(|| qymcad_meshfit::chord_deflection(p, tol.sharp_deg))),
            };
        }
        if stop() {
            return None;
        }
        traced = qymcad_meshfit::curves(b, &kept, tol);
        areas = describe(p, &kept, b, &traced).areas(tol)?;
    }
    if stop() {
        return None;
    }
    let mut made = build(p, &kept, b, &traced, tol)?;
    // AND WHERE THE SHELL STILL DID NOT CLOSE, THE REGIONS AT ITS UNMET SIDES GO BACK TO THE MESH, round by round while
    // fewer sides stay unmet; the best body is kept. Faces on regions that fit their surfaces only in patches meet one
    // another badly: measured on a cylinder of 10 by 20 with its corners moved by up to 0.005 mm, 140 sides stayed
    // unmet and the body was a shell, and on such a sphere moved by 0.02 mm, 46. A region as mesh closes by
    // construction - its triangles are its faces.
    for _ in 0..8 {
        if made.free_edges == 0 {
            break;
        }
        let mut at_vertex: std::collections::HashMap<u32, Vec<usize>> = std::collections::HashMap::new();
        for (r, region) in kept.iter().enumerate().filter(|(_, region)| region.surface.is_some()) {
            for &t in &region.tris {
                for &v in &p.mesh.tris[t as usize] {
                    at_vertex.entry(v).or_default().push(r);
                }
            }
        }
        let mut back: Vec<usize> = made
            .free_at
            .iter()
            .filter_map(|q| {
                let nearest = p.mesh.verts.iter().enumerate().min_by(|a, b| dist2(a.1, q).total_cmp(&dist2(b.1, q))).map(|(i, _)| i as u32)?;
                at_vertex.get(&nearest).cloned()
            })
            .flatten()
            .collect();
        back.sort_unstable();
        back.dedup();
        if back.is_empty() {
            break;
        }
        let mut tried = kept.clone();
        for &r in &back {
            tried[r].surface = None;
        }
        if stop() {
            return None;
        }
        let traced = qymcad_meshfit::curves(b, &tried, tol);
        let Some(next) = build(p, &tried, b, &traced, tol) else { break };
        if next.free_edges >= made.free_edges {
            break;
        }
        kept = tried;
        made = next;
    }
    Some(made)
}

/// The squared distance from a mesh corner to a point.
fn dist2(v: &qymcad_core::geom::Point3, q: &[f64; 3]) -> f64 {
    (v.x - q[0]).powi(2) + (v.y - q[1]).powi(2) + (v.z - q[2]).powi(2)
}

/// What the kernel is handed for a body: the corners, the edges, a face on every region with a surface and the regions
/// without one as their triangles; `owners[k]` is the region of face `k`.
struct Described {
    corners: Vec<[f64; 3]>,
    edges: Vec<FaceEdge>,
    faces: Vec<BoundedFace>,
    loose: Vec<[[f64; 3]; 3]>,
    owners: Vec<usize>,
    regions: usize,
}

impl Described {
    /// The area each region's face took, by region; `None` where the region is left as mesh or its face was not built.
    fn areas(&self, tol: &Tolerance) -> Option<Vec<Option<f64>>> {
        let took = Shape::face_areas(&self.corners, &self.edges, &self.faces, tol.distance)?;
        let mut by_region = vec![None; self.regions];
        for (k, a) in took.into_iter().enumerate() {
            by_region[self.owners[k]] = a;
        }
        Some(by_region)
    }
}

/// One pass: every region with a surface a face, every region without one a handful of flat triangles.
fn build(p: &Prepared, found: &[Region], b: &Boundaries, curves: &[Curve], tol: &Tolerance) -> Option<Recognised> {
    let d = describe(p, found, b, curves);
    let made = Shape::from_faces(&d.corners, &d.edges, &d.faces, &d.loose, tol.distance)?;
    let mut by_region = vec![None; found.len()];
    for (k, took) in made.areas.into_iter().enumerate() {
        by_region[d.owners[k]] = took;
    }
    Some(Recognised { shape: made.shape, areas: by_region, free_edges: made.free_edges, free_at: made.free_at, loose_dropped: made.loose_dropped })
}

/// The description of the body `found` makes, for `build` or for the areas alone.
fn describe(p: &Prepared, found: &[Region], b: &Boundaries, curves: &[Curve]) -> Described {
    let as_mesh = |f: u32| f == OPEN || found[f as usize].surface.is_none();
    let corner = |v: usize| {
        let q = p.mesh.verts[b.corners[v] as usize];
        [q.x, q.y, q.z]
    };
    // BESIDE A REGION LEFT AS MESH THE EDGE IS CUT INTO THE MESH'S OWN SIDES, and it ends in the mesh's own corners.
    //
    // Such a region goes in as its triangles, so on its side of the seam stand as many short sides as the mesh has,
    // while the face beside it would bring one long edge through the same corners: the geometry agrees, the topology
    // does not, and the sewing leaves both free. Measured on `cube_gears`: every side left free lay on such an edge -
    // gear 1 had 36 of them, each between a face and a region left as mesh, each an edge of seven points.
    //
    // The corners are the mesh's own, not the ones placed on the surfaces: a crumb's surface is a nonsense - a sphere
    // of thousands of millimetres on a part of thirty - and the corner placed on it stands as far as 0.4 mm from the
    // mesh corner its triangles keep.
    let mut corners: Vec<[f64; 3]> = b.vertices.clone();
    let mut edges: Vec<FaceEdge> = Vec::new();
    let mut pieces: Vec<Vec<usize>> = Vec::with_capacity(b.edges.len());
    for (e, c) in b.edges.iter().zip(curves) {
        let beside = e.faces.iter().any(|&f| as_mesh(f));
        let mut points = match c {
            Curve::Points(pts) => pts.clone(),
            _ => e.points.clone(),
        };
        if let (true, Some([first, last])) = (beside, e.ends) {
            if let Some(at) = points.first_mut() {
                *at = corner(first);
            }
            if let Some(at) = points.last_mut() {
                *at = corner(last);
            }
        }
        if beside && points.len() > 2 {
            let at: Vec<usize> = points
                .iter()
                .enumerate()
                .map(|(k, q)| match (k, k + 1 == points.len(), e.ends) {
                    (0, _, Some([first, _])) => first,
                    (_, true, Some([_, last])) => last,
                    _ => {
                        corners.push(*q);
                        corners.len() - 1
                    }
                })
                .collect();
            let (n, closed) = (points.len(), e.ends.is_none());
            let sides = if closed { n } else { n - 1 };
            pieces.push((0..sides).map(|k| edges.len() + k).collect());
            for k in 0..sides {
                let j = (k + 1) % n;
                edges.push(FaceEdge { curve: EdgeCurve::Polyline, ends: Some([at[k], at[j]]), points: vec![points[k], points[j]] });
            }
            continue;
        }
        pieces.push(vec![edges.len()]);
        edges.push(FaceEdge {
            curve: if beside {
                EdgeCurve::Polyline
            } else {
                match *c {
                    Curve::Line { point, dir } => EdgeCurve::Line { point, dir },
                    Curve::Circle { center, axis, radius } => EdgeCurve::Circle { center, axis, radius },
                    Curve::Ellipse { center, normal, major_dir, major, minor } => EdgeCurve::Ellipse { center, normal, major_dir, major, minor },
                    // AN EDGE OF A HELIX HOLDS CLOSE TO ITS POINTS: a thread's root is faces 0.11 mm wide along helices of
                    // 105 mm, and a curve let off by four tolerances bowed away between its points - those faces took 89
                    // to 105 % of their area and the narrow ones went back to the mesh; within one, 99.8 to 100.6 %. Not
                    // every edge: on a mesh of quads the tighter curves took its body from 5.0 s to 24.7 s.
                    // AN EDGE OF A FITTED WALL RUNS THROUGH ITS POINTS AS THEY ARE: a region's border follows the sides of
                    // its triangles in a zigzag, and a smooth curve held within four tolerances of it swung off the wall -
                    // measured on a smooth handle, 80 edges of walls ran up to 45 times the wall's own width away from it,
                    // and their faces took 23 times their area. The points lie on both faces the edge parts.
                    Curve::Points(_) if e.faces.iter().any(|&f| f != OPEN && matches!(found[f as usize].surface, Some(Surface::Spline(_)))) => EdgeCurve::Polyline,
                    Curve::Points(_) => EdgeCurve::Points { close: e.faces.iter().any(|&f| f != OPEN && matches!(found[f as usize].surface, Some(Surface::Helix { .. }))) },
                }
            },
            ends: e.ends,
            points,
        });
    }
    let mut owners = Vec::new();
    let faces: Vec<BoundedFace> = found
        .iter()
        .zip(&b.loops)
        .enumerate()
        .filter_map(|(i, (r, loops))| {
            let s = r.surface.as_ref()?;
            owners.push(i);
            let surface = match *s {
                Surface::Plane { point, normal } => FaceSurface::Plane { point, normal },
                Surface::Cylinder { point, axis, radius } => FaceSurface::Cylinder { point, axis, radius },
                Surface::Cone { apex, axis, half_angle } => FaceSurface::Cone { apex, axis, half_angle },
                Surface::Sphere { center, radius } => FaceSurface::Sphere { center, radius },
                Surface::Torus { center, axis, major, minor } => FaceSurface::Torus { center, axis, major, minor },
                Surface::Free { normal } => FaceSurface::Free { normal },
                Surface::Spline(ref b) => FaceSurface::Spline { nu: b.nu, nv: b.nv, frame: b.frame },
                Surface::Helix { point, axis, reference, rise, radial, axial, offset } => {
                    // the angles and distances the region's corners span: the face is trimmed within them
                    let (mut turn, mut reach) = ([f64::MAX, f64::MIN], [f64::MAX, f64::MIN]);
                    for &t in &r.tris {
                        for q in p.mesh.triangle(t as usize) {
                            if let Some((angle, far)) = s.turn_at([q.x, q.y, q.z]) {
                                turn = [turn[0].min(angle), turn[1].max(angle)];
                                reach = [reach[0].min(far), reach[1].max(far)];
                            }
                        }
                    }
                    FaceSurface::Helix { point, axis, reference, rise, radial, axial, offset, turn, reach }
                }
                Surface::RoundHelix { point, axis, reference, rise, middle, height, round } => {
                    // the angles along and round the circle the region's corners span, round the circle taken about their
                    // mean way so that a piece across the angle's jump stays one piece
                    let seen: Vec<(f64, f64)> = r.tris.iter().flat_map(|&t| p.mesh.triangle(t as usize)).filter_map(|q| s.turn_at([q.x, q.y, q.z])).collect();
                    let (sin, cos) = seen.iter().fold((0.0, 0.0), |(s, c), &(_, phi)| (s + phi.sin(), c + phi.cos()));
                    let mean = sin.atan2(cos);
                    let (mut turn, mut reach) = ([f64::MAX, f64::MIN], [f64::MAX, f64::MIN]);
                    for (angle, phi) in seen {
                        let phi = mean + (phi - mean + std::f64::consts::PI).rem_euclid(2.0 * std::f64::consts::PI) - std::f64::consts::PI;
                        turn = [turn[0].min(angle), turn[1].max(angle)];
                        reach = [reach[0].min(phi), reach[1].max(phi)];
                    }
                    FaceSurface::RoundHelix { point, axis, reference, rise, middle, height, round, turn, reach }
                }
                Surface::Coil { point, axis, reference, rise, radius, lift, wire } => {
                    // the angles the region's corners span along the wire: the face is trimmed within them
                    let mut turn = [f64::MAX, f64::MIN];
                    for &t in &r.tris {
                        for q in p.mesh.triangle(t as usize) {
                            if let Some((angle, _)) = s.turn_at([q.x, q.y, q.z]) {
                                turn = [turn[0].min(angle), turn[1].max(angle)];
                            }
                        }
                    }
                    FaceSurface::Coil { point, axis, reference, rise, radius, lift, wire, turn }
                }
            };
            let walks = loops
                .iter()
                .map(|l| {
                    l.iter()
                        .flat_map(|&(e, forward)| {
                            let mut made: Vec<(usize, bool)> = pieces[e].iter().map(|&piece| (piece, forward)).collect();
                            if !forward {
                                made.reverse();
                            }
                            made
                        })
                        .collect()
                })
                .collect();
            // a free form runs through its triangles' middles, up to 400 of them evenly picked
            let points = if let Surface::Spline(b) = s {
                b.poles.clone() // a fitted wall hands its poles, row by row
            } else if matches!(s, Surface::Free { .. }) {
                let step = r.tris.len().div_ceil(400).max(1);
                r.tris
                    .iter()
                    .step_by(step)
                    .map(|&t| {
                        let [a, b, c] = p.mesh.triangle(t as usize);
                        [(a.x + b.x + c.x) / 3.0, (a.y + b.y + c.y) / 3.0, (a.z + b.z + c.z) / 3.0]
                    })
                    .collect()
            } else {
                Vec::new()
            };
            // a fitted wall faces the region's mean normal by construction: its u and v run so that u x v points out
            let outward = matches!(s, Surface::Free { .. } | Surface::Spline(_)) || looks_out(p, r, s);
            Some(BoundedFace { surface, outward, inside: amid(p, r), loops: walks, points })
        })
        .collect();
    let loose: Vec<[[f64; 3]; 3]> = found.iter().filter(|r| r.surface.is_none()).flat_map(|r| r.tris.iter().map(|&t| p.mesh.triangle(t as usize).map(|q| [q.x, q.y, q.z]))).collect();
    Described { corners, edges, faces, loose, owners, regions: found.len() }
}

/// The area of the region's triangles.
fn area(p: &Prepared, r: &Region) -> f64 {
    r.tris
        .iter()
        .map(|&t| {
            let [a, b, c] = p.mesh.triangle(t as usize);
            let (u, v) = ([b.x - a.x, b.y - a.y, b.z - a.z], [c.x - a.x, c.y - a.y, c.z - a.z]);
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt() / 2.0
        })
        .sum()
}

/// A point of the region itself: the middle of its widest triangle. The face built on the region has to hold it, and the
/// middle of all its triangles weighted by area does not have to lie in the region at all - measured on the washer, it
/// stood in the hole of each flat ring; the ring that held its hole was refused, and the face kept was the disc with the
/// hole added to it, 392.7 mm^2 for a ring of 235.6.
fn amid(p: &Prepared, r: &Region) -> [f64; 3] {
    let widest = r.tris.iter().map(|&t| {
        let [a, b, c] = p.mesh.triangle(t as usize);
        let (u, v) = ([b.x - a.x, b.y - a.y, b.z - a.z], [c.x - a.x, c.y - a.y, c.z - a.z]);
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        (n[0] * n[0] + n[1] * n[1] + n[2] * n[2], [(a.x + b.x + c.x) / 3.0, (a.y + b.y + c.y) / 3.0, (a.z + b.z + c.z) / 3.0])
    });
    widest.max_by(|x, y| x.0.total_cmp(&y.0)).map_or([0.0; 3], |(_, mid)| mid)
}

/// Whether the surface's own normal - the plane's, away from the axis or the centre on the rest - points the way the
/// region's triangles face, out of the body: summed over the triangles, weighted by area.
fn looks_out(p: &Prepared, r: &Region, s: &Surface) -> bool {
    let sub = |a: [f64; 3], b: [f64; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let dot = |a: [f64; 3], b: [f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let along = |d: [f64; 3], axis: [f64; 3]| {
        let t = dot(d, axis);
        [d[0] - axis[0] * t, d[1] - axis[1] * t, d[2] - axis[2] * t]
    };
    let mut sum = 0.0;
    for &t in &r.tris {
        let [a, b, c] = p.mesh.triangle(t as usize);
        let g = [(a.x + b.x + c.x) / 3.0, (a.y + b.y + c.y) / 3.0, (a.z + b.z + c.z) / 3.0];
        let cross = {
            let (u, v) = ([b.x - a.x, b.y - a.y, b.z - a.z], [c.x - a.x, c.y - a.y, c.z - a.z]);
            [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]]
        };
        let area = dot(cross, cross).sqrt() / 2.0;
        let own = match *s {
            Surface::Plane { normal, .. } => normal,
            Surface::Cylinder { point, axis, .. } => along(sub(g, point), axis),
            Surface::Cone { apex, axis, half_angle } => {
                let radial = along(sub(g, apex), axis);
                let len = dot(radial, radial).sqrt().max(1e-300);
                let (c, s) = (half_angle.cos() / len, half_angle.sin());
                [radial[0] * c - axis[0] * s, radial[1] * c - axis[1] * s, radial[2] * c - axis[2] * s]
            }
            Surface::Sphere { center, .. } => sub(g, center),
            Surface::Torus { center, axis, major, .. } => {
                let radial = along(sub(g, center), axis);
                let len = dot(radial, radial).sqrt().max(1e-300);
                sub(g, [center[0] + radial[0] / len * major, center[1] + radial[1] / len * major, center[2] + radial[2] / len * major])
            }
            Surface::Free { normal } => normal,
            Surface::Spline(_) => p.normals[t as usize],
            // a helix's: the side (radial, axial) points to in the cut through the axis, turned by the rise; a round
            // helix's and a coil's: away from the circle's middle
            Surface::Helix { .. } | Surface::RoundHelix { .. } | Surface::Coil { .. } => s.normal_at(g).unwrap_or([0.0; 3]),
        };
        let len = dot(own, own).sqrt();
        if len > 1e-300 {
            sum += dot(own, p.normals[t as usize]) / len * area;
        }
    }
    sum > 0.0
}
