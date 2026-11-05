// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exercise the actual egui/wgpu viewport bridge and its publication fence, not a mocked key table.
use eframe::{egui, egui_wgpu, wgpu};
use om9_app::gpu_viewport::{
    GpuViewport, SharedResourcePool, ViewportRenderRequest, scene_camera, submit_window, window_target_diagnostics, window_overlay_diagnostics, window_known_buffer_bytes,
};
use om9_app::session::Controller;
use om9_core::{
    budget::Limits,
    geometry::{Geometry, Mesh, Transform},
    model::{Document, Entity},
};
use om9_scene::{
    camera::{CPlane, Camera},
    scene::Scene,
};
use std::{
    collections::BTreeSet,
    sync::Arc,
    time::{Duration, Instant},
};
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    loop {
        match future.as_mut().poll(&mut context) {
            std::task::Poll::Ready(value) => return value,
            std::task::Poll::Pending => std::thread::yield_now(),
        }
    }
}
fn state() -> Option<egui_wgpu::RenderState> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        flags: Default::default(),
        memory_budget_thresholds: Default::default(),
        backend_options: Default::default(),
        display: Default::default(),
    });
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
        apply_limit_buckets: false,
    }))
    .ok()?;
    // Deliberately no timestamp feature: final publication must work using queue completion alone.
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .ok()?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let renderer = egui_wgpu::Renderer::new(&device, format, Default::default());
    Some(egui_wgpu::RenderState {
        adapter: adapter.clone(),
        available_adapters: vec![adapter],
        instance,
        device,
        queue,
        target_format: format,
        renderer: Arc::new(egui::mutex::RwLock::new(renderer)),
        surface_config: egui_wgpu::SurfaceConfig::LOW_LATENCY,
    })
}
struct TestScene {
    controller: Controller,
}
impl TestScene {
    fn new(document: Document) -> Self {
        Self { controller: Controller::new(document, Limits::default()).unwrap() }
    }
    fn preview(&mut self, transform: Transform) {
        let id = self.objects[0].root;
        self.controller.select_objects(&[id]).unwrap();
        self.controller.preview_gumball(transform).unwrap();
    }
}
impl std::ops::Deref for TestScene {
    type Target = Scene;
    fn deref(&self) -> &Scene { self.controller.scene().unwrap() }
}
fn scene(count: usize) -> TestScene {
    let mut document = Document::new("GPU publication");
    for i in 0..count {
        let mut e = Entity::new(
            "triangle",
            Geometry::Mesh(Mesh {
                vertices: vec![[-1., -1., 0.], [1., -1., 0.], [0., 1., 0.]],
                faces: vec![vec![0, 1, 2]],
                normals: vec![],
                colors: vec![],

                corner_uvs: Vec::new(),
            }),
            document.current_layer,
        );
        e.placement = Transform::translation([i as f64 * 2., 0., 0.]);
        document.entities.insert(e.id, e);
    }
    TestScene::new(document)
}
struct Harness {
    ctx: egui::Context,
    port: GpuViewport,
    pool: SharedResourcePool,
    state: egui_wgpu::RenderState,
    frame: u64,
    hidden_sources: Option<[om9_core::identity::ObjectId; 2]>,
    selected: BTreeSet<om9_core::identity::ObjectId>,
    preview_revision: u64,
}

#[test]
fn empty_viewport_publishes_without_geometry_arena_and_adds_one_for_a_mesh() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let empty = scene(0);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&empty, &camera, 0);
    assert_eq!(h.port.diagnostics().unwrap().unique_geometry_bytes, Some(0));
    assert_eq!(h.port.frame_stats().unwrap().objects, 0);
    let filled = scene(1);
    h.finish(&filled, &camera, 1);
    assert!(h.port.diagnostics().unwrap().unique_geometry_bytes.unwrap() >= 4 * 1024 * 1024);
    assert_eq!(h.port.frame_stats().unwrap().objects, 1);
}

