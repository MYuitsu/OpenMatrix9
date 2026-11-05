// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::Plane;
use om9_addon_geom::mesh::cuboid;
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
    let mut r = Tools::step(&e).unwrap();
    for i in inputs {
        e.state = r.state.clone();
        e.input = i;
        r = Tools::step(&e).unwrap();
        assert!(r.error.is_none(), "{command}: {:?} ({})", r.error, r.prompt);
    }
    assert!(r.done, "{command}: {}", r.prompt);
    r
}

fn pt(p: Vec3) -> Input {
    Input::Point { point: p }
}

fn curve(id: &str, points: Vec<Vec3>, closed: bool) -> Selected {
    Selected {
        id: id.into(),
        kind: "curve".into(),
        name: id.into(),
        bbox: [[0.0; 3], [1.0; 3]],
        points,
        closed,
        ..Default::default()
    }
}

fn rail_sel(r: f64) -> Selected {
    let g = rail_circle(r);
    let mut s = curve("rail", nurbs::samples(&g, 128), true);
    s.points.pop();
    s.exact = Some(g);
    s
}

fn solid(id: &str, m: Mesh) -> Selected {
    Selected {
        id: id.into(),
        kind: "mesh".into(),
        name: id.into(),
        bbox: m.bbox(),
        mesh: Some(sel_mesh(&m)),
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

#[test]
fn ring_rail_profiles_and_outside_rail() {
    let r = run(
        RING_RAIL,
        vec![],
        json!({"rr_ring_size": 7}),
        vec![Input::Enter],
    );
    let p = nurbs::samples(&r.create[0], 64);
    assert!(
        p.iter()
            .all(|q| (norm(*q) - 17.3196 / 2.0).abs() < 1e-6 && q[1].abs() < 1e-12),
        "in the Front view"
    );
    assert!(
        dist(p[0], [0.0, 0.0, 17.3196 / 2.0]) < 1e-6,
        "starts at the top"
    );
    let r = run(
        PROFILE_PLACER,
        vec![rail_sel(8.0)],
        json!({}),
        vec![Input::Enter],
    );
    assert_eq!(r.create.len(), 4);
    // The top profile stands on the rail's top.
    if let Geo::Polyline { points, .. } = &r.create[0] {
        assert!(
            points
                .iter()
                .all(|q| q[2] >= 8.0 - 1e-9 && q[0].abs() < 1e-9)
        );
    }
    let r = run(
        OUTSIDE_RAIL,
        vec![rail_sel(8.0)],
        json!({"orr_top": 3.0, "orr_side": 2.0, "orr_bottom": 1.5}),
        vec![Input::Enter],
    );
    let p = nurbs::samples(&r.create[0], 400);
    let top = p.iter().map(|q| q[2]).fold(f64::MIN, f64::max);
    let bottom = p.iter().map(|q| q[2]).fold(f64::MAX, f64::min);
    assert!(
        (top - 11.0).abs() < 0.05 && (bottom + 9.5).abs() < 0.05,
        "{top} {bottom}"
    );
}

#[test]
fn custom_rail_follows_the_top_view_shape() {
    // A bypass: y = x / 4 across the ring.
    let shape = curve(
        "s",
        (0..=40)
            .map(|k| {
                let x = -10.0 + k as f64 * 0.5;
                [x, x / 4.0, 0.0]
            })
            .collect(),
        false,
    );
    let r = run(
        CUSTOM_RAIL,
        vec![rail_sel(8.0), shape],
        json!({}),
        vec![Input::Enter],
    );
    for q in nurbs::samples(&r.create[0], 50) {
        assert!((q[1] - q[0] / 4.0).abs() < 0.05, "{q:?}");
    }
}

#[test]
fn four_rail_profile_passes_through_four_curves() {
    let line = |id: &str, y: f64, z: f64| curve(id, vec![[-10.0, y, z], [10.0, y, z]], false);
    let sel = vec![
        line("a", 2.0, 0.0),
        line("b", -2.0, 0.0),
        line("c", 2.0, 3.0),
        line("d", -2.0, 3.0),
    ];
    let r = run(
        FOUR_PROFILE,
        sel,
        json!({}),
        vec![pt([0.0, -5.0, 0.0]), pt([0.0, 5.0, 0.0])],
    );
    let p = nurbs::samples(&r.create[0], 100);
    assert!(
        p.iter().all(|q| q[0].abs() < 1e-9),
        "the profile lies in the line's plane (x = 0)"
    );
}

#[test]
fn weights_and_resizer() {
    let cube = solid("c", cuboid([0.0; 3], [10.0, 10.0, 10.0]));
    let r = run(METAL_WEIGHTS, vec![cube], json!({}), vec![Input::Enter]);
    assert!(
        r.messages
            .iter()
            .any(|m| m.contains("18k yellow gold: 15.5 g")),
        "{:?}",
        r.messages
    );
    // A ring band resized from 7 to 9 keeps its thickness.
    let band = solid(
        "b",
        mesh::torus(8.66, 1.0, 64, 16).placed(&Frame::new(
            [0.0; 3],
            [0.0, 1.0, 0.0],
            [1.0, 0.0, 0.0],
        )),
    );
    let from = inner_diameter("us", 7.0) / 2.0;
    let to = inner_diameter("us", 9.0) / 2.0;
    let m = resized_ring(band.mesh.as_ref().unwrap(), from, to);
    let inner = m
        .vertices
        .iter()
        .map(|v| (v[0] * v[0] + v[2] * v[2]).sqrt())
        .fold(f64::MAX, f64::min);
    assert!((inner - (7.66 + to - from)).abs() < 0.01, "{inner}");
    let r = run(
        RESIZE,
        vec![band],
        json!({"rs_sizes": "6, 8, 10"}),
        vec![Input::Enter],
    );
    assert_eq!(r.create.len(), 3);
}

#[test]
fn smart_targets_blend() {
    let r = run(
        SMART_TARGET,
        vec![],
        json!({}),
        vec![pt([0.0; 3]), pt([1.0, 0.0, 0.0])],
    );
    assert_eq!(r.create.len(), 2);
    let t1 = curve("t1", vec![[-1.0, 0.0, 0.0], [1.0, 0.0, 0.0]], false);
    let t2 = curve("t2", vec![[10.0, 9.0, 0.0], [10.0, 11.0, 0.0]], false);
    let r = run(SMART_BLEND, vec![t1, t2], json!({}), vec![Input::Enter]);
    let p = nurbs::samples(&r.create[0], 100);
    assert!(dist(p[0], [0.0; 3]) < 1e-9 && dist(*p.last().unwrap(), [10.0, 10.0, 0.0]) < 1e-9);
    assert!(dot(unit(sub(p[1], p[0])), [1.0, 0.0, 0.0]) > 0.99);
    // Join two touching lines.
    let a = curve("a", vec![[0.0; 3], [1.0, 0.0, 0.0]], false);
    let b = curve("b", vec![[2.0, 0.0, 0.0], [1.0, 0.0, 0.0]], false);
    let r = run(JOIN_HISTORY, vec![a, b], json!({}), vec![Input::Enter]);
    assert_eq!(r.create.len(), 1);
    assert_eq!(r.delete.len(), 2);
}

#[test]
fn image_trace_finds_a_dark_square() {
    let dir = std::env::temp_dir().join(format!("om9-trace-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("sq.pgm");
    let (w, h) = (20usize, 20usize);
    let mut bytes = format!("P5\n{w} {h}\n255\n").into_bytes();
    for y in 0..h {
        for x in 0..w {
            bytes.push(if (5..15).contains(&x) && (5..15).contains(&y) {
                0
            } else {
                255
            });
        }
    }
    std::fs::write(&file, &bytes).unwrap();
    let img = read_image(&file).unwrap();
    let loops = trace(&img, 0.5);
    assert_eq!(loops.len(), 1);
    assert!(loops[0].1);
    let r = run(
        IMAGE_TRACE,
        vec![],
        json!({"it_file": file.to_string_lossy(), "it_width": 20.0, "it_smooth": false}),
        vec![pt([0.0; 3])],
    );
    assert_eq!(r.create.len(), 1);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn checker_repair_and_reducer() {
    let mut open = cuboid([0.0; 3], [2.0, 2.0, 2.0]);
    open.triangles.truncate(10);
    let r = run(
        OBJECT_CHECKER,
        vec![solid("o", open.clone())],
        json!({}),
        vec![Input::Enter],
    );
    assert!(
        r.messages.iter().any(|m| m.contains("PROBLEM")),
        "{:?}",
        r.messages
    );
    let r = run(
        MESH_REPAIR,
        vec![solid("o", open)],
        json!({}),
        vec![Input::Enter],
    );
    let m = mesh_of(&r.replace[0].geo);
    assert!(m.is_closed(), "repaired");
    let ball = solid("s", mesh::sphere(5.0, 64));
    let n0 = ball.mesh.as_ref().unwrap().triangles.len();
    let r = run(
        MESH_REDUCER,
        vec![ball],
        json!({"mred_percent": 25}),
        vec![Input::Enter],
    );
    let n1 = mesh_of(&r.replace[0].geo).triangles.len();
    assert!(n1 < n0 / 2 && n1 > n0 / 10, "{n0} → {n1}");
}

#[test]
fn inset_pullback_and_printing() {
    let block = solid("b", cuboid([-5.0, -5.0, 0.0], [5.0, 5.0, 3.0]));
    let sq = curve(
        "q",
        vec![
            [-2.0, -2.0, 3.0],
            [2.0, -2.0, 3.0],
            [2.0, 2.0, 3.0],
            [-2.0, 2.0, 3.0],
        ],
        true,
    );
    let r = run(
        SURFACE_INSET,
        vec![block.clone(), sq],
        json!({"si_depth": 1.0}),
        vec![Input::Enter],
    );
    let m = mesh_of(&r.replace[0].geo);
    assert!((m.volume() - (300.0 - 16.0)).abs() < 1e-6, "{}", m.volume());
    // Pull a flat curve onto the block's top.
    let flat = curve(
        "f",
        vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 1.0, 0.0]],
        false,
    );
    let mut e = event(SURFACE_PULLBACK, Input::Enter, vec![flat], json!({}));
    let r = Tools::step(&e).unwrap();
    e.state = r.state;
    e.selection = vec![block.clone()];
    let r = Tools::step(&e).unwrap();
    assert!(r.done);
    assert!(
        nurbs::samples(&r.create[0], 10)
            .iter()
            .all(|q| (q[2] - 3.0).abs() < 1e-6)
    );
    let r = run(PRINTING, vec![block], json!({}), vec![Input::Enter]);
    assert_eq!(r.create.len(), 4);
}

#[test]
fn curve_tools_and_libraries() {
    let line = curve("l", (0..=20).map(|k| [k as f64, 0.0, 0.0]).collect(), false);
    let p = bent(&line.points, 0.5, 0.5, PI / 2.0, [0.0, 0.0, 1.0]);
    assert!(
        dist(*p.last().unwrap(), [10.0, 10.0, 0.0]) < 1e-9,
        "{:?}",
        p.last()
    );
    let r = run(
        CURVE_TRANSFORM,
        vec![line.clone()],
        json!({"ct_hold": 50, "ct_bend": 50, "ct_angle": 90}),
        vec![Input::Enter],
    );
    assert_eq!(r.replace.len(), 1);
    let other = curve("m", (0..=20).map(|k| [k as f64, 4.0, 0.0]).collect(), false);
    let r = run(
        CENTER_LINE,
        vec![line, other],
        json!({}),
        vec![Input::Enter],
    );
    assert!(
        nurbs::samples(&r.create[0], 10)
            .iter()
            .all(|q| (q[1] - 2.0).abs() < 1e-9)
    );
    for kind in [
        "post",
        "bail",
        "jump_ring",
        "clutch",
        "clasp",
        "peg_head",
        "basket",
        "cup",
    ] {
        let m = finding(kind, 6.0);
        assert!(!m.triangles.is_empty() && m.volume() > 0.0, "{kind}");
    }
    let r = run(
        FINDINGS_LIBRARY,
        vec![],
        json!({"lib_finding": "bail"}),
        vec![pt([0.0; 3])],
    );
    assert_eq!(r.create.len(), 1);
    let sq = vec![
        curve(
            "a",
            vec![[0.0; 3], [2.0, 0.0, 0.0], [2.0, 2.0, 0.0], [0.0, 2.0, 0.0]],
            true,
        ),
        curve(
            "b",
            vec![
                [1.0, 0.0, 0.0],
                [3.0, 0.0, 0.0],
                [3.0, 2.0, 0.0],
                [1.0, 2.0, 0.0],
            ],
            true,
        ),
    ];
    let r = run(PROFILE_MERGE, sq, json!({}), vec![Input::Enter]);
    assert_eq!(r.create.len(), 1);
    let ring = curve(
        "p",
        (0..32)
            .map(|k| {
                let a = TAU * k as f64 / 32.0;
                [a.cos(), a.sin(), 0.0]
            })
            .collect(),
        true,
    );
    let r = run(PROFILE_END_CAP, vec![ring], json!({}), vec![Input::Enter]);
    let m = mesh_of(&r.create[0]);
    assert!(m.is_closed() && m.volume() > 0.0);
}

#[test]
fn every_command_starts() {
    for c in Tools::SESSION_COMMANDS {
        let r = Tools::step(&event(c, Input::Start, vec![], json!({}))).unwrap();
        assert!(!r.done && !r.prompt.is_empty(), "{c}");
    }
}
