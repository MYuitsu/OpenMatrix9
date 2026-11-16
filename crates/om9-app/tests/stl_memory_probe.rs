// SPDX-License-Identifier: LGPL-2.1-or-later
#![cfg(all(feature = "occt-brep", windows))]

use om9_app::session::{Controller, StlExportProfile};
use om9_core::{budget::Limits, geometry::{Geometry, Transform}, model::{Document, Entity}};
use std::ffi::{c_char, c_int, c_void};

#[test]
#[ignore = "isolated trimmed-face process memory diagnostic"]
fn trimmed_surface_stl_memory_probe() {
    use om9_core::{rhino_semantics::{set_trimmed_surface,TrimmedSurface},transaction::Cancellation};
    let case=std::env::var("OM9_TRIM_STL_MEMORY_CASE").expect("choose plane or cylinder");
    let limits=Limits::default();
    let surface=match case.as_str() {
        "plane"=>om9_core::surface_kernel::plane([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],10.,10.,&limits).unwrap(),
        "cylinder"=>om9_core::surface_kernel::cylinder([0.,0.,0.],[1.,0.,0.],[0.,1.,0.],2.,5.,&limits).unwrap(),
        _=>panic!("unknown fixture"),
    };
    let outer=if case=="plane" {(0..1020).map(|i|{let a=std::f64::consts::TAU*i as f64/1020.;[0.5+0.4*a.cos(),0.5+0.4*a.sin()]}).collect()}
    else {let mut p=Vec::new();for side in 0..4 {for i in 0..255 {let t=i as f64/255.;p.push(match side {
        0=>[0.1*(1.-t)+0.3*t,0.2],1=>[0.3,0.2*(1.-t)+0.8*t],
        2=>[0.3*(1.-t)+0.1*t,0.8],_=>[0.1,0.8*(1.-t)+0.2*t],
    });}}p};
    let hole=if case=="plane" {vec![[0.4,0.4],[0.6,0.4],[0.6,0.6],[0.4,0.6]]}
        else {vec![[0.15,0.4],[0.2,0.4],[0.2,0.6],[0.15,0.6]]};
    let trim=TrimmedSurface {underlying:surface.clone(),outer,inners:vec![hole],reversed:false};
    let mut entity=Entity::new("trim probe",Geometry::Surface(surface),None);set_trimmed_surface(&mut entity,&trim,&limits).unwrap();let id=entity.id;
    let mut doc=Document::with_template_tolerance("trim STL memory");doc.entities.insert(id,entity);
    let folder=std::env::temp_dir().join(format!("om9-trim-stl-memory-{}",uuid::Uuid::new_v4()));std::fs::create_dir(&folder).unwrap();
    struct Cleanup(std::path::PathBuf);impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
    let _cleanup=Cleanup(folder.clone());let input=folder.join("source.om9");let output=folder.join("output.stl");
    om9_archive::Archive::save(&input,&doc,&limits,&Cancellation::default()).unwrap();
    let mut c=Controller::from_file(&input,limits).unwrap();c.select_objects(&[id]).unwrap();let document_before=c.document().clone();
    let before=om9_native_owner::process_memory().expect("before process memory");let start=std::time::Instant::now();
    let receipt=c.export_stl_selected_with_profile(&output,StlExportProfile {linear_tolerance_mm:0.005,angular_tolerance_rad:0.3}).unwrap();
    let elapsed=start.elapsed();let after=om9_native_owner::process_memory().expect("after process memory");
    assert!(receipt.triangles>0);assert_eq!(std::fs::metadata(&output).unwrap().len(),receipt.bytes as u64);assert!(c.document()==&document_before);
    println!("TRIM_STL_MEMORY case={case} edges=1024 triangles={} elapsed_ms={:.3} before_working={} before_peak_working={} before_private={} before_peak_pagefile={} after_working={} after_peak_working={} after_private={} after_peak_pagefile={}",
        receipt.triangles,elapsed.as_secs_f64()*1000.,before.working_set_bytes,before.peak_working_set_bytes,before.private_bytes,before.peak_pagefile_bytes,
        after.working_set_bytes,after.peak_working_set_bytes,after.private_bytes,after.peak_pagefile_bytes);
}

