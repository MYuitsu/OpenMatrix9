// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(feature = "occt-brep")]

//! A display-mesh hit is a candidate: verify its error against an analytic
//! sphere and the existing single-face kernel ray adapter independently.
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Transform},
    model::{Document, Entity},
};
use om9_scene::{
    camera::{Camera, Viewport},
    scene::{PickIntent, PickKind, Subelement},
};

#[test]
fn brep_display_is_an_occluder_even_without_reference_permission() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
    let mut document = Document::with_template_tolerance("B-rep occlusion");
    let entity = Entity::new("front", Geometry::Brep(sphere), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut scene = om9_scene::scene::Scene::capture(std::sync::Arc::new(document), 0, &limits).unwrap();
    let object = scene.objects.iter_mut().find(|object| object.root == id).unwrap();
    object.selectable = false;
    object.reference_snappable = false;
    let camera = Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 10.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let cursor = camera.project([0., 0., 0.], viewport).unwrap().unwrap().screen;
    assert!(om9_scene::osnap::visible_display_occludes(&scene, &camera, viewport, cursor, [0., 0., -5.], 0., &[], None).unwrap());
    assert!(!om9_scene::osnap::visible_display_occludes(&scene, &camera, viewport, cursor, [0., 0., 5.], 0., &[], None).unwrap());
    assert!(!om9_scene::osnap::visible_display_occludes(&scene, &camera, viewport, cursor, [0., 0., -5.], 0., &[id], None).unwrap());
}

#[test]
fn analytic_sphere_silhouette_surface_candidate_coverage() {
    let limits = Limits::default();
    let radius = 5.0_f64;
    let sphere = om9_kernel::brep::sphere_brep(radius, radius * 0.1, &limits).unwrap();
    let mut document = Document::with_template_tolerance("sphere silhouette coverage");
    let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let controller = om9_app::session::Controller::new(document, limits).unwrap();
    let camera = Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let mut found = 0;
    let mut missed = 0;
    for fraction in [0.9_f64, 0.95, 0.98, 0.99, 0.995, 0.999] {
        let start_missed = missed;
        for i in 0..24 {
            let angle = std::f64::consts::TAU * (i as f64 + 0.37) / 24.;
            let x = radius * fraction * angle.cos();
            let y = radius * fraction * angle.sin();
            let expected = [x, y, (radius * radius - x * x - y * y).sqrt()];
            let cursor = camera.project(expected, viewport).unwrap().unwrap().screen;
            let candidates = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap();
            if let Some(candidate) = candidates.iter().find(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface && candidate.root == Some(id)) {
                let point = controller.refine_surface_snap(candidate, &camera, viewport).unwrap().world;
                let error = point.iter().zip(expected).map(|(actual, want)| (actual - want).powi(2)).sum::<f64>().sqrt();
                assert!(error < 1e-6, "analytic sphere error at fraction {fraction}: {error}");
                found += 1;
            } else {
                let ray = camera.visible_ray(cursor, viewport).unwrap();
                assert!(controller.scene().unwrap().spatial_index().unwrap().ray_candidates(ray.origin, ray.direction).contains(&0),
                    "the spatial index excluded an analytic sphere ray at fraction {fraction}");
                missed += 1;
            }
        }
        println!("radius_fraction={fraction} missed={}", missed - start_missed);
    }
    assert!(found > 0, "the fixture must exercise actual canonical refinement");
    println!("sphere silhouette: {found} refined hits, {missed} missing candidates, 144 rays");
}

