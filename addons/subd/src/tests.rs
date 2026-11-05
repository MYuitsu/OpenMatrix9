// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::{Plane, SelectedMesh};
use serde_json::Value;

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

fn run(command: &str, selection: Vec<Selected>, settings: Value, inputs: Vec<Input>) -> Reply {
    let mut e = event(command, Input::Start, selection, settings);
    let mut r = SubD::step(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = SubD::step(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    assert!(r.done, "{command}: {}", r.prompt);
    r
}

fn pt(p: Vec3) -> Input {
    Input::Point { point: p }
}
fn num(v: f64) -> Input {
    Input::Text { text: fmt(v) }
}

fn obj(id: &str, p: &Poly) -> Selected {
    let m = p.to_mesh();
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

fn mesh_of(g: &Geo) -> Mesh {
    match g {
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

fn cube(id: &str) -> (Selected, Poly) {
    let p = subd::box_cage([-1.0; 3], [1.0; 3], [2, 2, 2]);
    (obj(id, &p), p)
}

#[test]
fn primitives_make_closed_cages_and_smooth_shapes() {
    let r = run(
        BOX,
        vec![],
        json!({"sd_faces": 2}),
        vec![pt([0.0; 3]), pt([4.0, 4.0, 0.0]), num(4.0)],
    );
    let m = mesh_of(&r.create[0]);
    assert!(
        m.is_closed() && (m.volume() - 64.0).abs() < 1e-9,
        "{}",
        m.volume()
    );
    let r = run(
        SPHERE,
        vec![],
        json!({"sd_faces": 2, "sd_levels": 3}),
        vec![pt([0.0; 3]), num(5.0)],
    );
    let m = mesh_of(&r.create[0]);
    assert!(m.is_closed());
    let rr: Vec<f64> = m.vertices.iter().map(|v| norm(*v)).collect();
    assert!(
        rr.iter().cloned().fold(f64::MAX, f64::min) > 3.5,
        "smoothed sphere stays round"
    );
    for (cmd, inputs) in [
        (CYLINDER, vec![pt([0.0; 3]), num(2.0), num(5.0)]),
        (CONE, vec![pt([0.0; 3]), num(2.0), num(5.0)]),
        (TORUS, vec![pt([0.0; 3]), num(8.0), num(2.0)]),
    ] {
        let r = run(cmd, vec![], json!({"sd_levels": 2}), inputs);
        let m = mesh_of(&r.create[0]);
        assert!(m.is_closed() && m.volume() > 0.0, "{cmd}: {}", m.volume());
    }
    let r = run(
        PLANE,
        vec![],
        json!({}),
        vec![pt([0.0; 3]), pt([3.0, 3.0, 0.0])],
    );
    assert!(!mesh_of(&r.create[0]).triangles.is_empty());
}

#[test]
fn select_loop_then_extrude_the_selection() {
    let (s, p) = cube("cube-a");
    // Select the faces round the cube's middle (a loop across a vertical edge).
    let r = run(
        SELECT_LOOP,
        vec![s.clone()],
        json!({}),
        vec![pt([0.5, -1.0, 0.0]), Input::Enter],
    );
    assert!(r.messages[0].contains("faces selected"));
    let sel = selected_faces("cube-a", &p);
    assert!(sel.len() >= 4, "{sel:?}");
    let r = run(EXTRUDE, vec![s.clone()], json!({}), vec![num(0.5)]);
    let q = cage_of(&r.replace[0].geo);
    assert_eq!(
        q.faces.len(),
        p.faces.len() + 4 * sel.len() - 2 * (sel.len()),
        "side walls of a band"
    );
    assert!(q.to_mesh().is_closed());
}

#[test]
fn pick_a_face_to_inset_and_collapse() {
    let (s, p) = cube("cube-b");
    let r = run(
        INSET,
        vec![s.clone()],
        json!({}),
        vec![pt([0.5, 0.5, 0.0]), num(0.3)],
    );
    let q = cage_of(&r.replace[0].geo);
    assert_eq!(q.faces.len(), p.faces.len() + 4);
    assert!(q.to_mesh().is_closed());
    let r = run(
        EXTRACT,
        vec![obj("cube-c", &p)],
        json!({}),
        vec![pt([0.5, 0.5, 0.0])],
    );
    assert_eq!(r.create.len(), 1);
    assert_eq!(cage_of(&r.replace[0].geo).faces.len(), p.faces.len() - 1);
}

#[test]
fn quick_selections() {
    let (s, p) = cube("cube-d");
    run(SELECT_ALL, vec![s.clone()], json!({}), vec![Input::Enter]);
    assert_eq!(selected_faces("cube-d", &p).len(), p.faces.len());
    run(INVERT, vec![s.clone()], json!({}), vec![Input::Enter]);
    assert!(selected_faces("cube-d", &p).is_empty());
    run(
        PAINT,
        vec![s.clone()],
        json!({}),
        vec![pt([0.5, 0.5, 0.0]), Input::Enter],
    );
    assert_eq!(selected_faces("cube-d", &p).len(), 1);
    run(GROW, vec![s.clone()], json!({}), vec![Input::Enter]);
    assert_eq!(
        selected_faces("cube-d", &p).len(),
        5,
        "a face and its four neighbours"
    );
    run(SHRINK, vec![s.clone()], json!({}), vec![Input::Enter]);
    assert_eq!(selected_faces("cube-d", &p).len(), 1);
    run(SELECT_NONE, vec![s.clone()], json!({}), vec![Input::Enter]);
    assert!(selected_faces("cube-d", &p).is_empty());
    run(
        SELECT_COPLANAR,
        vec![s.clone()],
        json!({}),
        vec![pt([0.5, 0.5, 0.0]), Input::Enter],
    );
    assert_eq!(
        selected_faces("cube-d", &p).len(),
        4,
        "the top's four faces"
    );
    run(
        SELECTION_SETS,
        vec![s.clone()],
        json!({"sd_set_name": "top"}),
        vec![Input::Enter],
    );
    run(SELECT_NONE, vec![s.clone()], json!({}), vec![Input::Enter]);
    run(
        SELECTION_SETS,
        vec![s.clone()],
        json!({"sd_set_name": "top"}),
        vec![
            Input::Option {
                name: "Action".into(),
                value: Some("Load".into()),
            },
            Input::Enter,
        ],
    );
    assert_eq!(selected_faces("cube-d", &p).len(), 4);
}

#[test]
fn level_honours_creases_and_divide_splits() {
    let (s, p) = cube("cube-e");
    // Crease the top border loop and smooth: the top edge stays sharper than the bottom.
    run(
        CREASE,
        vec![s.clone()],
        json!({}),
        vec![pt([0.5, -1.0, 1.0])],
    );
    assert!(!creases("cube-e", &p).is_empty());
    let r = run(LEVEL, vec![s.clone()], json!({}), vec![num(3.0)]);
    let m = mesh_of(&r.replace[0].geo);
    assert!(m.is_closed());
    // Along the creased top border the cage edge stays; the smooth bottom edge rounds in.
    let reach = |up: bool| {
        m.vertices
            .iter()
            .filter(|v| if up { v[2] > 0.999 } else { v[2] < -0.999 })
            .map(|v| v[1].abs())
            .fold(0.0, f64::max)
    };
    assert!((reach(true) - 1.0).abs() < 1e-9, "creased: {}", reach(true));
    assert!(reach(false) < 0.9, "smooth: {}", reach(false));
    let r = run(
        DIVIDE,
        vec![obj("cube-f", &p)],
        json!({}),
        vec![Input::Enter],
    );
    assert_eq!(cage_of(&r.replace[0].geo).faces.len(), 4 * p.faces.len());
}

#[test]
fn knife_split_sides_shell_fill_symmetry() {
    let (_, p) = cube("cube-g");
    let r = run(
        KNIFE,
        vec![obj("cube-g", &p)],
        json!({}),
        vec![pt([-5.0, 0.25, 0.0]), pt([5.0, 0.25, 0.0])],
    );
    let q = cage_of(&r.replace[0].geo);
    assert!(q.faces.len() > p.faces.len());
    assert!(q.to_mesh().is_closed());
    let r = run(
        SPLIT_SIDES,
        vec![obj("cube-h", &p)],
        json!({}),
        vec![num(0.5), pt([0.5, -1.0, 1.0])],
    );
    assert!(cage_of(&r.replace[0].geo).faces.len() > p.faces.len());
    // Open a hole, fill it, shell it.
    let top: BTreeSet<usize> = (0..p.faces.len())
        .filter(|&f| (p.face_center(f)[2] - 1.0).abs() < 1e-9)
        .collect();
    let open = p.without_faces(&top);
    let r = run(FILL, vec![obj("o1", &open)], json!({}), vec![Input::Enter]);
    assert!(cage_of(&r.replace[0].geo).naked_edges().is_empty());
    let r = run(SHELL, vec![obj("o2", &open)], json!({}), vec![num(0.2)]);
    assert!(mesh_of(&r.replace[0].geo).is_closed());
    let r = run(SYMMETRY, vec![obj("o3", &p)], json!({}), vec![Input::Enter]);
    let q = cage_of(&r.replace[0].geo);
    assert!(
        q.to_mesh().is_closed(),
        "the mirrored halves weld at the seam"
    );
    assert_eq!(q.verts.len(), p.verts.len());
}

#[test]
fn bridge_two_openings_and_weld() {
    // A tube with both ends open.
    let rows: Vec<Vec<Vec3>> = (0..3)
        .map(|i| {
            (0..8)
                .map(|j| {
                    let a = TAU * j as f64 / 8.0;
                    [2.0 * a.cos(), 2.0 * a.sin(), i as f64]
                })
                .collect()
        })
        .collect();
    let tube = subd::grid(&rows, false, true);
    // Two tubes far apart in one object: bridge their facing openings.
    let mut two = tube.clone();
    let base = two.verts.len() as u32;
    two.verts
        .extend(tube.verts.iter().map(|v| [v[0], v[1], v[2] + 5.0]));
    two.faces.extend(
        tube.faces
            .iter()
            .map(|f| f.iter().map(|i| i + base).collect()),
    );
    let r = run(BRIDGE, vec![obj("t", &two)], json!({}), vec![num(2.0)]);
    let q = cage_of(&r.replace[0].geo);
    assert_eq!(q.faces.len(), two.faces.len() + 8 * 2);
    let r = run(
        WELD,
        vec![obj("w", &two.welded(1e-9))],
        json!({}),
        vec![Input::Enter],
    );
    assert!(r.done);
}

#[test]
fn curves_and_surfaces_become_cages() {
    let line = Selected {
        id: "c".into(),
        kind: "curve".into(),
        points: (0..=10).map(|k| [k as f64, 0.0, 0.0]).collect(),
        ..Default::default()
    };
    let r = run(
        PIPE,
        vec![line.clone()],
        json!({"sd_levels": 1}),
        vec![num(1.0)],
    );
    assert!(mesh_of(&r.create[0]).is_closed());
    let circle = Selected {
        id: "k".into(),
        kind: "curve".into(),
        closed: true,
        points: (0..32)
            .map(|k| {
                let a = TAU * k as f64 / 32.0;
                [3.0 * a.cos(), 3.0 * a.sin(), 0.0]
            })
            .collect(),
        ..Default::default()
    };
    let r = run(
        FROM_CURVE,
        vec![circle.clone()],
        json!({"sd_levels": 2}),
        vec![Input::Enter],
    );
    assert!(mesh_of(&r.create[0]).is_closed());
    let prof = Selected {
        id: "p".into(),
        kind: "curve".into(),
        points: vec![[2.0, 0.0, 0.0], [3.0, 0.0, 1.0], [2.0, 0.0, 2.0]],
        ..Default::default()
    };
    let r = run(
        REVOLVE,
        vec![prof],
        json!({}),
        vec![pt([0.0; 3]), pt([0.0, 0.0, 1.0])],
    );
    assert!(!mesh_of(&r.create[0]).triangles.is_empty());
    let upper = Selected {
        id: "u".into(),
        kind: "curve".into(),
        closed: true,
        points: circle.points.iter().map(|p| [p[0], p[1], 4.0]).collect(),
        ..Default::default()
    };
    let r = run(
        LOFT,
        vec![circle.clone(), upper.clone()],
        json!({}),
        vec![Input::Enter],
    );
    assert!(!mesh_of(&r.create[0]).triangles.is_empty());
    let r = run(
        FROM_2_CURVES,
        vec![circle, upper],
        json!({}),
        vec![Input::Enter],
    );
    assert!(!mesh_of(&r.create[0]).triangles.is_empty());
    // A grid cage converts to one NURBS surface.
    let rows: Vec<Vec<Vec3>> = (0..4)
        .map(|i| {
            (0..5)
                .map(|j| [i as f64, j as f64, ((i + j) % 2) as f64 * 0.3])
                .collect()
        })
        .collect();
    let g = subd::grid(&rows, false, false);
    assert!(grid_rows(&g).is_some());
    let r = run(TO_NURBS, vec![obj("g", &g)], json!({}), vec![Input::Enter]);
    assert!(matches!(r.create[0], Geo::Surface { size: [4, 5], .. }));
    let r = run(
        LIBRARY,
        vec![],
        json!({"sd_library": "heart"}),
        vec![pt([0.0; 3])],
    );
    assert!(mesh_of(&r.create[0]).is_closed());
}

#[test]
fn item_moves_only_the_selection() {
    let (s, p) = cube("cube-i");
    run(
        PAINT,
        vec![s.clone()],
        json!({}),
        vec![pt([0.5, 0.5, 0.0]), Input::Enter],
    );
    let r = run(
        ITEM,
        vec![s.clone()],
        json!({}),
        vec![pt([0.0; 3]), pt([0.0, 0.0, 1.0])],
    );
    let q = cage_of(&r.replace[0].geo);
    let top = q.verts.iter().map(|v| v[2]).fold(f64::MIN, f64::max);
    assert!((top - 2.0).abs() < 1e-9);
    assert_eq!(
        q.verts.iter().filter(|v| v[2] > 1.5).count(),
        4,
        "only that face's corners moved"
    );
    let _ = p;
}

#[test]
fn every_command_starts() {
    for c in SubD::SESSION_COMMANDS {
        let r = SubD::step(&event(c, Input::Start, vec![], json!({}))).unwrap();
        assert!(!r.done && !r.prompt.is_empty(), "{c}");
    }
}