// Test-only ABI mirror: the callbacks observe native stages and do not retain
// or dereference any of the arrays owned by the C++ adapter.
#[repr(C)]
struct NativeSnapshot {
    source: *const u8, source_len: usize,
    points: *const [f64; 3], points_len: usize,
    triangles: *const [u32; 4], triangles_len: usize,
    segments: *const [u32; 3], segments_len: usize,
    vertices: *const u32, vertices_len: usize,
    face_edges: *const [u32; 2], face_edges_len: usize,
    faces_count: usize, edges_count: usize,
    topology_handle: *const c_void,
}
unsafe extern "C" {
    fn om9_occt_brep_snapshot_grouped(source: *const u8, source_len: usize, deflection: f64, angular_deflection: f64,
        max_points: usize, max_nodes: usize, max_bytes: usize,
        sink: unsafe extern "C" fn(*mut c_void, *const NativeSnapshot) -> c_int, context: *mut c_void,
        shell_sink: unsafe extern "C" fn(*mut c_void, *const u32, usize, *const om9_kernel::brep::EdgeNodeBinding, usize, f64) -> c_int,
        shell_context: *mut c_void, include_edge_bindings: c_int, optimize_export_sphere: c_int, message: *mut c_char, capacity: c_int) -> c_int;
}
#[derive(Default)]
struct StageSamples {
    groups: Option<om9_native_owner::ProcessMemory>,
    snapshot: Option<om9_native_owner::ProcessMemory>,
    triangles: usize,
}
unsafe extern "C" fn record_groups(context: *mut c_void, _: *const u32, _: usize,
    _: *const om9_kernel::brep::EdgeNodeBinding, _: usize, _: f64) -> c_int {
    // SAFETY: the probe passes a live StageSamples as context for the complete synchronous call.
    let samples = unsafe { &mut *context.cast::<StageSamples>() };
    samples.groups = om9_native_owner::process_memory();
    i32::from(samples.groups.is_none())
}
unsafe extern "C" fn record_snapshot(context: *mut c_void, snapshot: *const NativeSnapshot) -> c_int {
    if snapshot.is_null() { return 1; }
    // SAFETY: the adapter passes a live snapshot and the probe's live context until this callback returns.
    let samples = unsafe { &mut *context.cast::<StageSamples>() };
    samples.triangles = unsafe { (*snapshot).triangles_len };
    samples.snapshot = om9_native_owner::process_memory();
    i32::from(samples.snapshot.is_none())
}