#[test]
fn coarse_sphere_face_hit_needs_kernel_refinement_for_world_coordinates() {
    let limits = Limits::default();
    for radius in [5.0_f64, 0.05] {
        let sphere = om9_kernel::brep::sphere_brep(radius, radius * 0.1, &limits).unwrap();
        let mut document = Document::with_template_tolerance("coarse sphere picking");
        let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
        let id = entity.id;
        document.entities.insert(id, entity);
        let controller = om9_app::session::Controller::new(document, limits).unwrap();
        let camera = Camera::orthographic(
            [0., 0., radius * 4.],
            [0., 0., 0.],
            [0., 1., 0.],
            radius * 2.4,
        )
        .unwrap();
        let viewport = Viewport {
            origin: [0., 0.],
            size: [600., 600.],
        };
        let x = radius * 0.426;
        let y = radius * 0.274;
        let expected = [x, y, (radius * radius - x * x - y * y).sqrt()];
        let screen = camera.project(expected, viewport).unwrap().unwrap().screen;
        let hit = controller
            .scene()
            .unwrap()
            .pick_kind(
                screen,
                &camera,
                viewport,
                1.,
                PickIntent::Select,
                PickKind::Face,
            )
            .unwrap()
            .unwrap();
        assert_eq!(hit.root, id);
        assert!(matches!(hit.subset, Subelement::BrepFace(_)));
        hit.revalidate_in_view(controller.coordinator(), &camera, viewport)
            .unwrap();
        let candidate_error = (hit.world_point[2] - expected[2]).abs();
        assert!(
            candidate_error > radius * 1e-6,
            "fixture must expose a measurable display-mesh approximation: {candidate_error}"
        );
        let snaps = om9_scene::osnap::candidates(
            controller.scene().unwrap(),
            &camera,
            viewport,
            screen,
            Default::default(),
            None,
        )
        .unwrap();
        let surface = snaps
            .iter()
            .find(|snap| snap.kind == om9_scene::osnap::SnapKind::Surface && snap.root == Some(id))
            .expect("surface snap candidate");
        let provenance = surface
            .surface_source
            .as_ref()
            .expect("B-rep surface candidate provenance");
        assert_eq!(provenance.source, hit.source);
        assert_eq!(provenance.scene_epoch, hit.world_epoch);
        assert_eq!(provenance.leaf, id);
        assert!(provenance.path.is_empty());
        assert_eq!(
            Some(hit.subset.clone()),
            provenance.face.map(Subelement::BrepFace)
        );
        let snap_error = (surface.world[2] - expected[2]).abs();
        assert!(
            snap_error > radius * 1e-6,
            "current surface snap must expose the same coarse-mesh limitation"
        );
        let refined = controller
            .refine_surface_snap(surface, &camera, viewport)
            .unwrap();
        assert!(
            refined
                .world
                .iter()
                .zip(expected)
                .map(|(actual, want)| (actual - want).powi(2))
                .sum::<f64>()
                .sqrt()
                < radius * 2e-7
        );
        assert!(
            refined.normal.is_none(),
            "an unverified display normal must not be presented as exact"
        );
        if std::env::var_os("OM9_BENCH_SURFACE_SNAP").is_some() {
            let mut durations = Vec::new();
            for _ in 0..40 {
                let start = std::time::Instant::now();
                controller
                    .refine_surface_snap(surface, &camera, viewport)
                    .unwrap();
                durations.push(start.elapsed().as_secs_f64() * 1000.);
            }
            durations.sort_by(f64::total_cmp);
            println!(
                "surface_refine_unoptimized_ms radius={radius} p50={:.4} p95={:.4} max={:.4}",
                durations[20], durations[38], durations[39]
            );
        }
        let mut stale = surface.clone();
        stale.surface_source.as_mut().unwrap().scene_epoch += 1;
        assert!(
            controller
                .refine_surface_snap(&stale, &camera, viewport)
                .is_err()
        );
        let mut bad_face = surface.clone();
        bad_face.surface_source.as_mut().unwrap().face = Some(uuid::Uuid::nil());
        assert!(
            controller
                .refine_surface_snap(&bad_face, &camera, viewport)
                .is_err()
        );
        let mut bad_path = surface.clone();
        bad_path.surface_source.as_mut().unwrap().path.push(id);
        assert!(
            controller
                .refine_surface_snap(&bad_path, &camera, viewport)
                .is_err()
        );
        let mut stale_placement = surface.clone();
        stale_placement.surface_source.as_mut().unwrap().placement =
            Transform::translation([0., 0., 1.]);
        assert!(
            controller
                .refine_surface_snap(&stale_placement, &camera, viewport)
                .is_err()
        );
        let ray = camera.ray(screen, viewport).unwrap();
        let geometry = &controller.document().entities[&id].geometry;
        let exact = om9_kernel::measurement::surface_ray_hit(
            geometry,
            Transform::identity(),
            ray.origin,
            ray.direction,
            hit.world_point,
            radius * 1e-7,
            controller.coordinator().limits(),
        )
        .unwrap();
        let error = exact
            .iter()
            .zip(expected)
            .map(|(actual, want)| (actual - want).powi(2))
            .sum::<f64>()
            .sqrt();
        assert!(
            error < radius * 2e-7,
            "kernel ray versus analytic sphere: {error}"
        );
        let Geometry::Brep(brep) = geometry else {
            unreachable!()
        };
        let indexed = om9_kernel::measurement::brep_face_ray_hit(
            brep,
            provenance.face.unwrap(),
            Transform::identity(),
            ray.origin,
            ray.direction,
            surface.world,
            radius * 1e-7,
            controller.coordinator().limits(),
        )
        .unwrap();
        assert!(
            indexed
                .iter()
                .zip(expected)
                .map(|(actual, want)| (actual - want).powi(2))
                .sum::<f64>()
                .sqrt()
                < radius * 2e-7
        );
        assert!(
            om9_kernel::measurement::brep_face_ray_hit(
                brep,
                uuid::Uuid::nil(),
                Transform::identity(),
                ray.origin,
                ray.direction,
                surface.world,
                radius * 1e-7,
                controller.coordinator().limits()
            )
            .is_err()
        );
        println!(
            "radius_mm={radius}, display_candidate_error_mm={candidate_error:.9}, surface_snap_error_mm={snap_error:.9}, single_face_ray_error_mm={error:.12}"
        );
    }
}

