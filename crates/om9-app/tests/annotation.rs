// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::{Controller, InvocationOrigin, annotation::FEATURES};
use om9_core::{
    annotation::{Annotation, Kind},
    budget::Limits,
    geometry::{Curve, Geometry},
    model::{Document, Entity},
};
fn controller() -> Controller {
    Controller::new(Document::new("annotations"), Limits::default()).unwrap()
}
fn enter(c: &mut Controller, values: &[&str]) {
    for value in values {
        c.submit(value).unwrap_or_else(|e| panic!("{value}: {e}"));
    }
}
fn last(c: &Controller) -> (om9_core::identity::ObjectId, Annotation) {
    let e = c
        .document()
        .entities
        .values()
        .find(|e| e.extensions.contains_key(om9_core::annotation::SLOT))
        .unwrap();
    (e.id, Annotation::read(e).unwrap())
}
#[test]
fn all_thirteen_catalog_ids_have_distinct_icons_and_one_dispatcher() {
    for feature in FEATURES {
        assert!(Controller::availability(feature).is_ok());
        assert!(
            om9_app::menu_catalog::MEASURE
                .iter()
                .any(|n| om9_app::icons::feature_for(n) == Some(feature)),
            "{feature}"
        );
    }
    assert_eq!(Controller::resolve_alias("Dim"), Some(FEATURES[0]));
    assert_eq!(Controller::resolve_alias("DimVertical"), Some(FEATURES[1]));
    assert_eq!(Controller::resolve_alias("TextBlock"), Some(FEATURES[8]));
    assert_ne!(Controller::resolve_alias("Text"), Some(FEATURES[8]));
}
#[test]
fn five_dimension_families_have_independent_value_oracles_and_one_undo() {
    for (index, inputs, expected) in [
        (0, vec!["0,0,0", "3,4,0", "1,8,0"], 3.),
        (1, vec!["0,0,0", "3,4,0", "8,1,0"], 4.),
        (2, vec!["0,0,0", "1,0,0", "0,1,0", "3,3,0"], 90.),
        (3, vec!["Angle=90", "0,0,0", "3,4,0", "8,1,0"], 4.),
        (4, vec!["0,0,0", "3,4,0", "1,8,0"], 5.),
    ] {
        for origin in [InvocationOrigin::Menu, InvocationOrigin::Cmd] {
            let mut c = controller();
            c.dispatch(FEATURES[index], origin).unwrap();
            enter(&mut c, &inputs);
            assert!(!c.is_running());
            let (id, r) = last(&c);
            let layout = r.layout(&Limits::default()).unwrap();
            assert!((layout.value.unwrap() - expected).abs() < 1e-8);
            assert_eq!(c.coordinator().undo_len(), 1);
            assert!(om9_core::annotation::SLOT == "om9.annotation");
            c.undo().unwrap();
            assert!(!c.document().entities.contains_key(&id));
            c.redo().unwrap();
            assert_eq!(Annotation::read(&c.document().entities[&id]).unwrap(), r);
        }
    }
}
#[test]
fn diameter_and_radius_use_the_placed_circle_and_keep_source() {
    for (index, expected) in [(5, 10.), (6, 5.)] {
        let circle = Entity::new(
            "circle",
            Geometry::Curve(Curve::Circle {
                center: [2., 3., 0.],
                x: [1., 0., 0.],
                y: [0., 1., 0.],
                radius: 5.,
                domain: [0., std::f64::consts::TAU],
                angle_domain: None,
            }),
            None,
        );
        let id = circle.id;
        let mut d = Document::new("radial");
        d.entities.insert(id, circle);
        let mut c = Controller::new(d, Limits::default()).unwrap();
        c.select_objects(&[id]).unwrap();
        c.dispatch(FEATURES[index], InvocationOrigin::Menu).unwrap();
        enter(&mut c, &["12,12,0"]);
        assert_eq!(c.document().entities.len(), 2);
        let (_, r) = last(&c);
        assert_eq!(r.layout(&Limits::default()).unwrap().value, Some(expected));
    }
}
#[test]
fn text_and_leader_edit_text_without_flattening_semantics() {
    for index in [7, 8] {
        let mut c = controller();
        c.dispatch(FEATURES[index], InvocationOrigin::Menu).unwrap();
        if index == 7 {
            enter(&mut c, &["0,0,0", "3,4,0"]);
        }
        enter(&mut c, &["5,6,0", "Đá chủ 0.75 ct"]);
        let (id, r) = last(&c);
        assert_eq!(r.text.as_deref(), Some("Đá chủ 0.75 ct"));
        c.select_objects(&[id]).unwrap();
        enter(&mut c, &["gvEditText", "Đổi chữ"]);
        assert_eq!(last(&c).1.text.as_deref(), Some("Đổi chữ"));
        c.undo().unwrap();
        assert_eq!(last(&c).1, r);
    }
}
#[test]
fn edits_options_and_recenter_preserve_identity_and_are_atomic() {
    let mut c = controller();
    enter(&mut c, &["Dim", "0,0,0", "10,0,0", "2,4,0"]);
    let (id, original) = last(&c);
    c.select_objects(&[id]).unwrap();
    enter(
        &mut c,
        &["DimEdit", "Point2=W20,0,0", "Position=W3,8,0", "Text=<> x2"],
    );
    assert_eq!(last(&c).1, original, "draft does not mutate");
    enter(&mut c, &[""]);
    let changed = last(&c).1;
    assert_eq!(changed.points[1], [20., 0., 0.]);
    assert_eq!(changed.text.as_deref(), Some("<> x2"));
    assert_eq!(last(&c).0, id);
    enter(
        &mut c,
        &[
            "DimOptions",
            "Height=4",
            "Arrow=2",
            "Precision=3",
            "Scale=2",
            "",
        ],
    );
    assert_eq!(last(&c).1.style.height_mm, 4.);
    c.dispatch(FEATURES[12], InvocationOrigin::Menu).unwrap();
    assert_eq!(last(&c).1.text_point, [10., 8., 0.]);
    c.undo().unwrap();
    assert_eq!(last(&c).1.text_point, [3., 8., 0.]);
    let doc = c.document().clone();
    enter(&mut c, &["DimEdit"]);
    assert!(c.submit("Precision=99").is_err());
    enter(&mut c, &["Cancel"]);
    assert_eq!(c.document(), &doc);
}
#[test]
fn preview_cancel_invalid_witness_and_native_archive_roundtrip() {
    let mut c = controller();
    enter(&mut c, &["DimAligned", "0,0,0", "3,4,0"]);
    let before = c.document().clone();
    let (_, preview) = c.annotation_preview([6., 7., 0.]).unwrap().unwrap();
    assert_eq!(preview.value, Some(5.));
    assert_eq!(c.document(), &before);
    enter(&mut c, &["6,7,0"]);
    let (_, r) = last(&c);
    assert_eq!(r.layout(&Limits::default()).unwrap(), preview);
    let path = std::env::temp_dir().join(format!("om9-annotation-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(
        &path,
        c.document(),
        &Limits::default(),
        &om9_core::transaction::Cancellation::default(),
    )
    .unwrap();
    let loaded = om9_archive::Archive::open(&path, &Limits::default()).unwrap();
    assert_eq!(&loaded, c.document());
    std::fs::remove_file(path).unwrap();
    c.clear_selection();
    enter(&mut c, &["TextBlock", "1,2,0", "Cancel"]);
    assert_eq!(c.document().entities.len(), 1);
    enter(&mut c, &["Dim", "1,1,0", "1,1,0"]);
    assert!(c.submit("2,2,0").is_err());
    assert!(c.is_running());
    enter(&mut c, &["Cancel"]);
}
#[test]
fn session_style_defaults_and_transformed_values_are_observable() {
    let mut c = controller();
    enter(&mut c, &["DimOptions", "Height=5", ""]);
    assert_eq!(c.coordinator().undo_len(), 0);
    enter(&mut c, &["DimAligned", "0,0,0", "3,4,0", "3,8,0"]);
    let (_, r) = last(&c);
    assert_eq!(r.style.height_mm, 5.);
    assert_eq!(r.kind, Kind::Aligned);
    let placed = r
        .placed(om9_core::geometry::Transform([
            [2., 0., 0., 7.],
            [0., 2., 0., 9.],
            [0., 0., 2., 0.],
            [0., 0., 0., 1.],
        ]))
        .unwrap();
    assert_eq!(placed.layout(&Limits::default()).unwrap().value, Some(10.));
}
#[test]
fn scene_displays_independent_dimension_strokes_without_connecting_gaps() {
    let mut c = controller();
    enter(&mut c, &["Dim", "0,0,0", "10,0,0", "4,7,0"]);
    let (_, record) = last(&c);
    let layout = record.layout(&Limits::default()).unwrap();
    let object = &c.scene().unwrap().objects[0];
    let om9_scene::scene::RenderBuffers::Mesh {
        vertices,
        edges,
        triangles,
    } = object.buffers.as_ref()
    else {
        panic!("independent strokes")
    };
    assert!(triangles.is_empty());
    assert_eq!(edges.len(), layout.lines.len());
    for ([i, j], [a, b]) in edges.iter().zip(layout.lines) {
        assert_eq!(vertices[*i as usize], a);
        assert_eq!(vertices[*j as usize], b);
    }
}
