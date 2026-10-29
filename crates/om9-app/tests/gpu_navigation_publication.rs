// SPDX-License-Identifier: LGPL-2.1-or-later
//! Exercise the actual egui/wgpu viewport bridge and its publication fence, not a mocked key table.
use eframe::{egui, egui_wgpu, wgpu};
use om9_app::gpu_viewport::{
    GpuViewport, SharedResourcePool, ViewportRenderRequest, scene_camera, submit_window,
};
use om9_core::{
    budget::{Limits, Usage},
    geometry::{Geometry, Mesh, Transform},
    model::{Document, Entity},
    transaction::{TransactionCoordinator, World},
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
fn scene(count: usize) -> Scene {
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
    let id = document.id;
    let coordinator = TransactionCoordinator::new(
        World::new(vec![document], &Limits::default()).unwrap(),
        Limits::default(),
    )
    .unwrap();
    Scene::capture_in_world(&coordinator, id, Usage::default()).unwrap()
}
struct Harness {
    ctx: egui::Context,
    port: GpuViewport,
    pool: SharedResourcePool,
    state: egui_wgpu::RenderState,
    frame: u64,
    hidden_sources: Option<[om9_core::identity::ObjectId; 2]>,
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
        })
    }
    fn frame(&mut self, scene: &Scene, camera: &Camera, generation: u64, rect: egui::Rect) {
        self.frame += 1;
        self.port.set_render_request(ViewportRenderRequest {
            frame_id: self.frame,
            generation,
            moving: false,
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
                scene_camera(camera),
                camera.revision,
                camera,
                Some(scene),
                CPlane::world_xy(),
                &BTreeSet::new(),
                0,
                0,
                self.hidden_sources,
                false,
            );
        });
        output.textures_delta.clear();
        submit_window(&self.pool, &self.state);
        self.port.window_submitted(&self.state.queue);
    }
    fn finish(&mut self, scene: &Scene, camera: &Camera, generation: u64) {
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
    let before = h.port.encoded_frame;
    h.finish(&second, &camera, 0);
    assert_ne!(h.port.encoded_frame, before);
    assert_eq!(h.port.frame_stats().unwrap().objects, 2);
}
