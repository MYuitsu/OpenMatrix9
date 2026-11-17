// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::Plane;
use serde_json::Value;
use std::f64::consts::PI;

fn event(command: &str, input: Input, selection: Vec<Selected>, settings: Value) -> Event {
    Event {
        command: command.into(),
        input,
        state: Value::Null,
        plane: Plane::default(),
        selection,
        pick: None,
        settings: serde_json::from_value(settings).unwrap(),
    }
}

fn run(command: &str, selection: Vec<Selected>, inputs: Vec<Input>) -> Reply {
    let mut e = event(command, Input::Start, selection, json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    r
}

fn pt(p: Vec3) -> Input {
    Input::Point { point: p }
}
fn num(v: f64) -> Input {
    Input::Text { text: fmt(v) }
}

fn solid(id: &str, m: Mesh) -> Selected {
    Selected {
        id: id.into(),
        kind: "mesh".into(),
        name: id.into(),
        bbox: m.bbox(),
        mesh: Some(SelectedMesh {
            vertices: m.vertices,
            triangles: m.triangles,
        }),
        ..Default::default()
    }
}

fn closed_curve(id: &str, pts: Vec<Vec3>) -> Selected {
    Selected {
        id: id.into(),
        kind: "curve".into(),
        name: id.into(),
        bbox: [[0.0; 3], [1.0; 3]],
        points: pts,
        closed: true,
        ..Default::default()
    }
}

fn square(c: Vec3, half: f64) -> Vec<Vec3> {
    vec![
        add(c, [-half, -half, 0.0]),
        add(c, [half, -half, 0.0]),
        add(c, [half, half, 0.0]),
        add(c, [-half, half, 0.0]),
    ]
}

fn created(r: &Reply) -> Mesh {
    match &r.create[0] {
        Geo::Mesh {
            vertices,
            triangles,
            ..
        } => Mesh {
            vertices: vertices.clone(),
            triangles: triangles.clone(),
        },
        g => panic!("{g:?}"),
    }
}

fn replaced(r: &Reply, k: usize) -> Mesh {
    match &r.replace[k].geo {
        Geo::Mesh {
            vertices,
            triangles,
            ..
        } => Mesh {
            vertices: vertices.clone(),
            triangles: triangles.clone(),
        },
        g => panic!("{g:?}"),
    }
}

#[test]
fn primitives_have_their_volume() {
    let r = run(
        BOX,
        vec![],
        vec![pt([0.0; 3]), pt([2.0, 3.0, 0.0]), num(4.0)],
    );
    assert!((created(&r).volume() - 24.0).abs() < 1e-9);
    let r = run(SPHERE, vec![], vec![pt([0.0; 3]), num(2.0)]);
    assert!((created(&r).volume() / (4.0 / 3.0 * PI * 8.0) - 1.0).abs() < 0.01);
    let r = run(CYLINDER, vec![], vec![pt([0.0; 3]), num(1.0), num(3.0)]);
    assert!((created(&r).volume() / (PI * 3.0) - 1.0).abs() < 0.01);
    let r = run(
        TUBE,
        vec![],
        vec![pt([0.0; 3]), num(2.0), num(1.0), num(1.0)],
    );
    assert!((created(&r).volume() / (PI * 3.0) - 1.0).abs() < 0.01);
    let r = run(CONE, vec![], vec![pt([0.0; 3]), num(1.0), num(3.0)]);
    assert!((created(&r).volume() / PI - 1.0).abs() < 0.01);
    let r = run(
        TRUNCATED_CONE,
        vec![],
        vec![pt([0.0; 3]), num(2.0), num(1.0), num(3.0)],
    );
    assert!((created(&r).volume() / (PI * 3.0 / 3.0 * 7.0) - 1.0).abs() < 0.01);
    let r = run(PYRAMID, vec![], vec![pt([0.0; 3]), num(1.0), num(3.0)]);
    assert!(
        (created(&r).volume() - 2.0).abs() < 1e-9,
        "square pyramid of diagonal 2: {}",
        created(&r).volume()
    );
    let r = run(TORUS, vec![], vec![pt([0.0; 3]), num(8.0), num(1.0)]);
    assert!((created(&r).volume() / (2.0 * PI * PI * 8.0) - 1.0).abs() < 0.01);
    let r = run(
        ELLIPSOID,
        vec![],
        vec![pt([0.0; 3]), num(3.0), num(2.0), num(1.0)],
    );
    assert!((created(&r).volume() / (4.0 / 3.0 * PI * 6.0) - 1.0).abs() < 0.01);
    for (m, _) in [(created(&r), 0)] {
        assert!(m.is_closed());
    }
}

#[test]
fn booleans() {
    let a = solid("a", mesh::cuboid([0.0; 3], [2.0, 2.0, 2.0]));
    let b = solid("b", mesh::cuboid([1.0, 1.0, 1.0], [3.0, 3.0, 3.0]));
    let r = Solids::step(&event(
        UNION,
        Input::Enter,
        vec![a.clone(), b.clone()],
        json!({}),
    ))
    .unwrap();
    assert!((replaced(&r, 0).volume() - 15.0).abs() < 1e-6);
    assert_eq!(r.delete, vec!["b".to_string()]);
    // Difference: first set, then the cutters.
    let mut e = event(DIFFERENCE, Input::Enter, vec![a.clone()], json!({}));
    let r1 = Solids::step(&e).unwrap();
    assert!(r1.prompt.contains("now select"), "{}", r1.prompt);
    e.state = r1.state;
    e.selection = vec![b.clone()];
    let r = Solids::step(&e).unwrap();
    assert!((replaced(&r, 0).volume() - 7.0).abs() < 1e-6);
    assert_eq!(r.replace[0].id, "a");
    assert_eq!(r.delete, vec!["b".to_string()]);
    let r = run(
        BOOLEAN_TWO,
        vec![a, b],
        vec![
            Input::Selection,
            Input::Option {
                name: "Mode".into(),
                value: Some("Intersection".into()),
            },
            Input::Enter,
        ],
    );
    assert!((replaced(&r, 0).volume() - 1.0).abs() < 1e-6);
}

#[test]
fn extrusions_with_holes_and_surfaces() {
    let outer = closed_curve("o", square([0.0; 3], 2.0));
    let inner = closed_curve("i", square([0.0; 3], 1.0));
    let open = Selected {
        id: "l".into(),
        kind: "curve".into(),
        points: vec![[5.0, 0.0, 0.0], [8.0, 0.0, 0.0]],
        exact: Some(Geo::Line {
            start: [5.0, 0.0, 0.0],
            end: [8.0, 0.0, 0.0],
        }),
        ..Default::default()
    };
    let r = run(
        EXTRUDE_ALL,
        vec![outer, inner, open],
        vec![Input::Selection, num(3.0)],
    );
    let m = created(&r);
    assert!(m.is_closed());
    assert!(
        (m.volume() - (16.0 - 4.0) * 3.0).abs() < 1e-9,
        "{}",
        m.volume()
    );
    assert!(
        r.create
            .iter()
            .any(|g| matches!(g, Geo::Surface { size: [_, 2], .. })),
        "the open curve becomes a surface"
    );
}

#[test]
fn holes_shell_fillet_and_faces() {
    let plate = solid("p", mesh::cuboid([-5.0, -5.0, 0.0], [5.0, 5.0, 1.0]));
    let hole = closed_curve("h", square([0.0, 0.0, 1.0], 0.5));
    // Rhino 5 MakeHole: the curves and the solid, Enter at the depth prompt: through.
    let r = run(MAKE_HOLE, vec![plate.clone(), hole.clone()], vec![Input::Selection, Input::Enter]);
    assert!(
        (replaced(&r, 0).volume() - 99.0).abs() < 1e-6,
        "{}",
        replaced(&r, 0).volume()
    );
    let r = run(
        ARRAY_HOLE,
        vec![plate.clone(), hole.clone()],
        vec![Input::Selection, num(3.0), num(1.0), pt([0.0; 3]), pt([2.0, 0.0, 0.0]), pt([0.0, 2.0, 0.0])],
    );
    assert!((replaced(&r, 0).volume() - 97.0).abs() < 1e-6);
    // Move Hole: the hole made above, moved 3 mm in x (volume stays 99); Copy=Yes keeps
    // the old one too (98).
    let holed = run(MAKE_HOLE, vec![plate.clone(), hole.clone()], vec![Input::Selection, Input::Enter]);
    let holed = solid("p", replaced(&holed, 0));
    let r = run(MOVE_HOLE, vec![holed.clone(), hole.clone()], vec![Input::Selection, pt([0.0; 3]), pt([3.0, 0.0, 0.0])]);
    assert!((replaced(&r, 0).volume() - 99.0).abs() < 1e-6, "{}", replaced(&r, 0).volume());
    let mut e = event(MOVE_HOLE, Input::Start, vec![holed, hole.clone()], json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in [Input::Selection, Input::Option { name: "Copy".into(), value: None }, pt([0.0; 3]), pt([3.0, 0.0, 0.0])] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
    }
    assert!((replaced(&r, 0).volume() - 98.0).abs() < 1e-6, "{}", replaced(&r, 0).volume());
    // 2 × 2 with Rectangular (B forced square to A): 4 holes.
    let r = run(
        ARRAY_HOLE,
        vec![plate.clone(), hole.clone()],
        vec![Input::Selection, num(2.0), num(2.0), pt([0.0; 3]), pt([2.0, 0.0, 0.0]), pt([1.0, 2.5, 0.0])],
    );
    assert!((replaced(&r, 0).volume() - 96.0).abs() < 1e-6, "{}", replaced(&r, 0).volume());
    let r = run(
        ARRAY_HOLE_POLAR,
        vec![
            plate.clone(),
            closed_curve("h2", square([3.0, 0.0, 1.0], 0.5)),
        ],
        vec![Input::Selection, pt([0.0; 3]), num(4.0), num(360.0)],
    );
    assert!((replaced(&r, 0).volume() - 96.0).abs() < 1e-6);
    // Move Face: the top of the plate up by 1.
    let r = run(
        MOVE_FACE,
        vec![plate.clone()],
        vec![Input::Selection, pt([0.0, 0.0, 1.0]), pt([0.0, 0.0, 1.0]), pt([0.0, 0.0, 2.0])],
    );
    assert!(
        (replaced(&r, 0).volume() - 200.0).abs() < 1e-6,
        "{}",
        replaced(&r, 0).volume()
    );
    // DirectionConstraint=Normal: a slanted move (1 up, 3 sideways) moves the top only up.
    let mut e = event(MOVE_FACE, Input::Start, vec![plate.clone()], json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in [Input::Selection, Input::Option { name: "DirectionConstraint".into(), value: Some("Normal".into()) }, pt([0.0, 0.0, 1.0]), pt([0.0, 0.0, 1.0]), pt([3.0, 0.0, 2.0])] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
    }
    assert!((replaced(&r, 0).volume() - 200.0).abs() < 1e-6, "{}", replaced(&r, 0).volume());
    // A side face (x = 5) moved out 1: 11 × 10 × 1.
    let r = run(MOVE_FACE, vec![plate.clone()], vec![Input::Selection, pt([5.0, 0.0, 0.5]), pt([5.0, 0.0, 0.5]), pt([6.0, 0.0, 0.5])]);
    assert!((replaced(&r, 0).volume() - 110.0).abs() < 1e-6, "{}", replaced(&r, 0).volume());
    // Cap Planar Holes on an open box.
    let mut open = mesh::cuboid([0.0; 3], [1.0, 1.0, 1.0]);
    open.triangles.truncate(10);
    let r = Solids::step(&event(CAP, Input::Enter, vec![solid("o", open)], json!({}))).unwrap();
    assert!(replaced(&r, 0).is_closed());
    // Shell of a ball: two walls.
    let ball = solid("b", mesh::sphere(3.0, 32));
    let r = run(SHELL, vec![ball], vec![Input::Selection, num(0.5)]);
    let v = replaced(&r, 0).volume();
    let expected = 4.0 / 3.0 * PI * (27.0 - 2.5f64.powi(3));
    assert!((v / expected - 1.0).abs() < 0.08, "{v} vs {expected}");
}

#[test]
fn text_boss_and_pipe() {
    let mut e = event(
        TEXT,
        Input::Start,
        vec![],
        json!({"text_string": "OM9", "text_height": 3.0, "text_depth": 0.5}),
    );
    let r = Solids::step(&e).unwrap();
    e.state = r.state;
    e.input = pt([0.0; 3]);
    let r = Solids::step(&e).unwrap();
    let m = created(&r);
    let b = m.bbox();
    assert!((b[1][2] - 0.5).abs() < 1e-9 && b[1][0] > 4.0, "{b:?}");
    let plate = solid("p", mesh::cuboid([-5.0, -5.0, -1.0], [5.0, 5.0, 0.0]));
    let r = run(
        BOSS,
        vec![plate, closed_curve("c", square([0.0; 3], 1.0))],
        vec![Input::Selection, num(2.0)],
    );
    assert!((replaced(&r, 0).volume() - 108.0).abs() < 1e-6);
    let curve = Selected {
        id: "c".into(),
        kind: "curve".into(),
        points: vec![[0.0; 3], [10.0, 0.0, 0.0]],
        ..Default::default()
    };
    let r = run(PIPE, vec![curve], vec![Input::Selection, num(1.0)]);
    assert!((created(&r).volume() / (PI * 10.0) - 1.0).abs() < 0.02);
}

#[test]
fn solid_points_on_moves_a_corner() {
    let b = solid("b", mesh::cuboid([0.0; 3], [2.0, 2.0, 2.0]));
    let r = run(
        POINTS_ON,
        vec![b],
        vec![pt([2.0, 2.0, 2.0]), pt([3.0, 3.0, 3.0]), Input::Enter],
    );
    assert!(r.done, "{}", r.prompt);
    let m = replaced(&r, 0);
    // Clicked in the Top view: the point moves in the CPlane, keeping its height.
    assert!(m.vertices.contains(&[3.0, 3.0, 2.0]));
    assert!(!m.vertices.contains(&[2.0, 2.0, 2.0]));
    assert_eq!(m.vertices.len(), 8);
}

/// Rhino 5 Slab, MakeHole (depth, BothSides, Direction, DeleteInput) and Rib, checked by
/// volume against the formulas (the meshes are polyhedral: volumes are exact).
#[test]
fn slab_make_hole_and_rib_follow_rhino5() {
    // Slab: an open L-shaped polyline (6 + 4 mm) offset 1 mm to its inside, 2 mm high: the
    // band's area is the L's (6·1 + 4·1 − 1·1 at the mitered corner) = 9 mm², volume 18.
    let l = Selected { closed: false, ..closed_curve("l", vec![[0.0, 0.0, 0.0], [6.0, 0.0, 0.0], [6.0, 4.0, 0.0]]) };
    let r = run(SLAB, vec![l.clone()], vec![Input::Selection, pt([5.0, 1.0, 0.0]), num(2.0)]);
    assert!((created(&r).volume().abs() - 18.0).abs() < 1e-9, "{}", created(&r).volume());
    // BothSides: 2 mm wide band → 6·2 + 4·2 = 20 mm² (the miters add and take back one square)... ×2 high = 40.
    let mut e = event(SLAB, Input::Start, vec![l.clone()], json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in [Input::Selection, Input::Option { name: "BothSides".into(), value: None }, pt([5.0, 1.0, 0.0]), num(2.0)] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
    }
    assert!((created(&r).volume().abs() - 40.0).abs() < 1e-9, "{}", created(&r).volume());
    // A closed square 4 × 4, offset 1 outward (mitered: 6 × 6), height 1: 36 − 16 = 20.
    let sq = closed_curve("s", square([0.0, 0.0, 0.0], 2.0));
    let r = run(SLAB, vec![sq.clone()], vec![Input::Selection, pt([3.0, 0.0, 0.0]), num(1.0)]);
    assert!((created(&r).volume().abs() - 20.0).abs() < 1e-9, "{}", created(&r).volume());
    // ThroughPoint: the side point 0.5 inside → 16 − 9 = 7.
    let mut e = event(SLAB, Input::Start, vec![sq], json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in [Input::Selection, Input::Option { name: "ThroughPoint".into(), value: None }, pt([1.5, 0.0, 0.0]), num(1.0)] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
    }
    assert!((created(&r).volume().abs() - 7.0).abs() < 1e-9, "{}", created(&r).volume());

    // MakeHole into a plate 10 × 10 × 2: a 1 × 1 square on top cut 0.5 deep (depth point);
    // BothSides from the middle; DeleteInput removes the curve.
    let plate = solid("p", mesh::cuboid([-5.0, -5.0, 0.0], [5.0, 5.0, 2.0]));
    let top = closed_curve("h", square([0.0, 0.0, 2.0], 0.5));
    let r = run(MAKE_HOLE, vec![plate.clone(), top.clone()], vec![Input::Selection, pt([0.0, 0.0, 1.5])]);
    assert!((replaced(&r, 0).volume() - (200.0 - 0.5)).abs() < 1e-6, "{}", replaced(&r, 0).volume());
    let mid = closed_curve("m", square([0.0, 0.0, 1.0], 0.5));
    let mut e = event(MAKE_HOLE, Input::Start, vec![plate.clone(), mid], json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in [Input::Selection, Input::Option { name: "BothSides".into(), value: None }, Input::Option { name: "DeleteInput".into(), value: None }, num(0.25)] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
    }
    assert!((replaced(&r, 0).volume() - (200.0 - 0.5)).abs() < 1e-6, "{}", replaced(&r, 0).volume());
    assert_eq!(r.delete, vec!["m".to_owned()]);
    // Direction X: a square in the x = 5 side face, through: 10 mm long hole of 1 mm².
    let side = closed_curve("x", vec![[5.0, -0.5, 0.5], [5.0, 0.5, 0.5], [5.0, 0.5, 1.5], [5.0, -0.5, 1.5]]);
    let mut e = event(MAKE_HOLE, Input::Start, vec![plate.clone(), side], json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in [Input::Selection, Input::Option { name: "Direction".into(), value: Some("X".into()) }, Input::Enter] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
    }
    assert!((replaced(&r, 0).volume() - 190.0).abs() < 1e-6, "{}", replaced(&r, 0).volume());

    // Rib: a straight line 4 mm long at z = 0 under a roof at z = 3 (the boundary), 1 mm
    // thick: 4 × 1 × 3 = 12 (InCurvePlane, run up to the roof).
    let roof = solid("roof", mesh::cuboid([-10.0, -10.0, 3.0], [10.0, 10.0, 4.0]));
    let line = Selected { closed: false, ..closed_curve("r", vec![[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]]) };
    let mut e = event(RIB, Input::Start, vec![roof.clone(), line.clone()], json!({}));
    let mut r = Solids::step(&e).unwrap();
    // A line has no plane of its own: InCurvePlane takes the CPlane's (z up).
    for i in [Input::Selection, Input::Enter] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
    }
    assert!(r.done, "{:?} {}", r.error, r.prompt);
    assert!((created(&r).volume().abs() - 12.0).abs() < 1e-6, "{}", created(&r).volume());
    // Tapered, DraftAngle 10°: the walls lean out by 3·tan 10° each → the section is a
    // trapezoid 1 → 1 + 2·0.529 wide: 4 × 3 × (1 + 3·tan 10°) = 18.35.
    let mut e = event(RIB, Input::Start, vec![roof.clone(), line.clone()], json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in [Input::Selection, Input::Option { name: "Mode".into(), value: Some("Tapered".into()) }, Input::Option { name: "DraftAngle".into(), value: Some("10".into()) }, Input::Enter] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
    }
    let want = 4.0 * 3.0 * (1.0 + 3.0 * 10f64.to_radians().tan());
    assert!((created(&r).volume().abs() - want).abs() < 1e-6, "{} vs {want}", created(&r).volume());
    // NormalToCurvePlane: an arc-free profile in the xz plane (z = 1 from x 0..4) runs down
    // to a floor at z = 0, 1 mm thick across: 4 × 1 × 1 = 4.
    let floor = solid("floor", mesh::cuboid([-10.0, -10.0, -1.0], [10.0, 10.0, 0.0]));
    let profile = Selected { closed: false, ..closed_curve("q", vec![[0.0, 0.0, 1.0], [2.0, 0.0, 1.0], [4.0, 0.0, 1.0]]) };
    let mut e = event(RIB, Input::Start, vec![floor, profile], json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in [Input::Selection, Input::Option { name: "Offset".into(), value: Some("NormalToCurvePlane".into()) }, Input::Enter] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
    }
    assert!(r.done, "{:?} {}", r.error, r.prompt);
    assert!((created(&r).volume().abs() - 4.0).abs() < 1e-6, "{}", created(&r).volume());
}

