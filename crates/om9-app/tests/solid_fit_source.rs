// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]

use om9_app::session::Controller;
use om9_core::{budget::Limits, geometry::{Geometry, Transform}, model::{Document, Entity}};

#[test]
fn sphere_fit_uses_native_vertices_and_world_placement_independent_of_display() {
    let limits = Limits::default();
    let box_brep = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let source_only_limits = Limits { max_points: 8, ..limits };
    assert!(box_brep.validate(&source_only_limits).is_err());
    assert_eq!(om9_kernel::brep::source_vertices(&box_brep, &source_only_limits).unwrap().len(), 8);
    assert_eq!(om9_kernel::brep::source_vertices_admitted(&box_brep, &source_only_limits, |_, _| Ok(())).unwrap().len(), 8);
    let mut observed = Vec::new();
    for deflection in [0.5, 0.01] {
        let body = om9_kernel::brep::read_brep(box_brep.source(), deflection, &limits).unwrap();
        let mut entity = Entity::new("box", Geometry::Brep(body), None);
        entity.placement = Transform::translation([10., -3., 7.]);
        let id = entity.id;
        let mut document = Document::new("fit source vertices");
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.select_objects(&[id]).unwrap();
        let mut points = controller.solid_fit_selection_points().unwrap();
        points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])).then(a[2].total_cmp(&b[2])));
        let mut expected = Vec::new();
        for x in [10., 14.] { for y in [-3., 2.] { for z in [7., 13.] { expected.push([x, y, z]); } } }
        assert_eq!(points, expected);
        controller.submit("Sphere").unwrap();
        controller.submit("FitPoints").unwrap();
        controller.submit("Selection").unwrap();
        controller.submit("").unwrap();
        let result = controller.document().entities.values().find(|entity| entity.id != id).unwrap();
        let center = result.placement.point([0., 0., 0.]).unwrap();
        for (actual, expected) in center.into_iter().zip([12., -0.5, 10.]) {
            assert!((actual - expected).abs() < 1e-7, "{center:?}");
        }
        let Geometry::Brep(sphere) = &result.geometry else { panic!("expected B-rep sphere") };
        let bounds = om9_kernel::brep::world_bounds(sphere, &Transform::identity(), &limits).unwrap();
        let radius = (19.25_f64).sqrt();
        assert!((bounds[3] - radius).abs() < 1e-7);
        controller.undo().unwrap();
        assert_eq!(controller.document().entities.len(), 1);
        observed.push(points);
    }
    assert_eq!(observed[0], observed[1]);
}