#[test]
fn surface_scheduler_coalesces_input_and_rejects_stale_pending_work() {
    use om9_core::budget::memory::MemoryClass;
    let limits = Limits::default();
    let mut document = Document::with_template_tolerance("surface scheduler");
    let entity = Entity::new("sphere", Geometry::Brep(om9_kernel::brep::sphere_brep(5., 0.2, &limits).unwrap()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = om9_app::session::Controller::new(document, limits).unwrap();
    let camera = Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let input = |controller: &om9_app::session::Controller, x: f64| {
        let cursor = camera.project([x, 1., (24. - x*x).sqrt()], viewport).unwrap().unwrap().screen;
        let candidates: Vec<_> = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
            .into_iter().filter(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface).collect();
        assert_eq!(candidates.len(), 1);
        (cursor, candidates)
    };
    let mut scheduler = om9_app::session::SurfaceSnapScheduler::default();
    // Another scheduler or worker can occupy the shared account. Keep only
    // the newest input while waiting instead of losing it as an error.
    let account = controller.coordinator().external_memory_account();
    let slot = account.try_acquire_surface_snap().unwrap();
    for x in [1., 1.1, 1.2] {
        let (cursor, candidates) = input(&controller, x);
        let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        scheduler.submit(&controller, candidates, camera.clone(), viewport, cursor, guard).unwrap();
        assert!(scheduler.poll(&controller).is_none());
        assert!(scheduler.is_busy());
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 1024 * 1024);
    }
    drop(slot);
    let (last_cursor, _) = input(&controller, 1.2);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let receipt = loop {
        if let Some(result) = scheduler.poll(&controller) { break result.unwrap(); }
        assert!(scheduler.is_busy() && std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    let hit = receipt.accept(&controller, &camera, viewport, last_cursor).unwrap().unwrap();
    assert!((hit.world[0] - 1.2).abs() < 1e-6);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    let slot = account.try_acquire_surface_snap().unwrap();
    let (cursor, candidates) = input(&controller, 1.);
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    scheduler.submit(&controller, candidates.clone(), camera.clone(), viewport, cursor, guard).unwrap();
    scheduler.cancel();
    assert!(!scheduler.is_busy());
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    scheduler.submit(&controller, candidates, camera.clone(), viewport, cursor, guard).unwrap();
    controller.select_objects(&[id]).unwrap();
    assert!(matches!(scheduler.poll(&controller), Some(Err(om9_core::Error::Stale(_)))));
    assert!(!scheduler.is_busy());
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    drop(slot);
    controller.clear_selection();
    let mut last_cursor = [0.; 2];
    for i in 0..30 {
        let (cursor, candidates) = input(&controller, 1. + i as f64 * 0.01);
        let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        scheduler.submit(&controller, candidates, camera.clone(), viewport, cursor, guard).unwrap();
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) <= 2 * 1024 * 1024);
        last_cursor = cursor;
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let result = loop {
        if let Some(result) = scheduler.poll(&controller) { break result.unwrap(); }
        assert!(scheduler.is_busy() && std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    let hit = result.accept(&controller, &camera, viewport, last_cursor).unwrap().unwrap();
    assert!((hit.world[0] - 1.29).abs() < 1e-6);
    assert!(!scheduler.is_busy());
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);

    // Identical candidate IDs/revisions in a replaced document must schedule
    // fresh work, not deduplicate against the older active request.
    let same_ids = om9_app::session::Controller::new(controller.document().clone(), limits).unwrap();
    let (cursor, candidates) = input(&controller, 1.2);
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    scheduler.submit(&controller, candidates.clone(), camera.clone(), viewport, cursor, guard).unwrap();
    let guard = same_ids.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    scheduler.submit(&same_ids, candidates.clone(), camera.clone(), viewport, cursor, guard).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let receipt = loop {
        if let Some(result) = scheduler.poll(&same_ids) { break result.unwrap(); }
        assert!(scheduler.is_busy() && std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    assert!(receipt.accept(&same_ids, &camera, viewport, cursor).unwrap().is_some());
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    assert_eq!(same_ids.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);

    for x in [1., 1.1] {
        let (cursor, candidates) = input(&controller, x);
        let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        scheduler.submit(&controller, candidates, camera.clone(), viewport, cursor, guard).unwrap();
    }
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let result = loop {
        if let Some(result) = scheduler.poll(&same_ids) { break result; }
        assert!(scheduler.is_busy() && std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    assert!(matches!(result, Err(om9_core::Error::Stale(_))), "pending work must not migrate to a replacement snapshot");
    assert!(!scheduler.is_busy());
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);

    for x in [1., 1.1] {
        let (cursor, candidates) = input(&controller, x);
        let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        scheduler.submit(&controller, candidates, camera.clone(), viewport, cursor, guard).unwrap();
    }
    controller.select_objects(&[id]).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let error = loop {
        if let Some(result) = scheduler.poll(&controller) { break result.err().expect("pending request must be stale"); }
        assert!(scheduler.is_busy() && std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    assert!(matches!(error, om9_core::Error::Stale(_)));
    assert!(!scheduler.is_busy());
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    for x in [1., 1.1] {
        let (cursor, candidates) = input(&controller, x);
        let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        scheduler.submit(&controller, candidates, camera.clone(), viewport, cursor, guard).unwrap();
    }
    scheduler.cancel();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while scheduler.is_busy() {
        assert!(scheduler.poll(&controller).is_none(), "cancelled work must not be published");
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);

    let (cursor, candidates) = input(&controller, 1.2);
    let mut receipt = None;
    for _ in 0..100 {
        let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        scheduler.submit(&controller, candidates.clone(), camera.clone(), viewport, cursor, guard).unwrap();
        assert!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) <= 1024 * 1024,
            "identical input must not create a second retained request");
        std::thread::sleep(std::time::Duration::from_millis(2));
        if let Some(result) = scheduler.poll(&controller) { receipt = Some(result.unwrap()); break; }
    }
    let hit = receipt.expect("unchanged repeated input must not starve the worker")
        .accept(&controller, &camera, viewport, cursor).unwrap().unwrap();
    assert!((hit.world[0] - 1.2).abs() < 1e-6);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
}

#[test]
fn surface_worker_and_stl_export_use_the_same_immutable_source() {
    use om9_core::budget::memory::MemoryClass;
    use om9_app::session::{StlExportProfile, SurfaceSnapJob};
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.2, &limits).unwrap();
    let original_source = sphere.source().to_owned();
    let mut document = Document::with_template_tolerance("surface and STL concurrency");
    let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = om9_app::session::Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    let capture = controller.capture_stl_selected().unwrap();
    let camera = Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let cursor = camera.project([2., 1., 20_f64.sqrt()], viewport).unwrap().unwrap().screen;
    let candidate = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
        .into_iter().find(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface && candidate.root == Some(id)).unwrap();
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 8 * 1024 * 1024, None).unwrap();
    let mut snap = SurfaceSnapJob::start(&controller, vec![candidate; 256], camera.clone(), viewport, cursor, guard).unwrap();
    assert!(snap.try_result().is_none(), "fixture must begin while the snap worker is active");
    let target = std::env::temp_dir().join(format!("om9-surface-export-{id:?}.stl"));
    let result = capture.export_with_profile(&target, StlExportProfile { linear_tolerance_mm: 0.01, angular_tolerance_rad: 0.3 }).unwrap();
    let completed_at_export_return = snap.try_result();
    println!("surface result available at STL return: {}", completed_at_export_return.is_some());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    let receipt = if let Some(result) = completed_at_export_return { result.unwrap() } else {
        loop {
            if let Some(result) = snap.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline, "Surface worker timed out");
            std::thread::yield_now();
        }
    };
    let hit = receipt.accept(&controller, &camera, viewport, cursor).unwrap().unwrap();
    assert!((hit.world[2] - 20_f64.sqrt()).abs() < 1e-6);
    let bytes = std::fs::read(&target).unwrap();
    std::fs::remove_file(&target).unwrap();
    let count = u32::from_le_bytes(bytes[80..84].try_into().unwrap()) as usize;
    assert_eq!(bytes.len(), 84 + count * 50);
    assert_eq!(count, result.triangles);
    assert!(count > 100);
    for triangle in bytes[84..].chunks_exact(50) {
        for vertex in triangle[12..48].chunks_exact(12) {
            let coordinates: Vec<_> = vertex.chunks_exact(4).map(|word| f32::from_le_bytes(word.try_into().unwrap()) as f64).collect();
            let radius = coordinates.iter().map(|v| v*v).sum::<f64>().sqrt();
            assert!((radius - 5.).abs() < 1e-4, "encoded vertex departs from source sphere: {radius}");
        }
    }
    let Geometry::Brep(current) = &controller.document().entities[&id].geometry else { panic!("source changed type") };
    assert_eq!(current.source(), original_source);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
}

#[test]
fn surface_worker_receipt_rejects_stale_cursor_camera_and_document() {
    use om9_core::budget::memory::MemoryClass;
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.2, &limits).unwrap();
    let mut document = Document::with_template_tolerance("surface worker");
    let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = om9_app::session::Controller::new(document, limits).unwrap();
    let camera = Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let cursor = camera.project([2., 1., 20_f64.sqrt()], viewport).unwrap().unwrap().screen;
    let candidates: Vec<_> = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
        .into_iter().filter(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface).collect();
    assert_eq!(candidates.len(), 1);
    let old_floor = 2 * std::mem::size_of::<om9_scene::osnap::SnapCandidate>() as u64;
    for extra_path in [false, true] {
        let mut oversized = candidates.clone();
        if extra_path { oversized[0].surface_source.as_mut().unwrap().path.reserve_exact(4096); }
        else { oversized.reserve_exact(4096); }
        let insufficient = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, old_floor, None).unwrap();
        assert!(matches!(om9_app::session::SurfaceSnapJob::start(&controller, oversized, camera.clone(), viewport, cursor, insufficient), Err(om9_core::Error::Budget(_))),
            "reservation must include retained {} capacity", if extra_path { "instance path" } else { "candidate vector" });
        assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    }
    let account = controller.coordinator().external_memory_account();
    let held_slot = account.try_acquire_surface_snap().unwrap();
    let blocked_guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    assert!(om9_app::session::SurfaceSnapJob::start(&controller, candidates.clone(), camera.clone(), viewport, cursor, blocked_guard).is_err());
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0, "rejected worker must release its reservation");
    drop(held_slot);
    let get_receipt = |controller: &om9_app::session::Controller| {
        let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        let mut job = om9_app::session::SurfaceSnapJob::start(controller, candidates.clone(), camera.clone(), viewport, cursor, guard).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        loop {
            if let Some(result) = job.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline, "surface worker timed out");
            std::thread::yield_now();
        }
    };
    let receipt = get_receipt(&controller);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 1024 * 1024);
    let hit = receipt.accept(&controller, &camera, viewport, cursor).unwrap().unwrap();
    assert!((hit.world[2] - 20_f64.sqrt()).abs() < 1e-6);
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    assert!(get_receipt(&controller).accept(&controller, &camera, viewport, [cursor[0] + 1., cursor[1]]).is_err());
    let changed_camera = Camera::orthographic([1., 0., 20.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
    assert!(get_receipt(&controller).accept(&controller, &changed_camera, viewport, cursor).is_err());
    let another = om9_app::session::Controller::new(Document::with_template_tolerance("another document"), limits).unwrap();
    assert!(get_receipt(&controller).accept(&another, &camera, viewport, cursor).is_err());
    let same_ids = om9_app::session::Controller::new(controller.document().clone(), limits).unwrap();
    assert!(get_receipt(&controller).accept(&same_ids, &camera, viewport, cursor).is_err(),
        "a replacement document allocation with identical IDs and revisions must reject an old receipt");
    let wrong_guard = another.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    assert!(om9_app::session::SurfaceSnapJob::start(&controller, candidates.clone(), camera.clone(), viewport, cursor, wrong_guard).is_err());
    let small_guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1, None).unwrap();
    assert!(om9_app::session::SurfaceSnapJob::start(&controller, candidates.clone(), camera.clone(), viewport, cursor, small_guard).is_err());
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    let mut job = om9_app::session::SurfaceSnapJob::start(&controller, candidates.clone(), camera.clone(), viewport, cursor, guard).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let receipt = loop {
        if let Some(result) = job.try_result() { break result.unwrap(); }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    job.cancel();
    assert!(matches!(receipt.accept(&controller, &camera, viewport, cursor), Err(om9_core::Error::Canceled)));
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    let mut cancelled = om9_app::session::SurfaceSnapJob::start(&controller, candidates.clone(), camera.clone(), viewport, cursor, guard).unwrap();
    cancelled.cancel();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if let Some(result) = cancelled.try_result() {
            match result {
                Err(om9_core::Error::Canceled) => {},
                Ok(receipt) => assert!(matches!(receipt.accept(&controller, &camera, viewport, cursor), Err(om9_core::Error::Canceled))),
                Err(error) => panic!("unexpected cancellation result: {error}"),
            }
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
    assert_eq!(controller.coordinator().external_reserved_bytes(MemoryClass::Kernel), 0);
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    let dropped = om9_app::session::SurfaceSnapJob::start(&controller, vec![candidates[0].clone(); 20], camera.clone(), viewport, cursor, guard).unwrap();
    drop(dropped);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while controller.coordinator().external_reserved_bytes(MemoryClass::Kernel) != 0 {
        assert!(std::time::Instant::now() < deadline, "dropped worker must eventually release its reservation");
        std::thread::yield_now();
    }
    let stale_selection = get_receipt(&controller);
    controller.select_objects(&[id]).unwrap();
    assert!(stale_selection.accept(&controller, &camera, viewport, cursor).is_err());
    let stale_preview = get_receipt(&controller);
    controller.preview_gumball(Transform::translation([1., 0., 0.])).unwrap();
    assert!(stale_preview.accept(&controller, &camera, viewport, cursor).is_err());
    controller.clear_gumball_preview();
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    let mut running = om9_app::session::SurfaceSnapJob::start(&controller, vec![candidates[0].clone(); 20], camera.clone(), viewport, cursor, guard).unwrap();
    controller.numeric_move([1., 0., 0.]).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let stale = loop {
        if let Some(result) = running.try_result() { break result.unwrap(); }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    assert!(stale.accept(&controller, &camera, viewport, cursor).is_err());
}

#[test]
fn face_refinement_chooses_the_first_forward_sphere_intersection() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    assert_eq!(sphere.display.faces.len(), 1);
    let z = (25_f64 - 1. - 0.25).sqrt();
    let hit = om9_kernel::measurement::brep_face_ray_hit(
        &sphere, sphere.display.faces[0], Transform::identity(),
        [1., 0.5, 20.], [0., 0., -1.], [1., 0.5, -z], 0.001, &limits,
    ).unwrap();
    assert!((hit[2] - z).abs() < 1e-7, "the back-side display reference must not select the rear hit: {hit:?}");
    let inside = om9_kernel::measurement::brep_face_ray_hit(
        &sphere, sphere.display.faces[0], Transform::identity(),
        [1., 0.5, 0.], [0., 0., 1.], [1., 0.5, -z], 0.001, &limits,
    ).unwrap();
    assert!((inside[2] - z).abs() < 1e-7, "an inside-origin ray must use its forward exit");
    assert!(om9_kernel::measurement::brep_face_ray_hit(
        &sphere, sphere.display.faces[0], Transform::identity(),
        [1., 0.5, 20.], [0., 0., 1.], [1., 0.5, z], 0.001, &limits,
    ).is_err(), "a backwards-facing ray must not select behind its origin");
}

#[test]
fn a_single_box_exposes_each_intersected_face_for_canonical_depth_comparison() {
    let limits = Limits::default();
    let brep = om9_kernel::creation::primitive(
        om9_kernel::creation::Primitive::Box { x_mm: 10., y_mm: 10., z_mm: 10. }, 0.1, &limits,
    ).unwrap();
    let entity = Entity::new("box", Geometry::Brep(brep), None);
    let id = entity.id;
    let mut document = Document::with_template_tolerance("box face candidates");
    document.entities.insert(id, entity);
    let controller = om9_app::session::Controller::new(document, limits).unwrap();
    let camera = Camera::orthographic([5., 5., 30.], [5., 5., 0.], [0., 1., 0.], 15.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let cursor = camera.project([4., 4., 10.], viewport).unwrap().unwrap().screen;
    let candidates = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap();
    let surfaces: Vec<_> = candidates.iter().filter(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface && candidate.root == Some(id)).collect();
    assert_eq!(surfaces.len(), 2, "both the front and back face must reach canonical refinement");
    assert_ne!(surfaces[0].surface_source.as_ref().unwrap().face, surfaces[1].surface_source.as_ref().unwrap().face);
    let single: Vec<_> = surfaces.iter().map(|candidate| controller.refine_surface_snap(candidate, &camera, viewport).unwrap().world).collect();
    let Geometry::Brep(brep) = &controller.document().entities[&id].geometry else { panic!("B-rep expected") };
    let face_ids: Vec<_> = surfaces.iter().map(|candidate| candidate.surface_source.as_ref().unwrap().face.unwrap()).collect();
    let ray = camera.visible_ray(cursor, viewport).unwrap();
    let batch = om9_kernel::measurement::brep_face_ray_hits(brep, &face_ids, Transform::identity(), ray.origin, ray.direction, 0.001, &limits).unwrap();
    let fixed = om9_kernel::measurement::brep_face_ray_hits_fixed(brep, &face_ids, Transform::identity(), ray.origin, ray.direction, 0.001, &limits).unwrap();
    assert_eq!(fixed.as_slice(), batch.as_slice());
    for (a, b) in batch.iter().zip(&single) {
        assert!(a.iter().zip(b).all(|(x, y)| (x-y).abs() < 1e-7));
    }
    let mut invalid = face_ids.clone(); invalid.push(uuid::Uuid::nil());
    assert!(om9_kernel::measurement::brep_face_ray_hits(brep, &invalid, Transform::identity(), ray.origin, ray.direction, 0.001, &limits).is_err());
    assert!(om9_kernel::measurement::brep_face_ray_hits(brep, &[], Transform::identity(), ray.origin, ray.direction, 0.001, &limits).is_err());
    assert!(om9_kernel::measurement::brep_face_ray_hits(brep, &[face_ids[0]; 9], Transform::identity(), ray.origin, ray.direction, 0.001, &limits).is_err());
    let guard = controller.coordinator().reserve_external_memory(om9_core::budget::memory::MemoryClass::Kernel, 1024 * 1024, None).unwrap();
    let mut job = om9_app::session::SurfaceSnapJob::start(&controller, surfaces.iter().map(|candidate| (*candidate).clone()).collect(), camera.clone(), viewport, cursor, guard).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let receipt = loop {
        if let Some(result) = job.try_result() { break result.unwrap(); }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    };
    let front = receipt.accept(&controller, &camera, viewport, cursor).unwrap().unwrap();
    assert!((front.world[2] - 10.).abs() < 1e-7);
    let mut z: Vec<_> = single.iter().map(|point| point[2]).collect();
    z.sort_by(f64::total_cmp);
    assert!((z[0] - 0.).abs() < 1e-7 && (z[1] - 10.).abs() < 1e-7);
}

#[test]
fn overlapping_surface_candidates_prefer_the_visible_front_sphere() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap();
    let first = Entity::new("first", Geometry::Brep(sphere.clone()), None);
    let second = Entity::new("second", Geometry::Brep(sphere), None);
    let (mut back, mut front) = if first.id < second.id { (first, second) } else { (second, first) };
    back.placement = Transform::translation([0., 0., -5.]);
    front.placement = Transform::translation([0., 0., 5.]);
    let front_id = front.id;
    let mut document = Document::with_template_tolerance("overlapping surface picking");
    document.entities.insert(back.id, back);
    document.entities.insert(front.id, front);
    let controller = om9_app::session::Controller::new(document, limits).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [640., 640.] };
    let expected = [1., 0.5, 5. + (9_f64 - 1. - 0.25).sqrt()];
    for camera in [
        Camera::orthographic([0., 0., 30.], [0., 0., 0.], [0., 1., 0.], 16.).unwrap(),
        Camera::perspective([0., 0., 30.], [0., 0., 0.], [0., 1., 0.], 0.8).unwrap(),
    ] {
        let cursor = camera.project(expected, viewport).unwrap().unwrap().screen;
        let snaps = om9_scene::osnap::candidates(
            controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None,
        ).unwrap();
        let surfaces: Vec<_> = snaps.iter().filter(|snap| snap.kind == om9_scene::osnap::SnapKind::Surface).collect();
        assert_eq!(surfaces.len(), 2);
        assert_eq!(surfaces[0].root, Some(front_id), "visible sphere must win regardless of ID");
        let refined = controller.refine_surface_snap(surfaces[0], &camera, viewport).unwrap();
        let error = refined.world.iter().zip(expected).map(|(a,b)| (a-b).powi(2)).sum::<f64>().sqrt();
        assert!(error < 1e-6, "wrong surface point: {error}");
        let guard = controller.coordinator().reserve_external_memory(om9_core::budget::memory::MemoryClass::Kernel, 1024 * 1024, None).unwrap();
        let mut worker = om9_app::session::SurfaceSnapJob::start(&controller, surfaces.iter().map(|candidate| (*candidate).clone()).collect(), camera.clone(), viewport, cursor, guard).unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let receipt = loop {
            if let Some(result) = worker.try_result() { break result.unwrap(); }
            assert!(std::time::Instant::now() < deadline);
            std::thread::yield_now();
        };
        let background = receipt.accept(&controller, &camera, viewport, cursor).unwrap().unwrap();
        assert_eq!(background, refined, "worker and direct refinement must produce the same candidate");
    }
}

#[test]
fn perspective_surface_snap_matches_sphere_and_rejects_outdated_context() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let mut document = Document::with_template_tolerance("perspective sphere picking");
    let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = om9_app::session::Controller::new(document, limits).unwrap();
    let camera = Camera::perspective([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 0.8).unwrap();
    let viewport = Viewport { origin: [12., 18.], size: [640., 480.] };
    let expected = [2., 1., 20_f64.sqrt()];
    let screen = camera.project(expected, viewport).unwrap().unwrap().screen;
    let snaps = om9_scene::osnap::candidates(
        controller.scene().unwrap(), &camera, viewport, screen, Default::default(), None,
    ).unwrap();
    let candidate = snaps.iter().find(|snap| {
        snap.kind == om9_scene::osnap::SnapKind::Surface && snap.root == Some(id)
    }).expect("perspective surface candidate");
    let refined = controller.refine_surface_snap(candidate, &camera, viewport).unwrap();
    let distance = refined.world.iter().zip(expected)
        .map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt();
    assert!(distance < 1e-6, "analytic sphere mismatch: {distance}");
    let other_camera = Camera::perspective([1., 0., 20.], [0., 0., 0.], [0., 1., 0.], 0.8).unwrap();
    assert!(controller.refine_surface_snap(candidate, &other_camera, viewport).is_err());
    controller.select_objects(&[id]).unwrap();
    controller.numeric_move([1., 0., 0.]).unwrap();
    assert!(controller.refine_surface_snap(candidate, &camera, viewport).is_err());
}

#[test]
fn perspective_surface_snap_refines_shared_triangle_edge_after_preview() {
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::plane([0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2., 2., &limits).unwrap();
    let mut document = Document::with_template_tolerance("triangle seam snap");
    let mut entity = Entity::new("plane", Geometry::Surface(surface), None);
    entity.placement = Transform::translation([-25., -15., 0.]).compose(Transform::scale([1.5, 0.75, 1.]).unwrap()).unwrap();
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = om9_app::session::Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    controller.preview_gumball(Transform::translation([0., 20., 0.])).unwrap();
    let camera = Camera::perspective([0., 0., 80.], [0., 0., 0.], [0., 1., 0.], 0.9).unwrap();
    let viewport = Viewport { origin: [10., 20.], size: [800., 600.] };
    let expected = [-23.5, 5.75, 0.];
    let screen = camera.project(expected, viewport).unwrap().unwrap().screen;
    let candidates = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, screen, Default::default(), None).unwrap();
    let candidate = candidates.iter().find(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface && candidate.root == Some(id)).expect("shared triangle edge candidate");
    let hit = controller.refine_surface_snap(candidate, &camera, viewport).unwrap();
    assert!(hit.world.iter().zip(expected).all(|(a, b)| (a - b).abs() < 1e-7), "{:?}", hit.world);
    controller.clear_gumball_preview();
    assert!(controller.refine_surface_snap(candidate, &camera, viewport).is_err());
}

#[test]
fn surface_candidate_index_tracks_preview_and_cancellation() {
    let limits = Limits::default();
    let mut document = Document::with_template_tolerance("surface preview index");
    let entity = Entity::new("sphere", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut controller = om9_app::session::Controller::new(document, limits).unwrap();
    controller.select_objects(&[id]).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let find = |controller: &om9_app::session::Controller, x: f64| {
        let camera = Camera::orthographic([x, 0., 100.], [x, 0., 0.], [0., 1., 0.], 20.).unwrap();
        let candidates = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, [300., 300.], Default::default(), None).unwrap();
        let candidate = candidates.iter().find(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface && candidate.root == Some(id)).expect("surface candidate");
        let point = controller.refine_surface_snap(candidate, &camera, viewport).unwrap().world;
        assert!((point[0] - x).abs() < 1e-7 && (point[2] - 3.).abs() < 1e-7, "{point:?}");
    };
    find(&controller, 0.);
    controller.preview_gumball(Transform::translation([1000., 0., 0.])).unwrap();
    find(&controller, 1000.);
    controller.clear_gumball_preview();
    find(&controller, 0.);
}

#[test]
fn surface_snap_uses_first_hit_inside_the_camera_clip_slab() {
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::cylinder(
        [0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2., 10., &limits,
    ).unwrap();
    let sphere = om9_kernel::brep::sphere_brep(2., 0.1, &limits).unwrap();
    for is_brep in [false, true] {
    let mut document = Document::with_template_tolerance("clipped surface");
    let geometry = if is_brep { Geometry::Brep(sphere.clone()) } else { Geometry::Surface(surface.clone()) };
    let mut entity = Entity::new("surface", geometry, None);
    if is_brep { entity.placement = Transform::translation([0., 0., 5.]); }
    let id = entity.id;
    document.entities.insert(id, entity);
    let controller = om9_app::session::Controller::new(document, limits).unwrap();
    for perspective in [false, true] {
    let mut camera = if perspective {
        Camera::perspective([20., 0., 5.], [0., 0., 5.], [0., 0., 1.], 0.8).unwrap()
    } else { Camera::orthographic([20., 0., 5.], [0., 0., 5.], [0., 0., 1.], 10.).unwrap() };
    camera.near_mm = 20.;
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let screen = [300., 300.];
    let snaps = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, screen, Default::default(), None).unwrap();
    let candidate = snaps.iter().find(|snap| snap.kind == om9_scene::osnap::SnapKind::Surface && snap.root == Some(id)).expect("visible rear surface candidate");
    let hit = controller.refine_surface_snap(candidate, &camera, viewport).unwrap();
    assert!((hit.world[0] + 2.).abs() < 1e-7, "{:?}", hit.world);
    camera.far_mm = 21.;
    let clipped = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, screen, Default::default(), None).unwrap();
    assert!(!clipped.iter().any(|snap| snap.kind == om9_scene::osnap::SnapKind::Surface && snap.root == Some(id)));
    }
    }
}

