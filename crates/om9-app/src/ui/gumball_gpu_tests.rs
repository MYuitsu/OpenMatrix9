// SPDX-License-Identifier: LGPL-2.1-or-later
//! Perspective input and the real offscreen GPU viewport in one MatrixApp.
use super::*;
use eframe::{egui_wgpu, wgpu};
use om9_core::{geometry::{Curve, Geometry, Transform}, model::{Document, Entity}};
use std::{sync::Arc, time::{Duration, Instant}};

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

fn gpu() -> Option<egui_wgpu::RenderState> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(), flags: Default::default(),
        memory_budget_thresholds: Default::default(), backend_options: Default::default(),
        display: Default::default(),
    });
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: false,
        compatible_surface: None, apply_limit_buckets: false,
    })).ok()?;
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(), required_limits: adapter.limits(),
        ..Default::default()
    })).ok()?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let renderer = egui_wgpu::Renderer::new(&device, format, Default::default());
    Some(egui_wgpu::RenderState {
        adapter: adapter.clone(), available_adapters: vec![adapter], instance, device, queue,
        target_format: format, renderer: Arc::new(egui::mutex::RwLock::new(renderer)),
        surface_config: egui_wgpu::SurfaceConfig::LOW_LATENCY,
    })
}

fn frame(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<egui::Event>) {
    frame_with_size(app, ctx, events, egui::vec2(1280., 720.));
}

fn frame_with_size(app: &mut MatrixApp, ctx: &egui::Context, events: Vec<egui::Event>, size: egui::Vec2) {
    // The GPU path uses wall-clock frame pacing rather than synthetic egui time.
    std::thread::sleep(Duration::from_millis(40));
    let mut output = ctx.run_ui(egui::RawInput {
        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, size)),
        events, ..Default::default()
    }, |ui| app.show(ui));
    output.textures_delta.clear();
    app.gpu_state.as_ref().unwrap().device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
}

fn settle(app: &mut MatrixApp, ctx: &egui::Context) {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        frame(app, ctx, vec![]);
        if app.gpu_viewports.get(&ViewId::Perspective).is_some_and(|p| p.final_complete() && p.displayed_camera().is_some()) { return; }
        assert!(Instant::now() < deadline, "Perspective GPU image never completed");
        std::thread::sleep(Duration::from_millis(2));
    }
}

fn viewport_pixel(app: &MatrixApp, view: ViewId, point: [f64; 3]) -> [u8; 4] {
    let port = &app.gpu_viewports[&view];
    let texture = port.published_texture().expect("published texture");
    let rect = app.view_rects[&view];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let screen = port.displayed_camera().unwrap().project(point, viewport).unwrap().unwrap().screen;
    let x = ((screen[0] - viewport.origin[0]) * texture.width() as f64 / viewport.size[0]).floor() as u32;
    let y = ((screen[1] - viewport.origin[1]) * texture.height() as f64 / viewport.size[1]).floor() as u32;
    texture_pixel(app, texture, x, y)
}

fn texture_pixel(app: &MatrixApp, texture: &wgpu::Texture, x: u32, y: u32) -> [u8; 4] {
    assert!(x < texture.width() && y < texture.height());
    let state = app.gpu_state.as_ref().unwrap();
    let buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("mode test pixel"), size: 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false,
    });
    let mut encoder = state.device.create_command_encoder(&Default::default());
    let mut source = texture.as_image_copy();
    source.origin = wgpu::Origin3d { x, y, z: 0 };
    encoder.copy_texture_to_buffer(source, wgpu::TexelCopyBufferInfo {
        buffer: &buffer,
        layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(256), rows_per_image: Some(1) },
    }, wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 });
    state.queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| { let _ = sender.send(result); });
    state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let bytes = buffer.slice(..).get_mapped_range().unwrap();
    let result = [bytes[0], bytes[1], bytes[2], bytes[3]];
    drop(bytes);
    buffer.unmap();
    result
}

fn texture_rgba(app: &MatrixApp, texture: &wgpu::Texture) -> Vec<u8> {
    let state = app.gpu_state.as_ref().unwrap();
    let row = texture.width() * 4;
    let padded = row.div_ceil(256) * 256;
    let buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("viewport image comparison"), size: u64::from(padded) * u64::from(texture.height()),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false,
    });
    let mut encoder = state.device.create_command_encoder(&Default::default());
    encoder.copy_texture_to_buffer(texture.as_image_copy(), wgpu::TexelCopyBufferInfo {
        buffer: &buffer, layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(texture.height()) },
    }, wgpu::Extent3d { width: texture.width(), height: texture.height(), depth_or_array_layers: 1 });
    state.queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |result| { let _ = tx.send(result); });
    state.device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    let mapped = buffer.slice(..).get_mapped_range().unwrap();
    let mut out = Vec::with_capacity(row as usize * texture.height() as usize);
    for line in mapped.chunks_exact(padded as usize) { out.extend_from_slice(&line[..row as usize]); }
    drop(mapped);
    buffer.unmap();
    out
}

#[test]
fn viewport_snapshot_recovers_after_shared_ram_pressure() {
    use om9_core::budget::memory::MemoryClass;
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("viewport snapshot pressure");
    let point = Entity::new("point", Geometry::Point([0.; 3]), None);
    let id = point.id;
    document.entities.insert(point.id, point);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.select_objects(&[id]).unwrap();
    let ctx = egui::Context::default();
    let held = app.controller.scene_cache_usage().unwrap();
    let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, held,
    ).unwrap();
    frame(&mut app, &ctx, vec![]);
    assert!(app.gpu_viewports[&ViewId::Perspective].preparation_error().is_some());
    let port = app.gpu_viewports.get_mut(&ViewId::Perspective).unwrap();
    assert!(port.snapshot_retry_deadline().is_some());
    port.delay_snapshot_retry_for_test(Duration::from_secs(60));
    let deadline = port.snapshot_retry_deadline().unwrap();
    assert!(deadline > Instant::now());
    let before_preview = app.controller.preview_revision();
    app.controller.preview_gumball(Transform::translation([2., 0., 0.])).unwrap();
    assert_ne!(app.controller.preview_revision(), before_preview);
    for _ in 0..2 { frame(&mut app, &ctx, vec![]); }
    assert_eq!(app.gpu_viewports[&ViewId::Perspective].snapshot_retry_deadline(), Some(deadline),
        "preview and frames before the deadline must not retry the same allocation");
    drop(pressure);
    app.gpu_viewports.get_mut(&ViewId::Perspective).unwrap().delay_snapshot_retry_for_test(Duration::ZERO);
    settle(&mut app, &ctx);
    assert!(app.gpu_viewports[&ViewId::Perspective].preparation_error().is_none());
}

#[test]
fn viewport_snapshot_new_scene_bypasses_old_retry_deadline() {
    use om9_core::budget::memory::MemoryClass;
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("viewport stale retry");
    let point = Entity::new("point", Geometry::Point([0.; 3]), None);
    document.entities.insert(point.id, point);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    let ctx = egui::Context::default();
    let held = app.controller.scene_cache_usage().unwrap();
    let pressure = app.controller.coordinator().reserve_remaining_external_memory_with_held(
        MemoryClass::Kernel, None, held,
    ).unwrap();
    frame(&mut app, &ctx, vec![]);
    let port = app.gpu_viewports.get_mut(&ViewId::Perspective).unwrap();
    assert!(port.snapshot_retry_deadline().is_some());
    port.delay_snapshot_retry_for_test(Duration::from_secs(60));
    drop(pressure);
    app.controller.submit("Point").unwrap();
    app.controller.submit("1,2,3").unwrap();
    settle(&mut app, &ctx);
    assert_eq!(app.controller.document().entities.len(), 2);
    assert!(app.gpu_viewports[&ViewId::Perspective].preparation_error().is_none());
}

