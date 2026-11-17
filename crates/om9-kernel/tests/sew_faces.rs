// SPDX-License-Identifier: LGPL-2.1-or-later
//! Bodies sewn from add-on faces (`editing::sew_faces`): six trimmed planes make a closed
//! box of the right volume, an open set stays a shell (and is refused when a solid is asked
//! for), and solids read face by face with `face_nurbs` sew back to the same volume — the
//! round trip an add-on makes when it reads a solid and writes it back.
//! Run: `cargo test -p om9-kernel --test sew_faces`.
use om9_core::{
    budget::Limits,
    geometry::{NurbsCurve, NurbsSurface, Point3, Transform},
    surface_kernel,
};
use om9_kernel::{
    brep::box_brep,
    creation::{self, Primitive, measure},
    editing::{self, BooleanOp, sew_faces},
    face_nurbs::{FaceNurbs, face_nurbs},
    transform_ops::transform_brep,
};

type Face = (NurbsSurface, Vec<Vec<NurbsCurve>>);

fn limits() -> Limits {
    Limits::default()
}

fn line(a: Point3, b: Point3) -> NurbsCurve {
    NurbsCurve {
        degree: 1,
        poles: vec![a, b],
        weights: vec![1.0; 2],
        knots: vec![0.0, 0.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    }
}

/// A w × h rectangle on the plane through `o` spanned by unit `x`, `y`, bounded by its four sides.
fn rect(o: Point3, x: Point3, y: Point3, w: f64, h: f64) -> Face {
    let s = surface_kernel::plane(o, x, y, w, h, &limits()).unwrap();
    let c = |a: f64, b: f64| [o[0] + x[0] * a + y[0] * b, o[1] + x[1] * a + y[1] * b, o[2] + x[2] * a + y[2] * b];
    let corners = [c(0.0, 0.0), c(w, 0.0), c(w, h), c(0.0, h)];
    let ring = (0..4).map(|i| line(corners[i], corners[(i + 1) % 4])).collect();
    (s, vec![ring])
}

fn box_faces() -> Vec<Face> {
    let (x, y, z) = ([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]);
    vec![
        rect([0.0, 0.0, 0.0], x, y, 10.0, 10.0),
        rect([0.0, 0.0, 10.0], x, y, 10.0, 10.0),
        rect([0.0, 0.0, 0.0], x, z, 10.0, 10.0),
        rect([0.0, 10.0, 0.0], x, z, 10.0, 10.0),
        rect([0.0, 0.0, 0.0], y, z, 10.0, 10.0),
        rect([10.0, 0.0, 0.0], y, z, 10.0, 10.0),
    ]
}

fn to_core(f: &FaceNurbs) -> Face {
    let ends = |k: &[f64], d: usize, n: usize| [k[d], k[n]];
    let surface = NurbsSurface {
        degree: f.degree,
        size: f.size,
        poles: f.poles.clone(),
        weights: f.weights.clone(),
        knots: f.knots.clone(),
        domain: [ends(&f.knots[0], f.degree[0], f.size[0]), ends(&f.knots[1], f.degree[1], f.size[1])],
        periodic: [false, false],
        reversed: false,
    };
    let loops = f
        .loops
        .iter()
        .map(|lp| {
            lp.iter()
                .map(|c| NurbsCurve {
                    degree: c.degree,
                    poles: c.poles.clone(),
                    weights: c.weights.clone(),
                    knots: c.knots.clone(),
                    domain: ends(&c.knots, c.degree, c.poles.len()),
                    periodic: false,
                    reversed: false,
                    seam: None,
                })
                .collect()
        })
        .collect();
    (surface, loops)
}

#[test]
fn six_trimmed_planes_close_into_the_box() {
    let body = sew_faces(&box_faces(), true, 0.01, 0.05, &limits()).unwrap();
    let m = measure(&body, &limits()).unwrap();
    assert_eq!((m.solids, m.faces, m.naked_edges), (1, 6, 0), "{m:?}");
    assert!(m.closed);
    assert!((m.volume_mm3 - 1000.0).abs() < 1e-6, "{}", m.volume_mm3);
}

#[test]
fn an_open_set_stays_a_shell_and_is_refused_as_a_solid() {
    let mut faces = box_faces();
    faces.pop();
    assert!(sew_faces(&faces, true, 0.01, 0.05, &limits()).is_err(), "five faces are no solid");
    let shell = sew_faces(&faces, false, 0.01, 0.05, &limits()).unwrap();
    let m = measure(&shell, &limits()).unwrap();
    assert_eq!((m.solids, m.faces, m.naked_edges), (0, 5, 4), "{m:?}");
    // One face is a face; nothing is refused.
    assert!(sew_faces(&faces[..1], false, 0.01, 0.05, &limits()).is_ok());
    assert!(sew_faces(&faces[..1], true, 0.01, 0.05, &limits()).is_err());
    assert!(sew_faces(&[], false, 0.01, 0.05, &limits()).is_err());
}

#[test]
fn solids_read_face_by_face_sew_back_to_their_volume() {
    let l = limits();
    let check = |body: &om9_core::brep::Brep, want: f64| {
        let faces: Vec<Face> = face_nurbs(body, &l).unwrap().iter().map(to_core).collect();
        let again = sew_faces(&faces, true, 0.01, 0.05, &l).unwrap();
        let m = measure(&again, &l).unwrap();
        assert_eq!(m.solids, 1, "{m:?}");
        assert!((m.volume_mm3 - want).abs() < want * 1e-5, "{} vs {want}", m.volume_mm3);
    };
    check(&box_brep([10.0, 10.0, 4.0], 0.05, &l).unwrap(), 400.0);
    let plate = box_brep([10.0, 10.0, 4.0], 0.05, &l).unwrap();
    let pin = creation::primitive(Primitive::Cylinder { radius_mm: 2.0, height_mm: 4.0 }, 0.05, &l).unwrap();
    let pin = transform_brep(&pin, &Transform([[1.0, 0.0, 0.0, 5.0], [0.0, 1.0, 0.0, 5.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]), 0.05, &l).unwrap();
    let drilled = editing::boolean(&plate, &pin, BooleanOp::Difference, 0.05, &l).unwrap();
    check(drilled.body().expect("a body"), 400.0 - std::f64::consts::PI * 4.0 * 4.0);
    // A closed (seamed) side: the cylinder r 3, h 5.
    let cyl = creation::primitive(Primitive::Cylinder { radius_mm: 3.0, height_mm: 5.0 }, 0.05, &l).unwrap();
    check(&cyl, std::f64::consts::PI * 9.0 * 5.0);
}

/// The rational circle of radius `r` round (0, 0, z): nine poles, degree 2.
fn circle(r: f64, z: f64) -> NurbsCurve {
    let h = std::f64::consts::FRAC_1_SQRT_2;
    let p = [[1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [-1.0, 1.0], [-1.0, 0.0], [-1.0, -1.0], [0.0, -1.0], [1.0, -1.0], [1.0, 0.0]];
    NurbsCurve {
        degree: 2,
        poles: p.iter().map(|q| [q[0] * r, q[1] * r, z]).collect(),
        weights: (0..9).map(|i| if i % 2 == 1 { h } else { 1.0 }).collect(),
        knots: vec![0.0, 0.0, 0.0, 0.25, 0.25, 0.5, 0.5, 0.75, 0.75, 1.0, 1.0, 1.0],
        domain: [0.0, 1.0],
        periodic: false,
        reversed: false,
        seam: None,
    }
}

/// What an add-on's ExtrudeCrv Solid=Yes sends: the circle swept up (degree 1 across, its
/// seam two coincident edges) and two planar caps bounded by the circle. They sew into the
/// cylinder r 2, h 3.
#[test]
fn an_extruded_circle_with_its_caps_closes_into_the_cylinder() {
    let (bottom, top) = (circle(2.0, 0.0), circle(2.0, 3.0));
    let mut poles = Vec::new();
    let mut weights = Vec::new();
    for (a, b) in bottom.poles.iter().zip(&top.poles) {
        poles.extend([*a, *b]);
    }
    for w in &bottom.weights {
        weights.extend([*w, *w]);
    }
    let side = NurbsSurface {
        degree: [2, 1],
        size: [9, 2],
        poles,
        weights,
        knots: [bottom.knots.clone(), vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    let cap = |c: NurbsCurve, z: f64| {
        let s = surface_kernel::plane([-3.0, -3.0, z], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 6.0, 6.0, &limits()).unwrap();
        (s, vec![vec![c]])
    };
    let faces = vec![(side, Vec::new()), cap(bottom, 0.0), cap(top, 3.0)];
    let body = sew_faces(&faces, true, 0.01, 0.05, &limits()).unwrap();
    let m = measure(&body, &limits()).unwrap();
    assert_eq!(m.solids, 1, "{m:?}");
    let want = std::f64::consts::PI * 4.0 * 3.0;
    assert!((m.volume_mm3 - want).abs() < want * 1e-6, "{} vs {want}", m.volume_mm3);
}

/// Many faces sew in one pass: a 60 × 60 plate with 6 × 6 holes read face by face (42 faces)
/// sews back to its volume quickly (folding pairwise took seconds and grew with the square of
/// the face count).
#[test]
fn many_faces_sew_back_in_one_pass() {
    let l = limits();
    let mut plate = box_brep([60.0, 60.0, 4.0], 0.05, &l).unwrap();
    for i in 0..6 {
        for j in 0..6 {
            let pin = creation::primitive(Primitive::Cylinder { radius_mm: 2.0, height_mm: 4.0 }, 0.05, &l).unwrap();
            let at = Transform([[1.0, 0.0, 0.0, 5.0 + 10.0 * i as f64], [0.0, 1.0, 0.0, 5.0 + 10.0 * j as f64], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]);
            let pin = transform_brep(&pin, &at, 0.05, &l).unwrap();
            plate = editing::boolean(&plate, &pin, BooleanOp::Difference, 0.05, &l).unwrap().body().expect("a body").clone();
        }
    }
    let faces: Vec<Face> = face_nurbs(&plate, &l).unwrap().iter().map(to_core).collect();
    assert_eq!(faces.len(), 42);
    let t = std::time::Instant::now();
    let body = sew_faces(&faces, true, 0.01, 0.05, &l).unwrap();
    let took = t.elapsed();
    let m = measure(&body, &l).unwrap();
    let want = 60.0 * 60.0 * 4.0 - 36.0 * std::f64::consts::PI * 4.0 * 4.0;
    assert_eq!((m.solids, m.faces), (1, 42), "{m:?}");
    assert!((m.volume_mm3 - want).abs() < want * 1e-6, "{} vs {want}", m.volume_mm3);
    eprintln!("sew_faces: 42 faces in {took:?}");
    assert!(took.as_secs_f64() < 10.0, "{took:?}");
}