#[test]
fn sphere_fit_selection_includes_native_brep_vertices() {
    let limits = Limits::default();
    let source = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&source, 0.001, &limits).unwrap();
    let mut entity = Entity::new("native box", Geometry::NativeBrep(native), None);
    entity.placement = Transform::translation([10., -3., 7.]);
    let mut document = Document::with_template_tolerance("fit native selection");
    let mut ids = vec![entity.id];
    document.entities.insert(entity.id, entity);
    let r = 19.25_f64.sqrt();
    for point in [[12.+r,-0.5,10.], [12.-r,-0.5,10.], [12.,-0.5+r,10.], [12.,-0.5,10.+r]] {
        let entity = Entity::new("point", Geometry::Point(point), None);
        ids.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    let path = std::env::temp_dir().join(format!("om9-fit-native-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    for menu in [false, true] {
        let mut controller = Controller::from_file(&path, limits).unwrap();
        controller.select_objects(&ids).unwrap();
        let points = controller.solid_fit_selection_points().unwrap();
        assert_eq!(points.len(), 12, "selection omitted NativeBrep source vertices");
        assert!(points.contains(&[10., -3., 7.]));
        assert!(points.contains(&[14., 2., 13.]));
        let before = controller.document().clone();
        if menu {
            controller.dispatch("SolidSphereFitSpheretoPoints", om9_app::session::InvocationOrigin::Menu).unwrap();
        } else {
            controller.submit("Sphere").unwrap();
            controller.submit("FitPoints").unwrap();
        }
        controller.submit("Selection").unwrap();
        controller.submit("").unwrap();
        let after = controller.document().clone();
        assert_eq!(after.entities.len(), before.entities.len() + 1);
        let result = after.entities.values().find(|entity| !ids.contains(&entity.id)).unwrap();
        let center = result.placement.point([0., 0., 0.]).unwrap();
        for (actual, expected) in center.into_iter().zip([12., -0.5, 10.]) {
            assert!((actual - expected).abs() < 1e-7, "center {center:?}, menu {menu}");
        }
        let Geometry::Brep(sphere) = &result.geometry else { panic!("expected sphere") };
        let bounds = om9_kernel::brep::world_bounds(sphere, &Transform::identity(), &limits).unwrap();
        assert!((bounds[3] - r).abs() < 1e-7, "radius bounds {bounds:?}, menu {menu}");
        controller.undo().unwrap();
        assert_eq!(controller.document(), &before);
        controller.redo().unwrap();
        assert_eq!(controller.document(), &after);
    }
    std::fs::remove_file(path).unwrap();
}

#[test]
fn sphere_fit_native_brep_scaled_and_reflected_placement_after_reopen() {
    let limits = Limits::default();
    let source = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&source, 0.001, &limits).unwrap();
    let mut entity = Entity::new("native box", Geometry::NativeBrep(native), None);
    entity.placement = Transform([[2., 0., 0., 10.], [0., 0.5, 0., -3.], [0., 0., -1., 7.], [0., 0., 0., 1.]]);
    let id = entity.id;
    let mut document = Document::with_template_tolerance("fit affine native selection");
    document.entities.insert(id, entity);
    let path = std::env::temp_dir().join(format!("om9-fit-affine-native-{}.om9", uuid::Uuid::new_v4()));
    om9_archive::Archive::save(&path, &document, &limits, &om9_core::transaction::Cancellation::default()).unwrap();
    let mut controller = Controller::from_file(&path, limits).unwrap();
    std::fs::remove_file(path).unwrap();
    controller.select_objects(&[id]).unwrap();
    let points = controller.solid_fit_selection_points().unwrap();
    assert_eq!(points.len(), 8);
    for x in [10., 18.] { for y in [-3., -0.5] { for z in [1., 7.] {
        assert!(points.contains(&[x, y, z]), "missing transformed corner {x}, {y}, {z}");
    } } }
    let before = controller.document().clone();
    controller.submit("Sphere").unwrap();
    controller.submit("FitPoints").unwrap();
    controller.submit("Selection").unwrap();
    controller.submit("").unwrap();
    let after = controller.document().clone();
    let result = after.entities.values().find(|entity| entity.id != id).unwrap();
    let center = result.placement.point([0., 0., 0.]).unwrap();
    for (actual, expected) in center.into_iter().zip([14., -1.75, 4.]) {
        assert!((actual - expected).abs() < 1e-7, "center {center:?}");
    }
    let Geometry::Brep(sphere) = &result.geometry else { panic!("expected sphere") };
    let bounds = om9_kernel::brep::world_bounds(sphere, &Transform::identity(), &limits).unwrap();
    assert!((bounds[3] - 26.5625_f64.sqrt()).abs() < 1e-7, "radius bounds {bounds:?}");
    controller.undo().unwrap(); assert_eq!(controller.document(), &before);
    controller.redo().unwrap(); assert_eq!(controller.document(), &after);
}

