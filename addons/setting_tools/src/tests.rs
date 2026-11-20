// SPDX-License-Identifier: LGPL-2.1-or-later
use super::*;
use om9_addon_abi::session::Plane;
use om9_addon_geom::gems::{Cut, Gem};
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

fn step_all(e: &mut Event, inputs: Vec<(Input, Option<Vec<Selected>>)>) -> Reply {
    let step = |e: &Event| {
        if e.command == BOOLEAN_BUILDER {
            BooleanTool::step(e)
        } else {
            SettingTools::step(e)
        }
    };
    let mut r = step(e).unwrap();
    for (i, sel) in inputs {
        e.state = r.state.clone();
        e.input = i;
        if let Some(s) = sel {
            e.selection = s;
        }
        r = step(e).unwrap();
        assert!(
            r.error.is_none(),
            "{}: {:?} ({})",
            e.command,
            r.error,
            r.prompt
        );
    }
    assert!(r.done, "{}: {}", e.command, r.prompt);
    r
}

fn gem_sel(id: &str, c: Vec3, d: f64) -> Selected {
    let g = Gem::new(Cut::Round, d, d);
    let f = Frame::new(c, [0.0, 0.0, 1.0], [1.0, 0.0, 0.0]);
    Selected {
        id: id.into(),
        kind: "mesh".into(),
        name: g.name(),
        bbox: [sub(c, [d; 3]), add(c, [d; 3])],
        gem: Some(g.record(&f, "test", "diamond")),
        ..Default::default()
    }
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
fn subd_bezel_is_a_closed_ring_around_the_stone() {
    let mut e = event(
        SUBD_BEZEL,
        Input::Start,
        vec![gem_sel("g", [0.0; 3], 5.0)],
        json!({}),
    );
    let r = step_all(&mut e, vec![(Input::Enter, None)]);
    let m = mesh_of(&r.create[0]);
    assert!(m.is_closed(), "closed");
    assert!(m.volume() > 0.0, "{}", m.volume());
    let b = m.bbox();
    assert!(b[1][0] > 2.5 && b[1][0] < 4.0, "{b:?}");
    assert_eq!(r.records.len(), 1, "the stone remembers the bezel style");
}

#[test]
fn pull_to_rail_lands_the_bottom_on_the_rail() {
    let head = solid("h", cuboid([-1.0, -1.0, 5.0], [1.0, 1.0, 8.0]));
    let rail = solid("r", cuboid([-10.0, -10.0, 0.0], [10.0, 10.0, 3.0]));
    let mut e = event(
        PULL_TO_RAIL,
        Input::Enter,
        vec![head],
        json!({"ptr_zone": 50}),
    );
    let r = step_all(&mut e, vec![(Input::Enter, Some(vec![rail]))]);
    let m = mesh_of(&r.replace[0].geo);
    let b = m.bbox();
    assert!((b[0][2] - 3.0).abs() < 1e-9, "{b:?}");
    assert!((b[1][2] - 8.0).abs() < 1e-9, "the top stays");
}

#[test]
fn prong_editor_measures_and_rebuilds() {
    let p = prong_mesh(0.3, 2.0, 0.5, 0.0, 1.0);
    let s = SelectedMesh {
        vertices: p.vertices.clone(),
        triangles: p.triangles.clone(),
    };
    let (axis, base, r, len, _) = measure_prong(&s).unwrap();
    assert!(axis[2] > 0.999, "{axis:?}");
    assert!((r - 0.3).abs() < 0.02, "{r}");
    assert!((len - 2.5).abs() < 1e-9 && (base[2] + 0.5).abs() < 1e-9);
    let mut e = event(
        PRONG_EDITOR,
        Input::Start,
        vec![solid("p", p)],
        json!({"pe_diameter": 1.0, "pe_height": 1.0}),
    );
    let r = step_all(&mut e, vec![(Input::Enter, None)]);
    let b = mesh_of(&r.replace[0].geo).bbox();
    assert!(
        (b[1][0] - 0.5).abs() < 0.02 && (b[1][2] - 3.0).abs() < 1e-6,
        "{b:?}"
    );
}

#[test]
fn prongs_and_beads_on_a_surface() {
    let top = solid("t", cuboid([-10.0, -10.0, 0.0], [10.0, 10.0, 2.0]));
    let mut e = event(PRONG_ON_SURFACE, Input::Start, vec![top.clone()], json!({}));
    let r = step_all(
        &mut e,
        vec![
            (
                Input::Point {
                    point: [1.0, 1.0, 0.0],
                },
                None,
            ),
            (
                Input::Point {
                    point: [3.0, 1.0, 0.0],
                },
                None,
            ),
            (Input::Enter, None),
        ],
    );
    assert_eq!(r.create.len(), 2);
    let b = mesh_of(&r.create[0]).bbox();
    assert!(
        (b[1][2] - 3.0).abs() < 1e-6 && (b[0][2] - 1.2).abs() < 1e-6,
        "{b:?}"
    );
    let mut e = event(
        BEAD_ON_SURFACE,
        Input::Start,
        vec![top],
        json!({"bos_size": 1.0}),
    );
    let r = step_all(
        &mut e,
        vec![
            (Input::Point { point: [0.0; 3] }, None),
            (Input::Enter, None),
        ],
    );
    let b = mesh_of(&r.create[0]).bbox();
    assert!((b[1][2] - 2.5).abs() < 1e-6, "{b:?}");
}

#[test]
fn metal_from_a_line_of_stones() {
    let line: Vec<Selected> = (0..5)
        .map(|i| gem_sel(&i.to_string(), [i as f64 * 1.7, 0.0, 0.0], 1.5))
        .collect();
    for mode in [
        "outside",
        "inside",
        "outside_center",
        "top",
        "round_corners",
        "pinched_ends",
    ] {
        let mut e = event(
            METAL_PIECE,
            Input::Start,
            line.clone(),
            json!({"metal_mode": mode}),
        );
        let r = step_all(&mut e, vec![(Input::Enter, None)]);
        for g in &r.create {
            let m = mesh_of(g);
            assert!(m.is_closed(), "{mode}");
            assert!(m.volume() > 0.0, "{mode}: {}", m.volume());
            // No wall cuts through a stone: the stones' centers stay clear.
            if mode != "top" && mode != "inside" && mode != "outside_center" {
                let b = m.bbox();
                assert!(b[0][1] < -0.75 || b[1][1] > 0.75, "{mode}");
            }
        }
    }
}

#[test]
fn boolean_builder_cuts_and_checks() {
    let a = solid("a", cuboid([0.0; 3], [4.0, 4.0, 4.0]));
    let c = solid("c", cuboid([2.0, 2.0, 2.0], [6.0, 6.0, 6.0]));
    let mut e = event(BOOLEAN_BUILDER, Input::Enter, vec![a], json!({}));
    let r = step_all(&mut e, vec![(Input::Enter, Some(vec![c]))]);
    let m = mesh_of(&r.replace[0].geo);
    assert!((m.volume() - 56.0).abs() < 1e-6, "{}", m.volume());
    assert!(check(&SelectedMesh { vertices: m.vertices.clone(), triangles: m.triangles.clone() }).is_none(), "Boolean output must be a closed consistently oriented mesh");
    assert_eq!(r.delete, vec!["c".to_string()]);
    let open = SelectedMesh {
        vertices: vec![[0.0; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        triangles: vec![[0, 1, 2]],
    };
    assert!(check(&open).is_some());
}

#[test]
fn boolean_builder_refuses_invalid_inputs_and_keeps_the_selection_stage() {
    let valid = solid("valid", cuboid([0.; 3], [4.; 3]));
    let mut open = solid("open", cuboid([1.; 3], [3.; 3]));
    open.mesh.as_mut().unwrap().triangles.pop();
    let mut reversed = valid.clone();
    reversed.mesh.as_mut().unwrap().triangles[0].swap(0, 1);
    let mut invalid_index = valid.clone();
    invalid_index.mesh.as_mut().unwrap().triangles[0][0] = u32::MAX;
    let mut degenerate = valid.clone();
    let index = degenerate.mesh.as_ref().unwrap().triangles[0][0];
    degenerate.mesh.as_mut().unwrap().triangles[0][1] = index;
    let mut nonfinite = valid.clone();
    nonfinite.mesh.as_mut().unwrap().vertices[0][0] = f64::NAN;
    let mut brep = valid.clone();
    brep.kind = "brep".into();
    for invalid in [open, reversed, invalid_index, degenerate, nonfinite, brep] {
        let mut e = event(BOOLEAN_BUILDER, Input::Enter, vec![invalid.clone()], json!({}));
        let refusal = BooleanTool::step(&e).unwrap();
        assert!(refusal.error.is_some(), "{invalid:?}");
        assert!(!refusal.done && refusal.replace.is_empty() && refusal.delete.is_empty());
        assert!(load::<Objects>(&Event { state: refusal.state.clone(), ..e.clone() }).objects.is_empty());

        e.selection = vec![valid.clone()];
        let accepted = BooleanTool::step(&e).unwrap();
        assert!(accepted.error.is_none());
        e.state = accepted.state;
        e.selection = vec![invalid];
        e.selection[0].id = "cutter".into();
        let refusal = BooleanTool::step(&e).unwrap();
        assert!(refusal.error.is_some());
        assert!(!refusal.done && refusal.replace.is_empty() && refusal.delete.is_empty() && refusal.preview.is_empty());
        assert_eq!(load::<Objects>(&Event { state: refusal.state, ..e }).objects.len(), 1);
    }
}

#[test]
fn boolean_preview_changes_for_same_bounds_mesh_content() {
    let object = solid("object", cuboid([0.; 3], [4.; 3]));
    let cutter = solid("cutter", cuboid([2.; 3], [6.; 3]));
    let mut changed = cutter.clone();
    changed.mesh.as_mut().unwrap().vertices[0][0] += 0.5;
    assert_eq!(changed.bbox, cutter.bbox);
    assert_eq!(changed.mesh.as_ref().unwrap().vertices.len(), cutter.mesh.as_ref().unwrap().vertices.len());

    let mut e = event(BOOLEAN_BUILDER, Input::Enter, vec![object.clone()], json!({}));
    let first = BooleanTool::step(&e).unwrap();
    e.state = first.state;
    e.input = Input::Selection;
    e.selection = vec![cutter.clone()];
    let preview = BooleanTool::step(&e).unwrap().preview;
    e.selection = vec![changed.clone()];
    let changed_preview = BooleanTool::step(&e).unwrap().preview;
    let expected = boolean_mesh(&to_mesh(object.mesh.as_ref().unwrap()),
        &to_mesh(changed.mesh.as_ref().unwrap()), Op::Difference, 1e-8).unwrap().wire();
    assert_eq!(changed_preview, expected);
    assert_ne!(preview, changed_preview);
    e.input = Input::Enter;
    let completed = BooleanTool::step(&e).unwrap();
    assert!(completed.done, "{:?}", completed.error);
    let result = mesh_of(&completed.replace[0].geo);
    assert!(check_mesh(&result.vertices, &result.triangles).is_none());

    let objects = Objects { objects: vec![(object.id.clone(), object.mesh.clone().unwrap())] };
    let original = boolean_preview_key(&objects, &[&cutter], Op::Difference, 1e-8);
    assert_ne!(original, boolean_preview_key(&objects, &[&changed], Op::Difference, 1e-8));
    assert_ne!(original, boolean_preview_key(&objects, &[&cutter], Op::Union, 1e-8));
    assert_ne!(original, boolean_preview_key(&objects, &[&cutter], Op::Difference, 0.01));
    let mut changed_topology = cutter.clone();
    changed_topology.mesh.as_mut().unwrap().triangles[0].swap(0, 1);
    assert_ne!(original, boolean_preview_key(&objects, &[&changed_topology], Op::Difference, 1e-8));
    let mut changed_object = Objects { objects: objects.objects.clone() };
    changed_object.objects[0].1.vertices[0][0] += 0.5;
    assert_ne!(original, boolean_preview_key(&changed_object, &[&cutter], Op::Difference, 1e-8));
}

#[test]
fn empty_intersection_does_not_delete_inputs() {
    let object = solid("object", cuboid([0.; 3], [1.; 3]));
    let cutter = solid("cutter", cuboid([3.; 3], [4.; 3]));
    let mut e = event(BOOLEAN_BUILDER, Input::Enter, vec![object], json!({"bb_mode": "intersection"}));
    let first = BooleanTool::step(&e).unwrap();
    assert!(first.error.is_none());
    e.state = first.state;
    e.selection = vec![cutter];
    let result = BooleanTool::step(&e).unwrap();
    assert!(result.error.as_deref().unwrap_or_default().contains("empty"));
    assert!(!result.done && result.replace.is_empty() && result.delete.is_empty());
}

#[test]
fn boolean_mesh_modes_and_multiple_cutters_produce_closed_outputs() {
    for (mode, volume, keep) in [("difference", 56., false), ("union", 120., true), ("intersection", 8., false)] {
        let object = solid("object", cuboid([0.; 3], [4.; 3]));
        let cutter = solid("cutter", cuboid([2.; 3], [6.; 3]));
        let mut e = event(BOOLEAN_BUILDER, Input::Enter, vec![object], json!({"bb_mode": mode, "bb_keep_cutters": keep}));
        let first = BooleanTool::step(&e).unwrap();
        e.state = first.state;
        e.selection = vec![cutter];
        let r = BooleanTool::step(&e).unwrap();
        assert!(r.done, "{mode}: {:?}", r.error);
        let result = mesh_of(&r.replace[0].geo);
        assert!(check_mesh(&result.vertices, &result.triangles).is_none());
        assert!((result.volume().abs() - volume).abs() < 1e-6, "{mode}: {}", result.volume());
        assert_eq!(r.delete.is_empty(), keep);
    }
    let object = solid("object", cuboid([0.; 3], [6.; 3]));
    let cutters = vec![solid("a", cuboid([1., 1., 1.], [2., 2., 7.])),
        solid("b", cuboid([4., 4., 1.], [5., 5., 7.]))];
    let mut e = event(BOOLEAN_BUILDER, Input::Enter, vec![object], json!({}));
    let first = BooleanTool::step(&e).unwrap();
    e.state = first.state;
    e.selection = cutters;
    let r = BooleanTool::step(&e).unwrap();
    assert!(r.done, "{:?}", r.error);
    let result = mesh_of(&r.replace[0].geo);
    assert!(check_mesh(&result.vertices, &result.triangles).is_none());
    assert!((result.volume().abs() - 206.).abs() < 1e-6);
    assert_eq!(r.delete.len(), 2);
}

#[test]
fn boolean_builder_keeps_batch_outputs_matched_to_source_objects() {
    let objects = vec![solid("left", cuboid([0.; 3], [4.; 3])),
        solid("right", cuboid([10., 0., 0.], [14., 4., 4.]))];
    let cutters = vec![solid("a", cuboid([2.; 3], [6.; 3])),
        solid("b", cuboid([12., 2., 2.], [16., 6., 6.]))];
    let mut e = event(BOOLEAN_BUILDER, Input::Enter, objects, json!({}));
    let accepted = BooleanTool::step(&e).unwrap();
    e.state = accepted.state;
    e.selection = cutters;
    let result = BooleanTool::step(&e).unwrap();
    assert!(result.done, "{:?}", result.error);
    assert_eq!(result.replace.iter().map(|item| item.id.as_str()).collect::<Vec<_>>(), vec!["left", "right"]);
    for item in &result.replace { assert!((mesh_of(&item.geo).volume().abs() - 56.).abs() < 1e-6); }
    assert_eq!(result.delete, vec!["a", "b"]);
}

#[test]
fn boolean_mesh_rejects_oversized_input_before_conversion() {
    let mut huge = cuboid([0.; 3], [1.; 3]);
    huge.triangles.resize(25_000, huge.triangles[0]);
    let cutter = cuboid([0.5; 3], [1.5; 3]);
    assert!(boolean_mesh(&huge, &cutter, Op::Difference, 1e-8).unwrap_err().contains("triangle budget"));
}

#[test]
fn boolean_mesh_uses_document_tolerance() {
    let object = cuboid([0.; 3], [0.01; 3]);
    let cutter = cuboid([0.005; 3], [0.015; 3]);
    assert!(boolean_mesh(&object, &cutter, Op::Difference, 1e-8).is_ok());
    assert!(boolean_mesh(&object, &cutter, Op::Difference, 0.1).is_err());
}

#[test]
fn every_command_starts() {
    for c in SettingTools::SESSION_COMMANDS {
        let r = SettingTools::step(&event(c, Input::Start, vec![], json!({}))).unwrap();
        assert!(!r.done && !r.prompt.is_empty(), "{c}");
    }
}