#[test]
fn shared_world_lines_match_legacy_perspective_pixels() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mesh = om9_core::geometry::Mesh {
        vertices: vec![[-5., -5., 0.], [5., -5., 0.], [5., 5., 0.], [-5., 5., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("world-line pixel comparison");
    let mesh = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let mesh_id = mesh.id;
    let line = Entity::new("line", Geometry::Curve(Curve::Line {
        start: [-4., 0., 2.], end: [4., 1., 2.], domain: [0., 1.],
    }), None);
    let line_id = line.id;
    let point = Entity::new("point", Geometry::Point([0., 0., 3.]), None);
    let point_id = point.id;
    for entity in [mesh, line, point] { document.entities.insert(entity.id, entity); }
    let mut legacy = MatrixApp::new(document.clone()).unwrap();
    let mut shared = MatrixApp::new(document).unwrap();
    legacy.gpu_state = Some(state.clone());
    shared.gpu_state = Some(state.clone());
    for id in legacy.controller.workspace.views.keys().copied().collect::<Vec<_>>() {
        let mut port = crate::gpu_viewport::GpuViewport::new(&state, legacy.shared_scene.clone());
        port.use_legacy_world_lines_for_test();
        legacy.gpu_viewports.insert(id, port);
    }
    let legacy_ctx = egui::Context::default();
    let shared_ctx = egui::Context::default();
    for mode in [om9_scene::workspace::DisplayMode::Wireframe, om9_scene::workspace::DisplayMode::Shaded] {
        for app in [&mut legacy, &mut shared] {
            app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().mode = mode;
        }
        let mut unselected_pixels = None;
        for selected in [false, true] {
            let ids = [mesh_id];
            for app in [&mut legacy, &mut shared] {
                app.controller.select_objects(if selected { &ids } else { &[] }).unwrap();
            }
            settle(&mut legacy, &legacy_ctx);
            settle(&mut shared, &shared_ctx);
            for app in [&legacy, &shared] {
                let port = &app.gpu_viewports[&ViewId::Perspective];
                let view = &app.controller.workspace.views[&ViewId::Perspective];
                assert!(port.displays_content(app.controller.scene().unwrap(), app.controller.selection_state().revision,
                    app.controller.preview_revision(), view.cplane.revision,
                    app.controller.solid_boolean_two_hidden_sources(), mode), "comparison must use the current selection and display mode");
                assert_eq!(port.displayed_camera(), Some(&view.camera));
            }
            let old = legacy.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
            let new = shared.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
            assert_eq!((old.width(), old.height()), (new.width(), new.height()));
            let old_pixels = texture_rgba(&legacy, old);
            let new_pixels = texture_rgba(&shared, new);
            let different = old_pixels.chunks_exact(4).zip(new_pixels.chunks_exact(4)).filter(|(a, b)| a != b).count();
            assert_eq!(different, 0, "Perspective pixels differ for {mode:?}, selected={selected}");
            if selected {
                assert!(unselected_pixels.as_ref().is_some_and(|previous| *previous != old_pixels), "selection must visibly change the test image");
            } else {
                unselected_pixels = Some(old_pixels);
            }
        }
    }
    for id in [line_id, point_id] {
        for app in [&mut legacy, &mut shared] { app.controller.select_objects(&[id]).unwrap(); }
        settle(&mut legacy, &legacy_ctx);
        settle(&mut shared, &shared_ctx);
        for app in [&legacy, &shared] {
            let view = &app.controller.workspace.views[&ViewId::Perspective];
            assert!(app.gpu_viewports[&ViewId::Perspective].displays_content(
                app.controller.scene().unwrap(), app.controller.selection_state().revision,
                app.controller.preview_revision(), view.cplane.revision,
                app.controller.solid_boolean_two_hidden_sources(), view.mode));
        }
        let old = legacy.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
        let new = shared.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
        assert!(texture_rgba(&legacy, old) == texture_rgba(&shared, new), "Perspective pixels differ for selected line/point");
    }
    for app in [&mut legacy, &mut shared] { app.controller.select_objects(&[mesh_id]).unwrap(); }
    settle(&mut legacy, &legacy_ctx);
    settle(&mut shared, &shared_ctx);
    let before = texture_rgba(&legacy, legacy.gpu_viewports[&ViewId::Perspective].published_texture().unwrap());
    for app in [&mut legacy, &mut shared] {
        app.controller.preview_gumball(Transform::translation([2., 0., 0.])).unwrap();
    }
    settle(&mut legacy, &legacy_ctx);
    settle(&mut shared, &shared_ctx);
    for app in [&legacy, &shared] {
        let view = &app.controller.workspace.views[&ViewId::Perspective];
        assert!(app.gpu_viewports[&ViewId::Perspective].displays_content(
            app.controller.scene().unwrap(), app.controller.selection_state().revision,
            app.controller.preview_revision(), view.cplane.revision,
            app.controller.solid_boolean_two_hidden_sources(), view.mode));
    }
    let old = texture_rgba(&legacy, legacy.gpu_viewports[&ViewId::Perspective].published_texture().unwrap());
    let new = texture_rgba(&shared, shared.gpu_viewports[&ViewId::Perspective].published_texture().unwrap());
    assert!(old != before, "preview must visibly move the selected mesh");
    assert!(old == new, "Perspective pixels differ during Gumball preview");
    for app in [&mut legacy, &mut shared] { app.controller.clear_gumball_preview(); }
    for action in 0..3 {
        for app in [&mut legacy, &mut shared] {
            match action {
                0 => {
                    let stamp = app.controller.document().stamp(line_id).unwrap();
                    app.controller.set_objects_hidden(vec![stamp], true).unwrap();
                }
                1 => app.controller.undo().unwrap(),
                _ => app.controller.redo().unwrap(),
            }
            assert_eq!(app.controller.document().permissions(line_id, true).unwrap().visible, action == 1);
        }
        settle(&mut legacy, &legacy_ctx);
        settle(&mut shared, &shared_ctx);
        for app in [&legacy, &shared] {
            let view = &app.controller.workspace.views[&ViewId::Perspective];
            assert!(app.gpu_viewports[&ViewId::Perspective].displays_content(
                app.controller.scene().unwrap(), app.controller.selection_state().revision,
                app.controller.preview_revision(), view.cplane.revision,
                app.controller.solid_boolean_two_hidden_sources(), view.mode));
        }
        let old = legacy.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
        let new = shared.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
        let old_pixels = texture_rgba(&legacy, old);
        let new_pixels = texture_rgba(&shared, new);
        assert!(old_pixels == new_pixels, "Perspective pixels differ after visibility/Undo/Redo step {action}");
    }
    for app in [&mut legacy, &mut shared] {
        app.controller.undo().unwrap();
        app.controller.select_objects(&[]).unwrap();
    }
    for action in 0..3 {
        for app in [&mut legacy, &mut shared] {
            let camera = &mut app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera;
            match action {
                0 => camera.orbit(0.2, 0.1).unwrap(),
                1 => camera.pan([0.2, -0.1]).unwrap(),
                _ => camera.zoom(1.1).unwrap(),
            }
        }
        settle(&mut legacy, &legacy_ctx);
        settle(&mut shared, &shared_ctx);
        for app in [&legacy, &shared] {
            let camera = &app.controller.workspace.views[&ViewId::Perspective].camera;
            assert_eq!(app.gpu_viewports[&ViewId::Perspective].displayed_camera(), Some(camera));
        }
        let old = legacy.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
        let new = shared.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
        assert!(texture_rgba(&legacy, old) == texture_rgba(&shared, new), "Perspective pixels differ after camera step {action}");
    }
}

#[test]
fn side_view_gpu_grid_draws_blue_y_horizontally_and_yellow_z_vertically() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut app = MatrixApp::new(Document::new("side grid colours")).unwrap();
    app.gpu_state = Some(state);
    let ctx = egui::Context::default();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        frame(&mut app, &ctx, vec![]);
        if app.gpu_viewports.get(&ViewId::SideView).is_some_and(|p| p.final_complete() && p.published_texture().is_some()) { break; }
        assert!(Instant::now() < deadline, "Side View GPU image never completed");
    }
    let port = &app.gpu_viewports[&ViewId::SideView];
    let texture = port.published_texture().unwrap();
    let rect = app.view_rects[&ViewId::SideView];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = port.displayed_camera().unwrap();
    let sample = |point| {
        let screen = camera.project(point, viewport).unwrap().unwrap().screen;
        let x = ((screen[0] - viewport.origin[0]) * texture.width() as f64 / viewport.size[0]).floor() as u32;
        let y = ((screen[1] - viewport.origin[1]) * texture.height() as f64 / viewport.size[1]).floor() as u32;
        let mut pixels = Vec::new();
        for dy in -1_i32..=1 { for dx in -1_i32..=1 {
            let (px, py) = (x as i32 + dx, y as i32 + dy);
            if px >= 0 && py >= 0 && (px as u32) < texture.width() && (py as u32) < texture.height() {
                pixels.push(texture_pixel(&app, texture, px as u32, py as u32));
            }
        }}
        pixels
    };
    let blue = sample([0., 5.37, 0.]);
    let yellow = sample([0., 0., 5.37]);
    assert!(blue.iter().any(|p| p[1] > 80 && p[2] > 80 && p[0] < 60), "world Y grid axis must be blue: {blue:?}");
    assert!(yellow.iter().any(|p| p[0] > 80 && p[1] > 60 && p[2] < 60), "world Z grid axis must be yellow: {yellow:?}");
}

