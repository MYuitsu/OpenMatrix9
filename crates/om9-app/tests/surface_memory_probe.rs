// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(all(feature = "occt-brep", windows))]
//! Run each case in a fresh release process; counters cover the whole process.
use om9_app::session::{Controller, SurfaceSnapJob};
use om9_core::{budget::{Limits, memory::MemoryClass}, geometry::{Geometry, Transform}, model::{Document, Entity}};
use om9_scene::{camera::{Camera, Viewport}, osnap::{self, SnapKind}};

#[test]
#[ignore = "isolated release Surface memory diagnostic"]
fn surface_memory_probe() {
    let case = std::env::var("OM9_SURFACE_MEMORY_CASE").expect("choose one, overlap50, prepare16 or holes16");
    let limits = Limits::default();
    if case == "prepare16" {
        use om9_kernel::{creation::{self, Primitive, Profile}, editing::{self, BooleanOp}};
        let profile = Profile::new(vec![[-10., -10., 0.], [10., -10., 0.], [10., 10., 0.], [-10., 10., 0.]]).unwrap();
        let mut body = creation::extrude(&profile, [0., 0., 10.], true, 0.2, &limits).unwrap();
        let cylinder = creation::primitive(Primitive::Cylinder { radius_mm: 0.7, height_mm: 10. }, 0.2, &limits).unwrap();
        for x in [-6., -2., 2., 6.] { for y in [-6., -2., 2., 6.] {
            let cutter = om9_kernel::transform_ops::transform_brep(&cylinder, &Transform::translation([x, y, 0.]), 0.2, &limits).unwrap();
            body = editing::boolean(&body, &cutter, BooleanOp::Difference, 0.2, &limits).unwrap().body().unwrap().clone();
        }}
        let path = std::env::var_os("OM9_SURFACE_BREP_PATH").expect("temporary fixture path");
        std::fs::write(path, body.source()).unwrap();
        println!("SURFACE_FIXTURE holes=16 source_bytes={} faces={}", body.source().len(), body.display.faces.len());
        return;
    }
    let count = match case.as_str() { "one" | "holes16" => 1, "overlap50" => 50, _ => panic!("unknown case") };
    let brep = if case == "holes16" {
        let path = std::env::var_os("OM9_SURFACE_BREP_PATH").expect("temporary fixture path");
        om9_kernel::brep::read_brep(&std::fs::read_to_string(path).unwrap(), 0.1, &limits).unwrap()
    } else { om9_kernel::brep::sphere_brep(5., 0.1, &limits).unwrap() };
    let source_bytes = brep.source().len();
    let display_triangles = brep.display.triangles.len();
    let faces = brep.display.faces.len();
    let mut document = Document::with_template_tolerance("surface memory probe");
    for i in 0..count {
        let mut entity = Entity::new("probe body", Geometry::Brep(brep.clone()), None);
        entity.placement = Transform::translation([0., 0., i as f64 * 0.02]);
        document.entities.insert(entity.id, entity);
    }
    let controller = Controller::new(document, limits).unwrap();
    let camera = Camera::orthographic([0., 0., 30.], [0., 0., 0.], [0., 1., 0.], 12.).unwrap();
    let viewport = Viewport { origin: [0., 0.], size: [600., 600.] };
    let cursor = [300., 300.];
    let candidates: Vec<_> = osnap::candidates(controller.scene().unwrap(), &camera, viewport, cursor, Default::default(), None).unwrap()
        .into_iter().filter(|candidate| candidate.kind == SnapKind::Surface).collect();
    assert!(candidates.len() >= count);
    let candidate_count = candidates.len();
    let before = om9_native_owner::process_memory().expect("before sample");
    // Diagnostic reservation only: not an asserted upper bound on native work.
    let guard = controller.coordinator().reserve_external_memory(MemoryClass::Kernel, 32 * 1024 * 1024, None).unwrap();
    let start = std::time::Instant::now();
    let mut job = SurfaceSnapJob::start(&controller, candidates, camera.clone(), viewport, cursor, guard).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
    let receipt = loop {
        if let Some(result) = job.try_result() { break result.unwrap(); }
        assert!(std::time::Instant::now() < deadline, "worker timeout");
        std::thread::yield_now();
    };
    let elapsed = start.elapsed();
    let hit = receipt.accept(&controller, &camera, viewport, cursor).unwrap().unwrap();
    let expected_z = if case == "holes16" { 10. } else { 5. + (count - 1) as f64 * 0.02 };
    assert!((hit.world[2] - expected_z).abs() < 1e-6);
    let after = om9_native_owner::process_memory().expect("after sample");
    println!("SURFACE_MEMORY case={case} candidates={candidate_count} source_bytes={source_bytes} faces={faces} display_triangles={display_triangles} elapsed_ms={:.3} before_private={} before_peak_commit={} before_peak_working={} after_private={} after_peak_commit={} after_peak_working={}",
        elapsed.as_secs_f64()*1000., before.private_bytes, before.peak_pagefile_bytes, before.peak_working_set_bytes,
        after.private_bytes, after.peak_pagefile_bytes, after.peak_working_set_bytes);
}