#[test]
fn off_axis_sphere_snap_respects_camera_clip_planes() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(2., 0.05, &limits).unwrap();
    let mut document = Document::with_template_tolerance("off-axis clipping");
    let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let controller = om9_app::session::Controller::new(document, limits).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let cursor = [330., 270.];
    for perspective in [false, true] {
        let mut camera = if perspective {
            Camera::perspective([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 0.8).unwrap()
        } else {
            Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 10.).unwrap()
        };
        camera.near_mm = 20.;
        camera.far_mm = 23.;
        let ray = camera.ray(cursor, viewport).unwrap();
        let dot = |a: [f64; 3], b: [f64; 3]| a.into_iter().zip(b).map(|(x, y)| x * y).sum::<f64>();
        let b = dot(ray.origin, ray.direction);
        let discriminant = b * b - (dot(ray.origin, ray.origin) - 4.);
        assert!(discriminant > 0.);
        let t = -b + discriminant.sqrt();
        let expected = std::array::from_fn::<_, 3, _>(|i| ray.origin[i] + t * ray.direction[i]);
        let candidates = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap();
        let candidate = candidates.iter().find(|c| c.kind == om9_scene::osnap::SnapKind::Surface && c.root == Some(id)).expect("visible back surface");
        let point = controller.refine_surface_snap(candidate, &camera, viewport).unwrap().world;
        assert!(point.iter().zip(expected).all(|(actual, want)| (actual - want).abs() < 1e-6), "{point:?} versus {expected:?}");
        camera.far_mm = 20.5;
        let clipped = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap();
        assert!(!clipped.iter().any(|c| c.kind == om9_scene::osnap::SnapKind::Surface && c.root == Some(id)));
    }
}