#[test]
fn sphere_fit_native_source_failures_preserve_pending_and_redo() {
    use om9_core::budget::memory::MemoryClass;
    let limits = Limits::default();
    let shape = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&shape, 0.001, &limits).unwrap();
    let mut malformed = native.clone();
    malformed.payload = vec![1, 2, 3, 4].into();
    let mut good = Entity::new("native", Geometry::NativeBrep(native), None);
    let mut bad = Entity::new("malformed native", Geometry::NativeBrep(malformed), None);
    good.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(1));
    bad.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(2));
    let (good_id, bad_id) = (good.id, bad.id);
    let mut document = Document::with_template_tolerance("fit source errors");
    document.entities.insert(good_id, good);
    document.entities.insert(bad_id, bad);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.submit("Point").unwrap();
    controller.submit("100,100,100").unwrap();
    controller.undo().unwrap();
    let before = controller.document().clone();
    controller.select_objects(&[good_id, bad_id]).unwrap();
    controller.submit("Sphere").unwrap();
    controller.submit("FitPoints").unwrap();
    assert!(controller.submit("Selection").is_err());
    assert!(controller.solid_preview_markers().is_empty());
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().redo_len(), 1);

    controller.select_objects(&[good_id]).unwrap();
    let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap(),
    ).unwrap();
    assert!(matches!(controller.submit("Selection"), Err(om9_core::Error::Budget(_))));
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().redo_len(), 1);
    drop(pressure);
    controller.submit("Selection").unwrap();
    controller.submit("").unwrap();
    let after = controller.document().clone();
    controller.undo().unwrap();
    assert_eq!(controller.document(), &before);
    controller.redo().unwrap();
    assert_eq!(controller.document(), &after);
}

#[test]
fn source_vertex_allocation_refusal_is_typed_and_retryable() {
    let limits = Limits::default();
    let source = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let mut refused = false;
    let result = om9_kernel::brep::source_vertices_from_source_admitted(source.source(), &limits, |_, pending| {
        if pending > 0 { refused = true; return Err(om9_core::Error::Budget("test vertices")); }
        Ok(())
    });
    assert!(refused);
    assert!(matches!(result, Err(om9_core::Error::Budget(_))));
    let actual = om9_kernel::brep::source_vertices_from_source_admitted(source.source(), &limits, |_, _| Ok(())).unwrap();
    assert_eq!(actual.len(), 8);
}

#[test]
fn sphere_fit_native_reuses_existing_point_capacity_under_pressure() {
    use om9_core::budget::memory::MemoryClass;
    let limits = Limits { max_bytes: 64 * 1024 * 1024, ..Limits::default() };
    let source = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let native = om9_kernel::brep::to_3dm(&source, 0.001, &limits).unwrap();
    let mut document = Document::with_template_tolerance("fit capacity");
    let mut ids = Vec::new();
    for i in 1..=17u128 {
        let mut point = Entity::new("point", Geometry::Point([i as f64, 0., 0.]), None);
        point.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(i));
        ids.push(point.id); document.entities.insert(point.id, point);
    }
    let mut shape = Entity::new("native", Geometry::NativeBrep(native), None);
    shape.id = om9_core::identity::ObjectId(uuid::Uuid::from_u128(100));
    ids.push(shape.id); document.entities.insert(shape.id, shape);
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&ids).unwrap();
    let mut minimum = [None, None];
    for (case, selection) in [&ids[ids.len()-1..], ids.as_slice()].into_iter().enumerate() {
        controller.select_objects(selection).unwrap();
        for slack in (2048..=16384).step_by(128) {
            let mut pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
                MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap()).unwrap();
            pressure.shrink_to(pressure.bytes() - slack).unwrap();
            let result = controller.solid_fit_selection_points();
            drop(pressure);
            if let Ok(points) = result {
                assert_eq!(points.len(), if case == 0 { 8 } else { 25 });
                minimum[case] = Some(slack);
                break;
            }
        }
    }
    let single = minimum[0].expect("single source must fit within the probe budget");
    let mixed = minimum[1].expect("mixed selection must fit within the probe budget");
    let existing_capacity = 32 * std::mem::size_of::<[f64; 3]>() as u64;
    assert!(mixed <= single + existing_capacity + 128, "single={single}, mixed={mixed}");
}

