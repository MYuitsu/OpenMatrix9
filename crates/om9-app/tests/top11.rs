// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::{Controller, LINE};
use om9_core::{
    budget::Limits,
    geometry::{Geometry, NurbsSurface, Transform},
    model::{Document, Entity},
    top11::{GripMode, Top11Action},
};

fn controller() -> Controller {
    Controller::new(Document::new("top11"), Limits::default()).unwrap()
}

fn draw_line(c: &mut Controller, start: [f64; 3], end: [f64; 3]) -> om9_core::identity::ObjectId {
    let before: std::collections::BTreeSet<_> = c.document().entities.keys().copied().collect();
    c.submit(LINE).unwrap();
    c.submit(&format!("{},{},{}", start[0], start[1], start[2]))
        .unwrap();
    c.submit(&format!("{},{},{}", end[0], end[1], end[2]))
        .unwrap();
    *c.document()
        .entities
        .keys()
        .find(|id| !before.contains(id))
        .expect("new line")
}

fn select(c: &mut Controller, ids: &[om9_core::identity::ObjectId]) {
    c.select_objects(ids).unwrap();
}

#[test]
fn top11_move_can_start_before_selection_and_undo() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    c.select_objects(&[]).unwrap();
    c.top11_invoke(Top11Action::Move).unwrap();
    assert!(c.top11_needs_object());
    assert!(c.submit("").is_err());
    c.top11_accept_object(id).unwrap();
    c.submit("").unwrap();
    assert!(!c.top11_needs_object());
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([2.0, 3.0, 0.0]).unwrap();
    assert_eq!(
        c.document().entities[&id]
            .placement
            .point([0.0; 3])
            .unwrap(),
        [2.0, 3.0, 0.0]
    );
    c.undo().unwrap();
    assert_eq!(
        c.document().entities[&id]
            .placement
            .point([0.0; 3])
            .unwrap(),
        [0.0; 3]
    );
}

#[test]
fn mirror_can_start_before_selection_and_cancel() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    c.select_objects(&[]).unwrap();
    c.submit("Mirror").unwrap();
    c.top11_accept_object(id).unwrap();
    c.cancel();
    assert!(!c.is_running());
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn every_top11_action_can_open_object_selection_and_cancel() {
    for action in Top11Action::ALL {
        if action == Top11Action::RingRail {
            continue;
        }
        let mut c = controller();
        let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
        c.select_objects(&[]).unwrap();
        let before = c.document().clone();
        assert!(
            c.top11_availability()
                .iter()
                .any(|state| state.action == action && state.enabled)
        );
        c.top11_invoke(action).unwrap();
        assert!(c.top11_needs_object(), "{action:?}");
        c.top11_accept_object(id).unwrap();
        c.cancel();
        assert_eq!(c.document(), &before, "{action:?}");
    }
}

#[test]
fn top11_can_remove_picked_selection_and_cutters_before_commit() {
    let mut c = controller();
    let target = draw_line(&mut c, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let cutter = draw_line(&mut c, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    c.select_objects(&[]).unwrap();
    c.top11_invoke(Top11Action::Split).unwrap();
    c.top11_accept_object(target).unwrap();
    c.top11_remove_object(target).unwrap();
    assert!(c.submit("").is_err());
    c.top11_accept_object(target).unwrap();
    c.submit("").unwrap();
    c.top11_accept_object(cutter).unwrap();
    c.top11_remove_object(cutter).unwrap();
    assert!(c.submit("").is_err());
    c.top11_accept_object(cutter).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 3);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
    assert!(c.document().entities.contains_key(&target));
    assert!(c.document().entities.contains_key(&cutter));
}

#[test]
fn split_multiple_curves_is_one_undo_and_preserves_unhit_target() {
    let mut c = controller();
    let first = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let second = draw_line(&mut c, [-10., 4., 0.], [10., 4., 0.]);
    let untouched = draw_line(&mut c, [30., 0., 0.], [40., 0., 0.]);
    let cutter = draw_line(&mut c, [0., -5., 0.], [0., 10., 0.]);
    c.select_objects(&[first, second, untouched]).unwrap();
    let undo_before = c.coordinator().undo_len();
    c.top11_invoke(Top11Action::Split).unwrap();
    c.top11_accept_object(cutter).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 6);
    assert!(c.document().entities.contains_key(&untouched));
    assert!(c.document().entities.contains_key(&cutter));
    assert!(!c.document().entities.contains_key(&first));
    assert!(!c.document().entities.contains_key(&second));
    assert_eq!(c.coordinator().undo_len(), undo_before + 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 4);
    assert!(c.document().entities.contains_key(&first));
    assert!(c.document().entities.contains_key(&second));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 6);
}