#[test]
fn nurbs_cylinder_snap_selects_first_forward_hit_and_rejects_stale_context() {
    let limits = Limits::default();
    let surface = om9_core::surface_kernel::cylinder(
        [0., 0., 0.], [1., 0., 0.], [0., 1., 0.], 2., 10., &limits,
    ).unwrap();
    for scale_x in [2., -2.] {
        let mut document = Document::with_template_tolerance("NURBS cylinder snap");
        let mut entity = Entity::new("cylinder", Geometry::Surface(surface.clone()), None);
        entity.placement = Transform::scale([scale_x, 0.5, 1.5]).unwrap();
        let id = entity.id;
        document.entities.insert(id, entity);
        let mut controller = om9_app::session::Controller::new(document, limits).unwrap();
        let camera = Camera::perspective([20., 0., 7.5], [0., 0., 7.5], [0., 0., 1.], 0.8).unwrap();
        let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
        let screen = [300., 300.];
        let snaps = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, screen, Default::default(), None).unwrap();
        let candidate = snaps.iter().find(|snap| snap.kind == om9_scene::osnap::SnapKind::Surface && snap.root == Some(id)).expect("cylinder candidate");
        let hit = controller.refine_surface_snap(candidate, &camera, viewport).unwrap();
        assert!((hit.world[0] - 4.).abs() < 1e-7 && hit.world[1].abs() < 1e-7 && (hit.world[2] - 7.5).abs() < 1e-7, "{:?}", hit.world);
        let other_camera = Camera::perspective([20., 1., 7.5], [0., 0., 7.5], [0., 0., 1.], 0.8).unwrap();
        assert!(controller.refine_surface_snap(candidate, &other_camera, viewport).is_err());
        controller.select_objects(&[id]).unwrap();
        controller.numeric_move([1., 0., 0.]).unwrap();
        assert!(controller.refine_surface_snap(candidate, &camera, viewport).is_err());
    }
}