#[test]
fn sphere_fit_selection_collection_refuses_pressure_for_each_source_kind() {
    use om9_core::{budget::memory::MemoryClass, geometry::{Curve, Mesh}};
    let limits = Limits::default();
    let brep = om9_kernel::brep::box_brep([4., 5., 6.], 0.1, &limits).unwrap();
    let sources = [
        (Geometry::Point([1., 0., 0.]), 1),
        (Geometry::Curve(Curve::Line { start: [0., 0., 0.], end: [1., 0., 0.], domain: [0., 1.] }), 16),
        (Geometry::Mesh(Mesh { vertices: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], faces: vec![vec![0,1,2]], normals: vec![], colors: vec![], corner_uvs: vec![] }), 3),
        (Geometry::Brep(brep), 8),
    ];
    for (geometry, expected) in sources {
        let entity = Entity::new("source", geometry, None);
        let id = entity.id;
        let mut document = Document::with_template_tolerance("fit collection pressure");
        document.entities.insert(id, entity);
        let mut controller = Controller::new(document, limits).unwrap();
        controller.submit("Point").unwrap(); controller.submit("9,0,0").unwrap(); controller.undo().unwrap();
        controller.select_objects(&[id]).unwrap();
        controller.submit("Sphere FitPoints").unwrap();
        let before = controller.document().clone();
        let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap(),
        ).unwrap();
        assert!(matches!(controller.submit("Selection"), Err(om9_core::Error::Budget(_))));
        assert!(controller.solid_preview_markers().is_empty());
        assert_eq!(controller.document(), &before);
        assert_eq!(controller.coordinator().redo_len(), 1);
        drop(pressure);
        controller.submit("Selection").unwrap();
        assert_eq!(controller.solid_preview_markers().len(), expected);
    }
}

#[test]
fn sphere_fit_selection_accounts_for_other_live_pending_data() {
    use om9_core::{budget::memory::MemoryClass, geometry::Curve};
    let mut document = Document::with_template_tolerance("fit retained curve memory");
    let curve = Entity::new("curve", Geometry::Curve(Curve::Line {
        start: [0.,0.,0.], end: [10.,0.,0.], domain: [0.,1.],
    }), None);
    let curve_id = curve.id; document.entities.insert(curve_id, curve);
    let point = Entity::new("point", Geometry::Point([1.,0.,0.]), None);
    let point_id = point.id; document.entities.insert(point_id, point);
    let mut reference = Controller::new(document.clone(), Limits::default()).unwrap();
    reference.select_objects(&[point_id]).unwrap();
    reference.submit("Sphere FitPoints").unwrap();
    let base_held = reference.scene_cache_usage().unwrap().bytes;
    let mut baseline = None;
    for slack in (0..8192).step_by(128) {
        let mut pressure = reference.coordinator().reserve_remaining_external_memory_with_held(
            MemoryClass::Kernel, None, reference.scene_cache_usage().unwrap(),
        ).unwrap();
        pressure.shrink_to(pressure.bytes() - slack).unwrap();
        let result = reference.submit("Selection");
        drop(pressure);
        if result.is_ok() { baseline = Some(slack); break; }
        assert!(matches!(result, Err(om9_core::Error::Budget(_))));
    }
    let baseline = baseline.expect("point selection must fit");
    drop(reference);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[curve_id]).unwrap();
    controller.submit("Sphere").unwrap(); controller.submit("AroundCurve").unwrap();
    controller.submit("FitPoints").unwrap();
    controller.select_objects(&[point_id]).unwrap();
    let before = controller.document().clone();
    let retained = controller.scene_cache_usage().unwrap().bytes.checked_sub(base_held).unwrap();
    assert!(retained >= 512, "fixture must retain curve data: {retained}");
    let mut pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap(),
    ).unwrap();
    let slack = baseline.checked_sub(retained / 2).expect("fixture requires positive remaining headroom");
    pressure.shrink_to(pressure.bytes() - slack).unwrap();
    let result = controller.submit("Selection");
    assert!(matches!(result, Err(om9_core::Error::Budget(_))), "baseline={baseline}, retained={retained}, result={result:?}");
    assert!(controller.solid_preview_markers().is_empty());
    assert_eq!(controller.document(), &before);
    drop(pressure);
    controller.submit("Selection").unwrap();
    assert_eq!(controller.solid_preview_markers(), vec![[1.,0.,0.]]);
}