#[test]
fn resize_preserves_the_last_published_texture_until_replacement() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mesh = om9_core::geometry::Mesh {
        vertices: vec![[-8., -8., 0.], [8., -8., 0.], [8., 8., 0.], [-8., 8., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("published image resize");
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    document.entities.insert(entity.id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.workspace.active = ViewId::Perspective;
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera =
        Camera::orthographic([0., 0., 50.], [0., 0., 0.], [0., 1., 0.], 30.).unwrap();
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let old = app.gpu_viewports[&ViewId::Perspective].published_texture().unwrap().clone();
    let x = old.width() / 2;
    let y = old.height() / 2;
    let old_pixel = texture_pixel(&app, &old, x, y);
    let old_frame = app.gpu_viewports[&ViewId::Perspective].encoded_frame;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        frame_with_size(&mut app, &ctx, vec![], egui::vec2(1500., 900.));
        let port = &app.gpu_viewports[&ViewId::Perspective];
        if port.encoded_frame != old_frame {
            assert!(port.published_texture().unwrap() == &old, "resize published incomplete texture");
            assert_eq!(texture_pixel(&app, port.published_texture().unwrap(), x, y), old_pixel);
            break;
        }
        assert!(Instant::now() < deadline, "resized frame not encoded");
    }
    let next_camera = {
        let camera = &mut app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera;
        camera.pan([1., 0.]).unwrap();
        camera.clone()
    };
    frame_with_size(&mut app, &ctx, vec![], egui::vec2(1500., 900.));
    let port = &app.gpu_viewports[&ViewId::Perspective];
    assert!(port.published_texture().unwrap() == &old, "obsolete resize frame was published after camera changed");
    assert_eq!(texture_pixel(&app, port.published_texture().unwrap(), x, y), old_pixel);
    loop {
        frame_with_size(&mut app, &ctx, vec![], egui::vec2(1500., 900.));
        let port = &app.gpu_viewports[&ViewId::Perspective];
        if port.final_complete() && port.published_texture().unwrap() != &old {
            assert_eq!(port.displayed_camera(), Some(&next_camera), "only the latest camera may be published");
            break;
        }
        assert!(Instant::now() < deadline, "resized image not published");
    }
}

#[test]
fn selected_mesh_and_ghost_buffers_are_shared_across_four_viewports() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let n = 64u32;
    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    for y in 0..=n { for x in 0..=n { vertices.push([f64::from(x) - 32., f64::from(y) - 32., 0.]); } }
    for y in 0..n { for x in 0..n {
        let a = y * (n + 1) + x;
        let b = a + 1;
        let c = a + n + 1;
        let d = c + 1;
        faces.push(vec![a, b, d]);
        faces.push(vec![a, d, c]);
    } }
    let mut document = Document::new("selected mesh memory");
    let mesh = Entity::new("grid mesh", Geometry::Mesh(om9_core::geometry::Mesh {
        vertices, faces, normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let id = mesh.id;
    document.entities.insert(id, mesh);
    let mut legacy = MatrixApp::new(document.clone()).unwrap();
    let mut shared = MatrixApp::new(document).unwrap();
    legacy.gpu_state = Some(state.clone());
    shared.gpu_state = Some(state.clone());
    for app in [&mut legacy, &mut shared] { app.controller.select_objects(&[id]).unwrap(); }
    for view in legacy.controller.workspace.views.keys().copied().collect::<Vec<_>>() {
        let mut port = crate::gpu_viewport::GpuViewport::new(&state, legacy.shared_scene.clone());
        port.use_legacy_world_lines_for_test();
        legacy.gpu_viewports.insert(view, port);
    }
    let legacy_ctx = egui::Context::default();
    let shared_ctx = egui::Context::default();
    for (app, ctx) in [(&mut legacy, &legacy_ctx), (&mut shared, &shared_ctx)] {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            frame(app, ctx, vec![]);
            if app.gpu_viewports.len() == 4 && app.gpu_viewports.values().all(|port| port.final_complete()) { break; }
            assert!(Instant::now() < deadline, "selected mesh did not settle in four viewports");
        }
    }
    let before = crate::gpu_viewport::window_overlay_diagnostics(&legacy.shared_scene, legacy.gpu_viewports.values()).unwrap().unique_bytes;
    let after = crate::gpu_viewport::window_overlay_diagnostics(&shared.shared_scene, shared.gpu_viewports.values()).unwrap().unique_bytes;
    let infos: Vec<_> = shared.gpu_viewports.values().map(|port| port.shared_world_test_capacities().unwrap()).collect();
    assert!(infos.iter().all(|entry| *entry == infos[0]));
    let selected_bytes = infos[0].1[1] + infos[0].1[2];
    assert!(selected_bytes > 0 && before > after + 2 * selected_bytes,
        "selected and ghost geometry should not be duplicated per viewport: {before} -> {after}, shared={selected_bytes}");
    eprintln!("selected mesh logical overlay bytes: {before} -> {after}; shared selected+ghost={selected_bytes}");
    let old = legacy.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
    let new = shared.gpu_viewports[&ViewId::Perspective].published_texture().unwrap();
    assert!(texture_rgba(&legacy, old) == texture_rgba(&shared, new), "selected Perspective image changed");
}

#[test]
fn unselected_world_edges_are_shared_and_selection_invalidates_them() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mesh = om9_core::geometry::Mesh {
        vertices: vec![[-8., -8., 0.], [8., -8., 0.], [8., 8., 0.], [-8., 8., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("shared viewport edges");
    let first = Entity::new("first", Geometry::Mesh(mesh.clone()), None);
    let selected = first.id;
    let second = Entity::new("second", Geometry::Mesh(mesh), None);
    document.entities.insert(first.id, first);
    document.entities.insert(second.id, second);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    let ctx = egui::Context::default();
    let settle_all = |app: &mut MatrixApp| {
        let deadline = Instant::now() + Duration::from_secs(15);
        loop {
            frame(app, &ctx, vec![]);
            if app.gpu_viewports.len() == 4 && app.gpu_viewports.values().all(|port| port.final_complete()) { break; }
            assert!(Instant::now() < deadline, "four viewport edges did not settle");
        }
    };
    settle_all(&mut app);
    let before: Vec<_> = app.gpu_viewports.values().map(|port| port.shared_world_test_info().unwrap()).collect();
    // Keep the old allocation's weak header alive so its address cannot be
    // recycled for the replacement after the GPU owners release it.
    let _old_owner = app.gpu_viewports.values().next().unwrap().shared_world_test_weak_owner().unwrap();
    assert!(before[0].1 > 0);
    assert!(before.iter().all(|entry| *entry == before[0]), "four views must share the same world-line owner");
    for port in app.gpu_viewports.values_mut() { port.enable_diagnostics(); }
    let per_view: u64 = app.gpu_viewports.values().map(|port| port.diagnostics().unwrap().overlay_buffer_bytes).sum();
    let union = crate::gpu_viewport::window_overlay_diagnostics(&app.shared_scene, app.gpu_viewports.values()).unwrap();
    assert!(union.unique_bytes <= per_view - 3 * before[0].1, "shared edge buffers must be counted once");

    app.controller.select_objects(&[selected]).unwrap();
    settle_all(&mut app);
    let after: Vec<_> = app.gpu_viewports.values().map(|port| port.shared_world_test_info().unwrap()).collect();
    assert!(after.iter().all(|entry| *entry == after[0]));
    assert_ne!(after[0].0, before[0].0, "selection must invalidate shared geometry");
    assert!(after[0].1 < before[0].1, "selected geometry must leave the shared unselected batch");
    let selected_batches: Vec<_> = app.gpu_viewports.values().map(|port| port.shared_world_test_capacities().unwrap()).collect();
    assert!(selected_batches.iter().all(|entry| *entry == selected_batches[0]));
    assert!(selected_batches[0].1[1] > 0 && selected_batches[0].1[2] > 0,
        "selected and ghost edge buffers must be present and shared");
    let shared_bytes: u64 = selected_batches[0].1.iter().sum();
    let per_view: u64 = app.gpu_viewports.values().map(|port| port.diagnostics().unwrap().overlay_buffer_bytes).sum();
    let union = crate::gpu_viewport::window_overlay_diagnostics(&app.shared_scene, app.gpu_viewports.values()).unwrap();
    assert!(union.unique_bytes <= per_view - 3 * shared_bytes, "selected and ghost buffers must be counted once");
}

#[test]
fn window_target_budget_counts_all_panes_during_resize() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut app = MatrixApp::new(Document::new("window target budget")).unwrap();
    app.gpu_state = Some(state);
    let ctx = egui::Context::default();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        frame(&mut app, &ctx, vec![]);
        if app.gpu_viewports.len() == 4 && app.gpu_viewports.values().all(|port| port.final_complete()) { break; }
        assert!(Instant::now() < deadline, "four viewports did not settle");
    }
    let inventory = crate::gpu_viewport::window_target_diagnostics(&app.shared_scene, app.gpu_viewports.values()).unwrap();
    let old = app.gpu_viewports[&ViewId::Perspective].published_texture().unwrap().clone();
    let one_view_upper = u64::from(old.width()) * u64::from(old.height()) * 40;
    assert!(inventory.unique_bytes > one_view_upper * 2, "fixture needs more than one viewport's targets");
    // A single new viewport fits this ceiling, but not alongside all live targets.
    app.gpu_viewports.get_mut(&ViewId::Perspective).unwrap().set_target_ceiling_for_test(Some(inventory.unique_bytes));
    loop {
        frame_with_size(&mut app, &ctx, vec![], egui::vec2(1300., 740.));
        let port = &app.gpu_viewports[&ViewId::Perspective];
        if port.preparation_error().is_some() {
            assert_eq!(port.published_texture(), Some(&old));
            break;
        }
        assert!(Instant::now() < deadline, "combined target pressure did not refuse resize");
    }
    app.gpu_viewports.get_mut(&ViewId::Perspective).unwrap().set_target_ceiling_for_test(None);
    loop {
        frame_with_size(&mut app, &ctx, vec![], egui::vec2(1300., 740.));
        if app.gpu_viewports[&ViewId::Perspective].final_complete() { break; }
        assert!(Instant::now() < deadline, "viewport did not recover");
    }
}

#[test]
fn target_budget_refusal_retains_last_image_and_recovers_after_budget_returns() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut app = MatrixApp::new(Document::new("target budget recovery")).unwrap();
    app.gpu_state = Some(state);
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let port = &app.gpu_viewports[&ViewId::Perspective];
    let old = port.published_texture().unwrap().clone();
    let old_frame = port.encoded_frame;
    let old_pixel = texture_pixel(&app, &old, old.width() / 2, old.height() / 2);
    app.gpu_viewports.get_mut(&ViewId::Perspective).unwrap().set_target_ceiling_for_test(Some(1));
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        frame_with_size(&mut app, &ctx, vec![], egui::vec2(1500., 900.));
        let port = &app.gpu_viewports[&ViewId::Perspective];
        if port.preparation_error().is_some() {
            assert!(port.preparation_error().unwrap().contains("GPU ceiling"));
            assert_eq!(port.encoded_frame, old_frame);
            assert_eq!(port.published_texture(), Some(&old));
            assert_eq!(texture_pixel(&app, &old, old.width() / 2, old.height() / 2), old_pixel);
            break;
        }
        assert!(Instant::now() < deadline, "low-budget resize was not refused");
    }
    app.gpu_viewports.get_mut(&ViewId::Perspective).unwrap().set_target_ceiling_for_test(None);
    loop {
        frame_with_size(&mut app, &ctx, vec![], egui::vec2(1500., 900.));
        let port = &app.gpu_viewports[&ViewId::Perspective];
        if port.final_complete() && port.published_texture().is_some_and(|texture| texture != &old) {
            assert!(port.preparation_error().is_none());
            break;
        }
        assert!(Instant::now() < deadline, "viewport did not recover after budget returned");
    }
}

#[test]
fn viewport_wireframe_and_shaded_publish_distinct_face_pixels() {
    use om9_scene::workspace::DisplayMode;
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mesh = om9_core::geometry::Mesh {
        vertices: vec![[-8., -8., 0.], [8., -8., 0.], [8., 8., 0.], [-8., 8., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("viewport mode pixels");
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    document.entities.insert(entity.id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.workspace.active = ViewId::Perspective;
    let view = app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap();
    view.camera = Camera::orthographic([0., 0., 50.], [0., 0., 0.], [0., 1., 0.], 30.).unwrap();
    view.mode = DisplayMode::Wireframe;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let wire = viewport_pixel(&app, ViewId::Perspective, [2.3, 3.2, 0.]);
    // Compare to the same empty view: a cleared viewport must not pass as wireframe.
    // Sampling a narrow strip accommodates rasterization/MSAA coverage at the exact edge.
    let mut empty = MatrixApp::new(Document::new("empty viewport mode pixels")).unwrap();
    empty.gpu_state = app.gpu_state.clone();
    empty.controller.workspace.active = ViewId::Perspective;
    let empty_view = empty.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap();
    empty_view.camera = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
    empty_view.mode = DisplayMode::Wireframe;
    let empty_ctx = egui::Context::default();
    settle(&mut empty, &empty_ctx);
    let mut different_edge_pixels = 0;
    for y in [-8.15, -8.10, -8.05, -8.0, -7.95, -7.90, -7.85] {
        let point = [2.3, y, 0.];
        if viewport_pixel(&app, ViewId::Perspective, point)
            != viewport_pixel(&empty, ViewId::Perspective, point) {
            different_edge_pixels += 1;
        }
    }
    assert!(different_edge_pixels > 0, "wireframe must draw the authored mesh edge, not only clear the target");
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().mode = DisplayMode::Shaded;
    let port = &app.gpu_viewports[&ViewId::Perspective];
    let view = &app.controller.workspace.views[&ViewId::Perspective];
    assert!(!port.displays_content(app.controller.scene().unwrap(), app.controller.selection_state().revision,
        app.controller.preview_revision(), view.cplane.revision,
        app.controller.solid_boolean_two_hidden_sources(), view.mode), "old mode must not be accepted for picking");
    frame(&mut app, &ctx, vec![]);
    settle(&mut app, &ctx);
    let port = &app.gpu_viewports[&ViewId::Perspective];
    let view = &app.controller.workspace.views[&ViewId::Perspective];
    assert!(port.displays_content(app.controller.scene().unwrap(), app.controller.selection_state().revision,
        app.controller.preview_revision(), view.cplane.revision,
        app.controller.solid_boolean_two_hidden_sources(), view.mode));
    let shaded = viewport_pixel(&app, ViewId::Perspective, [2.3, 3.2, 0.]);
    assert_ne!(wire, shaded, "face interior must differ between Wireframe and Shaded: {wire:?} {shaded:?}");
}

#[test]
fn published_viewport_pixels_remain_stable_until_new_frame_is_published() {
    use om9_scene::workspace::DisplayMode;
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mesh = om9_core::geometry::Mesh {
        vertices: vec![[-8., -8., 0.], [8., -8., 0.], [8., 8., 0.], [-8., 8., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("published image isolation");
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    document.entities.insert(entity.id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.workspace.active = ViewId::Perspective;
    let view = app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap();
    view.camera = Camera::orthographic([0., 0., 50.], [0., 0., 0.], [0., 1., 0.], 30.).unwrap();
    view.mode = DisplayMode::Wireframe;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let original = viewport_pixel(&app, ViewId::Perspective, [2.3, 3.2, 0.]);
    let first_texture = app.gpu_viewports[&ViewId::Perspective].published_texture().unwrap().clone();
    let old_encoded = app.gpu_viewports[&ViewId::Perspective].encoded_frame;
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().mode = DisplayMode::Shaded;
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        frame(&mut app, &ctx, vec![]);
        let port = &app.gpu_viewports[&ViewId::Perspective];
        if port.encoded_frame != old_encoded {
            assert_eq!(port.displayed_mode(), Some(DisplayMode::Wireframe), "old image should remain published until completion is accepted");
            assert_eq!(viewport_pixel(&app, ViewId::Perspective, [2.3, 3.2, 0.]), original,
                "published pixels changed before published image metadata");
            break;
        }
        assert!(Instant::now() < deadline, "new viewport frame was not encoded");
    }
    settle(&mut app, &ctx);
    assert_eq!(app.gpu_viewports[&ViewId::Perspective].displayed_mode(), Some(DisplayMode::Shaded));
    assert!(app.gpu_viewports[&ViewId::Perspective].published_texture().unwrap() != &first_texture);
    assert_ne!(viewport_pixel(&app, ViewId::Perspective, [2.3, 3.2, 0.]), original);
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().mode = DisplayMode::Wireframe;
    settle(&mut app, &ctx);
    assert!(app.gpu_viewports[&ViewId::Perspective].published_texture().unwrap() == &first_texture,
        "the previous colour texture should be reused after publication");
    assert_eq!(viewport_pixel(&app, ViewId::Perspective, [2.3, 3.2, 0.]), original);
}

#[test]
fn gumball_preview_pixels_move_only_after_publication_in_two_viewports() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mesh = om9_core::geometry::Mesh {
        vertices: vec![[-8., -2., 0.], [-4., -2., 0.], [-4., 2., 0.], [-8., 2., 0.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    };
    let mut document = Document::new("preview pixels in two views");
    let entity = Entity::new("mesh", Geometry::Mesh(mesh), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    for view in [ViewId::Perspective, ViewId::LookingDown] {
        let view = app.controller.workspace.views.get_mut(&view).unwrap();
        view.camera = Camera::orthographic([0., 0., 50.], [0., 0., 0.], [0., 1., 0.], 30.).unwrap();
        view.mode = om9_scene::workspace::DisplayMode::Shaded;
    }
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera =
        Camera::perspective([0., -22., 32.], [0., 0., 0.], [0., 0., 1.], 0.9).unwrap();
    let ctx = egui::Context::default();
    let settle_both = |app: &mut MatrixApp| {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            frame(app, &ctx, vec![]);
            if [ViewId::Perspective, ViewId::LookingDown].iter().all(|view|
                app.gpu_viewports.get(view).is_some_and(|port| port.final_complete())) { break; }
            assert!(Instant::now() < deadline, "viewports failed to publish preview");
        }
    };
    settle_both(&mut app);
    let old_point = [-6.3, 0.7, 0.];
    let new_point = [5.7, 0.7, 0.];
    let before = [ViewId::Perspective, ViewId::LookingDown].map(|view|
        (viewport_pixel(&app, view, old_point), viewport_pixel(&app, view, new_point)));
    for (old, new) in before { assert_ne!(old, new, "fixture must start with a filled face and an empty destination"); }
    let old_encoded = [ViewId::Perspective, ViewId::LookingDown].map(|view| app.gpu_viewports[&view].encoded_frame);
    let old_published = [ViewId::Perspective, ViewId::LookingDown].map(|view| app.gpu_viewports[&view].published_frame);
    app.controller.preview_gumball(Transform::translation([12., 0., 0.])).unwrap();
    let mut observed = [false; 2];
    let deadline = Instant::now() + Duration::from_secs(10);
    while !observed.iter().all(|seen| *seen) {
        frame(&mut app, &ctx, vec![]);
        for (index, view) in [ViewId::Perspective, ViewId::LookingDown].into_iter().enumerate() {
            let port = &app.gpu_viewports[&view];
            if !observed[index] && port.encoded_frame != old_encoded[index] {
                assert_eq!(port.published_frame, old_published[index], "preview published before completion was accepted in {view:?}");
                assert_eq!(viewport_pixel(&app, view, old_point), before[index].0,
                    "old image changed before preview publication in {view:?}");
                assert_eq!(viewport_pixel(&app, view, new_point), before[index].1,
                    "new preview appeared before publication in {view:?}");
                observed[index] = true;
            }
        }
        assert!(Instant::now() < deadline, "preview was not encoded for both viewports");
    }
    settle_both(&mut app);
    for (index, view) in [ViewId::Perspective, ViewId::LookingDown].into_iter().enumerate() {
        let old = viewport_pixel(&app, view, old_point);
        let new = viewport_pixel(&app, view, new_point);
        assert_ne!(old, before[index].0, "old geometry position remained filled in {view:?}");
        assert_ne!(new, before[index].1, "preview did not appear at the moved position in {view:?}");
    }
    assert_eq!(app.controller.document().entities[&id].placement, Transform::identity());
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[cfg(feature = "occt-brep")]
#[test]
fn snappy_gumball_uses_published_mode_for_occlusion() {
    use om9_scene::{osnap::SnapKind, workspace::DisplayMode};
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let limits = om9_core::budget::Limits::default();
    let sphere = Entity::new("sphere", Geometry::Brep(om9_kernel::brep::sphere_brep(3., 0.1, &limits).unwrap()), None);
    let mesh = Entity::new("occluder", Geometry::Mesh(om9_core::geometry::Mesh {
        vertices: vec![[-5., -5., 10.], [5., -5., 10.], [5., 5., 10.], [-5., 5., 10.]],
        faces: vec![vec![0, 1, 2, 3]], normals: vec![], colors: vec![], corner_uvs: vec![],
    }), None);
    let line = Entity::new("moving line", Geometry::Curve(Curve::Line {
        start: [10., 0., 0.], end: [12., 0., 0.], domain: [0., 2.],
    }), None);
    let id = line.id;
    let mut document = Document::with_template_tolerance("published mode snap");
    for entity in [sphere, mesh, line] { document.entities.insert(entity.id, entity); }
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.workspace.active = ViewId::Perspective;
    let view = app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap();
    view.camera = Camera::orthographic([0., 0., 50.], [0., 0., 0.], [0., 1., 0.], 30.).unwrap();
    view.mode = DisplayMode::Shaded;
    app.controller.select_objects(&[id]).unwrap();
    app.controller.submit("Gumball Snappy").unwrap();
    app.set_snap_override(Some(SnapKind::Surface));
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let rect = app.view_rects[&ViewId::Perspective];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap().clone();
    let cursor = camera.project([1., 0.5, (9_f64 - 1. - 0.25).sqrt()], viewport).unwrap().unwrap().screen;
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().mode = DisplayMode::Wireframe;
    assert!(app.gumball_snap_candidate(&camera, viewport, cursor).is_err(), "a shaded published image must still occlude");
    frame(&mut app, &ctx, vec![]);
    settle(&mut app, &ctx);
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().mode = DisplayMode::Shaded;
    assert!(app.gumball_snap_candidate(&camera, viewport, cursor).unwrap().is_some(), "a wireframe published image must allow the visible surface");

    frame(&mut app, &ctx, vec![]);
    settle(&mut app, &ctx);
    let start = camera.project([11., 0., 0.], viewport).unwrap().unwrap().screen;
    let start = egui::pos2(start[0] as f32, start[1] as f32);
    let end = egui::pos2(cursor[0] as f32, cursor[1] as f32);
    let button = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, true)]);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start + egui::vec2(-8., 0.))]);
    assert!(app.gumball_session.is_some(), "Gumball must capture the drag");
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().mode = DisplayMode::Wireframe;
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)]);
    assert_eq!(app.gpu_viewports[&ViewId::Perspective].displayed_mode(), Some(DisplayMode::Shaded), "the occlusion decision must be made while the old image is published");
    frame(&mut app, &ctx, vec![button(end, false)]);
    assert_eq!(app.controller.document().entities[&id].placement, Transform::identity(), "an occluded surface on the published image must not commit");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(app.status_line().contains("occluded"), "{}", app.status_line());
    settle(&mut app, &ctx);
    assert_eq!(app.gpu_viewports[&ViewId::Perspective].displayed_mode(), Some(DisplayMode::Wireframe));
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, true)]);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start + egui::vec2(-8., 0.))]);
    assert!(app.gumball_session.is_some(), "second drag must be captured");
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().mode = DisplayMode::Shaded;
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end), button(end, false)]);
    assert_eq!(app.gpu_viewports[&ViewId::Perspective].displayed_mode(), Some(DisplayMode::Wireframe));
    let placement = app.controller.document().entities[&id].placement;
    let snapped = [11. + placement.0[0][3], placement.0[1][3], placement.0[2][3]];
    let radius = snapped.iter().map(|x| x * x).sum::<f64>().sqrt();
    assert!((radius - 3.).abs() < 1e-6, "the committed pivot must land on the exact sphere: {snapped:?}");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