#[test]
fn indexed_brep_face_ray_resolves_every_transformed_box_face() {
    let limits = Limits::default();
    let brep = om9_kernel::brep::box_brep([2., 4., 6.], 0.1, &limits).unwrap();
    for scale_x in [2., -2.] {
        let placement = Transform::scale([scale_x, 0.5, 1.5]).unwrap();
        let mut document = Document::new("placed box face picking");
        let mut entity = Entity::new("box", Geometry::Brep(brep.clone()), None);
        entity.placement = placement;
        document.entities.insert(entity.id, entity);
        let controller = om9_app::session::Controller::new(document, limits).unwrap();
        let center = [scale_x, 1., 4.5];
        let viewport = Viewport {
            origin: [0., 0.],
            size: [600., 600.],
        };
        for axis in 0..3 {
            for side in [-1., 1.] {
                let mut eye = center;
                eye[axis] += side * 30.;
                let up = if axis == 2 {
                    [0., 1., 0.]
                } else {
                    [0., 0., 1.]
                };
                let camera = Camera::orthographic(eye, center, up, 15.).unwrap();
                let screen = [300., 300.];
                let hit = controller
                    .scene()
                    .unwrap()
                    .pick_kind(
                        screen,
                        &camera,
                        viewport,
                        1.,
                        PickIntent::Select,
                        PickKind::Face,
                    )
                    .unwrap()
                    .unwrap();
                let Subelement::BrepFace(face) = hit.subset else {
                    panic!("B-rep face expected")
                };
                let ray = camera.ray(screen, viewport).unwrap();
                let point = om9_kernel::measurement::brep_face_ray_hit(
                    &brep,
                    face,
                    placement,
                    ray.origin,
                    ray.direction,
                    hit.world_point,
                    1e-7,
                    &limits,
                )
                .unwrap();
                let grouped = om9_kernel::measurement::brep_face_ray_hits(
                    &brep, &[face, face], placement, ray.origin, ray.direction, 1e-7, &limits,
                ).unwrap();
                assert!(grouped.iter().all(|p| p.iter().zip(point).all(|(a,b)| (a-b).abs() < 1e-7)));
                let expected_axis = match (axis, side < 0.) {
                    (0, true) => (2. * scale_x).min(0.),
                    (0, false) => (2. * scale_x).max(0.),
                    (_, true) => 0.,
                    (_, false) => [4., 2., 9.][axis],
                };
                assert!(
                    (point[axis] - expected_axis).abs() < 1e-7,
                    "axis={axis} side={side} point={point:?}"
                );
            }
        }
    }
}