#[test]
fn sphere_fit_existing_points_refuse_unadmitted_selection_append() {
    use om9_core::budget::memory::MemoryClass;
    let entity = Entity::new("point", Geometry::Point([1., 0., 0.]), None);
    let id = entity.id;
    let mut doc = Document::with_template_tolerance("fit append pressure");
    doc.entities.insert(id, entity);
    let mut controller = Controller::new(doc, Limits::default()).unwrap();
    controller.submit("Point").unwrap(); controller.submit("9,0,0").unwrap(); controller.undo().unwrap();
    controller.select_objects(&[id]).unwrap();
    controller.submit("Sphere FitPoints").unwrap();
    controller.accept_world_point([0., 1., 0.]).unwrap();
    let before = controller.document().clone();
    let pressure = controller.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, controller.scene_cache_usage().unwrap(),
    ).unwrap();
    assert!(matches!(controller.submit("Selection"), Err(om9_core::Error::Budget(_))));
    assert_eq!(controller.solid_preview_markers(), vec![[0., 1., 0.]]);
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().redo_len(), 1);
    drop(pressure);
    controller.submit("Selection").unwrap();
    assert_eq!(controller.solid_preview_markers().len(), 2);
}

#[test]
fn sphere_fit_applies_placement_to_authored_points() {
    let mut entity = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    entity.placement = Transform::translation([10., -3., 7.]);
    let id = entity.id;
    let mut document = Document::new("fit point placement");
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    assert_eq!(controller.solid_fit_selection_points().unwrap(), [[11., -1., 10.]]);
}

#[test]
fn sphere_fit_direct_points_enforce_limit_and_preserve_session() {
    let limits = Limits { max_points: 4, ..Limits::default() };
    let mut controller = Controller::new(Document::with_template_tolerance("fit point limit"), limits).unwrap();
    controller.submit("Sphere FitPoints").unwrap();
    for point in [[1.,0.,0.], [-1.,0.,0.], [0.,1.,0.], [0.,0.,1.]] {
        controller.accept_world_point(point).unwrap();
    }
    let before = controller.document().clone();
    let points = controller.solid_preview_markers();
    assert!(matches!(controller.accept_world_point([0.,-1.,0.]), Err(om9_core::Error::Budget("Sphere FitPoints"))));
    assert_eq!(controller.solid_preview_markers(), points);
    assert_eq!(controller.document(), &before);
    assert!(controller.is_running());
}

#[test]
fn sphere_fit_repeated_selection_respects_point_limit_without_commit() {
    let entity = Entity::new("point", Geometry::Point([1., 2., 3.]), None);
    let id = entity.id;
    let mut document = Document::new("fit point limit");
    document.entities.insert(id, entity);
    let mut controller = Controller::new(document, Limits { max_points: 20, ..Limits::default() }).unwrap();
    controller.select_objects(&[id]).unwrap();
    let before = controller.document().clone();
    controller.submit("Sphere").unwrap(); controller.submit("FitPoints").unwrap();
    for _ in 0..20 { controller.submit("Selection").unwrap(); }
    assert!(matches!(controller.submit("Selection"), Err(om9_core::Error::Budget(_))));
    assert_eq!(controller.document(), &before);
    assert_eq!(controller.coordinator().undo_len(), 0);
}
