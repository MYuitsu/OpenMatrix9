// SPDX-License-Identifier: LGPL-2.1-or-later
use om9_app::session::{
    Controller,
    measure::{self, MeasureUnit},
};
use om9_core::{
    budget::Limits,
    geometry::{Curve, Geometry, Transform},
    model::{Definition, Document, Entity, Extension, GEM_EXTENSION_SLOT},
    surface_kernel,
};
fn controller(entities: Vec<Entity>) -> Controller {
    let mut d = Document::new("extended measure");
    for e in entities {
        d.entities.insert(e.id, e);
    }
    Controller::new(d, Limits::default()).unwrap()
}
fn enter(c: &mut Controller, v: &[&str]) {
    for s in v {
        c.submit(s).unwrap_or_else(|e| panic!("{s}: {e}"));
    }
}
#[test]
fn dynamic_ortho_and_continuous_distance_are_read_only() {
    let mut c = controller(vec![]);
    let before = c.document().clone();
    enter(
        &mut c,
        &["Distance", "Continuous=Yes", "Ortho=Yes", "0,0,0"],
    );
    let (label, wire) = c.measure_preview([3., 4., 0.]).unwrap().unwrap();
    assert_eq!(label, "Distance = 4 mm");
    assert_eq!(wire[0][1], [0., 4., 0.]);
    assert_eq!(c.document(), &before);
    enter(&mut c, &["3,4,0"]);
    assert!(c.is_running());
    assert_eq!(c.last_measurement(), Some("Distance = 4 mm"));
    enter(&mut c, &["3,4,0"]);
    assert_eq!(c.last_measurement(), Some("Distance = 3 mm"));
    enter(&mut c, &["Cancel"]);
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    enter(
        &mut c,
        &["Angle", "Ortho=Yes", "0,0,0", "3,4,0", "10,10,0", "14,12,0"],
    );
    assert_eq!(
        c.last_measurement(),
        Some("Angle = 90 deg"),
        "the second angle line has an independent start point"
    );
}
#[test]
fn radius_on_an_ellipse_is_pointwise_and_a_line_has_no_finite_curvature() {
    let circle = Curve::Circle {
        center: [0.; 3],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 1.,
        domain: [0., std::f64::consts::TAU],
        angle_domain: None,
    };
    let ellipse = om9_core::top11::bake_curve(
        &circle,
        Transform([
            [4., 0., 0., 0.],
            [0., 2., 0., 0.],
            [0., 0., 1., 0.],
            [0., 0., 0., 1.],
        ]),
    )
    .unwrap();
    let e = Entity::new("ellipse", Geometry::Curve(ellipse), None);
    let id = e.id;
    let mut c = controller(vec![e]);
    c.select_objects(&[id]).unwrap();
    assert!((c.curvature_at_fraction(0.).unwrap().value - 1.).abs() < 1e-8);
    assert!((c.curvature_at_fraction(0.25).unwrap().value - 8.).abs() < 1e-8);
    enter(&mut c, &["Radius", "Station=0.25"]);
    assert!(c.last_measurement().unwrap().contains("8 mm"));
    let line = Curve::Line {
        start: [0.; 3],
        end: [1., 0., 0.],
        domain: [0., 1.],
    };
    assert!(om9_core::measure_geometry::curvature_radius(&line, 0.5, &Limits::default()).is_err());
    let kink = Curve::Nurbs(om9_core::geometry::NurbsCurve {
        degree: 2,
        poles: vec![
            [0., 0., 0.],
            [1., 1., 0.],
            [2., 0., 0.],
            [3., -1., 0.],
            [4., 0., 0.],
        ],
        weights: vec![1.; 5],
        knots: vec![0., 0., 0., 0.5, 0.5, 1., 1., 1.],
        domain: [0., 1.],
        periodic: false,
        reversed: false,
        seam: None,
    });
    assert!(
        om9_core::measure_geometry::curvature_radius(&kink, 0.5, &Limits::default()).is_err(),
        "opposite one-sided curvature is not a radius"
    );
}
#[test]
fn isocurve_and_surface_area_use_world_placement() {
    let surface = surface_kernel::plane(
        [0.; 3],
        [1., 0., 0.],
        [0., 1., 0.],
        10.,
        20.,
        &Limits::default(),
    )
    .unwrap();
    let mut e = Entity::new("surface", Geometry::Surface(surface), None);
    e.placement = Transform([
        [2., 0., 0., 3.],
        [0., 3., 0., 4.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]);
    let id = e.id;
    let mut c = controller(vec![e]);
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    let area = c.measure_selected_integral(measure::AREA).unwrap();
    assert_eq!(area.unit, MeasureUnit::SquareMillimetre);
    assert!((area.value - 1200.).abs() < 1e-6);
    enter(&mut c, &["Length", "U=0.5", ""]);
    assert!(c.last_measurement().unwrap().contains("20 mm"));
    assert_eq!(c.document(), &before);
    assert!(c.measure_selected_integral(measure::VOLUME).is_err());
}
#[test]
fn summary_groups_instances_and_counts_only_placed_explicit_carats() {
    let mut gem = Entity::new("gem", Geometry::Point([0.; 3]), None);
    gem.extensions.insert(
        GEM_EXTENSION_SLOT.into(),
        Extension::new(1, r#"{"carat":0.75}"#).unwrap(),
    );
    let def = Definition::new("gem definition", vec![gem]);
    let mut d = Document::new("summary");
    let id = def.id;
    d.definitions.insert(id, def);
    for _ in 0..2 {
        let e = Entity::new("placed gem", Geometry::Instance(id), None);
        d.entities.insert(e.id, e);
    }
    let mut unknown = Entity::new("unknown gem", Geometry::Point([0.; 3]), None);
    unknown.extensions.insert(
        GEM_EXTENSION_SLOT.into(),
        Extension::new(1, r#"{"shape":"round"}"#).unwrap(),
    );
    d.entities.insert(unknown.id, unknown);
    let c = Controller::new(d, Limits::default()).unwrap();
    let s = c.document_summary();
    assert_eq!(s.total, 3);
    assert_eq!(s.definitions, 1);
    assert_eq!(s.gems, 3);
    assert_eq!(s.known_carats, 1.5);
    assert_eq!(s.gems_without_weight, 1);
    assert_eq!(s.by_definition.values().sum::<u64>(), 2);
    assert_eq!(s.by_layer.values().sum::<u64>(), 3);
}
#[test]
#[cfg(feature = "occt-brep")]
fn canonical_brep_edges_area_volume_and_object_distance_have_numeric_oracles() {
    use om9_kernel::creation::{Primitive, primitive};
    let b = primitive(
        Primitive::Box {
            x_mm: 2.,
            y_mm: 3.,
            z_mm: 4.,
        },
        0.1,
        &Limits::default(),
    )
    .unwrap();
    let e = Entity::new("box", Geometry::Brep(b), None);
    let id = e.id;
    let mut point = Entity::new("point", Geometry::Point([0.; 3]), None);
    point.placement = Transform::translation([5., 1., 2.]);
    let p = point.id;
    let mut c = controller(vec![e, point]);
    c.select_objects(&[id]).unwrap();
    let before = c.document().clone();
    assert!((c.measure_selected_length().unwrap().value - 36.).abs() < 1e-7);
    assert!((c.measure_selected_integral(measure::AREA).unwrap().value - 52.).abs() < 1e-7);
    assert!((c.measure_selected_integral(measure::VOLUME).unwrap().value - 24.).abs() < 1e-7);
    c.select_objects(&[id, p]).unwrap();
    enter(&mut c, &["Distance", "Objects"]);
    assert!(c.last_measurement().unwrap().ends_with("3 mm"));
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().undo_len(), 0);
    c.clear_selection();
    let edges = match &c.document().entities[&id].geometry {
        Geometry::Brep(b) => b
            .display
            .edges
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let exact_line_length = b
                    .display
                    .segments
                    .iter()
                    .filter(|s| s[2] as usize == index)
                    .map(|s| {
                        om9_core::measure::distance_mm(
                            b.display.points[s[0] as usize],
                            b.display.points[s[1] as usize],
                        )
                        .unwrap()
                    })
                    .sum::<f64>();
                (*id, exact_line_length)
            })
            .collect::<Vec<_>>(),
        _ => unreachable!(),
    };
    for (edge, expected) in edges {
        c.clear_subselection();
        c.select_subobject(id, om9_scene::scene::Subelement::BrepEdge(edge))
            .unwrap();
        assert!(
            (c.measure_selected_length().unwrap().value - expected).abs() < 1e-7,
            "native edge ordinal must match the selected edge UUID"
        );
    }
    let mut transformed = before.clone();
    transformed.entities.get_mut(&id).unwrap().placement = Transform([
        [2., 0., 0., 7.],
        [0., 3., 0., 9.],
        [0., 0., 4., 11.],
        [0., 0., 0., 1.],
    ]);
    let mut placed = Controller::new(transformed, Limits::default()).unwrap();
    placed.select_objects(&[id]).unwrap();
    assert!((placed.measure_selected_length().unwrap().value - 116.).abs() < 1e-7);
    assert!(
        (placed
            .measure_selected_integral(measure::AREA)
            .unwrap()
            .value
            - 488.)
            .abs()
            < 1e-7
    );
    assert!(
        (placed
            .measure_selected_integral(measure::VOLUME)
            .unwrap()
            .value
            - 576.)
            .abs()
            < 1e-7
    );
}
#[test]
fn placed_arc_keeps_its_trim_under_nonuniform_scale() {
    let arc = Curve::Circle {
        center: [3., 4., 0.],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 2.,
        domain: [0., 1.],
        angle_domain: Some([0., std::f64::consts::FRAC_PI_2]),
    };
    let transform = Transform([
        [2., 0., 0., 7.],
        [0., 3., 0., 9.],
        [0., 0., 1., 0.],
        [0., 0., 0., 1.],
    ]);
    let c = om9_core::top11::bake_curve(&arc, transform).unwrap();
    for (f, expected) in [(0., [17., 21., 0.]), (1., [13., 27., 0.])] {
        let p = om9_core::curve_ops::point_at(&c, f, &Limits::default()).unwrap();
        for i in 0..3 {
            assert!((p[i] - expected[i]).abs() < 1e-8);
        }
    }
    assert!(!om9_core::curve_ops::is_closed(&c, 1e-8, &Limits::default()).unwrap());
}
#[test]
#[cfg(feature = "occt-brep")]
fn object_distance_uses_trimmed_curves_and_finite_surface_domains() {
    let l = Limits::default();
    let surface = surface_kernel::plane([0.; 3], [1., 0., 0.], [0., 1., 0.], 10., 20., &l).unwrap();
    let s = Geometry::Surface(surface);
    let p = Geometry::Point([3., 4., 5.]);
    let d =
        om9_kernel::measurement::distance(&s, Transform::identity(), &p, Transform::identity(), &l)
            .unwrap();
    assert!((d.millimetres - 5.).abs() < 1e-8);
    let arc = Geometry::Curve(Curve::Circle {
        center: [0.; 3],
        x: [1., 0., 0.],
        y: [0., 1., 0.],
        radius: 2.,
        domain: [0., 1.],
        angle_domain: Some([0., std::f64::consts::FRAC_PI_2]),
    });
    let p = Geometry::Point([-2., 0., 0.]);
    let d = om9_kernel::measurement::distance(
        &arc,
        Transform::identity(),
        &p,
        Transform::identity(),
        &l,
    )
    .unwrap();
    assert!((d.millimetres - 8_f64.sqrt()).abs() < 1e-7);
}