#[test]
fn split_multiple_curves_accepts_cutters_that_hit_different_targets() {
    let mut c = controller();
    let first = draw_line(&mut c, [-10., 0., 0.], [10., 0., 0.]);
    let second = draw_line(&mut c, [-10., 4., 0.], [10., 4., 0.]);
    let cutter_a = draw_line(&mut c, [0., -2., 0.], [0., 2., 0.]);
    let cutter_b = draw_line(&mut c, [2., 2., 0.], [2., 6., 0.]);
    c.select_objects(&[first, second]).unwrap();
    let undo_before = c.coordinator().undo_len();
    c.top11_invoke(Top11Action::Split).unwrap();
    c.top11_accept_object(cutter_a).unwrap();
    c.top11_accept_object(cutter_b).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 6);
    assert_eq!(c.coordinator().undo_len(), undo_before + 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 4);
    assert!(c.document().entities.contains_key(&first));
    assert!(c.document().entities.contains_key(&second));
}

#[test]
fn split_point_line_accepts_world_coordinates_and_undo_redo() {
    let mut doc = Document::with_template_tolerance("split point");
    let mut entity = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0., 0., 0.],
            end: [10., 0., 0.],
            domain: [0., 1.],
        }),
        None,
    );
    entity.placement = om9_core::geometry::Transform::translation([5., 2., 0.]);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Split").unwrap();
    c.submit("Point").unwrap();
    assert!(c.submit("W9,3,0").is_err());
    assert!(c.is_running());
    c.submit("W9,2,0").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let mut lengths: Vec<f64> = c
        .document()
        .entities
        .values()
        .map(|e| {
            let Geometry::Curve(curve) = &e.geometry else {
                panic!("curve")
            };
            om9_core::curve_ops::arc_length(curve, c.coordinator().limits()).unwrap()
        })
        .collect();
    lengths.sort_by(f64::total_cmp);
    assert!(
        (lengths[0] - 4.).abs() < 1e-8 && (lengths[1] - 6.).abs() < 1e-8,
        "{lengths:?}"
    );
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert!(c.document().entities.contains_key(&id));
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn split_point_command_alias_and_cancel_leave_line_intact() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.; 3], [10., 0., 0.]);
    select(&mut c, &[id]);
    c.submit("Split Point").unwrap();
    assert_eq!(c.top11_prompt(), Some("pick a point on the line"));
    c.cancel();
    assert!(c.document().entities.contains_key(&id));
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn split_point_rejects_non_line_without_document_changes() {
    let mut doc = Document::with_template_tolerance("split circle");
    let entity = Entity::new(
        "circle",
        Geometry::Curve(om9_core::geometry::Curve::Circle {
            center: [0.; 3],
            x: [1., 0., 0.],
            y: [0., 1., 0.],
            radius: 5.,
            domain: [0., std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    c.submit("Split").unwrap();
    assert!(c.submit("Point").is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    assert!(c.is_running());
}

#[test]
fn m03_03_08_006_toolbar_has_eleven_actions_and_ring_rail_opens_the_builder() {
    let mut c = controller();
    let states = c.top11_availability();
    assert_eq!(states.len(), 11);
    let ring = states
        .iter()
        .find(|state| state.action == Top11Action::RingRail)
        .unwrap();
    // M04 owns the builder now; with no command running the row is available.
    assert!(ring.enabled);
    assert!(c.top11_invoke(Top11Action::Duplicate).is_ok());
    assert!(c.top11_needs_object());
    c.cancel();
    let result = c.top11_invoke(Top11Action::RingRail).unwrap();
    assert!(result.message().contains("Ring Rail"));
    assert!(c.builder().is_some());
    let running = c.top11_availability();
    for state in &running {
        assert!(!state.enabled, "an open builder must disable the toolbar");
    }
    assert!(c.top11_invoke(Top11Action::RingRail).is_err());
    c.builder_cancel();
    // The line command path still works after the builder closes.
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit(LINE).unwrap();
    for state in c.top11_availability() {
        assert!(!state.enabled, "running command must disable all buttons");
    }
    assert!(c.top11_invoke(Top11Action::Duplicate).is_err());
    c.cancel();
    println!("top11_buttons={}", states.len());
    println!("ringrail_enabled={}", ring.enabled);
}

#[test]
fn m03_03_08_001_duplicate_is_in_place_one_undo_and_selects_the_copy() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    let before = c.document().entities[&id].geometry.clone();
    let result = c.top11_duplicate_in_place().unwrap();
    assert!(result.message().contains("Duplicate"));
    assert_eq!(c.document().entities.len(), 2);
    assert_eq!(c.selected().len(), 1);
    let copy_id = *c.selected().iter().next().unwrap();
    assert_ne!(copy_id, id);
    assert_eq!(c.document().entities[&copy_id].geometry, before);
    assert_eq!(c.coordinator().undo_len(), 2);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
    println!("duplicate_entities={}", c.document().entities.len());
    println!("duplicate_undo_redo_passed=true");
}

#[test]
fn m03_03_08_001_duplicate_base_point_places_multiple_copies_until_enter() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Copy").unwrap();
    assert_eq!(c.top11_prompt(), Some("pick the base point"));
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 3);
    assert!(c.is_running());
    c.submit("").unwrap();
    assert!(!c.is_running());
    let placements: Vec<f64> = c
        .document()
        .entities
        .values()
        .map(|entity| entity.placement.point([0.0; 3]).unwrap()[0])
        .collect();
    assert!(placements.contains(&0.0));
    assert!(placements.contains(&5.0));
    assert!(placements.contains(&10.0));
    println!("duplicate_base_copies=2");
    println!("duplicate_total_objects={}", c.document().entities.len());
}

#[test]
fn m03_03_08_003_move_and_rotate_collect_points_and_commit_placements() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Move").unwrap();
    assert!(c.top11_pending().is_some());
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([5.0, -2.0, 0.0]).unwrap();
    assert!(!c.is_running());
    let placement = c.document().entities[&id].placement;
    assert_eq!(placement.point([0.0; 3]).unwrap(), [5.0, -2.0, 0.0]);
    c.undo().unwrap();
    select(&mut c, &[id]);
    c.submit("Rotate").unwrap();
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 10.0, 0.0]).unwrap();
    let placement = c.document().entities[&id].placement;
    let end = placement.point([10.0, 0.0, 0.0]).unwrap();
    assert!(
        end[0].abs() < 1e-6 && (end[1] - 10.0).abs() < 1e-6,
        "{end:?}"
    );
    assert_eq!(c.coordinator().undo_len(), 2);
    println!("move_base_mm=[5.0, -2.0, 0.0]");
    println!("rotate_end_mm={end:?}");
}