#[test]
fn one_moving_view_reuses_other_images_and_shared_geometry() {
    let Some(state) = state() else { eprintln!("SKIP: no GPU adapter"); return };
    let pool = SharedResourcePool::default();
    let ctx = egui::Context::default();
    let mut ports: Vec<_> = (0..4).map(|_| {
        let mut port = GpuViewport::new(&state, pool.clone());
        port.enable_diagnostics();
        port
    }).collect();
    let scene = scene(4);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    let mut cameras = vec![camera; 4];
    let mut frame = 0_u64;
    let mut render = |ports: &mut Vec<GpuViewport>, cameras: &[Camera], moving: bool| {
        frame += 1;
        let input = egui::RawInput { screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(640., 480.))), ..Default::default() };
        let mut output = ctx.run_ui(input, |ui| {
            for (i, port) in ports.iter_mut().enumerate() {
                let camera = &cameras[i];
                port.set_render_request(ViewportRenderRequest { frame_id: frame, generation: 0, moving: moving && i == 0, render_due: true, quality: Default::default() });
                let origin = egui::pos2((i % 2) as f32 * 320., (i / 2) as f32 * 240.);
                port.show(ui, egui::Rect::from_min_size(origin, egui::vec2(320., 240.)), &state, &scene.controller, scene_camera(camera), camera.revision, camera, Some(&scene), CPlane::world_xy(), &BTreeSet::new(), 0, 0, None, false);
            }
        });
        output.textures_delta.clear();
        submit_window(&pool, &state);
        for port in ports.iter_mut() { port.window_submitted(&state.queue); }
        state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    while !ports.iter().all(GpuViewport::final_complete) {
        render(&mut ports, &cameras, false);
        assert!(Instant::now() < deadline, "viewports did not settle");
        std::thread::sleep(Duration::from_millis(1));
    }
    let before: Vec<_> = ports.iter().map(|p| p.diagnostics().unwrap()).collect();
    let window_targets = window_target_diagnostics(&pool, ports.iter()).unwrap();
    assert_eq!(window_targets.live_bytes, before.iter().map(|c| c.known_target_logical_bytes).sum::<u64>());
    assert_eq!(window_targets.unique_bytes, window_targets.live_bytes);
    assert_eq!(window_targets.retired_command_bytes, 0);
    assert!(window_targets.live_count >= 12, "four views must keep their independent targets");
    for _ in 0..4 {
        cameras[0].eye[0] += 0.25;
        cameras[0].revision += 1;
        render(&mut ports, &cameras, true);
    }
    let after: Vec<_> = ports.iter().map(|p| p.diagnostics().unwrap()).collect();
    assert!(after[0].encoded_frames > before[0].encoded_frames);
    assert!(before[0].main_scene_camera_snapshot_bytes > 0);
    assert!(before[0].main_scene_instance_snapshot_bytes > 0);
    println!("moving view camera snapshot bytes={}, instance snapshot bytes={}",
        after[0].main_scene_camera_snapshot_bytes - before[0].main_scene_camera_snapshot_bytes,
        after[0].main_scene_instance_snapshot_bytes - before[0].main_scene_instance_snapshot_bytes);
    for i in 0..4 {
        assert_eq!(after[i].scene_preparations, before[i].scene_preparations, "view {i}");
        assert_eq!(after[i].overlay_preparations, before[i].overlay_preparations, "view {i}");
        assert_eq!(after[i].geometry_upload_bytes, before[i].geometry_upload_bytes, "view {i}");
        assert_eq!(after[i].overlay_upload_bytes, before[i].overlay_upload_bytes, "view {i}");
        assert_eq!(after[i].target_rebuilds, before[i].target_rebuilds, "view {i}");
        if i != 0 {
            assert_eq!(after[i].encoded_frames, before[i].encoded_frames, "view {i}");
            assert_eq!(after[i].main_scene_camera_snapshot_bytes, before[i].main_scene_camera_snapshot_bytes, "view {i}");
            assert_eq!(after[i].main_scene_instance_snapshot_bytes, before[i].main_scene_instance_snapshot_bytes, "view {i}");
            assert_eq!(after[i].readiness_checks, before[i].readiness_checks, "view {i}");
            assert!(after[i].cached_frames > before[i].cached_frames, "view {i}");
        }
    }
    while !ports.iter().all(GpuViewport::final_complete) {
        render(&mut ports, &cameras, false);
        assert!(Instant::now() < deadline, "viewports did not settle after navigation");
    }
    let before_mode: Vec<_> = ports.iter().map(|p| p.diagnostics().unwrap()).collect();
    ports[0].set_display_mode(om9_scene::workspace::DisplayMode::Wireframe);
    render(&mut ports, &cameras, false);
    let after_mode: Vec<_> = ports.iter().map(|p| p.diagnostics().unwrap()).collect();
    assert!(after_mode[0].encoded_frames > before_mode[0].encoded_frames);
    for i in 0..4 {
        assert_eq!(after_mode[i].scene_preparations, before_mode[i].scene_preparations, "view {i}");
        assert_eq!(after_mode[i].geometry_upload_bytes, before_mode[i].geometry_upload_bytes, "view {i}");
        if i != 0 {
            assert_eq!(after_mode[i].encoded_frames, before_mode[i].encoded_frames, "view {i}");
        }
    }
}
impl Harness {
    fn new() -> Option<Self> {
        let state = state()?;
        let pool = SharedResourcePool::default();
        Some(Self {
            ctx: Default::default(),
            port: GpuViewport::new(&state, pool.clone()),
            pool,
            state,
            frame: 0,
            hidden_sources: None,
            selected: BTreeSet::new(),
            preview_revision: 0,
        })
    }
    fn frame(&mut self, scene: &TestScene, camera: &Camera, generation: u64, rect: egui::Rect) {
        self.frame_with_submission(scene, camera, generation, rect, false, true);
    }
    fn frame_with_submission(&mut self, scene: &TestScene, camera: &Camera, generation: u64, rect: egui::Rect, moving: bool, notify: bool) {
        self.frame += 1;
        self.port.set_render_request(ViewportRenderRequest {
            frame_id: self.frame,
            generation,
            moving,
            render_due: true,
            quality: Default::default(),
        });
        let raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(320., 240.),
            )),
            time: Some(self.frame as f64 / 60.),
            ..Default::default()
        };
        let mut output = self.ctx.run_ui(raw, |ui| {
            self.port.show(
                ui,
                rect,
                &self.state,
                &scene.controller,
                scene_camera(camera),
                camera.revision,
                camera,
                Some(scene),
                CPlane::world_xy(),
                &self.selected,
                self.selected.len() as u64,
                self.preview_revision,
                self.hidden_sources,
                false,
            );
        });
        output.textures_delta.clear();
        submit_window(&self.pool, &self.state);
        if notify { self.port.window_submitted(&self.state.queue); }
    }
    fn finish(&mut self, scene: &TestScene, camera: &Camera, generation: u64) {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            self.frame(scene, camera, generation, rect());
            self.state
                .device
                .poll(wgpu::PollType::wait_indefinitely())
                .unwrap();
            if self.port.final_complete() {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "final frame did not complete: {:?}",
                self.port.preparation_error()
            );
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}

#[test]
fn moving_view_waits_for_its_previous_submission_and_uses_latest_camera() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    camera.eye[0] += 1.;
    camera.revision += 1;
    h.frame_with_submission(&scene, &camera, 1, rect(), true, false);
    let encoded = h.port.diagnostics().unwrap().encoded_frames;
    camera.eye[0] += 1.;
    camera.revision += 1;
    h.frame_with_submission(&scene, &camera, 1, rect(), true, false);
    assert_eq!(h.port.diagnostics().unwrap().encoded_frames, encoded, "must not encode another frame before the previous submission is acknowledged");
    h.port.window_submitted(&h.state.queue);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    h.frame_with_submission(&scene, &camera, 1, rect(), true, true);
    assert_eq!(h.port.diagnostics().unwrap().encoded_frames, encoded + 1);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    h.frame_with_submission(&scene, &camera, 1, rect(), true, true);
    assert_eq!(h.port.displayed_camera(), Some(&camera));
}

#[test]
fn camera_only_motion_reuses_unchanged_instance_snapshot() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    let before = h.port.diagnostics().unwrap();
    for _ in 0..3 {
        camera.eye[0] += 0.01;
        camera.revision += 1;
        h.frame_with_submission(&scene, &camera, 1, rect(), true, true);
        h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
    let after = h.port.diagnostics().unwrap();
    assert!(after.encoded_frames > before.encoded_frames, "fixture must encode moving frames");
    assert!(after.main_scene_camera_snapshot_bytes > before.main_scene_camera_snapshot_bytes);
    assert_eq!(after.main_scene_instance_snapshot_bytes, before.main_scene_instance_snapshot_bytes,
        "unchanged instance data should not be uploaded for camera-only motion");
    let index_bytes = after.shared_scene_instance_snapshot_cpu_bytes.unwrap();
    println!("retained instance cache visible-index bytes={index_bytes}");
    assert!(index_bytes > 0 && index_bytes < 160, "one visible object should not retain a full serialized instance copy: {index_bytes}");
}