/// Rhino 5 hole tools by "Select a hole": a click on the edge of a hole already in the solid
/// (no curve) — ArrayHole, ArrayHolePolar, MoveHole; and MoveFace ToBoundary.
#[test]
fn hole_tools_pick_the_hole_and_move_face_to_boundary() {
    let plate = solid("p", mesh::cuboid([-5.0, -5.0, 0.0], [5.0, 5.0, 1.0]));
    let hole = closed_curve("h", square([0.0, 0.0, 1.0], 0.5));
    let holed = run(MAKE_HOLE, vec![plate.clone(), hole], vec![Input::Selection, Input::Enter]);
    let holed = solid("p", replaced(&holed, 0));
    assert!((holed.mesh.as_ref().map(|m| to_mesh(m).volume()).unwrap() - 99.0).abs() < 1e-6);
    // The hole is found from a click on its edge (on the top face).
    let found = rhino_tools::pick_hole(holed.mesh.as_ref().unwrap(), [0.5, 0.1, 1.0]).expect("the hole");
    assert_eq!(found.len(), 4, "{found:?}");
    assert!(found.iter().all(|p| (p[2] - 1.0).abs() < 1e-9 && (p[0].abs() - 0.5).abs() < 1e-9 && (p[1].abs() - 0.5).abs() < 1e-9));
    // A click on the outer edge is not a hole.
    assert!(rhino_tools::pick_hole(holed.mesh.as_ref().unwrap(), [5.0, 0.0, 1.0]).is_none());
    // ArrayHole from the picked hole: 3 × 1, 2 mm apart → two more holes (97).
    let r = run(ARRAY_HOLE, vec![holed.clone()], vec![Input::Selection, pt([0.5, 0.1, 1.0]), num(3.0), num(1.0), pt([0.0; 3]), pt([2.0, 0.0, 0.0]), pt([0.0, 2.0, 0.0])]);
    assert!((replaced(&r, 0).volume() - 97.0).abs() < 1e-6, "{}", replaced(&r, 0).volume());
    // ArrayHolePolar round (0, 2): 4 holes, the first is the existing one → 96.
    let r = run(ARRAY_HOLE_POLAR, vec![holed.clone()], vec![Input::Selection, pt([0.5, 0.1, 1.0]), pt([0.0, 2.0, 0.0]), num(4.0), num(360.0)]);
    assert!((replaced(&r, 0).volume() - 96.0).abs() < 1e-6, "{}", replaced(&r, 0).volume());
    // MoveHole: 3 mm in x → still 99, and the old place filled (a click there finds nothing).
    let r = run(MOVE_HOLE, vec![holed.clone()], vec![Input::Selection, pt([0.5, 0.1, 1.0]), pt([0.0; 3]), pt([3.0, 0.0, 0.0])]);
    let moved = replaced(&r, 0);
    assert!((moved.volume() - 99.0).abs() < 1e-6, "{}", moved.volume());
    let sm = solid("q", moved);
    let again = rhino_tools::pick_hole(sm.mesh.as_ref().unwrap(), [3.5, 0.1, 1.0]).expect("the moved hole");
    assert!(again.iter().all(|p| (p[0] - 3.0).abs() <= 0.5 + 1e-9));
    // MoveFace ToBoundary: the top of a 10 × 10 × 1 plate up to a slab whose bottom is at z 4:
    // 10 × 10 × 4 = 400.
    let roof = solid("roof", mesh::cuboid([-20.0, -20.0, 4.0], [20.0, 20.0, 5.0]));
    let mut e = event(MOVE_FACE, Input::Start, vec![plate.clone(), roof.clone()], json!({}));
    let mut r = Solids::step(&e).unwrap();
    for i in [Input::Selection, Input::Option { name: "DirectionConstraint".into(), value: Some("ToBoundary".into()) }, pt([0.0, 0.0, 1.0])] {
        e.state = r.state.clone();
        e.input = i;
        r = Solids::step(&e).unwrap();
        assert!(r.error.is_none(), "{:?} {}", r.error, r.prompt);
    }
    assert!(r.done, "{}", r.prompt);
    assert!((replaced(&r, 0).volume() - 400.0).abs() < 1e-6, "{}", replaced(&r, 0).volume());
}
