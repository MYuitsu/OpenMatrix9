// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in timing fixture; this is not desktop presentation or input latency.
use om9_app::session::Controller;
use om9_core::{budget::Limits, geometry::{Geometry, Transform}, model::{Document, Entity}};
use om9_scene::{camera::{Camera, Viewport}, scene::{PickIntent, PickKind}};

#[test]
#[ignore = "opt-in release timing fixture"]
fn preview_picking_index_rebuild_timing() {
    let mut document = Document::with_template_tolerance("preview index timing");
    let target = Entity::new("moving", Geometry::Point([0., 0., 0.]), None);
    let id = target.id;
    document.entities.insert(id, target);
    for i in 0..2000 {
        let x = (i % 50) as f64 * 4. + 10.;
        let y = (i / 50) as f64 * 4. + 10.;
        let point = Entity::new("fixture", Geometry::Point([x, y, 0.]), None);
        document.entities.insert(point.id, point);
    }
    let mut controller = Controller::new(document, Limits::default()).unwrap();
    controller.select_objects(&[id]).unwrap();
    let camera = Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    controller.scene().unwrap().spatial_index().unwrap();
    let mut samples = Vec::new();
    for i in 1..=40 {
        let x = i as f64 * 0.02;
        let screen = camera.project([x, 0., 0.], viewport).unwrap().unwrap().screen;
        let start = std::time::Instant::now();
        controller.preview_gumball(Transform::translation([x, 0., 0.])).unwrap();
        let hit = controller.scene().unwrap().pick_kind(screen, &camera, viewport, 2., PickIntent::Select, PickKind::Object).unwrap().unwrap();
        assert_eq!(hit.root, id);
        samples.push(start.elapsed().as_secs_f64() * 1000.);
    }
    samples.sort_by(f64::total_cmp);
    println!("preview_pick_ms n=40 objects=2001 p50={:.3} p95={:.3} p99={:.3} max={:.3}", samples[20], samples[37], samples[39], samples[39]);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "opt-in release timing fixture"]