fn drag_after_orbit(wait_for_refinement: bool) {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("GPU Gumball interaction");
    let entity = Entity::new("line", Geometry::Curve(Curve::Line {
        start: [-10., 0., 0.], end: [10., 0., 0.], domain: [0., 20.],
    }), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let rect = app.view_rects[&ViewId::Perspective];
    let orbit_start = rect.center() + egui::vec2(100., 70.);
    let orbit_end = orbit_start + egui::vec2(35., 15.);
    let button = |pos, button, pressed| egui::Event::PointerButton { pos, button, pressed, modifiers: egui::Modifiers::NONE };
    let before = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(orbit_start), button(orbit_start, egui::PointerButton::Secondary, true)]);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(orbit_end)]);
    frame(&mut app, &ctx, vec![button(orbit_end, egui::PointerButton::Secondary, false)]);
    assert_ne!(app.controller.workspace.views[&ViewId::Perspective].camera, before);
    if wait_for_refinement { settle(&mut app, &ctx); }
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap().clone();
    let rect = app.view_rects[&ViewId::Perspective];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    let size = world_per_pixel(&camera, viewport) * 90.;
    let project = |x: f64| { let p = camera.project([pivot[0] + x, pivot[1], pivot[2]], viewport).unwrap().unwrap().screen; egui::pos2(p[0] as f32, p[1] as f32) };
    let start = project(size * 0.55);
    let end = project(size * 1.4);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, egui::PointerButton::Primary, true)]);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)]);
    assert_ne!(app.controller.scene().unwrap().objects[0].display_transform().unwrap(), Transform::identity(), "GPU-backed Gumball preview did not move");
    frame(&mut app, &ctx, vec![button(end, egui::PointerButton::Primary, false)]);
    assert!(app.controller.document().entities[&id].placement.0[0][3].abs() > 1e-6, "GPU-backed Gumball did not commit");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[test]
