#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::budget::Limits;
use om9_core::geometry::Geometry;
use om9_core::model::{Document, Entity};

#[test]
fn cross_section_profiles_generate_multiple_sections() {
    let mut doc = Document::new("cross sections");
    let surface = om9_core::surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        30.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("cylinder", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.set_cplane_origin_rotation([0.0, 0.0, 15.0], 0.0)
        .unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("CrossSectionProfiles").unwrap();
    c.submit("Count=2").unwrap();
    c.submit("Spacing=5").unwrap();
    c.submit("").unwrap();
    let mut levels: Vec<i64> = c
        .document()
        .entities
        .values()
        .filter(|entity| entity.label == "Section")
        .filter_map(|entity| {
            let Geometry::Curve(om9_core::geometry::Curve::Polycurve { segments, .. }) =
                &entity.geometry
            else {
                return None;
            };
            let om9_core::geometry::Curve::Line { start, .. } = &segments[0] else {
                return None;
            };
            Some((start[2] * 100.0).round() as i64)
        })
        .collect();
    levels.sort_unstable();
    levels.dedup();
    assert_eq!(levels, vec![1250, 1750]);
}


#[test]
fn cross_section_exposes_count_and_spacing_tokens() {
    let mut doc = Document::new("cross token");
    let surface = om9_core::surface_kernel::cylinder(
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        10.0,
        30.0,
        &Limits::default(),
    )
    .unwrap();
    let entity = Entity::new("cylinder", Geometry::Surface(surface), None);
    let id = entity.id;
    doc.entities.insert(id, entity);
    let mut c = Controller::new(doc, Limits::default()).unwrap();
    c.select_objects(&[id]).unwrap();
    c.submit("CrossSectionProfiles").unwrap();
    let tokens = c.cross_section_option_tokens();
    for expected in ["Count=", "Spacing="] {
        assert!(tokens.iter().any(|(_, command)| *command == expected));
    }
}
