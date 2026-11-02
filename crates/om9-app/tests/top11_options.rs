use om9_app::command_options::{CommandOptionEffect, CommandOptionInterface};
use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn duplicate_flags_are_exposed_and_toggle_owner_state() {
    let mut doc = Document::new("duplicate options");
    let entity = Entity::new("point", Geometry::Point([0.0, 0.0, 0.0]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Duplicate").unwrap();
    let vertical = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Vertical")
        .expect("Vertical option");
    assert_eq!(
        c.activate_command_option(vertical.id).unwrap(),
        CommandOptionEffect::Applied
    );
    assert!(c.prompt().contains("pick"));
    assert!(
        c.command_options()
            .iter()
            .any(|option| option.id.name() == "Vertical")
    );
}

#[test]
fn duplicate_in_place_option_uses_one_undo_and_redo() {
    let mut doc = Document::new("duplicate in place option");
    let entity = Entity::new("point", Geometry::Point([2., 3., 4.]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Copy").unwrap();
    let option = c
        .command_options()
        .into_iter()
        .find(|item| item.id.name() == "InPlace")
        .expect("InPlace option");
    c.activate_command_option(option.id).unwrap();
    assert_eq!(c.document().entities.len(), 2);
    assert!(
        c.document()
            .entities
            .values()
            .all(|entity| entity.geometry == Geometry::Point([2., 3., 4.]))
    );
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn mirror_copy_flag_uses_the_shared_option_entry() {
    use om9_core::geometry::Curve;
    let mut doc = Document::new("mirror options");
    let entity = Entity::new(
        "line",
        Geometry::Curve(Curve::Line {
            start: [1.0, 0.0, 0.0],
            end: [4.0, 0.0, 0.0],
            domain: [0.0, 1.0],
        }),
        None,
    );
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Mirror").unwrap();
    let copy = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Copy")
        .expect("Copy option");
    assert_eq!(
        c.activate_command_option(copy.id).unwrap(),
        CommandOptionEffect::Applied
    );
    c.accept_world_point([0.0, -5.0, 0.0]).unwrap();
    c.accept_world_point([0.0, 5.0, 0.0]).unwrap();
    assert_eq!(c.document().entities.len(), 1);
    let Geometry::Curve(Curve::Line { start, .. }) = &c.document().entities[&id].geometry else {
        panic!("mirrored line");
    };
    assert!((start[0] + 1.0).abs() < 1e-9, "{start:?}");
}

#[test]
fn move_vertical_uses_active_cplane_normal_and_undo() {
    let mut doc = Document::new("move vertical");
    let entity = Entity::new("point", Geometry::Point([0.0; 3]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.set_cplane(om9_scene::camera::CPlane::new([0.0; 3], [0., 1., 0.], [0., 0., 1.]).unwrap())
        .unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Move").unwrap();
    let option = c
        .command_options()
        .into_iter()
        .find(|o| o.id.name() == "Vertical")
        .expect("Vertical option");
    c.activate_command_option(option.id).unwrap();
    c.accept_world_point([1., 2., 3.]).unwrap();
    c.accept_world_point([-4., 8., 9.]).unwrap();
    let point = c.document().entities[&id].placement.point([0.; 3]).unwrap();
    assert!(
        (point[0] + 5.).abs() < 1e-9 && point[1].abs() < 1e-9 && point[2].abs() < 1e-9,
        "{point:?}"
    );
    c.undo().unwrap();
    assert_eq!(
        c.document().entities[&id].placement.point([0.; 3]).unwrap(),
        [0.; 3]
    );
}

#[test]
fn move_vertical_typed_world_points_preserve_both_signs_and_cancel() {
    for z in [-6.0, 6.0] {
        let mut doc = Document::new("move vertical typed");
        let entity = Entity::new("point", Geometry::Point([0.0; 3]), None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap();
        c.submit("Move").unwrap();
        c.submit("Vertical=Yes").unwrap();
        assert!(c.submit("not-a-point").is_err());
        assert!(c.is_running());
        c.submit("W1,2,3").unwrap();
        c.submit(&format!("W9,8,{}", 3.0 + z)).unwrap();
        let moved = c.document().entities[&id].placement.point([0.; 3]).unwrap();
        assert_eq!(moved, [0., 0., z]);
        c.undo().unwrap();
        c.submit("Move").unwrap();
        c.submit("Vertical=Yes").unwrap();
        c.cancel();
        assert_eq!(
            c.document().entities[&id].placement.point([0.; 3]).unwrap(),
            [0.; 3]
        );
    }
}

#[test]
fn rotate_copy_accepts_typed_center_and_angle_and_undo_redo() {
    let mut doc = Document::new("rotate copy");
    let entity = Entity::new("point", Geometry::Point([2., 0., 0.]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Rotate").unwrap();
    let copy = c
        .command_options()
        .into_iter()
        .find(|option| option.id.name() == "Copy")
        .expect("Copy option");
    c.activate_command_option(copy.id).unwrap();
    c.submit("W0,0,0").unwrap();
    c.submit("90").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    let original = &c.document().entities[&id];
    assert_eq!(
        original.placement.point([2., 0., 0.]).unwrap(),
        [2., 0., 0.]
    );
    let duplicate = c
        .document()
        .entities
        .values()
        .find(|entity| entity.id != id)
        .unwrap();
    let point = duplicate.placement.point([2., 0., 0.]).unwrap();
    assert!(
        point[0].abs() < 1e-9 && (point[1] - 2.).abs() < 1e-9,
        "{point:?}"
    );
    assert_eq!(c.coordinator().undo_len(), 1);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    c.redo().unwrap();
    assert_eq!(c.document().entities.len(), 2);
}

#[test]
fn rotate_reference_points_move_original_and_cancel_preserves_it() {
    let mut doc = Document::new("rotate reference");
    let entity = Entity::new("point", Geometry::Point([2., 0., 0.]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Rotate").unwrap();
    c.accept_world_point([0.; 3]).unwrap();
    c.accept_world_point([1., 0., 0.]).unwrap();
    c.accept_world_point([0., -1., 0.]).unwrap();
    let point = c.document().entities[&id]
        .placement
        .point([2., 0., 0.])
        .unwrap();
    assert!(
        point[0].abs() < 1e-9 && (point[1] + 2.).abs() < 1e-9,
        "{point:?}"
    );
    c.undo().unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Rotate").unwrap();
    c.submit("Copy=Yes").unwrap();
    c.cancel();
    assert_eq!(c.document().entities.len(), 1);
    assert_eq!(
        c.document().entities[&id]
            .placement
            .point([2., 0., 0.])
            .unwrap(),
        [2., 0., 0.]
    );
}

#[test]
fn rotate_copy_option_can_be_supplied_with_command() {
    let mut doc = Document::new("rotate copy command");
    let entity = Entity::new("point", Geometry::Point([1., 0., 0.]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Rotate Copy=Yes").unwrap();
    c.submit("W0,0,0").unwrap();
    c.submit("90").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn mirror_cplane_axes_use_the_shared_options_and_undo_redo() {
    for (axis, expected) in [("XAxis", [8., 3., 0.]), ("YAxis", [12., -3., 0.])] {
        let mut doc = Document::new("mirror cplane axes");
        let entity = Entity::new("point", Geometry::Point([12., 3., 0.]), None);
        let id = entity.id;
        doc.entities.insert(id, entity);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        c.set_cplane(
            om9_scene::camera::CPlane::new([10., 0., 0.], [0., 1., 0.], [-1., 0., 0.]).unwrap(),
        )
        .unwrap();
        c.select_objects(&[id]).unwrap();
        c.submit("Mirror").unwrap();
        c.submit("Copy=No").unwrap();
        let option = c
            .command_options()
            .into_iter()
            .find(|option| option.id.name() == axis)
            .expect("axis option");
        c.activate_command_option(option.id).unwrap();
        let Geometry::Point(point) = &c.document().entities[&id].geometry else {
            panic!("point")
        };
        for i in 0..3 {
            assert!((point[i] - expected[i]).abs() < 1e-9, "{point:?}");
        }
        assert_eq!(c.coordinator().undo_len(), 1);
        c.undo().unwrap();
        assert_eq!(
            c.document().entities[&id].geometry,
            Geometry::Point([12., 3., 0.])
        );
        c.redo().unwrap();
        let Geometry::Point(point) = &c.document().entities[&id].geometry else {
            panic!("point")
        };
        for i in 0..3 {
            assert!((point[i] - expected[i]).abs() < 1e-9, "{point:?}");
        }
    }
}

#[test]
fn mirror_axis_default_copies_and_cancel_is_non_mutating() {
    let mut doc = Document::new("mirror axis copy");
    let entity = Entity::new("point", Geometry::Point([1., 2., 0.]), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Mirror").unwrap();
    c.submit("XAxis").unwrap();
    assert_eq!(
        c.document().entities[&id].geometry,
        Geometry::Point([1., 2., 0.])
    );
    assert!(
        c.document()
            .entities
            .values()
            .any(|entity| entity.id != id && entity.geometry == Geometry::Point([1., -2., 0.]))
    );
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
    c.select_objects(&[id]).unwrap();
    c.submit("Mirror").unwrap();
    c.cancel();
    assert_eq!(c.document().entities.len(), 1);
}
