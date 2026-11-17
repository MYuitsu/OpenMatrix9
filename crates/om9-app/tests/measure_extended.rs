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
#[cfg(feature = "occt-brep")]
fn reopened_reordered_edge_length_measures_selected_edge() {
    use om9_kernel::creation::{primitive, Primitive};
    let limits = Limits::default();
    let shape = primitive(Primitive::Box { x_mm: 2., y_mm: 3., z_mm: 4. }, 0.1, &limits).unwrap();
    let mut display = shape.display().clone();
    let length = |index: usize| {
        let s = display.segments.iter().find(|s| s[2] as usize == index).unwrap();
        om9_core::measure::distance_mm(display.points[s[0] as usize], display.points[s[1] as usize]).unwrap()
    };
    let first = 0;
    let other = (1..display.edges.len()).find(|&index| (length(first) - length(index)).abs() > 0.5).unwrap();
    display.edges.swap(first, other);
    for entry in &mut display.face_edges { if entry[1] as usize == first { entry[1] = other as u32; } else if entry[1] as usize == other { entry[1] = first as u32; } }
    for entry in &mut display.segments { if entry[2] as usize == first { entry[2] = other as u32; } else if entry[2] as usize == other { entry[2] = first as u32; } }
    let segment = display.segments.iter().find(|entry| entry[2] as usize == first).unwrap();
    let expected = om9_core::measure::distance_mm(display.points[segment[0] as usize], display.points[segment[1] as usize]).unwrap();
    let edge = display.edges[first];
    let shape = om9_core::brep::Brep::from_native(shape.source().to_owned(), display, &limits).unwrap();
    let mut document = Document::new("reopened edge length");
    let entity = Entity::new("box", Geometry::Brep(shape), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-length-reordered-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    c.select_subobject(id, om9_scene::scene::Subelement::BrepEdge(edge)).unwrap();
    let measured = c.measure_selected_length().unwrap().value;
    assert!((measured - expected).abs() < 1e-7, "selected edge length {expected}, measured {measured}");
    c.measure_selection(measure::LENGTH).unwrap();
    assert_eq!(c.last_measurement(), Some("Length = 3 mm"));
}

#[test]
#[cfg(feature = "occt-brep")]
fn selected_length_checks_curved_source_and_rejects_wrong_display() {
    use om9_kernel::creation::{primitive, Primitive};
    let limits = Limits::default();
    let cylinder = primitive(Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.1, &limits).unwrap();
    let ring = (0..cylinder.topology().edges.len()).find(|&i| {
        om9_kernel::measurement::edge_length(&cylinder, Some(i), Transform::identity(), &limits)
            .is_ok_and(|length| (length - 4. * std::f64::consts::PI).abs() < 1e-6)
    }).unwrap();
    let edge = cylinder.display().edges[ring];
    let mut entity = Entity::new("cylinder", Geometry::Brep(cylinder.clone()), None);
    entity.placement = Transform::translation([11., 13., 17.]);
    let id = entity.id;
    let mut c = controller(vec![entity]);
    c.select_subobject(id, om9_scene::scene::Subelement::BrepEdge(edge)).unwrap();
    let before = c.document().clone();
    assert!((c.measure_selected_length().unwrap().value - 4. * std::f64::consts::PI).abs() < 1e-6);
    assert_eq!(c.document(), &before);

    let mut display = cylinder.display().clone();
    let segment = *display.segments.iter().find(|segment| segment[2] as usize == ring).unwrap();
    for point in [segment[0], segment[1]] { display.points[point as usize][2] += 100.; }
    let bad = om9_core::brep::Brep::from_native(cylinder.source().to_owned(), display, &limits).unwrap();
    let entity = Entity::new("bad display", Geometry::Brep(bad), None);
    let id = entity.id;
    let mut c = controller(vec![entity]);
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_subobject(id, om9_scene::scene::Subelement::BrepEdge(edge)).unwrap();
    let before = c.document().clone();
    assert!(c.measure_selected_length().is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
#[cfg(feature = "occt-brep")]
fn selected_length_rejects_mixed_segments_on_the_same_display_edge() {
    use om9_kernel::creation::{primitive, Primitive};
    let limits = Limits::default();
    let cylinder = primitive(Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, 0.1, &limits).unwrap();
    let rings: Vec<_> = (0..cylinder.topology().edges.len()).filter(|&i| {
        om9_kernel::measurement::edge_length(&cylinder, Some(i), Transform::identity(), &limits)
            .is_ok_and(|length| (length - 4. * std::f64::consts::PI).abs() < 1e-6)
    }).collect();
    assert_eq!(rings.len(), 2);
    let mut display = cylinder.display().clone();
    let edge = display.edges[rings[0]];
    let mut foreign = *display.segments.iter().find(|s| s[2] as usize == rings[1]).unwrap();
    foreign[2] = rings[0] as u32;
    display.segments.push(foreign);
    let shape = om9_core::brep::Brep::from_native(cylinder.source().to_owned(), display, &limits).unwrap();
    let entity = Entity::new("mixed display", Geometry::Brep(shape), None);
    let id = entity.id;
    let mut document = Document::new("mixed displayed edge");
    document.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-length-mixed-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut c = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    c.submit("Point").unwrap(); c.submit("1,2,3").unwrap(); c.undo().unwrap();
    c.select_subobject(id, om9_scene::scene::Subelement::BrepEdge(edge)).unwrap();
    let before = c.document().clone();
    assert!(c.measure_selected_length().is_err(), "a foreign displayed segment must not identify another source edge");
    assert!(c.measure_selection(measure::LENGTH).is_err());
    assert_eq!(c.document(), &before);
    assert_eq!(c.coordinator().redo_len(), 1);
}

#[test]
#[cfg(feature = "occt-brep")]
#[ignore = "manual debug timing probe; not a desktop performance test"]
fn selected_length_segment_verification_timing() {
    use om9_kernel::creation::{primitive, Primitive};
    use std::time::Instant;
    let limits = Limits::default();
    for deflection in [0.1, 0.01, 0.001, 0.00001] {
        let shape = primitive(Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, deflection, &limits).unwrap();
        let ring = (0..shape.topology().edges.len()).find(|&i| {
            om9_kernel::measurement::edge_length(&shape, Some(i), Transform::identity(), &limits)
                .is_ok_and(|length| (length - 4. * std::f64::consts::PI).abs() < 1e-6)
        }).unwrap();
        let display = shape.display();
        let segment_count = display.segments.iter().filter(|s| s[2] as usize == ring).count();
        let first = display.segments.iter().find(|s| s[2] as usize == ring).unwrap();
        let witness = [display.points[first[0] as usize], display.points[first[1] as usize]];
        let mut elapsed = Vec::new();
        let mut single_elapsed = Vec::new();
        for _ in 0..5 {
            let start = Instant::now();
            let single = om9_kernel::measurement::verified_edge_length(
                &shape, Transform::identity(), witness, deflection + 1e-6, 1e-6, &limits).unwrap();
            single_elapsed.push(start.elapsed().as_secs_f64() * 1000.);
            assert!((single - 4. * std::f64::consts::PI).abs() < 1e-6);
            let start = Instant::now();
            let measured = om9_kernel::measurement::verified_display_edge_length(
                &shape, Transform::identity(), &display.points, &display.segments, ring as u32,
                deflection + 1e-6, 1e-6, &limits).unwrap();
            assert!((measured - 4. * std::f64::consts::PI).abs() < 1e-6);
            elapsed.push(start.elapsed().as_secs_f64() * 1000.);
        }
        println!("deflection_mm={deflection} selected_segments={segment_count} single_ms={single_elapsed:?} all_ms={elapsed:?}");
    }
}

#[test]
#[cfg(feature = "occt-brep")]
fn selected_cylinder_edge_length_ignores_display_profile_under_affine_scale() {
    use om9_kernel::creation::{primitive, Primitive};
    let limits = Limits::default();
    // Independent composite Simpson integration of the ellipse's speed.
    let n = 10000;
    let h = std::f64::consts::TAU / n as f64;
    let speed = |t: f64| ((4. * t.sin()).powi(2) + (6. * t.cos()).powi(2)).sqrt();
    let expected = (speed(0.) + speed(std::f64::consts::TAU)
        + (1..n).map(|i| speed(i as f64 * h) * if i % 2 == 0 { 2. } else { 4. }).sum::<f64>()) * h / 3.;
    let mut actual = Vec::new();
    for deflection in [0.1, 0.01] {
        let shape = primitive(Primitive::Cylinder { radius_mm: 2., height_mm: 5. }, deflection, &limits).unwrap();
        let ring = (0..shape.topology().edges.len()).find(|&i| {
            om9_kernel::measurement::edge_length(&shape, Some(i), Transform::identity(), &limits)
                .is_ok_and(|length| (length - 4. * std::f64::consts::PI).abs() < 1e-6)
        }).unwrap();
        let edge = shape.display().edges[ring];
        let mut entity = Entity::new("scaled cylinder", Geometry::Brep(shape), None);
        entity.placement = Transform([[2.,0.,0.,10.], [0.,3.,0.,20.], [0.,0.,1.,30.], [0.,0.,0.,1.]]);
        let id = entity.id;
        let mut c = controller(vec![entity]);
        c.select_subobject(id, om9_scene::scene::Subelement::BrepEdge(edge)).unwrap();
        actual.push(c.measure_selected_length().unwrap().value);
    }
    for length in actual {
        assert!((length - expected).abs() < 1e-6, "expected ellipse circumference {expected}, got {length}");
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
            .display()
            .edges
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let exact_line_length = b
                    .display()
                    .segments
                    .iter()
                    .filter(|s| s[2] as usize == index)
                    .map(|s| {
                        om9_core::measure::distance_mm(
                            b.display().points[s[0] as usize],
                            b.display().points[s[1] as usize],
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