fn perspective_axis_drag_after_orbit_uses_published_gpu_camera() {
    drag_after_orbit(true);
}

#[test]
fn perspective_axis_drag_immediately_after_orbit_keeps_its_capture() {
    drag_after_orbit(false);
}

#[cfg(feature = "occt-brep")]
#[test]
fn perspective_box_gumball_drag_immediately_after_zoom() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let limits = om9_core::budget::Limits::default();
    let mut document = Document::new("post-zoom box Gumball");
    let entity = Entity::new("box", Geometry::Brep(om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    let trace_path = std::env::temp_dir().join(format!("om9-gumball-route-{}.jsonl", uuid::Uuid::new_v4()));
    app.interaction_log = Some(crate::interaction_log::InteractionLog::open(trace_path.clone()).expect("diagnostic log"));
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let rect = app.view_rects[&ViewId::Perspective];
    let scroll_pos = rect.center();
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(scroll_pos)]);
    let before = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
    frame(&mut app, &ctx, vec![egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line, delta: egui::vec2(0., 1.), modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move,
    }]);
    frame(&mut app, &ctx, vec![]);
    assert_ne!(app.controller.workspace.views[&ViewId::Perspective].camera, before, "wheel must change the camera");
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap().clone();
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    let size = world_per_pixel(&camera, viewport) * 90.;
    let project = |z: f64| { let p = camera.project([pivot[0], pivot[1], pivot[2] + z], viewport).unwrap().unwrap().screen; egui::pos2(p[0] as f32, p[1] as f32) };
    let start = project(size * 0.7);
    let end = project(size * 1.3);
    let button = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, true)]);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)]);
    assert!(app.gumball_session.is_some(), "Gumball must capture the first drag after zoom");
    frame(&mut app, &ctx, vec![button(end, false)]);
    assert!(app.controller.document().entities[&id].placement.0[2][3].abs() > 1e-6, "first post-zoom drag must commit");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.flush_interaction_log();
    let records: Vec<serde_json::Value> = std::fs::read_to_string(&trace_path).unwrap().lines()
        .map(|line|serde_json::from_str(line).unwrap()).collect();
    assert!(records.iter().any(|r| r["event"] == "gumball_route" && r["primary_pressed"] == true));
    assert!(records.iter().any(|r| r["event"] == "gumball_route" && r["drag_started"] == true && r["scene_current"] == true && r["press_matches"] == true));
    assert!(records.iter().any(|r| r["event"] == "gumball_hit" && r["handle"].is_string()));
    assert!(records.iter().any(|r| r["event"] == "gumball_commit" && r["success"] == true));
    assert!(records.iter().all(|r| r["dropped_records"] == 0));
    drop(app.interaction_log.take());
    std::fs::remove_file(trace_path).unwrap();
}