#[test]
fn surface_candidate_resolves_the_correct_shared_instance_and_rejects_stale_context() {
    use om9_core::model::Definition;
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.5, &limits).unwrap();
    let mut document = Document::with_template_tolerance("shared sphere snaps");
    let member = Entity::new("member", Geometry::Brep(sphere), None);
    let member_id = member.id;
    let definition = Definition::new("sphere definition", vec![member]);
    let definition_id = definition.id;
    document.definitions.insert(definition_id, definition);
    let mut first = Entity::new("left", Geometry::Instance(definition_id), None);
    first.placement = Transform::translation([-10., 0., 0.]);
    let first_id = first.id;
    let mut second = Entity::new("right", Geometry::Instance(definition_id), None);
    second.placement = Transform::translation([10., 0., 0.]);
    let second_id = second.id;
    document.entities.insert(first_id, first);
    document.entities.insert(second_id, second);
    let mut controller = om9_app::session::Controller::new(document, limits).unwrap();
    let camera = Camera::orthographic([0., 0., 40.], [0., 0., 0.], [0., 1., 0.], 40.).unwrap();
    let viewport = Viewport {
        origin: [0., 0.],
        size: [800., 800.],
    };
    let mut saved = None;
    for (root, x) in [(first_id, -10.), (second_id, 10.)] {
        let exact = [
            x + 2.13,
            1.37,
            (25.0_f64 - 2.13_f64.powi(2) - 1.37_f64.powi(2)).sqrt(),
        ];
        let cursor = camera.project(exact, viewport).unwrap().unwrap().screen;
        let snaps = om9_scene::osnap::candidates(
            controller.scene().unwrap(),
            &camera,
            viewport,
            cursor,
            Default::default(),
            None,
        )
        .unwrap();
        let candidate = snaps
            .iter()
            .find(|candidate| {
                candidate.kind == om9_scene::osnap::SnapKind::Surface
                    && candidate.root == Some(root)
            })
            .unwrap();
        let source = candidate.surface_source.as_ref().unwrap();
        assert_eq!(source.path, vec![member_id]);
        assert_eq!(source.leaf, member_id);
        assert_eq!(source.source.object, root);
        let refined = controller
            .refine_surface_snap(candidate, &camera, viewport)
            .unwrap();
        assert!(
            refined
                .world
                .iter()
                .zip(exact)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt()
                < 1e-7
        );
        saved = Some(candidate.clone());
    }
    controller.select_objects(&[second_id]).unwrap();
    controller.numeric_move([0., 0., 1.]).unwrap();
    assert!(
        controller
            .refine_surface_snap(&saved.unwrap(), &camera, viewport)
            .is_err(),
        "edited occurrence must invalidate the old candidate"
    );
}