#[test]
fn cold_picking_bounds_preserve_rendered_objects_and_warming_does_not_reupload_geometry() {
    use om9_core::budget::memory::MemoryClass;
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(4);
    let before_document = scene.controller.coordinator().snapshot();
    let charge = scene.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache);
    let pending = scene.controller.coordinator().external_pending_bytes(MemoryClass::CpuCache);
    let near = Camera::perspective([3., -0.1, 0.1], [3., 0., 0.], [0., 0., 1.], 0.6).unwrap();
    assert!(scene.cached_object_spheres().is_none());
    for _ in 0..240 { assert!(!GpuViewport::close_mode(&near, Some(&scene))); }
    assert_eq!(scene.controller.coordinator().external_reserved_bytes(MemoryClass::CpuCache), charge);
    assert_eq!(scene.controller.coordinator().external_pending_bytes(MemoryClass::CpuCache), pending);
    let mut camera = Camera::perspective([3., -12., 7.], [3., 0., 0.], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    assert_eq!(h.port.frame_stats().unwrap().objects, 4);
    assert!(scene.cached_object_spheres().is_none(), "render must not materialize picking data");
    let before = h.port.diagnostics().unwrap();
    scene.spatial_index().unwrap();
    assert!(GpuViewport::close_mode(&near, Some(&scene)));
    camera.eye[0] += 0.25;
    camera.revision += 1;
    h.finish(&scene, &camera, 1);
    let after = h.port.diagnostics().unwrap();
    assert_eq!(h.port.frame_stats().unwrap().objects, 4);
    assert_eq!(after.geometry_upload_bytes, before.geometry_upload_bytes);
    assert_eq!(after.main_scene_instance_snapshot_bytes, before.main_scene_instance_snapshot_bytes);
    assert_eq!(after.scene_preparations, before.scene_preparations);
    assert_eq!(scene.controller.coordinator().snapshot(), before_document);
}

#[test]
fn changed_preview_replaces_the_instance_snapshot() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let mut scene = scene(1);
    h.selected.insert(scene.objects[0].root);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    let before = h.port.diagnostics().unwrap();
    scene.preview(Transform::translation([1., 0., 0.]));
    h.preview_revision += 1;
    h.finish(&scene, &camera, 0);
    let after = h.port.diagnostics().unwrap();
    assert!(after.encoded_frames > before.encoded_frames);
    assert!(after.main_scene_instance_snapshot_bytes > before.main_scene_instance_snapshot_bytes,
        "changed preview must create a fresh immutable instance snapshot");
    scene.preview(Transform::identity());
    h.preview_revision += 1;
    h.finish(&scene, &camera, 0);
    assert!(h.port.diagnostics().unwrap().main_scene_instance_snapshot_bytes > after.main_scene_instance_snapshot_bytes,
        "cancelled preview must replace the prior snapshot");
}

#[test]
fn scene_replacement_evicts_cache_still_held_by_another_viewport() {
    let Some(mut first) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    let state = first.state.clone();
    let pool = first.pool.clone();
    let mut second = Harness {
        ctx: Default::default(), port: GpuViewport::new(&state, pool.clone()), pool, state,
        frame: 0, hidden_sources: None, selected: BTreeSet::new(), preview_revision: 0,
    };
    first.port.enable_diagnostics();
    second.port.enable_diagnostics();
    let old = scene(1);
    let new = scene(1);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    first.finish(&old, &camera, 0);
    second.finish(&old, &camera, 0);
    assert!(second.port.diagnostics().unwrap().shared_scene_instance_snapshot_cpu_bytes.unwrap() > 0);
    first.frame(&new, &camera, 1, rect());
    assert_eq!(second.port.diagnostics().unwrap().shared_scene_instance_snapshot_cpu_bytes, Some(0),
        "old shared scene must release its optional cache while another viewport still owns it");
    assert!(second.port.displays_scene(&old), "evicting cache must not invalidate the retained completed image");
}

#[test]
fn submitted_instance_buffer_is_tracked_until_queue_completion() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    assert_eq!(h.port.diagnostics().unwrap().unique_instance_bytes, Some(160));
    let geometry = h.port.diagnostics().unwrap().unique_geometry_bytes.unwrap();
    assert!(geometry >= 4 * 1024 * 1024, "allocated vertex and index arenas must be counted");
    camera.eye[0] += 0.1;
    camera.revision += 1;
    h.frame_with_submission(&scene, &camera, 1, rect(), true, true);
    let inflight = h.port.diagnostics().unwrap();
    assert_eq!(inflight.inflight_instance_buffers, Some(1));
    assert_eq!(inflight.inflight_instance_bytes, Some(160));
    assert_eq!(inflight.unique_instance_bytes, Some(160), "cache and submission must be counted once together");
    assert_eq!(inflight.inflight_geometry_bytes, Some(geometry));
    assert_eq!(inflight.inflight_geometry_buffers, Some(2));
    assert_eq!(inflight.unique_geometry_bytes, Some(geometry), "live and submitted shared geometry must be counted once");
    assert!(inflight.shared_scene_buffer_bytes.unwrap() >= 160,
        "the same buffer is cached and in flight; counters overlap");
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let finished = h.port.diagnostics().unwrap();
    assert_eq!(finished.inflight_instance_buffers, Some(0));
    assert_eq!(finished.inflight_instance_bytes, Some(0));
    assert_eq!(finished.unique_instance_bytes, Some(160), "completed buffer remains resident in the cache");
    assert_eq!(finished.inflight_geometry_bytes, Some(0));
    assert_eq!(finished.unique_geometry_bytes, Some(geometry), "queue completion must not erase live geometry");
    assert!(finished.shared_scene_buffer_bytes.unwrap() >= 160, "queue completion must not evict the live cache");
}

