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

#[test]
fn rotate_copy_secondary_survives_post_selection() {
    use om9_core::top11::Top11Action;
    let mut doc = Document::with_template_tolerance("Rotate Copy");
    let point = Entity::new("point", Geometry::Point([2., 0., 0.]), None);
    let id = point.id;
    doc.entities.insert(id, point);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.top11_alternate(Top11Action::Rotate).unwrap();
    c.top11_accept_object(id).unwrap();
    c.submit("").unwrap();
    c.submit("W0,0,0").unwrap();
    c.submit("90").unwrap();
    assert_eq!(c.document().entities.len(), 2);
    c.undo().unwrap();
    assert_eq!(c.document().entities.len(), 1);
}

#[test]
fn trim_options_compute_fractions_on_the_original_line() {
    use om9_core::geometry::Curve;
    for (option, start, end) in [
        ("ExtendLines=Yes", [4., 3., 0.], [4., 6., 0.]),
        ("ApparentIntersections=Yes", [4., -2., 5.], [4., 2., 5.]),
    ] {
        let mut doc = Document::with_template_tolerance("Trim options");
        let line = |a, b| Entity::new("line", Geometry::Curve(Curve::Line { start: a, end: b, domain: [0.,1.] }), None);
        let target = line([0.;3], [10.,0.,0.]);
        let cutter = line(start, end);
        let ids = [target.id, cutter.id];
        doc.entities.insert(ids[0], target);
        doc.entities.insert(ids[1], cutter);
        let mut c = Controller::new(doc, Limits::default()).unwrap();
        c.activate_view("Looking Down").unwrap();
        c.select_objects(&ids).unwrap();
        let before = c.document().clone();
        c.submit(&format!("Trim {option}")).unwrap();
        c.top11_trim_many_accept_pick(ids[0], [2.,0.,0.]).unwrap();
        c.submit("").unwrap();
        let Geometry::Curve(Curve::Line { start, end, .. }) = &c.document().entities[&ids[0]].geometry else { panic!("retained line"); };
        assert!((start[0]-4.).abs()<1e-7 && (end[0]-10.).abs()<1e-7);
        assert_eq!((start[2],end[2]), (0.,0.));
        c.undo().unwrap();
        assert_eq!(c.document(), &before);
    }
}

#[test]
fn explode_dimension_keeps_editable_text_then_explodes_the_glyph() {
    use om9_core::{annotation::{Annotation, Kind, SLOT, Style}, top11::Top11Action};
    let limits = Limits::default();
    let annotation = Annotation { kind:Kind::Aligned, points:vec![[0.;3],[10.,0.,0.]],
        text_point:[5.,4.,0.], text:Some("O".into()), x:[1.,0.,0.], y:[0.,1.,0.],
        rotation_degrees:0., style:Style::default() };
    let mut dim = Entity::new("dim", annotation.geometry(&limits).unwrap(), None);
    dim.extensions.insert(SLOT.into(),annotation.extension(&limits).unwrap());
    let mut doc = Document::with_template_tolerance("dimension");
    let id = dim.id;
    doc.entities.insert(id,dim);
    let mut c = Controller::new(doc,limits).unwrap();
    let before = c.document().clone();
    c.select_objects(&[id]).unwrap();
    c.top11_invoke(Top11Action::Explode).unwrap();
    let txt = c.document().entities.values().find(|e| Annotation::read(e).is_ok_and(|a| a.kind==Kind::Text)).unwrap().id;
    c.select_objects(&[txt]).unwrap();
    c.submit("Explode").unwrap();
    assert!(c.document().entities.values().all(|e| matches!(e.geometry,Geometry::Curve(_)) && !e.extensions.contains_key(SLOT)));
    assert_eq!(c.selected().len(),2, "inner and outer contours");
    for id in c.selected() {
        let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
            &c.document().entities[id].geometry else { panic!("closed glyph contour"); };
        assert!(segments.iter().any(|c| matches!(c, om9_core::geometry::Curve::Nurbs(n) if n.degree >= 2)));
    }
    c.undo().unwrap(); c.undo().unwrap();
    assert_eq!(c.document(), &before);
}

#[test]
fn exploding_a_cage_releases_objects_and_keeps_edges_and_faces() {
    let limits = Limits::default();
    let corners = om9_core::cage::box_corners([0.;3], [4.,5.,6.]);
    let mesh = om9_core::cage::box_mesh(corners);
    let object = Entity::new("caged mesh", Geometry::Mesh(mesh), None);
    let id = object.id;
    let mut document = Document::with_template_tolerance("cage");
    document.entities.insert(id,object);
    let mut c = Controller::new(document,limits).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("Cage").unwrap();
    let before = c.document().clone();
    let cage = *before.entities.keys().find(|key| **key!=id).unwrap();
    c.select_objects(&[cage]).unwrap();
    c.submit("Explode").unwrap();
    let after = c.document();
    assert_eq!(after.entities.values().filter(|e| matches!(e.geometry, Geometry::Surface(_))).count(), 6);
    assert_eq!(after.entities.values().filter(|e| matches!(e.geometry, Geometry::Curve(_))).count(), 12);
    assert!(after.entities.values().all(|e| !e.extensions.contains_key(om9_core::model::CAGE_EXTENSION_SLOT)));
    assert_eq!(after.entities[&id].geometry, before.entities[&id].geometry);
    c.undo().unwrap();
    assert_eq!(c.document(), &before);
}
