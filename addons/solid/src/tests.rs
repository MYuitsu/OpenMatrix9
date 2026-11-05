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
    let r = Solids::step(&event(
        MAKE_HOLE,
        Input::Enter,
        vec![plate.clone(), hole.clone()],
        json!({}),
    ))
    .unwrap();
    assert!(
        (replaced(&r, 0).volume() - 99.0).abs() < 1e-6,
        "{}",
        replaced(&r, 0).volume()
    );
    let r = run(
        ARRAY_HOLE,
        vec![plate.clone(), hole.clone()],
        vec![Input::Selection, num(3.0), num(1.0), num(2.0), num(2.0)],
    );
    assert!((replaced(&r, 0).volume() - 97.0).abs() < 1e-6);
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
        vec![Input::Selection, pt([0.0, 0.0, 0.0]), num(1.0)],
    );
    assert!(
        (replaced(&r, 0).volume() - 200.0).abs() < 1e-6,
        "{}",
        replaced(&r, 0).volume()
    );
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