#[test]
fn resized_away_targets_stay_tracked_through_gpu_completion() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    camera.eye[0] += 0.1;
    camera.revision += 1;
    let expected = 320u64 * 240 * 4 * (1 + 4 + 4);
    for (step, viewport_rect) in [rect(), egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.))].into_iter().enumerate() {
        h.port.set_render_request(ViewportRenderRequest { frame_id: 1000 + step as u64, generation: 1,
            moving: true, render_due: true, quality: Default::default() });
        let mut output = h.ctx.run_ui(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(400., 300.))),
            ..Default::default()
        }, |ui| {
            h.port.show(ui, viewport_rect, &h.state, &scene.controller, scene_camera(&camera), camera.revision, &camera,
                Some(&scene), CPlane::world_xy(), &BTreeSet::new(), 0, 0, None, false);
        });
        output.textures_delta.clear();
        let diagnostics = h.port.diagnostics().unwrap();
        assert_eq!(diagnostics.encoded_target_bytes, Some(expected), "encoded old target must remain tracked after resize");
        assert_eq!(diagnostics.tracked_target_bytes, Some(expected));
    }
    assert!(submit_window(&h.pool, &h.state));
    let all = window_target_diagnostics(&h.pool, [&h.port]).unwrap();
    assert!(all.retired_command_bytes > 0, "resized-away command targets remain in the window union");
    assert_eq!(all.unique_bytes, all.live_bytes + all.retired_command_bytes);
    let submitted = h.port.diagnostics().unwrap();
    assert_eq!(submitted.encoded_target_bytes, Some(0));
    assert_eq!(submitted.inflight_target_count, Some(3));
    assert_eq!(submitted.inflight_target_bytes, Some(expected));
    h.port.window_submitted(&h.state.queue);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(h.port.diagnostics().unwrap().tracked_target_bytes, Some(0));
    let completed = window_target_diagnostics(&h.pool, [&h.port]).unwrap();
    assert_eq!(completed.retired_command_bytes, 0);
    assert_eq!(completed.unique_bytes, completed.live_bytes);
    let other = SharedResourcePool::default();
    assert!(window_target_diagnostics(&other, [&h.port]).is_none(), "a mixed window must not report a partial total");
    let guard = h.pool.lock().unwrap();
    assert!(window_target_diagnostics(&h.pool, [&h.port]).is_none(), "diagnostics must not block on a busy pool");
    drop(guard);
}

#[test]
fn distinct_preview_and_ghost_instance_buffers_are_both_counted() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let mut scene = scene(1);
    h.selected.insert(scene.objects[0].root);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    assert_eq!(h.port.diagnostics().unwrap().unique_instance_bytes, Some(160));
    let prior = h.port.diagnostics().unwrap().encoded_frames;
    scene.preview(Transform::translation([1., 0., 0.]));
    h.preview_revision += 1;
    h.frame_with_submission(&scene, &camera, 0, rect(), false, true);
    let submitted = h.port.diagnostics().unwrap();
    assert!(submitted.encoded_frames > prior, "fixture must draw the preview");
    assert_eq!(submitted.inflight_instance_buffers, Some(2));
    assert_eq!(submitted.inflight_camera_bytes, Some(160), "scene and ghost have separate camera uniforms");
    assert_eq!(submitted.inflight_instance_bytes, Some(320));
    assert_eq!(submitted.unique_instance_bytes, Some(320), "live cache must not add duplicates");
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let completed = h.port.diagnostics().unwrap();
    assert_eq!(completed.inflight_instance_bytes, Some(0));
    assert_eq!(completed.inflight_camera_bytes, Some(0));
    assert_eq!(completed.unique_instance_bytes, Some(320), "both caches remain after submission completes");
}

#[test]
fn evicted_scene_buffer_remains_counted_until_its_submission_completes() {
    let Some(mut first) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    first.port.enable_diagnostics();
    let old = scene(1);
    let new = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    first.finish(&old, &camera, 0);
    camera.eye[0] += 0.1;
    camera.revision += 1;
    let mut second = GpuViewport::new(&first.state, first.pool.clone());
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(320., 240.))),
        ..Default::default()
    }, |ui| {
        first.port.set_render_request(ViewportRenderRequest { frame_id: 1000, generation: 1, moving: true,
            render_due: true, quality: Default::default() });
        first.port.show(ui, rect(), &first.state, &old.controller, scene_camera(&camera), camera.revision, &camera,
            Some(&old), CPlane::world_xy(), &BTreeSet::new(), 0, 0, None, false);
        second.set_render_request(ViewportRenderRequest { frame_id: 1000, generation: 1, moving: false,
            render_due: true, quality: Default::default() });
        second.show(ui, rect(), &first.state, &new.controller, scene_camera(&camera), camera.revision, &camera,
            Some(&new), CPlane::world_xy(), &BTreeSet::new(), 0, 0, None, false);
    });
    output.textures_delta.clear();
    let encoded = first.port.diagnostics().unwrap();
    assert_eq!(encoded.shared_scene_instance_snapshot_cpu_bytes, Some(0), "old cache should be evicted");
    assert_eq!(encoded.encoded_instance_bytes, Some(160));
    assert_eq!(encoded.unique_instance_bytes, Some(160), "encoded command must retain the evicted buffer");
    assert!(submit_window(&first.pool, &first.state));
    let submitted = first.port.diagnostics().unwrap();
    assert_eq!(submitted.inflight_instance_bytes, Some(160));
    assert_eq!(submitted.unique_instance_bytes, Some(160));
    first.port.window_submitted(&first.state.queue);
    second.window_submitted(&first.state.queue);
    first.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(first.port.diagnostics().unwrap().unique_instance_bytes, Some(0));
}

