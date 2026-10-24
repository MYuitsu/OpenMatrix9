#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

fn controller_with_lines() -> Controller {
    let mut doc = Document::new("blend");
    let first = om9_core::curve_build::polyline(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]).unwrap();
    let second = om9_core::curve_build::polyline(&[[0.0, 10.0, 0.0], [10.0, 10.0, 0.0]]).unwrap();
    for (name, curve) in [("a", first), ("b", second)] {
        let entity = Entity::new(name, Geometry::Curve(curve), None);
        doc.entities.insert(entity.id, entity);
    }
    Controller::new(doc, Limits::default()).unwrap()
}

#[test]
fn blend_curve_bridges_two_curve_ends() {
    let mut c = controller_with_lines();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("Blend").unwrap();
    c.submit("").unwrap();
    let blend = c
        .document()
        .entities
        .values()
        .find(|entity| entity.label == "Blend")
        .unwrap();
    assert!(matches!(
        blend.geometry,
        Geometry::Curve(om9_core::geometry::Curve::Nurbs(_))
    ));
}


#[test]
fn blend_accepts_tangent_scale_options() {
    let mut c = controller_with_lines();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("Blend").unwrap();
    c.submit("Scale1=2").unwrap();
    c.submit("Scale2=0.5").unwrap();
    c.submit("").unwrap();
    assert!(c
        .document()
        .entities
        .values()
        .any(|entity| entity.label == "Blend"));
}


#[test]
fn adjustable_blend_crv_dispatches_to_the_same_builder() {
    let mut c = controller_with_lines();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("BlendCrv").unwrap();
    c.submit("").unwrap();
    assert!(c
        .document()
        .entities
        .values()
        .any(|entity| entity.label == "Blend"));
}


#[test]
fn match_uses_the_blend_builder_with_the_match_label() {
    let mut c = controller_with_lines();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("Match").unwrap();
    c.submit("").unwrap();
    assert!(c
        .document()
        .entities
        .values()
        .any(|entity| entity.label == "Match"));
}


#[test]
fn blend_exposes_scale_tokens() {
    let mut c = controller_with_lines();
    let ids: Vec<_> = c.document().entities.keys().copied().collect();
    c.select_objects(&ids).unwrap();
    c.submit("Blend").unwrap();
    for expected in ["Scale1=", "Scale2="] {
        assert!(c
            .blend_option_tokens()
            .iter()
            .any(|(_, command)| *command == expected));
    }
}