#[test]
fn indexed_face_refinement_respects_a_trimmed_through_hole() {
    use om9_kernel::{creation::{self, Primitive, Profile}, editing::{self, BooleanOp, BooleanOutcome}};
    let limits = Limits::default();
    let profile = Profile::new(vec![[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 0.]]).unwrap();
    let block = creation::extrude(&profile, [0., 0., 10.], true, 0.2, &limits).unwrap();
    let cutter = creation::primitive(Primitive::Cylinder { radius_mm: 2., height_mm: 10. }, 0.2, &limits).unwrap();
    let BooleanOutcome::Body(body) = editing::boolean(&block, &cutter, BooleanOp::Difference, 0.2, &limits).unwrap() else { panic!("cut must retain a solid") };
    let mut document = Document::with_template_tolerance("trimmed hole snap");
    let entity = Entity::new("drilled block", Geometry::Brep(body), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let controller = om9_app::session::Controller::new(document, limits).unwrap();
    let camera = Camera::orthographic([0., 0., 30.], [0., 0., 0.], [0., 1., 0.], 14.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [700., 700.] };
    let cursor = camera.project([3., 1., 10.], viewport).unwrap().unwrap().screen;
    let snaps = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap();
    let candidate = snaps.iter().find(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface && candidate.root == Some(id)).unwrap();
    let refined = controller.refine_surface_snap(candidate, &camera, viewport).unwrap();
    assert!((refined.world[2] - 10.).abs() < 1e-7);
    let mut through_hole = candidate.clone();
    through_hole.surface_source.as_mut().unwrap().cursor = camera.project([0., 0., 10.], viewport).unwrap().unwrap().screen;
    assert!(controller.refine_surface_snap(&through_hole, &camera, viewport).is_err(), "the trimmed top face must not intersect inside its hole");
    let Geometry::Brep(body) = &controller.document().entities[&id].geometry else { panic!("B-rep expected") };
    let hole_cursor = through_hole.surface_source.as_ref().unwrap().cursor;
    let ray = camera.visible_ray(hole_cursor, viewport).unwrap();
    let face = through_hole.surface_source.as_ref().unwrap().face.unwrap();
    assert!(om9_kernel::measurement::brep_face_ray_hits(body, &[face, face], Transform::identity(), ray.origin, ray.direction, 0.001, &limits).is_err(),
        "grouped refinement must also reject a ray through a trimmed hole");
}

#[test]
fn surface_snap_refines_near_both_sides_of_periodic_sphere_seam() {
    let limits = Limits::default();
    let sphere = om9_kernel::brep::sphere_brep(5., 0.1, &limits).unwrap();
    let mut document = Document::with_template_tolerance("sphere seam snap");
    let entity = Entity::new("sphere", Geometry::Brep(sphere), None);
    document.entities.insert(entity.id, entity);
    let controller = om9_app::session::Controller::new(document, limits).unwrap();
    let camera = Camera::orthographic([0., 0., 20.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    for y in [-0.0001_f64, 0.0, 0.0001] {
        let expected = [3.5, y, (25. - 3.5_f64.powi(2) - y * y).sqrt()];
        let cursor = camera.project(expected, viewport).unwrap().unwrap().screen;
        let candidates = om9_scene::osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap();
        let candidate = candidates.iter().find(|candidate| candidate.kind == om9_scene::osnap::SnapKind::Surface).expect("surface near seam");
        let refined = controller.refine_surface_snap(candidate, &camera, viewport).unwrap();
        let error = refined.world.iter().zip(expected).map(|(a,b)| (a-b).powi(2)).sum::<f64>().sqrt();
        assert!(error < 1e-6, "y={y} error={error}");
    }
}