#[test]
fn replaced_scene_geometry_remains_counted_until_submission_completes() {
    let Some(mut first) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    let old = scene(1);
    let new = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    first.finish(&old, &camera, 0);
    let mut inspector = GpuViewport::new(&first.state, first.pool.clone());
    inspector.enable_diagnostics();
    let old_bytes = inspector.diagnostics().unwrap().unique_geometry_bytes.unwrap();
    assert!(window_known_buffer_bytes(&first.pool, [&first.port]).unwrap() >= old_bytes);
    drop(inspector);
    assert!(old_bytes >= 4 * 1024 * 1024);
    camera.eye[0] += 0.1;
    camera.revision += 1;
    let mut second = GpuViewport::new(&first.state, first.pool.clone());
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(320., 240.))),
        ..Default::default()
    }, |ui| {
        first.port.set_render_request(ViewportRenderRequest { frame_id: 1000, generation: 1, moving: true,
            render_due: true, quality: Default::default() });
        first.port.show(ui, rect(), &first.state, &old.controller, scene_camera(&camera), camera.revision, &camera,
            Some(&old), CPlane::world_xy(), &BTreeSet::new(), 0, 0, None, false);
        second.set_render_request(ViewportRenderRequest { frame_id: 1000, generation: 1, moving: false,
            render_due: true, quality: Default::default() });
        second.show(ui, rect(), &first.state, &new.controller, scene_camera(&camera), camera.revision, &camera,
            Some(&new), CPlane::world_xy(), &BTreeSet::new(), 0, 0, None, false);
    });
    output.textures_delta.clear();
    assert!(submit_window(&first.pool, &first.state));
    first.port.window_submitted(&first.state.queue);
    second.window_submitted(&first.state.queue);
    drop(first.port);
    drop(second);
    let mut observer = GpuViewport::new(&first.state, first.pool.clone());
    observer.enable_diagnostics();
    let submitted = observer.diagnostics().unwrap();
    assert_eq!(submitted.inflight_geometry_bytes, Some(old_bytes));
    assert!(submitted.inflight_camera_bytes.unwrap() > 0, "camera buffers must remain tracked without logging");
    assert!(submitted.inflight_overlay_bytes.unwrap() > 0, "overlay buffers must remain tracked without logging");
    let deadline = Instant::now() + Duration::from_secs(5);
    let accounted = loop {
        if let Some(bytes) = window_known_buffer_bytes(&first.pool, std::iter::empty()) { break bytes; }
        assert!(Instant::now() < deadline, "canceled preparation did not finish");
        std::thread::yield_now();
    };
    assert!(accounted >= old_bytes, "accounting must work even when the submitting viewport has diagnostics disabled");
    let before = submitted.unique_geometry_bytes.unwrap();
    assert!(before >= old_bytes);
    first.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let completed = observer.diagnostics().unwrap();
    assert_eq!(completed.inflight_geometry_bytes, Some(0));
    assert!(completed.unique_geometry_bytes.unwrap() <= before - old_bytes,
        "replaced geometry must be released from the diagnostic tracker after completion");
}

#[test]
fn shared_instance_buffer_is_counted_once_per_submission() {
    let Some(mut first) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    let state = first.state.clone();
    let pool = first.pool.clone();
    let mut second = Harness {
        ctx: Default::default(), port: GpuViewport::new(&state, pool.clone()), pool, state,
        frame: 0, hidden_sources: None, selected: BTreeSet::new(), preview_revision: 0,
    };
    first.port.enable_diagnostics();
    second.port.enable_diagnostics();
    let scene = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    first.finish(&scene, &camera, 0);
    second.finish(&scene, &camera, 0);
    let first_before = first.port.diagnostics().unwrap().encoded_frames;
    let second_before = second.port.diagnostics().unwrap().encoded_frames;
    camera.eye[0] += 0.1;
    camera.revision += 1;
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(320., 240.))),
        ..Default::default()
    }, |ui| {
        for port in [&mut first.port, &mut second.port] {
            port.set_render_request(ViewportRenderRequest { frame_id: 999, generation: 1, moving: true,
                render_due: true, quality: Default::default() });
            port.show(ui, rect(), &first.state, &scene.controller, scene_camera(&camera), camera.revision, &camera,
                Some(&scene), CPlane::world_xy(), &BTreeSet::new(), 0, 0, None, false);
        }
    });
    output.textures_delta.clear();
    assert!(first.port.diagnostics().unwrap().encoded_frames > first_before);
    assert!(second.port.diagnostics().unwrap().encoded_frames > second_before);
    let encoded = first.port.diagnostics().unwrap();
    assert_eq!(encoded.encoded_instance_bytes, Some(160));
    assert_eq!(encoded.tracked_instance_bytes, Some(160));
    assert_eq!(encoded.unique_instance_bytes, Some(160));
    assert_eq!(encoded.encoded_camera_bytes, Some(160), "two views retain two distinct camera uniforms");
    assert_eq!(encoded.tracked_camera_bytes, Some(160));
    let overlay_bytes = encoded.encoded_overlay_bytes.unwrap();
    assert!(overlay_bytes > 0, "the two viewport overlays must be tracked before submission");
    assert_eq!(encoded.tracked_overlay_bytes, Some(overlay_bytes));
    let overlay_window = window_overlay_diagnostics(&first.pool, [&first.port, &second.port]).unwrap();
    assert!(overlay_window.live_bytes > 0);
    assert!(overlay_window.retired_command_bytes > 0, "encoded uniforms remain charged outside live vertex buffers");
    assert!(overlay_window.unique_bytes >= overlay_bytes);
    let geometry = encoded.unique_geometry_bytes.unwrap();
    assert!(geometry >= 4 * 1024 * 1024);
    let known = window_known_buffer_bytes(&first.pool, [&first.port, &second.port]).unwrap();
    assert_eq!(known, geometry + 160 + 160 + overlay_window.unique_bytes,
        "shared scene, instance, camera and overlay handles must each be counted once");
    let target_bytes = 2 * 320u64 * 240 * 4 * (1 + 4 + 4);
    assert_eq!(encoded.encoded_target_bytes, Some(target_bytes), "two viewports own separate colour/depth targets");
    assert_eq!(encoded.tracked_target_bytes, Some(target_bytes));
    assert!(submit_window(&first.pool, &first.state));
    let inflight = first.port.diagnostics().unwrap();
    assert_eq!(inflight.inflight_instance_buffers, Some(1));
    assert_eq!(inflight.inflight_instance_bytes, Some(160));
    assert_eq!(inflight.encoded_instance_bytes, Some(0));
    assert_eq!(inflight.tracked_instance_bytes, Some(160));
    assert_eq!(inflight.unique_instance_bytes, Some(160));
    assert_eq!(inflight.encoded_camera_bytes, Some(0));
    assert_eq!(inflight.inflight_camera_bytes, Some(160));
    assert_eq!(inflight.tracked_camera_bytes, Some(160));
    assert_eq!(inflight.inflight_geometry_buffers, Some(2));
    assert_eq!(inflight.inflight_geometry_bytes, Some(geometry), "both views share the same arena buffers");
    assert_eq!(inflight.unique_geometry_bytes, Some(geometry));
    assert_eq!(inflight.encoded_overlay_bytes, Some(0));
    assert_eq!(inflight.inflight_overlay_bytes, Some(overlay_bytes));
    assert_eq!(inflight.tracked_overlay_bytes, Some(overlay_bytes));
    assert_eq!(window_overlay_diagnostics(&first.pool, [&first.port, &second.port]).unwrap().unique_bytes, overlay_window.unique_bytes);
    assert_eq!(inflight.inflight_target_count, Some(6));
    assert_eq!(inflight.inflight_target_bytes, Some(target_bytes));
    first.port.window_submitted(&first.state.queue);
    second.port.window_submitted(&first.state.queue);
    first.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(first.port.diagnostics().unwrap().inflight_instance_bytes, Some(0));
    assert_eq!(first.port.diagnostics().unwrap().tracked_instance_bytes, Some(0));
    assert_eq!(first.port.diagnostics().unwrap().tracked_camera_bytes, Some(0));
    assert_eq!(first.port.diagnostics().unwrap().unique_instance_bytes, Some(160));
    assert_eq!(first.port.diagnostics().unwrap().tracked_target_bytes, Some(0));
    assert_eq!(first.port.diagnostics().unwrap().inflight_overlay_bytes, Some(0));
    assert_eq!(first.port.diagnostics().unwrap().tracked_overlay_bytes, Some(0));
    let completed = window_overlay_diagnostics(&first.pool, [&first.port, &second.port]).unwrap();
    assert_eq!(completed.retired_command_bytes, 0);
    assert_eq!(completed.unique_bytes, completed.live_bytes);
    let unrelated = SharedResourcePool::default();
    assert!(window_overlay_diagnostics(&unrelated, [&first.port]).is_none());
    let lock = first.pool.lock().unwrap();
    assert!(window_overlay_diagnostics(&first.pool, [&first.port]).is_none());
    drop(lock);
}