fn preview_picking_repeated_brep_timing() {
    use om9_core::model::Definition;
    let limits = Limits::default();
    let brep = om9_kernel::brep::sphere_brep(3., 0.005, &limits).unwrap();
    let points = brep.display.points.len();
    let triangles = brep.display.triangles.len();
    let definition = Definition::new("shared sphere", vec![Entity::new("member", Geometry::Brep(brep), None)]);
    let definition_id = definition.id;
    let mut document = Document::with_template_tolerance("repeated brep preview timing");
    document.definitions.insert(definition_id, definition);
    let target = Entity::new("moving", Geometry::Point([0., 0., 0.]), None);
    let id = target.id;
    document.entities.insert(id, target);
    let count = std::env::var("OM9_BENCH_INSTANCES").ok()
        .and_then(|value| value.parse::<usize>().ok()).unwrap_or(200).clamp(2, 2000);
    for i in 0..count {
        let mut instance = Entity::new("sphere occurrence", Geometry::Instance(definition_id), None);
        instance.placement = Transform::translation([(i % 20) as f64 * 10. + 15., (i / 20) as f64 * 10. + 15., 0.]);
        document.entities.insert(instance.id, instance);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    let scene = controller.scene().unwrap();
    let mut spheres = scene.objects.iter().filter(|object| !object.path.is_empty());
    let first = spheres.next().unwrap();
    let second = spheres.next().unwrap();
    let shared = std::sync::Arc::ptr_eq(&first.buffers, &second.buffers);
    assert!(shared, "fixture must measure shared scene geometry");
    let camera = Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    controller.scene().unwrap().spatial_index().unwrap();
    let mut samples = Vec::new();
    for i in 1..=40 {
        let x = i as f64 * 0.02;
        let screen = camera.project([x, 0., 0.], viewport).unwrap().unwrap().screen;
        let start = std::time::Instant::now();
        controller.preview_gumball(Transform::translation([x, 0., 0.])).unwrap();
        let hit = controller.scene().unwrap().pick_kind(screen, &camera, viewport, 2., PickIntent::Select, PickKind::Object).unwrap().unwrap();
        assert_eq!(hit.root, id);
        samples.push(start.elapsed().as_secs_f64() * 1000.);
    }
    samples.sort_by(f64::total_cmp);
    println!("preview_pick_repeated_brep_ms n=40 instances={count} shared={shared} mesh_points={points} triangles={triangles} p50={:.3} p95={:.3} p99={:.3} max={:.3}", samples[20], samples[37], samples[39], samples[39]);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "opt-in release timing fixture"]
fn dispersed_surface_candidate_timing() {
    use om9_core::model::Definition;
    use om9_scene::osnap::{self, SnapKind, SnapSettings};
    let limits = Limits::default();
    let brep = om9_kernel::brep::sphere_brep(3., 0.05, &limits).unwrap();
    let triangles = brep.display.triangles.len();
    let definition = Definition::new("shared sphere", vec![Entity::new("member", Geometry::Brep(brep), None)]);
    let definition_id = definition.id;
    let mut document = Document::with_template_tolerance("dispersed surface timing");
    document.definitions.insert(definition_id, definition);
    let mut target = None;
    for i in 0..200 {
        let mut instance = Entity::new("sphere occurrence", Geometry::Instance(definition_id), None);
        instance.placement = Transform::translation([(i % 20) as f64 * 10., (i / 20) as f64 * 10., 0.]);
        if i == 0 { target = Some(instance.id); }
        document.entities.insert(instance.id, instance);
    }
    let controller = Controller::new(document, limits).unwrap();
    let scene = controller.scene().unwrap();
    let camera = Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    scene.spatial_index().unwrap();
    let mut samples = Vec::new();
    for i in 0..20 {
        let x = 1. + i as f64 * 0.005;
        let screen = camera.project([x, 0.5, (9. - x * x - 0.25).sqrt()], viewport).unwrap().unwrap().screen;
        let start = std::time::Instant::now();
        let candidates = osnap::candidates(scene, &camera, viewport, screen, SnapSettings::default(), None).unwrap();
        samples.push(start.elapsed().as_secs_f64() * 1000.);
        assert_eq!(candidates.iter().filter(|c| c.kind == SnapKind::Surface && c.root == target).count(), 1);
        assert_eq!(candidates.iter().filter(|c| c.kind == SnapKind::Surface).count(), 1);
    }
    samples.sort_by(f64::total_cmp);
    println!("dispersed_surface_candidates_ms n=20 instances=200 triangles_per_instance={triangles} p50={:.3} p95={:.3} max={:.3}", samples[10], samples[18], samples[19]);
}

#[cfg(feature = "occt-brep")]
#[test]
#[ignore = "opt-in release timing fixture"]
fn overlapping_surface_refinement_timing() {
    use om9_core::model::Definition;
    use om9_scene::osnap::{self, SnapKind, SnapSettings};
    let limits = Limits::default();
    let brep = om9_kernel::brep::sphere_brep(3., 0.05, &limits).unwrap();
    let points = brep.display.points.len();
    let triangles = brep.display.triangles.len();
    let source_bytes = brep.source().len();
    let definition = Definition::new("shared sphere", vec![Entity::new("member", Geometry::Brep(brep), None)]);
    let definition_id = definition.id;
    let mut document = Document::with_template_tolerance("overlapping surface refinement timing");
    document.definitions.insert(definition_id, definition);
    let count = std::env::var("OM9_BENCH_SURFACES").ok()
        .and_then(|value| value.parse::<usize>().ok()).unwrap_or(20).clamp(1, 50);
    for i in 0..count {
        let mut instance = Entity::new("sphere occurrence", Geometry::Instance(definition_id), None);
        instance.placement = Transform::translation([0., 0., -(i as f64) * 0.05]);
        document.entities.insert(instance.id, instance);
    }
    let controller = Controller::new(document, limits).unwrap();
    let camera = Camera::orthographic([0., 0., 100.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let mut samples = Vec::new();
    let mut candidate_samples = Vec::new();
    let mut kernel_samples = Vec::new();
    for i in 0..20 {
        let x = 1. + i as f64 * 0.005;
        let screen = camera.project([x, 0.5, (9. - x * x - 0.25).sqrt()], viewport).unwrap().unwrap().screen;
        let start = std::time::Instant::now();
        let candidates = osnap::candidates(controller.scene().unwrap(), &camera, viewport, screen, SnapSettings::default(), None).unwrap();
        candidate_samples.push(start.elapsed().as_secs_f64() * 1000.);
        let kernel_start = std::time::Instant::now();
        let mut refined = Vec::new();
        for candidate in candidates.iter().filter(|candidate| candidate.kind == SnapKind::Surface) {
            refined.push(controller.refine_surface_snap(candidate, &camera, viewport).unwrap());
        }
        kernel_samples.push(kernel_start.elapsed().as_secs_f64() * 1000.);
        assert_eq!(refined.len(), count);
        let closest = osnap::choose(&refined).unwrap();
        assert!((closest.world[2] - (9. - x * x - 0.25).sqrt()).abs() < 1e-6);
        samples.push(start.elapsed().as_secs_f64() * 1000.);
    }
    samples.sort_by(f64::total_cmp);
    candidate_samples.sort_by(f64::total_cmp);
    kernel_samples.sort_by(f64::total_cmp);
    println!("surface_candidates_refine_ms n=20 surfaces={count} shared=true mesh_points={points} triangles={triangles} source_bytes={source_bytes} p50={:.3} p95={:.3} p99={:.3} max={:.3} candidate_p50={:.3} kernel_p50={:.3}", samples[10], samples[18], samples[19], samples[19], candidate_samples[10], kernel_samples[10]);
}