#[cfg(feature = "occt-brep")]
fn perspective_box_batched_drag_after_zoom(release_same_frame: bool, outside: bool) {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let limits = om9_core::budget::Limits::default();
    let mut document = Document::new("batched post-zoom box Gumball");
    let entity = Entity::new("box", Geometry::Brep(om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    let trace_path = std::env::temp_dir().join(format!("om9-gumball-batched-{}.jsonl", uuid::Uuid::new_v4()));
    app.interaction_log = Some(crate::interaction_log::InteractionLog::open(trace_path.clone()).unwrap());
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let rect = app.view_rects[&ViewId::Perspective];
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(rect.center())]);
    let before = app.controller.workspace.views[&ViewId::Perspective].camera.clone();
    frame(&mut app, &ctx, vec![egui::Event::MouseWheel {
        unit: egui::MouseWheelUnit::Line, delta: egui::vec2(0., 1.), modifiers: egui::Modifiers::NONE, phase: egui::TouchPhase::Move,
    }]);
    frame(&mut app, &ctx, vec![]);
    assert_ne!(app.controller.workspace.views[&ViewId::Perspective].camera, before);
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap().clone();
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    let size = world_per_pixel(&camera, viewport) * 90.;
    let project = |z: f64| { let p = camera.project([pivot[0], pivot[1], pivot[2] + z], viewport).unwrap().unwrap().screen; egui::pos2(p[0] as f32, p[1] as f32) };
    let start = project(size * 0.7);
    let end = project(size * if outside { 1.3 } else { 0.85 });
    assert!(rect.contains(start) && rect.contains(end) != outside, "fixture viewport boundary: {rect:?} {start:?} {end:?}");
    let button = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
    if release_same_frame {
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, true), egui::Event::PointerMoved(end), button(end, false)]);
    } else {
        frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, true), egui::Event::PointerMoved(end)]);
        frame(&mut app, &ctx, vec![button(end, false)]);
    }
    app.flush_interaction_log();
    if app.controller.document().entities[&id].placement.0[2][3].abs() <= 1e-6 {
        for line in std::fs::read_to_string(&trace_path).unwrap().lines() {
            if line.contains("gumball_route") || line.contains("gumball_hit") || line.contains("navigation_input") { eprintln!("{line}"); }
        }
    }
    assert!(app.controller.document().entities[&id].placement.0[2][3].abs() > 1e-6, "batched press and movement followed by release must commit");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    let batched_placement = app.controller.document().entities[&id].placement;
    settle(&mut app, &ctx);
    assert_eq!(app.gpu_viewports[&ViewId::Perspective].displayed_camera(), Some(&app.controller.workspace.views[&ViewId::Perspective].camera), "publication must resume after release");
    app.controller.undo().unwrap();
    app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera = camera.clone();
    settle(&mut app, &ctx);
    assert_eq!(app.gpu_viewports[&ViewId::Perspective].displayed_camera(), Some(&camera));
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, true)]);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)]);
    frame(&mut app, &ctx, vec![button(end, false)]);
    let ordinary = app.controller.document().entities[&id].placement;
    for (a, b) in batched_placement.0.iter().flatten().zip(ordinary.0.iter().flatten()) {
        assert!((a - b).abs() < 1e-6, "batched drag must match the ordinary drag: {batched_placement:?} versus {ordinary:?}");
    }
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    app.controller.undo().unwrap();
    settle(&mut app, &ctx);
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap().clone();
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    let size = world_per_pixel(&camera, viewport) * 90.;
    let handle = camera.project([pivot[0], pivot[1], pivot[2] + size * 0.7], viewport).unwrap().unwrap().screen;
    let handle = egui::pos2(handle[0] as f32, handle[1] as f32);
    let close = handle + egui::vec2(1., 1.);
    let before = app.controller.document().entities[&id].placement;
    let undo = app.controller.coordinator().undo_len();
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(handle), button(handle, true), egui::Event::PointerMoved(close)]);
    frame(&mut app, &ctx, vec![button(close, false)]);
    assert_eq!(app.controller.document().entities[&id].placement, before, "a click must not transform geometry");
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    let drag_end = camera.project([pivot[0], pivot[1], pivot[2] + size * 0.85], viewport).unwrap().unwrap().screen;
    let drag_end = egui::pos2(drag_end[0] as f32, drag_end[1] as f32);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(handle), button(handle, true), egui::Event::PointerMoved(drag_end)]);
    frame(&mut app, &ctx, vec![egui::Event::Key { key: egui::Key::Escape, physical_key: None, pressed: true, repeat: false, modifiers: egui::Modifiers::NONE }]);
    frame(&mut app, &ctx, vec![button(drag_end, false)]);
    assert_eq!(app.controller.document().entities[&id].placement, before, "Escape must cancel a pending batched drag");
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    let outside_start = egui::pos2(handle.x, rect.top() - 1.);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(outside_start), button(outside_start, true), egui::Event::PointerMoved(handle)]);
    frame(&mut app, &ctx, vec![button(handle, false)]);
    assert_eq!(app.controller.document().entities[&id].placement, before, "a press outside the viewport must not capture its Gumball");
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    let elsewhere = rect.left_bottom() + egui::vec2(20., -20.);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(elsewhere), button(elsewhere, true), egui::Event::PointerMoved(handle)]);
    frame(&mut app, &ctx, vec![button(handle, false)]);
    assert_eq!(app.controller.document().entities[&id].placement, before, "a drag starting away from a handle must not transform geometry");
    assert_eq!(app.controller.coordinator().undo_len(), undo);
    drop(app.interaction_log.take());
    std::fs::remove_file(trace_path).unwrap();
}