#[test]
fn camera_culling_change_replaces_the_instance_snapshot() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(1);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    assert_eq!(h.port.frame_stats().unwrap().visible, 1);
    let before = h.port.diagnostics().unwrap();
    let mut away = camera.clone();
    away.target = [0., -20., 10.];
    away.revision += 1;
    h.frame_with_submission(&scene, &away, 1, rect(), true, true);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(h.port.frame_stats().unwrap().visible, 0);
    let after = h.port.diagnostics().unwrap();
    assert!(after.main_scene_instance_snapshot_bytes > before.main_scene_instance_snapshot_bytes,
        "changed visible set must replace the instance snapshot");
    h.frame_with_submission(&scene, &camera, 1, rect(), true, true);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    assert_eq!(h.port.frame_stats().unwrap().visible, 1);
    assert!(h.port.diagnostics().unwrap().main_scene_instance_snapshot_bytes > after.main_scene_instance_snapshot_bytes);
}

#[test]
fn moving_view_waits_for_a_superseded_stationary_submission() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    camera.eye[0] += 1.;
    camera.revision += 1;
    h.frame_with_submission(&scene, &camera, 1, rect(), false, false);
    let encoded = h.port.diagnostics().unwrap().encoded_frames;
    camera.eye[0] += 1.;
    camera.revision += 1;
    h.frame_with_submission(&scene, &camera, 2, rect(), true, false);
    assert_eq!(h.port.diagnostics().unwrap().encoded_frames, encoded, "a superseded stationary submission must finish before another frame is encoded");
    h.port.window_submitted(&h.state.queue);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    h.frame_with_submission(&scene, &camera, 2, rect(), true, true);
    assert_eq!(h.port.diagnostics().unwrap().encoded_frames, encoded + 1);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    h.frame_with_submission(&scene, &camera, 2, rect(), true, true);
    assert_eq!(h.port.displayed_camera(), Some(&camera));
}

#[test]
fn moving_view_publishes_only_completed_frames_and_continues_with_latest_camera() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    let initial = camera.clone();
    camera.eye[0] += 1.;
    camera.revision += 1;
    let first = camera.clone();
    h.frame_with_submission(&scene, &camera, 1, rect(), true, false);
    assert_eq!(h.port.displayed_camera(), Some(&initial), "a frame cannot publish before its queue completion");
    camera.eye[0] += 1.;
    camera.revision += 1;
    h.frame_with_submission(&scene, &camera, 1, rect(), true, false);
    assert_eq!(h.port.displayed_camera(), Some(&initial));
    h.port.window_submitted(&h.state.queue);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    h.frame_with_submission(&scene, &camera, 1, rect(), true, true);
    assert_eq!(h.port.displayed_camera(), Some(&first), "the completed image may publish while the latest camera is drawn");
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    h.frame_with_submission(&scene, &camera, 1, rect(), true, true);
    assert_eq!(h.port.displayed_camera(), Some(&camera), "the next completed image must use the latest camera");
}

#[test]
fn moving_frame_from_old_document_is_not_published() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    let old_scene = scene(1);
    let new_scene = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&old_scene, &camera, 0);
    camera.eye[0] += 1.;
    camera.revision += 1;
    h.frame_with_submission(&old_scene, &camera, 1, rect(), true, false);
    h.frame_with_submission(&new_scene, &camera, 2, rect(), true, false);
    assert!(!h.port.displays_scene(&old_scene));
    assert!(!h.port.displays_scene(&new_scene));
    h.port.window_submitted(&h.state.queue);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        h.frame_with_submission(&new_scene, &camera, 2, rect(), true, true);
        h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        assert!(!h.port.displays_scene(&old_scene), "old document frame must never be republished");
        if h.port.displays_scene(&new_scene) { break; }
        assert!(Instant::now() < deadline, "new document did not publish");
    }
}

#[test]
fn moving_frame_with_stale_selection_is_not_published() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(1);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    let original = camera.clone();
    camera.eye[0] += 1.;
    camera.revision += 1;
    h.frame_with_submission(&scene, &camera, 1, rect(), true, false);
    let stale_frame = h.port.encoded_frame;
    h.selected.insert(scene.objects[0].root);
    h.frame_with_submission(&scene, &camera, 1, rect(), true, false);
    assert_eq!(h.port.displayed_camera(), Some(&original));
    h.port.window_submitted(&h.state.queue);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let deadline = Instant::now() + Duration::from_secs(5);
    let old_published = h.port.published_frame;
    loop {
        h.frame_with_submission(&scene, &camera, 1, rect(), true, true);
        h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        if h.port.published_frame != old_published {
            assert_ne!(h.port.published_frame, stale_frame, "unselected frame must not publish after selection changes");
            assert_eq!(h.port.displayed_camera(), Some(&camera));
            break;
        }
        assert_eq!(h.port.displayed_camera(), Some(&original), "stale unselected frame must not publish");
        assert!(Instant::now() < deadline, "selected frame did not publish");
    }
}