/// Run each case in a fresh process; Windows peak values cover that process's
/// entire lifetime and cannot isolate allocations made by OCCT from Rust.
#[test]
#[ignore = "isolated release memory diagnostic"]
fn stl_memory_probe() {
    let case = std::env::var("OM9_STL_MEMORY_CASE").expect("choose light, dense, dense_native, dense_stage, repeated, repeated_instances, trimmed or nurbs");
    let (radius, tolerance, count) = match case.as_str() {
        "light" => (5., 0.05, 1),
        "dense" => (12., 0.001, 1),
        "dense_native" => (12., 0.001, 1),
        "dense_stage" => (12., 0.001, 1),
        "repeated" | "repeated_instances" => (5., 0.005, 8),
        "trimmed" | "nurbs" => (3., 0.01, 1),
        _ => panic!("unknown fixture"),
    };
    let tolerance = std::env::var("OM9_STL_MEMORY_TOLERANCE").ok().map(|value| value.parse::<f64>().unwrap()).unwrap_or(tolerance);
    assert!(tolerance.is_finite() && tolerance > 0.);
    let limits = Limits::default();
    let brep = om9_kernel::brep::sphere_brep(radius, 0.1, &limits).unwrap();
    let geometry = match case.as_str() {
        "trimmed" => {
            let block = om9_kernel::brep::box_brep([10.; 3], 0.1, &limits).unwrap();
            let cutter = om9_kernel::transform_ops::transform_brep(&brep, &Transform::translation([5., 5., 10.]), 0.1, &limits).unwrap();
            let pocket = om9_kernel::editing::boolean(&block, &cutter, om9_kernel::editing::BooleanOp::Difference, 0.1, &limits).unwrap();
            Geometry::Brep(pocket.body().unwrap().clone())
        }
        "nurbs" => Geometry::Surface(om9_core::geometry::NurbsSurface {
            degree: [2, 2], size: [3, 3],
            poles: (0..3).flat_map(|i| (0..3).map(move |j| [5. * f64::from(i), 5. * f64::from(j), f64::from(i * i + j * j)])).collect(),
            weights: vec![1.; 9],
            knots: [vec![0., 0., 0., 1., 1., 1.], vec![0., 0., 0., 1., 1., 1.]],
            domain: [[0., 1.], [0., 1.]], periodic: [false; 2], reversed: false,
        }),
        _ => Geometry::Brep(brep.clone()),
    };
    let mut document = Document::with_template_tolerance("STL memory probe");
    let definition = if case == "repeated_instances" {
        let definition = om9_core::model::Definition::new("shared sphere", vec![Entity::new("sphere", Geometry::Brep(brep.clone()), None)]);
        let id = definition.id;
        document.definitions.insert(id, definition);
        Some(id)
    } else { None };
    let mut selection = Vec::new();
    for i in 0..count {
        let geometry = definition.map_or_else(|| geometry.clone(), Geometry::Instance);
        let mut entity = Entity::new("sphere", geometry, None);
        entity.placement = Transform::translation([i as f64 * radius * 3., 0., 0.]);
        selection.push(entity.id);
        document.entities.insert(entity.id, entity);
    }
    let mut controller = Controller::new(document, limits).unwrap();
    controller.select_objects(&selection).unwrap();
    if let Some(path) = std::env::var_os("OM9_STL_MEMORY_SOURCE_OUT") {
        let Geometry::Brep(body) = &geometry else { panic!("this fixture has no canonical B-Rep text"); };
        std::fs::write(path, body.source()).unwrap();
    }
    let before = om9_native_owner::process_memory().expect("before memory sample");
    if case == "dense_stage" {
        let mut samples = StageSamples::default();
        let mut message = [0 as c_char; 512];
        let context = (&mut samples as *mut StageSamples).cast::<c_void>();
        let start = std::time::Instant::now();
        // SAFETY: source, message, callbacks and context stay valid for the synchronous adapter call.
        let status = unsafe { om9_occt_brep_snapshot_grouped(brep.source().as_ptr(), brep.source().len(), tolerance * 0.5, 0.3,
            limits.max_points as usize, limits.max_nodes as usize, limits.max_bytes as usize,
            record_snapshot, context, record_groups, context, 1, 1, message.as_mut_ptr(), message.len() as c_int) };
        let elapsed = start.elapsed();
        // SAFETY: the native adapter writes a NUL-terminated message to this initialized buffer.
        let diagnostic = unsafe { std::ffi::CStr::from_ptr(message.as_ptr()) }.to_string_lossy();
        assert_eq!(status, 0, "{diagnostic}");
        let groups = samples.groups.expect("group callback sample");
        let snapshot = samples.snapshot.expect("snapshot callback sample");
        let after = om9_native_owner::process_memory().expect("after memory sample");
        assert!(samples.triangles > 0);
        println!("STL_STAGE triangles={} source_bytes={} elapsed_ms={:.3} before_private={} before_peak={} groups_private={} groups_peak={} snapshot_private={} snapshot_peak={} after_private={} after_peak={}",
            samples.triangles, brep.source().len(), elapsed.as_secs_f64() * 1000., before.private_bytes, before.peak_pagefile_bytes,
            groups.private_bytes, groups.peak_pagefile_bytes, snapshot.private_bytes, snapshot.peak_pagefile_bytes,
            after.private_bytes, after.peak_pagefile_bytes);
        return;
    }
    if case == "dense_native" {
        let start = std::time::Instant::now();
        let snapshot = om9_kernel::brep::read_brep_with_shell_groups_and_edges(brep.source(), tolerance * 0.5, 0.3, &limits).unwrap();
        let elapsed = start.elapsed();
        let after = om9_native_owner::process_memory().expect("after memory sample");
        let triangles = snapshot.brep().display().triangles.len();
        let retained = snapshot.retained_usage(&limits).unwrap();
        assert!(triangles > 0);
        println!("STL_SNAPSHOT_TRACKED bytes={} points={} nodes={}", retained.bytes, retained.points, retained.nodes);
        println!("STL_MEMORY case={case} count={count} radius_mm={radius} tolerance_mm={tolerance} triangles={triangles} elapsed_ms={:.3} before_working={} before_peak_working={} before_private={} before_peak_pagefile={} after_working={} after_peak_working={} after_private={} after_peak_pagefile={}",
            elapsed.as_secs_f64()*1000., before.working_set_bytes, before.peak_working_set_bytes, before.private_bytes, before.peak_pagefile_bytes,
            after.working_set_bytes, after.peak_working_set_bytes, after.private_bytes, after.peak_pagefile_bytes);
        return;
    }
    let destination = std::env::temp_dir().join(format!("om9-stl-memory-{}.stl", uuid::Uuid::new_v4()));
    let start = std::time::Instant::now();
    let receipt = controller.export_stl_selected_with_profile(&destination, StlExportProfile {
        linear_tolerance_mm: tolerance, angular_tolerance_rad: 0.3,
    }).unwrap();
    let elapsed = start.elapsed();
    let after = om9_native_owner::process_memory().expect("after memory sample");
    assert!(receipt.triangles > 0);
    assert_eq!(std::fs::metadata(&destination).unwrap().len(), receipt.bytes as u64);
    std::fs::remove_file(destination).unwrap();
    println!("STL_MEMORY case={case} count={count} radius_mm={radius} tolerance_mm={tolerance} triangles={} elapsed_ms={:.3} before_working={} before_peak_working={} before_private={} before_peak_pagefile={} after_working={} after_peak_working={} after_private={} after_peak_pagefile={}",
        receipt.triangles, elapsed.as_secs_f64()*1000., before.working_set_bytes, before.peak_working_set_bytes, before.private_bytes, before.peak_pagefile_bytes,
        after.working_set_bytes, after.peak_working_set_bytes, after.private_bytes, after.peak_pagefile_bytes);
}