#[test]
fn m03_03_08_003_mirror_collects_a_plane_and_creates_a_copy() {
    let mut c = controller();
    let id = draw_line(&mut c, [1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Mirror").unwrap();
    c.accept_world_point([0.0, -5.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let copy = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) = &copy.geometry else {
        panic!("mirror did not copy a line");
    };
    assert!((start[0] + 1.0).abs() < 1e-9, "{start:?}");
    assert!((end[0] + 4.0).abs() < 1e-9, "{end:?}");
    println!("mirror_copies=2");
    println!("mirrored_start_x_mm={:.12}", start[0]);
}

#[test]
fn m03_03_08_004_join_explode_split_and_trim_run_from_the_toolbar() {
    let mut c = controller();
    let first = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    let second = draw_line(&mut c, [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
    select(&mut c, &[first, second]);
    c.top11_invoke(Top11Action::Join).unwrap();
    assert_eq!(c.document().entities.len(), 1);
    let joined = *c.document().entities.keys().next().unwrap();
    assert!(matches!(
        c.document().entities[&joined].geometry,
        Geometry::Curve(om9_core::geometry::Curve::Polycurve { .. })
    ));
    select(&mut c, &[joined]);
    c.top11_invoke(Top11Action::Explode).unwrap();
    assert_eq!(c.document().entities.len(), 2);

    let mut split = controller();
    let horizontal = draw_line(&mut split, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let vertical = draw_line(&mut split, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    select(&mut split, &[horizontal]);
    split.top11_invoke(Top11Action::Split).unwrap();
    assert_eq!(
        split.top11_prompt(),
        Some("pick cutting curves (Enter to execute)")
    );
    split.top11_accept_object(vertical).unwrap();
    split.submit("").unwrap();
    assert!(!split.is_running());
    assert_eq!(split.document().entities.len(), 3);
    assert_eq!(split.selected().len(), 2);

    let mut trim = controller();
    let horizontal = draw_line(&mut trim, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let vertical = draw_line(&mut trim, [0.0, -10.0, 0.0], [0.0, 10.0, 0.0]);
    select(&mut trim, &[horizontal]);
    trim.top11_invoke(Top11Action::Trim).unwrap();
    trim.top11_accept_object(vertical).unwrap();
    trim.submit("").unwrap();
    assert_eq!(trim.top11_prompt(), Some("pick the portion to remove"));
    trim.accept_world_point([-5.0, 0.0, 0.0]).unwrap();
    assert_eq!(trim.document().entities.len(), 2);
    let remainder = trim
        .document()
        .entities
        .values()
        .find(|entity| entity.id != vertical)
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, .. }) = &remainder.geometry else {
        panic!("trim did not leave a line");
    };
    assert!((start[0] - 0.0).abs() < 1e-9, "{start:?}");
    println!("joined_entities=1");
    println!("exploded_entities=2");
    println!("split_entities=3");
    println!("trim_remaining_entities=2");
    println!("trim_kept_start_x_mm={:.12}", start[0]);
}

#[test]
fn m03_03_08_003_mirror_delete_input_reflects_the_original_in_place() {
    let mut c = controller();
    let id = draw_line(&mut c, [1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Mirror Delete").unwrap();
    c.accept_world_point([0.0, -5.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 1);
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("mirror delete lost the line");
    };
    assert!((start[0] + 1.0).abs() < 1e-9, "{start:?}");
    assert!((end[0] + 4.0).abs() < 1e-9, "{end:?}");
    println!("mirror_delete_entities=1");
    println!("mirror_delete_start_x_mm={:.12}", start[0]);
}

#[test]
fn m03_03_08_002_grip_drag_moves_a_control_point_and_undo_restores_it() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    assert_eq!(handles.len(), 2);
    let handle = handles
        .into_iter()
        .find(|handle| handle.index == 1)
        .unwrap();
    c.top11_move_grip(handle, [15.0, 2.0, 0.0]).unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { end, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("grip edit lost the line");
    };
    assert!((end[0] - 15.0).abs() < 1e-9 && (end[1] - 2.0).abs() < 1e-9);
    c.undo().unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { end, .. }) =
        &c.document().entities[&id].geometry
    else {
        panic!("undo lost the line");
    };
    assert!((end[0] - 10.0).abs() < 1e-9);
    println!("grip_drag_end_mm=[15.0, 2.0, 0.0]");
    println!("grip_drag_undo_passed=true");
}

#[test]
fn m03_03_08_002_control_point_toggle_uses_f10_and_selection_only() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    let result = c.top11_invoke(Top11Action::ControlPoints).unwrap();
    assert!(result.message().contains("control points on"));
    assert_eq!(c.top11_grip_mode(), Some(GripMode::Control));
    assert_eq!(c.top11_grip_points().unwrap().len(), 2);
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    assert_eq!(c.top11_grip_mode(), None);
    c.cancel();
    assert_eq!(c.top11_grip_mode(), None);
    println!("grip_points_on_line=2");
    println!("grip_mode_toggle_passed=true");
}

#[test]
fn command_names_and_top11_prefix_share_the_toolbar_path() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Copy InPlace").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    c.undo().unwrap();
    select(&mut c, &[id]);
    c.submit("top11 duplicate inplace").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    c.undo().unwrap();
    select(&mut c, &[id]);
    c.submit("Split 0.25").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let half = *c.selected().iter().next().unwrap();
    select(&mut c, &[half]);
    c.submit("Trim 0.2 0.8").unwrap();
    let Geometry::Curve(curve) = &c.document().entities[&half].geometry else {
        panic!("trim lost the curve");
    };
    assert!(om9_core::curve_ops::arc_length(curve, c.coordinator().limits()).unwrap() > 0.0);
}

#[test]
fn m03_03_08_004_stress_join_explode_split_trim_cycles() {
    let mut completed = 0usize;
    for step in 0..50usize {
        let mut c = controller();
        let first = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
        let second = draw_line(&mut c, [10.0, 0.0, 0.0], [10.0, 10.0, 0.0]);
        select(&mut c, &[first, second]);
        c.top11_invoke(Top11Action::Join).unwrap();
        assert_eq!(c.document().entities.len(), 1);
        let joined = *c.document().entities.keys().next().unwrap();
        select(&mut c, &[joined]);
        c.top11_invoke(Top11Action::Explode).unwrap();
        assert_eq!(c.document().entities.len(), 2);
        let piece = *c.document().entities.keys().next().unwrap();
        select(&mut c, &[piece]);
        c.submit("Split 0.5").unwrap();
        assert_eq!(c.document().entities.len(), 3);
        let half = *c.selected().iter().next().unwrap();
        select(&mut c, &[half]);
        c.submit("Trim 0.1 0.9").unwrap();
        let Geometry::Curve(curve) = &c.document().entities[&half].geometry else {
            panic!("trim lost the curve");
        };
        let length = om9_core::curve_ops::arc_length(curve, c.coordinator().limits()).unwrap();
        assert!(length > 0.0 && length < 10.0, "step {step}: {length}");
        completed += 1;
    }
    assert_eq!(completed, 50);
    println!("stress_cycles={completed}");
}

#[test]
fn move_prompt_names_the_next_point() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [10.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Move").unwrap();
    assert_eq!(c.top11_prompt(), Some("pick the base point"));
    c.accept_world_point([0.0; 3]).unwrap();
    assert_eq!(c.top11_prompt(), Some("pick the destination point"));
    c.cancel();
    assert_eq!(c.top11_prompt(), None);
    assert!(!c.is_running());
}

#[test]
fn m03_03_08_001_duplicate_options_project_and_reuse_the_last_move() {
    let mut c = controller();
    let id = draw_line(&mut c, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Copy Vertical=Yes").unwrap();
    c.accept_world_point([0.0; 3]).unwrap();
    c.accept_world_point([3.0, 4.0, 5.0]).unwrap();
    c.submit("").unwrap();
    let vertical = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    let origin = vertical.placement.point([0.0; 3]).unwrap();
    assert!(
        origin[0].abs() < 1e-9 && origin[1].abs() < 1e-9,
        "{origin:?}"
    );
    assert!((origin[2] - 5.0).abs() < 1e-9, "{origin:?}");
    println!("duplicate_vertical_origin={origin:?}");

    let mut distance = controller();
    let id = draw_line(&mut distance, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut distance, &[id]);
    distance.submit("Copy UseLastDistance=Yes").unwrap();
    distance.accept_world_point([0.0; 3]).unwrap();
    distance.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    distance.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    distance.submit("").unwrap();
    assert!(distance.document().entities.values().any(|entity| {
        let point = entity.placement.point([0.0; 3]).unwrap();
        (point[1] - 10.0).abs() < 1e-9
    }));
    println!("duplicate_last_distance_y=10.0");

    let mut direction = controller();
    let id = draw_line(&mut direction, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut direction, &[id]);
    direction.submit("Copy UseLastDirection=Yes").unwrap();
    direction.accept_world_point([0.0; 3]).unwrap();
    direction.accept_world_point([10.0, 0.0, 0.0]).unwrap();
    direction.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    direction.submit("").unwrap();
    assert!(direction.document().entities.values().any(|entity| {
        let point = entity.placement.point([0.0; 3]).unwrap();
        (point[0] - 5.0).abs() < 1e-9
    }));
    println!("duplicate_last_direction_x=5.0");

    let mut from_last = controller();
    let id = draw_line(&mut from_last, [0.0; 3], [1.0, 0.0, 0.0]);
    select(&mut from_last, &[id]);
    from_last
        .submit("Copy FromLastPoint=Yes UseLastDistance=Yes")
        .unwrap();
    from_last.accept_world_point([0.0; 3]).unwrap();
    from_last.accept_world_point([5.0, 0.0, 0.0]).unwrap();
    from_last.accept_world_point([5.0, 5.0, 0.0]).unwrap();
    from_last.submit("").unwrap();
    assert!(from_last.document().entities.values().any(|entity| {
        let point = entity.placement.point([0.0; 3]).unwrap();
        (point[0] - 5.0).abs() < 1e-9 && (point[1] - 5.0).abs() < 1e-9
    }));
}

#[test]
fn m03_03_08_004_multiple_cutters_split_and_trim_one_target() {
    let mut split = controller();
    let horizontal = draw_line(&mut split, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let left = draw_line(&mut split, [-5.0, -5.0, 0.0], [-5.0, 5.0, 0.0]);
    let right = draw_line(&mut split, [5.0, -5.0, 0.0], [5.0, 5.0, 0.0]);
    select(&mut split, &[horizontal]);
    split.top11_invoke(Top11Action::Split).unwrap();
    split.top11_accept_object(left).unwrap();
    split.top11_accept_object(right).unwrap();
    split.submit("").unwrap();
    assert_eq!(split.document().entities.len(), 5);
    assert_eq!(split.selected().len(), 3);
    println!("multi_cutter_split_entities=5");

    let mut trim = controller();
    let horizontal = draw_line(&mut trim, [-10.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
    let left = draw_line(&mut trim, [-5.0, -5.0, 0.0], [-5.0, 5.0, 0.0]);
    let right = draw_line(&mut trim, [5.0, -5.0, 0.0], [5.0, 5.0, 0.0]);
    select(&mut trim, &[horizontal]);
    trim.top11_invoke(Top11Action::Trim).unwrap();
    trim.top11_accept_object(left).unwrap();
    trim.top11_accept_object(right).unwrap();
    trim.submit("").unwrap();
    trim.accept_world_point([-9.0, 0.0, 0.0]).unwrap();
    assert_eq!(trim.document().entities.len(), 4);
    println!("multi_cutter_trim_entities=4");
}

#[test]
fn m03_03_08_004_a_surface_can_be_the_cutting_object() {
    let mut doc = Document::new("surface cutter");
    let line_entity = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [-10.0, 0.0, 0.0],
            end: [10.0, 0.0, 0.0],
            domain: [0.0, 20.0],
        }),
        None,
    );
    let line_id = line_entity.id;
    let surface = NurbsSurface {
        degree: [1, 1],
        size: [2, 2],
        poles: vec![
            [0.0, -5.0, -5.0],
            [0.0, 5.0, -5.0],
            [0.0, -5.0, 5.0],
            [0.0, 5.0, 5.0],
        ],
        weights: vec![1.0; 4],
        knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    };
    let surface_entity = Entity::new("surface", Geometry::Surface(surface), None);
    let surface_id = surface_entity.id;
    doc.entities.insert(line_id, line_entity);
    doc.entities.insert(surface_id, surface_entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    select(&mut c, &[line_id]);
    c.top11_invoke(Top11Action::Split).unwrap();
    c.top11_accept_object(surface_id).unwrap();
    c.submit("").unwrap();
    assert_eq!(c.document().entities.len(), 3);
    assert_eq!(c.selected().len(), 2);
    println!("surface_cutter_entities=3");
}

#[test]
fn m03_03_08_003_three_point_plane_and_non_rigid_mirror() {
    let mut c = controller();
    let id = draw_line(&mut c, [1.0, 0.0, 0.0], [4.0, 0.0, 0.0]);
    select(&mut c, &[id]);
    c.submit("Mirror 3Point").unwrap();
    c.accept_world_point([0.0, 0.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 0.0, 1.0]).unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let copy = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { start, .. }) = &copy.geometry else {
        panic!("three-point mirror did not copy a line");
    };
    assert!((start[0] + 1.0).abs() < 1e-9, "{start:?}");
    println!("three_point_mirror_start_x_mm={:.12}", start[0]);

    let mut doc = Document::new("non-rigid");
    let mut entity = Entity::new(
        "line",
        Geometry::Curve(om9_core::geometry::Curve::Line {
            start: [0.0; 3],
            end: [1.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    entity.placement = Transform::scale([2.0, 1.0, 1.0]).unwrap();
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    select(&mut c, &[id]);
    c.top11_mirror(true).unwrap();
    c.accept_world_point([0.0, -1.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 1.0, 0.0]).unwrap();
    let copy = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Line { end, .. }) = &copy.geometry else {
        panic!("non-rigid mirror did not copy a line");
    };
    assert!((end[0] + 2.0).abs() < 1e-9, "{end:?}");
    println!("non_rigid_mirror_end_x_mm={:.12}", end[0]);
}

#[test]
fn m03_03_08_002_circle_and_surface_control_grips_reach_the_toolbar_adapter() {
    let mut doc = Document::new("grips");
    let circle = Entity::new(
        "circle",
        Geometry::Curve(om9_core::geometry::Curve::Circle {
            center: [0.0; 3],
            x: [1.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0],
            radius: 2.0,
            domain: [0.0, std::f64::consts::TAU],
            angle_domain: None,
        }),
        None,
    );
    let circle_id = circle.id;
    let surface = Entity::new(
        "surface",
        Geometry::Surface(NurbsSurface {
            degree: [1, 1],
            size: [2, 2],
            poles: vec![
                [0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
            ],
            weights: vec![1.0; 4],
            knots: [vec![0.0, 0.0, 1.0, 1.0], vec![0.0, 0.0, 1.0, 1.0]],
            domain: [[0.0, 1.0], [0.0, 1.0]],
            periodic: [false, false],
            reversed: false,
        }),
        None,
    );
    let surface_id = surface.id;
    doc.entities.insert(circle_id, circle);
    doc.entities.insert(surface_id, surface);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    select(&mut c, &[circle_id]);
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    assert_eq!(handles.len(), 5);
    c.top11_move_grip(handles[1], [5.0, 0.0, 0.0]).unwrap();
    let Geometry::Curve(om9_core::geometry::Curve::Circle { radius, .. }) =
        &c.document().entities[&circle_id].geometry
    else {
        panic!("circle grip lost the circle");
    };
    assert!((radius - 5.0).abs() < 1e-9);
    println!("circle_grip_radius={radius:.12}");
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    select(&mut c, &[surface_id]);
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    assert_eq!(handles.len(), 4);
    println!("surface_control_grips={}", handles.len());
    c.top11_invoke(Top11Action::ControlPoints).unwrap();
    c.top11_invoke(Top11Action::EditPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    assert_eq!(handles.len(), 4);
    let target = [
        handles[3].point[0] + 1.0,
        handles[3].point[1] + 1.0,
        handles[3].point[2],
    ];
    c.top11_move_grip(handles[3], target).unwrap();
    println!("surface_edit_grips={}", handles.len());
}

#[test]
fn m03_03_08_002_surface_edit_grips_keep_periodic_and_reversed_records() {
    let limits = Limits::default();
    let cylinder = om9_core::surface_kernel::cylinder(
        [0.0; 3],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        5.0,
        10.0,
        &limits,
    )
    .unwrap();
    let mut reversed = cylinder.clone();
    reversed.reversed = true;
    let mut periodic = cylinder;
    periodic.periodic = [true, false];
    let mut doc = Document::new("surface-edit");
    let reversed_entity = Entity::new("reversed", Geometry::Surface(reversed), None);
    let reversed_id = reversed_entity.id;
    let periodic_entity = Entity::new("periodic", Geometry::Surface(periodic), None);
    let periodic_id = periodic_entity.id;
    doc.entities.insert(reversed_id, reversed_entity);
    doc.entities.insert(periodic_id, periodic_entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();

    select(&mut c, &[reversed_id]);
    c.top11_invoke(Top11Action::EditPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    let target = [
        handles[10].point[0] + 1.0,
        handles[10].point[1],
        handles[10].point[2],
    ];
    c.top11_move_grip(handles[10], target).unwrap();
    let Geometry::Surface(edited) = &c.document().entities[&reversed_id].geometry else {
        panic!("reversed edit lost the surface");
    };
    assert!(edited.reversed);
    assert_eq!(edited.periodic, [false, false]);
    println!("surface_edit_reversed=true");

    c.top11_invoke(Top11Action::EditPoints).unwrap();
    select(&mut c, &[periodic_id]);
    c.top11_invoke(Top11Action::EditPoints).unwrap();
    let handles = c.top11_grip_handles().unwrap();
    let target = [
        handles[0].point[0],
        handles[0].point[1],
        handles[0].point[2] + 1.0,
    ];
    c.top11_move_grip(handles[0], target).unwrap();
    let Geometry::Surface(edited) = &c.document().entities[&periodic_id].geometry else {
        panic!("periodic edit lost the surface");
    };
    assert!(edited.periodic[0]);
    println!("surface_edit_periodic=true");
}

fn large_surface(size: usize) -> NurbsSurface {
    let degree = 3usize;
    let mut knots = vec![0.0; degree + 1];
    let spans = size - degree;
    for step in 1..spans {
        knots.push(step as f64 / spans as f64);
    }
    knots.extend(std::iter::repeat_n(1.0, degree + 1));
    let mut poles = Vec::with_capacity(size * size);
    for i in 0..size {
        for j in 0..size {
            let u = i as f64 / (size - 1) as f64;
            let v = j as f64 / (size - 1) as f64;
            poles.push([
                u * 100.0,
                v * 100.0,
                (u * 8.0).sin() * (v * 8.0).cos() * 5.0,
            ]);
        }
    }
    NurbsSurface {
        degree: [degree, degree],
        size: [size, size],
        weights: vec![1.0; size * size],
        poles,
        knots: [knots.clone(), knots],
        domain: [[0.0, 1.0], [0.0, 1.0]],
        periodic: [false, false],
        reversed: false,
    }
}

#[test]
fn m03_03_08_002_surface_edit_grid_is_fast_through_the_adapter() {
    let mut doc = Document::new("surface-perf");
    let entity = Entity::new("large", Geometry::Surface(large_surface(96)), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    select(&mut c, &[id]);
    c.top11_invoke(Top11Action::EditPoints).unwrap();
    let started = std::time::Instant::now();
    let handles = c.top11_grip_handles().unwrap();
    let handles_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(handles.len(), 96 * 96);
    assert!(
        handles_ms < 1500.0,
        "adapter handles took {handles_ms:.3} ms"
    );
    let started = std::time::Instant::now();
    let target = [
        handles[48 * 96 + 48].point[0] + 1.0,
        handles[48 * 96 + 48].point[1],
        handles[48 * 96 + 48].point[2],
    ];
    c.top11_move_grip(handles[48 * 96 + 48], target).unwrap();
    let move_ms = started.elapsed().as_secs_f64() * 1000.0;
    assert!(move_ms < 500.0, "adapter move took {move_ms:.3} ms");
    println!("surface_edit_handles_96_ms={handles_ms:.3}");
    println!("surface_edit_move_96_ms={move_ms:.3}");
}