#[test]
fn moving_frame_with_stale_preview_is_not_published() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut scene = scene(1);
    h.selected.insert(scene.objects[0].root);
    let mut camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    let original = camera.clone();
    camera.eye[0] += 1.;
    camera.revision += 1;
    h.frame_with_submission(&scene, &camera, 1, rect(), true, false);
    let stale_frame = h.port.encoded_frame;
    scene.preview(Transform::translation([2., 0., 0.]));
    h.preview_revision += 1;
    h.frame_with_submission(&scene, &camera, 1, rect(), true, false);
    assert_eq!(h.port.displayed_camera(), Some(&original));
    h.port.window_submitted(&h.state.queue);
    h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let old_published = h.port.published_frame;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        h.frame_with_submission(&scene, &camera, 1, rect(), true, true);
        h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        if h.port.published_frame != old_published {
            assert_ne!(h.port.published_frame, stale_frame, "old preview frame must not publish");
            assert_eq!(h.port.displayed_camera(), Some(&camera));
            break;
        }
        assert!(Instant::now() < deadline, "updated preview frame did not publish");
    }
}

#[test]
fn repeated_gumball_input_blocking_does_not_starve_viewport_publication() {
    use om9_app::navigation::NavigationController;
    use om9_scene::workspace::ViewId;
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    let scene = scene(1);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    let mut nav = NavigationController::default();
    let id = ViewId::Perspective;
    nav.begin_drag(id, egui::PointerButton::Secondary, egui::Pos2::ZERO, &camera, Instant::now());
    nav.cancel();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        nav.cancel();
        h.frame(&scene, &camera, nav.generation(id), rect());
        h.state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        if h.port.final_complete() { break; }
        assert!(Instant::now() < deadline, "repeated blocked input starved GPU publication");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(feature = "occt-brep")]
#[test]
fn box_undo_and_redo_invalidate_cached_viewport() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let mut fixture = TestScene::new(Document::new("undo viewport"));
    let camera = Camera::perspective([15., -20., 15.], [0., 0., 0.], [0., 0., 1.], 0.6).unwrap();
    h.finish(&fixture, &camera, 0);
    let c = &mut fixture.controller;
    assert!(h.port.displays_scene(c.scene().unwrap()));
    c.submit("Box").unwrap();
    c.accept_world_point([0., 0., 0.]).unwrap();
    c.accept_world_point([10., 20., 0.]).unwrap();
    c.accept_world_point([0., 0., 30.]).unwrap();
    assert!(!h.port.displays_scene(c.scene().unwrap()), "old image must not identify the edited scene");
    h.finish(&fixture, &camera, 0);
    assert!(h.port.displays_scene(&fixture));
    assert_eq!(h.port.frame_stats().unwrap().objects, 1);
    let created = h.port.encoded_frame;
    fixture.controller.undo().unwrap();
    assert!(!h.port.displays_scene(&fixture), "Undo invalidates the published scene receipt");
    h.finish(&fixture, &camera, 0);
    assert!(h.port.displays_scene(&fixture));
    assert_eq!(h.port.frame_stats().unwrap().objects, 0);
    assert_ne!(h.port.encoded_frame, created);
    let undone = h.port.encoded_frame;
    fixture.controller.redo().unwrap();
    assert!(!h.port.displays_scene(&fixture), "Redo must not revive an older receipt");
    h.finish(&fixture, &camera, 0);
    assert!(h.port.displays_scene(&fixture));
    assert_eq!(h.port.frame_stats().unwrap().objects, 1);
    assert_ne!(h.port.encoded_frame, undone);
}

#[test]
fn dpi_change_rebuilds_only_the_viewport_target() {
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    h.port.enable_diagnostics();
    let scene = scene(1);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    let before = h.port.diagnostics().unwrap();
    h.ctx.set_pixels_per_point(1.5);
    h.finish(&scene, &camera, 0);
    let after = h.port.diagnostics().unwrap();
    assert!(after.target_rebuilds > before.target_rebuilds);
    assert!(after.encoded_frames > before.encoded_frames);
    assert_eq!(after.scene_preparations, before.scene_preparations);
    assert_eq!(after.geometry_upload_bytes, before.geometry_upload_bytes);
}

#[cfg(feature = "occt-brep")]
#[test]
fn box_commit_publication_timing_smoke() {
    let Some(mut h)=Harness::new() else {eprintln!("SKIP: no GPU adapter");return};
    let mut fixture=TestScene::new(Document::new("box publication timing"));
    let camera=Camera::perspective([15.,-20.,15.],[0.,0.,0.],[0.,0.,1.],0.6).unwrap();
    h.finish(&fixture,&camera,0);
    let c=&mut fixture.controller;
    c.submit("Box").unwrap();c.accept_world_point([0.,0.,0.]).unwrap();c.accept_world_point([10.,20.,0.]).unwrap();
    let start=Instant::now();c.accept_world_point([0.,0.,30.]).unwrap();let commit=start.elapsed();
    let submitted=Instant::now();h.finish(&fixture,&camera,0);let publication=submitted.elapsed();
    let stats=h.port.frame_stats().unwrap();assert_eq!(stats.objects,1);assert_eq!(stats.visible,1);
    println!("box commit_ms={:.2}, offscreen_gpu_ready_ms={:.2}",commit.as_secs_f64()*1000.,publication.as_secs_f64()*1000.);
}

#[cfg(feature = "occt-brep")]
#[test]
fn spherical_brep_selection_rebuilds_viewport_without_mutating_document() {
    use om9_kernel::creation::{primitive, Primitive, DEFAULT_DEFLECTION_MM};
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("sphere selection display");
    let sphere = primitive(Primitive::Sphere {radius_mm: 5.}, DEFAULT_DEFLECTION_MM, &Limits::default()).unwrap();
    let object = Entity::new("sphere", Geometry::Brep(sphere.clone()), None);
    let mut other = Entity::new("other sphere", Geometry::Brep(sphere), None);
    other.placement = Transform::translation([12., 0., 0.]);
    let id = object.id; document.entities.insert(id, object); document.entities.insert(other.id, other);
    let scene = TestScene::new(document.clone());
    let camera = Camera::perspective([16., -28., 18.], [6., 0., 0.], [0., 0., 1.], 0.8).unwrap();
    h.finish(&scene, &camera, 0); let before = h.port.encoded_frame;
    h.selected.insert(id); h.finish(&scene, &camera, 0);
    assert_ne!(h.port.encoded_frame, before);
    assert_eq!(h.port.frame_stats().unwrap().objects, 2);
    h.selected.clear(); h.finish(&scene, &camera, 0);
    assert_eq!(&*scene.document, &document);
}