#[cfg(feature = "occt-brep")]
#[test]
fn perspective_box_drag_start_in_same_frame_as_press_after_zoom() {
    perspective_box_batched_drag_after_zoom(false, false);
}

#[cfg(feature = "occt-brep")]
#[test]
fn perspective_box_entire_drag_in_one_frame_after_zoom() {
    perspective_box_batched_drag_after_zoom(true, false);
}

#[cfg(feature = "occt-brep")]
#[test]
fn perspective_box_entire_drag_exits_viewport_in_one_frame() {
    perspective_box_batched_drag_after_zoom(true, true);
}

#[cfg(feature = "occt-brep")]
#[test]
fn perspective_box_drag_exits_viewport_before_next_frame() {
    perspective_box_batched_drag_after_zoom(false, true);
}

#[cfg(feature = "occt-brep")]
#[test]
fn focus_loss_cancels_batched_gumball_drag_and_allows_a_fresh_drag() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("batched Gumball focus loss");
    let entity = Entity::new("box", Geometry::Brep(om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &om9_core::budget::Limits::default()).unwrap()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let rect = app.view_rects[&ViewId::Perspective];
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap().clone();
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    let size = world_per_pixel(&camera, viewport) * 90.;
    let project = |z: f64| { let p = camera.project([pivot[0], pivot[1], pivot[2] + z], viewport).unwrap().unwrap().screen; egui::pos2(p[0] as f32, p[1] as f32) };
    let start = project(size * 0.7);
    let end = project(size * 0.85);
    let button = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
    let before = app.controller.document().entities[&id].placement;
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, true), egui::Event::PointerMoved(end)]);
    assert!(app.primary_press_scene.is_some(), "fixture must retain the published press basis before focus loss");
    let mut output = ctx.run_ui(egui::RawInput {
        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280., 720.))),
        focused: false, events: vec![egui::Event::WindowFocused(false), button(end, false)], ..Default::default()
    }, |ui| app.show(ui));
    output.textures_delta.clear();
    assert_eq!(app.controller.document().entities[&id].placement, before, "focus loss must cancel the pending drag");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
    assert!(app.primary_press_scene.is_none(), "focus loss must discard the pending press basis");
    frame(&mut app, &ctx, vec![egui::Event::WindowFocused(true)]);
    settle(&mut app, &ctx);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, true)]);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)]);
    frame(&mut app, &ctx, vec![button(end, false)]);
    assert_ne!(app.controller.document().entities[&id].placement, before, "fresh drag after focus returns must work");
    assert_eq!(app.controller.coordinator().undo_len(), 1);
}

#[cfg(feature = "occt-brep")]
fn gumball_does_not_capture_press_on_overlay(same_layer: bool) {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let limits = om9_core::budget::Limits::default();
    let mut document = Document::new("overlay Gumball");
    let entity = Entity::new("box", Geometry::Brep(om9_kernel::brep::box_brep([10., 10., 10.], 0.1, &limits).unwrap()), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let rect = app.view_rects[&ViewId::Perspective];
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap().clone();
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    let size = world_per_pixel(&camera, viewport) * 90.;
    let project = |z: f64| { let p = camera.project([pivot[0], pivot[1], pivot[2] + z], viewport).unwrap().unwrap().screen; egui::pos2(p[0] as f32, p[1] as f32) };
    let start = project(size * 0.7);
    let end = project(size * 1.0);
    let overlay = Rect::from_center_size(start, egui::vec2(12., 12.));
    assert!(rect.contains(start) && rect.contains(end) && !overlay.contains(end));
    let overlay_frame = |app: &mut MatrixApp, events| {
        std::thread::sleep(Duration::from_millis(40));
        let mut overlay_hovered = false;
        let mut output = ctx.run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280., 720.))), events, ..Default::default() }, |ui| {
            app.show(ui);
            if same_layer {
                overlay_hovered = ui.interact(overlay, egui::Id::new("test-gumball-same-layer-overlay"), egui::Sense::click_and_drag()).hovered();
            } else {
                overlay_hovered = egui::Area::new(egui::Id::new("test-gumball-overlay")).order(egui::Order::Foreground).fixed_pos(overlay.min).show(ui.ctx(), |ui| {
                    ui.allocate_exact_size(overlay.size(), egui::Sense::click_and_drag()).1.hovered()
                }).inner;
            }
        });
        output.textures_delta.clear();
        app.gpu_state.as_ref().unwrap().device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        overlay_hovered
    };
    overlay_frame(&mut app, vec![]);
    overlay_frame(&mut app, vec![]);
    if !same_layer {
        assert_eq!(ctx.layer_id_at(start).map(|layer| layer.order), Some(egui::Order::Foreground), "fixture must cover the press point");
    }
    assert!(overlay_frame(&mut app, vec![egui::Event::PointerMoved(start)]), "fixture widget must be hovered at the press point");
    let before = app.controller.document().entities[&id].placement;
    let button = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
    overlay_frame(&mut app, vec![egui::Event::PointerMoved(start), button(start, true), egui::Event::PointerMoved(end), button(end, false)]);
    assert_eq!(app.controller.document().entities[&id].placement, before, "overlay press must not drag the underlying Gumball");
    assert_eq!(app.controller.coordinator().undo_len(), 0);
}

#[cfg(feature = "occt-brep")]
#[test]
fn gumball_does_not_capture_press_on_foreground_overlay() {
    gumball_does_not_capture_press_on_overlay(false);
}

#[cfg(feature = "occt-brep")]
#[test]
fn gumball_does_not_capture_press_on_same_layer_overlay() {
    gumball_does_not_capture_press_on_overlay(true);
}

