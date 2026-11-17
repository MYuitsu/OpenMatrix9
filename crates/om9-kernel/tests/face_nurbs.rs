// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exact NURBS of BRep faces (`face_nurbs`), checked against the shapes' analytic geometry:
//! every loop is a closed chain lying on its face's surface, planes and holes come out where
//! they are, and a periodic cylinder is opened into a clamped surface that stays on the
//! cylinder. Run: `cargo test -p om9-kernel --test face_nurbs`.
use om9_core::{budget::Limits, geometry::Transform};
use om9_kernel::{
    brep::box_brep,
    creation::{self, Primitive},
    editing::{self, BooleanOp},
    face_nurbs::{FaceNurbs, NurbsCurveData, edge_nurbs, face_nurbs, parse},
    transform_ops::transform_brep,
};

fn limits() -> Limits {
    Limits::default()
}

/// Rational de Boor on one parameter direction (clamped knots, normalised parameter t).
fn eval(degree: usize, knots: &[f64], poles: &[[f64; 4]], t: f64) -> [f64; 4] {
    let (a, b) = (knots[degree], knots[knots.len() - degree - 1]);
    let u = a + (b - a) * t.clamp(0.0, 1.0);
    let n = poles.len();
    let mut k = degree;
    while k + 1 < n && knots[k + 1] <= u {
        k += 1;
    }
    let mut d: Vec<[f64; 4]> = (0..=degree).map(|j| poles[j + k - degree]).collect();
    for r in 1..=degree {
        for j in (r..=degree).rev() {
            let i = j + k - degree;
            let den = knots[i + degree + 1 - r] - knots[i];
            let alpha = if den.abs() < 1e-300 { 0.0 } else { (u - knots[i]) / den };
            d[j] = std::array::from_fn(|c| (1.0 - alpha) * d[j - 1][c] + alpha * d[j][c]);
        }
    }
    d[degree]
}

fn homog(p: [f64; 3], w: f64) -> [f64; 4] {
    [p[0] * w, p[1] * w, p[2] * w, w]
}

fn curve_at(c: &NurbsCurveData, t: f64) -> [f64; 3] {
    let h: Vec<[f64; 4]> = c.poles.iter().zip(&c.weights).map(|(p, w)| homog(*p, *w)).collect();
    let r = eval(c.degree, &c.knots, &h, t);
    [r[0] / r[3], r[1] / r[3], r[2] / r[3]]
}