#[cfg(feature = "occt-brep")]
#[test]
fn rectangular_brep_selection_rebuilds_viewport_without_mutating_document() {
    use om9_kernel::creation::{primitive, Primitive, DEFAULT_DEFLECTION_MM};
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("box selection display");
    let box_brep = primitive(Primitive::Box {x_mm: 4., y_mm: 6., z_mm: 8.}, DEFAULT_DEFLECTION_MM, &Limits::default()).unwrap();
    let object = Entity::new("box", Geometry::Brep(box_brep), None);
    let id = object.id; document.entities.insert(id, object);
    let scene = TestScene::new(document.clone());
    let camera = Camera::perspective([14., -16., 14.], [2., 3., 4.], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0); let before = h.port.encoded_frame;
    h.selected.insert(id); h.finish(&scene, &camera, 0);
    assert_ne!(h.port.encoded_frame, before);
    h.selected.clear(); h.finish(&scene, &camera, 0);
    assert_eq!(&*scene.document, &document);
}

#[test]
fn mixed_object_selection_preserves_model_colours() {
    use om9_core::geometry::Curve;
    let Some(mut h) = Harness::new() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document=Document::new("mixed object colours");
    let geometries=[
        Geometry::Point([0.,0.,0.]),
        Geometry::Curve(Curve::Line { start:[-2.,0.,0.],end:[2.,0.,0.],domain:[0.,1.] }),
        Geometry::Mesh(Mesh { vertices:vec![[-1.,-1.,1.],[1.,-1.,1.],[0.,1.,1.]],faces:vec![vec![0,1,2]],normals:vec![],colors:vec![],corner_uvs:Vec::new() }),
    ];
    for geometry in geometries {
        let mut object=Entity::new("coloured",geometry,None);object.attributes.color=Some([180,50,20,255]);
        document.entities.insert(object.id,object);
    }
    let scene=TestScene::new(document.clone());
    let camera=Camera::perspective([10.,-12.,10.],[0.;3],[0.,0.,1.],0.6).unwrap();
    h.finish(&scene,&camera,0);h.selected.extend(document.entities.keys().copied());h.finish(&scene,&camera,0);
    h.selected.clear();h.finish(&scene,&camera,0);
    assert_eq!(&*scene.document,&document);
    assert!(scene.document.entities.values().all(|e|e.attributes.color==Some([180,50,20,255])));
}

#[test]
fn preview_visibility_rebuilds_gpu_resources_and_restores_source_objects() {
    let Some(mut h) = Harness::new() else {
        eprintln!("SKIP: no GPU adapter");
        return;
    };
    let scene = scene(3);
    let ids = [scene.objects[0].root, scene.objects[1].root];
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    assert_eq!(h.port.frame_stats().unwrap().objects, 3);
    h.hidden_sources = Some(ids);
    h.finish(&scene, &camera, 0);
    assert_eq!(h.port.frame_stats().unwrap().objects, 1);
    assert_eq!(scene.objects.len(), 3);
    assert!(
        ids.iter()
            .all(|id| !scene.document.entities[id].attributes.hidden)
    );
    h.hidden_sources = None;
    h.finish(&scene, &camera, 0);
    assert_eq!(h.port.frame_stats().unwrap().objects, 3);
    h.hidden_sources = Some(ids);
    h.frame(&scene, &camera, 0, rect());
    h.hidden_sources = None;
    h.finish(&scene, &camera, 0);
    assert_eq!(
        h.port.frame_stats().unwrap().objects,
        3,
        "an obsolete masked frame must not publish after cancellation"
    );
}
fn rect() -> egui::Rect {
    egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(320., 240.))
}
#[test]
fn publication_rejects_old_generation_and_same_revision_pose_and_recovers_from_oversize() {
    let Some(mut h) = Harness::new() else {
        eprintln!("SKIP: no GPU adapter");
        return;
    };
    let scene = scene(1);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&scene, &camera, 0);
    let encoded = h.port.encoded_frame;
    let same_revision = Camera::perspective([10., 0., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    assert_eq!(camera.revision, same_revision.revision);
    h.finish(&scene, &same_revision, 1);
    assert_ne!(
        h.port.encoded_frame, encoded,
        "a same-revision restore must encode the new pose, not just relabel a cached texture"
    );
    let mut obsolete = camera.clone();
    obsolete.revision += 10;
    h.frame(&scene, &obsolete, 2, rect());
    let mut current = same_revision.clone();
    current.revision += 20;
    h.frame(&scene, &current, 3, rect());
    assert_ne!(
        h.port.displayed_camera(),
        Some(&obsolete),
        "old generation cannot publish"
    );
    h.finish(&scene, &current, 3);
    assert_eq!(h.port.displayed_camera(), Some(&current));
    let before = h.port.encoded_frame;
    let oversized = egui::Rect::from_min_size(
        egui::Pos2::ZERO,
        egui::vec2(
            h.state.device.limits().max_texture_dimension_2d as f32 + 1.,
            240.,
        ),
    );
    h.frame(&scene, &current, 4, oversized);
    assert!(!h.port.final_complete());
    assert!(h.port.preparation_error().is_some());
    assert_eq!(h.port.encoded_frame, before);
    h.finish(&scene, &current, 5);
    assert!(h.port.preparation_error().is_none());
    assert_eq!(scene.document.entities.len(), 1);
}
#[test]
fn document_identity_invalidates_equal_epoch_camera_selection_and_size() {
    let Some(mut h) = Harness::new() else {
        return;
    };
    let first = scene(1);
    let second = scene(2);
    assert_eq!(first.epoch, second.epoch);
    assert_ne!(first.document.id, second.document.id);
    let camera = Camera::perspective([0., -10., 5.], [0.; 3], [0., 0., 1.], 0.6).unwrap();
    h.finish(&first, &camera, 0);
    assert!(h.port.displays_scene(&first));
    assert!(!h.port.displays_scene(&second));
    let before = h.port.encoded_frame;
    h.finish(&second, &camera, 0);
    assert!(h.port.displays_scene(&second));
    assert!(!h.port.displays_scene(&first));
    assert_ne!(h.port.encoded_frame, before);
    assert_eq!(h.port.frame_stats().unwrap().objects, 2);
}