#[test]
fn perspective_drag_retains_camera_published_when_pressed() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("pending camera Gumball");
    let entity = Entity::new("line", Geometry::Curve(Curve::Line {
        start: [-10., 0., 0.], end: [10., 0., 0.], domain: [0., 20.],
    }), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let rect = app.view_rects[&ViewId::Perspective];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap().clone();
    let pivot = app.controller.gumball_pivot().unwrap().origin;
    let size = world_per_pixel(&camera, viewport) * 90.;
    let project = |x: f64| { let p = camera.project([pivot[0] + x, pivot[1], pivot[2]], viewport).unwrap().unwrap().screen; egui::pos2(p[0] as f32, p[1] as f32) };
    let start = project(size * 0.55);
    let mid = project(size * 0.8);
    let end = project(size * 1.4);
    let button = |pos, pressed| egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed, modifiers: egui::Modifiers::NONE };
    // An orbit has advanced the authoritative camera while the last published image
    // still represents the previous camera. Begin the gesture against that image.
    let next = &mut app.controller.workspace.views.get_mut(&ViewId::Perspective).unwrap().camera;
    next.eye[0] += 80.;
    next.revision += 1;
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(start), button(start, true)]);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(mid)]);
    assert!(app.gumball_session.is_some(), "gesture must capture against the published image");
    assert_eq!(app.gumball_basis.as_ref().unwrap().camera, camera, "capture must use the image present at press");
    let encoded_before = app.gpu_viewports[&ViewId::Perspective].encoded_frame.unwrap_or(0);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(end)]);
    assert!(app.gumball_session.is_some(), "gesture must stay captured");
    assert_eq!(app.gpu_viewports[&ViewId::Perspective].displayed_camera(), Some(&camera), "captured image must remain stable during the drag");
    let deadline = Instant::now() + Duration::from_secs(10);
    while app.gpu_viewports[&ViewId::Perspective].encoded_frame.unwrap_or(0) <= encoded_before {
        frame(&mut app, &ctx, vec![]);
        assert!(Instant::now() < deadline, "GPU preview must continue rendering during the drag");
    }
    frame(&mut app, &ctx, vec![button(end, false)]);
    assert!(app.controller.document().entities[&id].placement.0[0][3].abs() > 1e-6);
    assert_eq!(app.controller.coordinator().undo_len(), 1);
    settle(&mut app, &ctx);
    assert_eq!(app.gpu_viewports[&ViewId::Perspective].displayed_camera(), Some(&app.controller.workspace.views[&ViewId::Perspective].camera), "latest camera must be published after release");
}

#[test]
fn point_command_waits_for_the_current_preview_to_be_published() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("stale preview selection");
    let entity = Entity::new("line", Geometry::Curve(Curve::Line {
        start: [-3., 0., 0.], end: [3., 0., 0.], domain: [0., 6.],
    }), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    app.controller.submit("Line").unwrap();
    app.controller.preview_gumball(Transform::translation([12., 0., 0.])).unwrap();
    let rect = app.view_rects[&ViewId::Perspective];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap();
    let point = camera.project([15., 0., 0.], viewport).unwrap().unwrap().screen;
    let point = egui::pos2(point[0] as f32, point[1] as f32);
    assert!(rect.contains(point), "fixture point must be inside the viewport: {point:?} {rect:?}");
    let click = || vec![egui::Event::PointerMoved(point),
        egui::Event::PointerButton { pos: point, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE },
        egui::Event::PointerButton { pos: point, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE }];
    frame(&mut app, &ctx, click());
    assert!(app.controller.prompt_with_options().contains("Start"), "a point on a preview not yet shown must not be accepted: {}", app.controller.prompt_with_options());
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(point),
        egui::Event::PointerButton { pos: point, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE }]);
    settle(&mut app, &ctx);
    assert_eq!(app.controller.scene().unwrap().objects[0].display_transform().unwrap(), Transform::translation([12., 0., 0.]), "preview must remain active");
    frame(&mut app, &ctx, vec![egui::Event::PointerButton { pos: point, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE }]);
    assert!(app.controller.prompt_with_options().contains("Start"), "publication after press must not validate that old press");
    frame(&mut app, &ctx, click());
    assert!(app.controller.prompt_with_options().contains("End"), "point input should work after preview publication: {}", app.controller.prompt_with_options());
}

#[test]
fn hover_is_cleared_while_the_published_scene_is_outdated() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("hover during unpublished scene");
    let entity = Entity::new("line", Geometry::Curve(Curve::Line {
        start: [-10., 0., 0.], end: [10., 0., 0.], domain: [0., 20.],
    }), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.workspace.active = ViewId::Perspective;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    let rect = app.view_rects[&ViewId::Perspective];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap();
    let at = camera.project([0., 0., 0.], viewport).unwrap().unwrap().screen;
    let pointer = egui::pos2(at[0] as f32, at[1] as f32);
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(pointer)]);
    assert_eq!(app.hovered_object(), Some(id), "fixture must first hover the published object");
    app.controller.select_objects(&[id]).unwrap();
    app.controller.numeric_move([20., 0., 0.]).unwrap();
    app.controller.clear_selection();
    assert!(!app.gpu_viewports[&ViewId::Perspective].displays_scene(app.controller.scene().unwrap()));
    frame(&mut app, &ctx, vec![]);
    assert_eq!(app.hovered_object(), None, "old hover must be cleared while the displayed scene is stale");
    settle(&mut app, &ctx);
    frame(&mut app, &ctx, vec![]);
    assert_eq!(app.hovered_object(), None, "moved object must not be hovered at its old position");
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap();
    let at = camera.project([20., 0., 0.], viewport).unwrap().unwrap().screen;
    let moved_pointer = egui::pos2(at[0] as f32, at[1] as f32);
    assert!(rect.contains(moved_pointer));
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(moved_pointer)]);
    assert_eq!(app.hovered_object(), Some(id), "hover must resume against the newly published scene");
}

#[test]
fn selection_waits_for_the_current_scene_to_be_published() {
    let Some(state) = gpu() else { eprintln!("SKIP: no GPU adapter"); return };
    let mut document = Document::new("stale scene selection");
    let entity = Entity::new("line", Geometry::Curve(Curve::Line {
        start: [-10., 0., 0.], end: [10., 0., 0.], domain: [0., 20.],
    }), None);
    let id = entity.id;
    document.entities.insert(id, entity);
    let mut app = MatrixApp::new(document).unwrap();
    app.gpu_state = Some(state);
    app.controller.select_objects(&[id]).unwrap();
    app.controller.workspace.active = ViewId::Perspective;
    let ctx = egui::Context::default();
    settle(&mut app, &ctx);
    app.controller.numeric_move([3., 0., 0.]).unwrap();
    app.controller.clear_selection();
    assert!(!app.gpu_viewports[&ViewId::Perspective].displays_scene(app.controller.scene().unwrap()));
    let rect = app.view_rects[&ViewId::Perspective];
    let viewport = Viewport { origin: [rect.min.x as f64, rect.min.y as f64], size: [rect.width() as f64, rect.height() as f64] };
    let camera = app.gpu_viewports[&ViewId::Perspective].displayed_camera().unwrap();
    let point = camera.project([3., 0., 0.], viewport).unwrap().unwrap().screen;
    let point = egui::pos2(point[0] as f32, point[1] as f32);
    let click = || vec![egui::Event::PointerMoved(point),
        egui::Event::PointerButton { pos: point, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE },
        egui::Event::PointerButton { pos: point, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE }];
    frame(&mut app, &ctx, click());
    assert!(app.controller.selected().is_empty(), "a click on an old scene must not select the current geometry");
    settle(&mut app, &ctx);
    assert!(app.gpu_viewports[&ViewId::Perspective].displays_scene(app.controller.scene().unwrap()));
    frame(&mut app, &ctx, click());
    assert!(app.controller.selected().contains(&id), "selection must work once the current scene is published");
    app.controller.numeric_move([2., 0., 0.]).unwrap();
    app.controller.clear_selection();
    frame(&mut app, &ctx, vec![egui::Event::PointerMoved(point),
        egui::Event::PointerButton { pos: point, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE }]);
    settle(&mut app, &ctx);
    assert!(app.gpu_viewports[&ViewId::Perspective].displays_scene(app.controller.scene().unwrap()));
    frame(&mut app, &ctx, vec![egui::Event::PointerButton { pos: point, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE }]);
    assert!(app.controller.selected().is_empty(), "publication after the press must not make a stale click valid");
}