fn surface_at(f: &FaceNurbs, s: f64, t: f64) -> [f64; 3] {
    let [nu, nv] = f.size;
    let rows: Vec<[f64; 4]> = (0..nu)
        .map(|i| {
            let row: Vec<[f64; 4]> = (0..nv).map(|j| homog(f.poles[i * nv + j], f.weights[i * nv + j])).collect();
            eval(f.degree[1], &f.knots[1], &row, t)
        })
        .collect();
    let r = eval(f.degree[0], &f.knots[0], &rows, s);
    [r[0] / r[3], r[1] / r[3], r[2] / r[3]]
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Distance from `p` to the face's surface (a coarse grid, then a fine local search).
fn off_surface(f: &FaceNurbs, p: [f64; 3]) -> f64 {
    let n = 40;
    let mut best = (f64::MAX, 0.0, 0.0);
    for i in 0..=n {
        for j in 0..=n {
            let (s, t) = (i as f64 / n as f64, j as f64 / n as f64);
            let d = dist(surface_at(f, s, t), p);
            if d < best.0 {
                best = (d, s, t);
            }
        }
    }
    let mut h = 1.0 / n as f64;
    for _ in 0..200 {
        let (mut moved, (s0, t0)) = (false, (best.1, best.2));
        for (ds, dt) in [(h, 0.0), (-h, 0.0), (0.0, h), (0.0, -h)] {
            let (s, t) = ((s0 + ds).clamp(0.0, 1.0), (t0 + dt).clamp(0.0, 1.0));
            let d = dist(surface_at(f, s, t), p);
            if d < best.0 {
                best = (d, s, t);
                moved = true;
            }
        }
        if !moved {
            h *= 0.5;
            if h < 1e-15 {
                break;
            }
        }
    }
    best.0
}

/// End knots repeated degree + 1 times (periodic surfaces and closed edges are opened).
fn clamped(degree: usize, k: &[f64]) -> bool {
    k[..=degree].iter().all(|v| *v == k[0]) && k[k.len() - degree - 1..].iter().all(|v| *v == k[k.len() - 1])
}

/// Every surface and edge is clamped, and every loop is a closed chain of curves lying on the face.
fn check_loops(f: &FaceNurbs, tol: f64) {
    assert!(clamped(f.degree[0], &f.knots[0]) && clamped(f.degree[1], &f.knots[1]), "surface clamped");
    for lp in &f.loops {
        assert!(!lp.is_empty());
        for (i, c) in lp.iter().enumerate() {
            assert!(clamped(c.degree, &c.knots), "edge clamped");
            let next = &lp[(i + 1) % lp.len()];
            assert!(dist(curve_at(c, 1.0), curve_at(next, 0.0)) < tol, "loop chained");
            for k in 0..=4 {
                let p = curve_at(c, k as f64 / 4.0);
                assert!(off_surface(f, p) < tol, "edge point {p:?} on its surface");
            }
        }
    }
}

/// How a loop of a flat face winds round the surface's normal: + counter-clockwise.
fn winding(f: &FaceNurbs, lp: &[NurbsCurveData]) -> f64 {
    let o = surface_at(f, 0.0, 0.0);
    let (a, b) = (surface_at(f, 1.0, 0.0), surface_at(f, 0.0, 1.0));
    let (u, v) = ([a[0] - o[0], a[1] - o[1], a[2] - o[2]], [b[0] - o[0], b[1] - o[1], b[2] - o[2]]);
    let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
    let pts: Vec<[f64; 3]> = lp.iter().flat_map(|c| (0..8).map(move |k| curve_at(c, k as f64 / 8.0))).collect();
    let mut s = [0.0; 3];
    for i in 0..pts.len() {
        let (a, b) = (pts[i], pts[(i + 1) % pts.len()]);
        s = [s[0] + a[1] * b[2] - a[2] * b[1], s[1] + a[2] * b[0] - a[0] * b[2], s[2] + a[0] * b[1] - a[1] * b[0]];
    }
    s[0] * n[0] + s[1] * n[1] + s[2] * n[2]
}

#[test]
fn box_faces_are_its_six_planes_whole() {
    let b = box_brep([10.0, 10.0, 4.0], 0.05, &limits()).unwrap();
    let faces = face_nurbs(&b, &limits()).unwrap();
    assert_eq!(faces.len(), 6);
    // Each side is its whole plane: no loops, and the planes' areas add up to 2·(100 + 40 + 40).
    let mut area = 0.0;
    for f in &faces {
        assert!(f.loops.is_empty(), "a box side is its whole surface");
        check_loops(f, 1e-9);
        let (o, a, b) = (surface_at(f, 0.0, 0.0), surface_at(f, 1.0, 0.0), surface_at(f, 0.0, 1.0));
        let (u, v) = ([a[0] - o[0], a[1] - o[1], a[2] - o[2]], [b[0] - o[0], b[1] - o[1], b[2] - o[2]]);
        let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
        area += (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
    }
    assert!((area - 360.0).abs() < 1e-9, "{area}");
    // Faces whose normal is the surface's flipped are marked (a box has some either way).
    assert!(faces.iter().any(|f| f.reversed) && faces.iter().any(|f| !f.reversed));
}

#[test]
fn a_drilled_face_has_its_hole_as_a_second_loop_on_the_circle() {
    let plate = box_brep([10.0, 10.0, 4.0], 0.05, &limits()).unwrap();
    let pin = creation::primitive(Primitive::Cylinder { radius_mm: 2.0, height_mm: 4.0 }, 0.05, &limits()).unwrap();
    let pin = transform_brep(&pin, &Transform([[1.0, 0.0, 0.0, 5.0], [0.0, 1.0, 0.0, 5.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]), 0.05, &limits()).unwrap();
    let drilled = editing::boolean(&plate, &pin, BooleanOp::Difference, 0.05, &limits()).unwrap();
    let drilled = drilled.body().expect("a body").clone();
    let faces = face_nurbs(&drilled, &limits()).unwrap();
    for f in &faces {
        check_loops(f, 1e-7);
    }
    // The top and bottom faces: two loops, the second on the circle r 2 round (5, 5).
    let holed: Vec<&FaceNurbs> = faces.iter().filter(|f| f.loops.len() == 2).collect();
    assert_eq!(holed.len(), 2, "top and bottom");
    for f in holed {
        for c in &f.loops[1] {
            for k in 0..=8 {
                let p = curve_at(c, k as f64 / 8.0);
                assert!(((p[0] - 5.0).hypot(p[1] - 5.0) - 2.0).abs() < 1e-7, "{p:?}");
            }
        }
        assert!(winding(f, &f.loops[0]) > 0.0 && winding(f, &f.loops[1]) < 0.0, "outer anticlockwise, hole clockwise");
        // The outer loop is the square.
        let corners: Vec<[f64; 3]> = f.loops[0].iter().map(|c| curve_at(c, 0.0)).collect();
        assert!(corners.iter().all(|p| (p[0].abs() < 1e-9 || (p[0] - 10.0).abs() < 1e-9) && (p[1].abs() < 1e-9 || (p[1] - 10.0).abs() < 1e-9)), "{corners:?}");
    }
}

#[test]
fn a_cylinder_side_opens_into_a_clamped_surface_on_the_cylinder() {
    let cyl = creation::primitive(Primitive::Cylinder { radius_mm: 3.0, height_mm: 5.0 }, 0.05, &limits()).unwrap();
    let faces = face_nurbs(&cyl, &limits()).unwrap();
    let side: Vec<&FaceNurbs> = faces
        .iter()
        .filter(|f| (0..=6).any(|k| (surface_at(f, k as f64 / 6.0, 0.5)[2] - surface_at(f, k as f64 / 6.0, 0.0)[2]).abs() > 1e-6))
        .collect();
    assert_eq!(side.len(), 1, "one curved side");
    let f = side[0];
    assert!(f.loops.is_empty(), "the side is its whole surface (its seam is the surface's)");
    for i in 0..=24 {
        for j in 0..=4 {
            let p = surface_at(f, i as f64 / 24.0, j as f64 / 4.0);
            assert!((p[0].hypot(p[1]) - 3.0).abs() < 1e-9 && p[2] > -1e-9 && p[2] < 5.0 + 1e-9, "{p:?}");
        }
    }
    check_loops(f, 1e-7);
}

#[test]
fn a_sphere_and_a_torus_open_both_ways_and_stay_on_their_shapes() {
    let ball = creation::primitive(Primitive::Sphere { radius_mm: 4.0 }, 0.05, &limits()).unwrap();
    for f in &face_nurbs(&ball, &limits()).unwrap() {
        for i in 0..=12 {
            for j in 0..=12 {
                let p = surface_at(f, i as f64 / 12.0, j as f64 / 12.0);
                assert!((dist(p, [0.0; 3]) - 4.0).abs() < 1e-9, "{p:?}");
            }
        }
        check_loops(f, 1e-7);
    }
    // Ring 10, tube 2 round the z axis: periodic both ways, a single face.
    let ring = creation::primitive(Primitive::Torus { ring_radius_mm: 10.0, tube_radius_mm: 2.0 }, 0.05, &limits()).unwrap();
    let faces = face_nurbs(&ring, &limits()).unwrap();
    assert_eq!(faces.len(), 1);
    for i in 0..=16 {
        for j in 0..=16 {
            let p = surface_at(&faces[0], i as f64 / 16.0, j as f64 / 16.0);
            assert!(((p[0].hypot(p[1]) - 10.0).hypot(p[2]) - 2.0).abs() < 1e-9, "{p:?}");
        }
    }
    check_loops(&faces[0], 1e-7);
    // The ring with a quarter cut away (x > 0, z < 0): the trimmed tube's loop runs along the
    // seam and round both cut circles; its curves meet at the shape's vertices.
    let block = transform_brep(
        &box_brep([20.0, 40.0, 5.0], 0.05, &limits()).unwrap(),
        &Transform([[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, -20.0], [0.0, 0.0, 1.0, -5.0], [0.0, 0.0, 0.0, 1.0]]),
        0.05,
        &limits(),
    )
    .unwrap();
    let cut = editing::boolean(&ring, &block, BooleanOp::Difference, 0.05, &limits()).unwrap();
    let faces = face_nurbs(cut.body().expect("a body"), &limits()).unwrap();
    assert!(faces.iter().any(|f| f.loops.iter().any(|l| l.len() > 4)), "a long loop round the cut");
    // (The cut circles are the boolean's intersection curves: on the faces to its tolerance.)
    for f in &faces {
        check_loops(f, 1e-6);
    }
}

/// Faces and edges come in the display's order: every display triangle lies on the surface of
/// the face its index names, and every display segment on the curve of its edge — so a face
/// or edge picked by its display key finds its exact geometry (subobject picks).
#[test]
fn faces_and_edges_come_in_the_display_order() {
    let l = limits();
    let plate = box_brep([10.0, 10.0, 4.0], 0.05, &l).unwrap();
    let pin = creation::primitive(Primitive::Cylinder { radius_mm: 2.0, height_mm: 4.0 }, 0.05, &l).unwrap();
    let pin = transform_brep(&pin, &Transform([[1.0, 0.0, 0.0, 5.0], [0.0, 1.0, 0.0, 5.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]), 0.05, &l).unwrap();
    let drilled = editing::boolean(&plate, &pin, BooleanOp::Difference, 0.05, &l).unwrap();
    let ball = creation::primitive(Primitive::Sphere { radius_mm: 4.0 }, 0.05, &l).unwrap();
    for body in [drilled.body().expect("a body").clone(), ball] {
        let faces = face_nurbs(&body, &l).unwrap();
        let edges = edge_nurbs(&body, &l).unwrap();
        let d = &body.display;
        assert_eq!((faces.len(), edges.len()), (d.faces.len(), d.edges.len()));
        for t in d.triangles.iter().step_by(7) {
            let f = &faces[t[3] as usize];
            for k in 0..3 {
                let p = d.points[t[k] as usize];
                assert!(off_surface(f, p) < 1e-6, "triangle corner {p:?} on face {}", t[3]);
            }
        }
        for s in &d.segments {
            let c = edges[s[2] as usize].as_ref().expect("a drawn edge has a curve");
            for k in 0..2 {
                let p = d.points[s[k] as usize];
                let near = (0..=400).map(|i| dist(curve_at(c, i as f64 / 400.0), p)).fold(f64::MAX, f64::min);
                assert!(near < 0.05, "segment end {p:?} on edge {} ({near})", s[2]);
            }
        }
    }
    // A sphere's poles are degenerate edges: no curve.
    let ball = creation::primitive(Primitive::Sphere { radius_mm: 4.0 }, 0.05, &l).unwrap();
    let edges = edge_nurbs(&ball, &l).unwrap();
    assert_eq!(edges.iter().filter(|e| e.is_none()).count(), 2);
    // The box's twelve edges: four of each length, every curve a straight line between corners.
    let edges = edge_nurbs(&box_brep([10.0, 6.0, 4.0], 0.05, &l).unwrap(), &l).unwrap();
    let mut lengths: Vec<f64> = edges.iter().map(|e| { let c = e.as_ref().unwrap(); dist(curve_at(c, 0.0), curve_at(c, 1.0)) }).collect();
    lengths.sort_by(f64::total_cmp);
    assert_eq!(lengths.len(), 12);
    for (i, want) in [4.0, 6.0, 10.0].iter().enumerate() {
        assert!(lengths[i * 4..i * 4 + 4].iter().all(|v| (v - want).abs() < 1e-12), "{lengths:?}");
    }
}

#[test]
fn malformed_text_is_refused() {
    let l = limits();
    assert!(parse("", &l).is_err());
    assert!(parse("F 1\nS 1 1 2 2 0\n0 0 1 1\n0 0 1 1\n", &l).is_err());
    assert!(parse("F 1\nS 0 1 2 2 0\n", &l).is_err());
    assert!(parse("X 1", &l).is_err());
}
