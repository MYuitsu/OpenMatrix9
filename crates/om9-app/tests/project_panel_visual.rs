// SPDX-License-Identifier: LGPL-2.1-or-later
//! Offscreen rendering of the actual egui Projects panel for visual review.
use eframe::{egui, egui_wgpu, wgpu};
use om9_app::ui::MatrixApp;
use om9_core::{geometry::Geometry, model::{Document, Entity}};

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

#[test]
#[ignore = "requires a GPU and OM9_PROJECT_PANEL_IMAGE output path"]
fn render_projects_panel_offscreen() {
    let output = std::path::PathBuf::from(std::env::var_os("OM9_PROJECT_PANEL_IMAGE").expect("output path"));
    let scale: f32 = std::env::var("OM9_PROJECT_PANEL_SCALE").unwrap_or_else(|_| "1".into()).parse().unwrap();
    assert!([1.0, 1.25, 1.5].contains(&scale));
    let width_px = (1280.0 * scale) as u32;
    let height_px = (900.0 * scale) as u32;
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(), flags: Default::default(), memory_budget_thresholds: Default::default(),
        backend_options: Default::default(), display: Default::default(),
    });
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: false,
        compatible_surface: None, apply_limit_buckets: false,
    })).expect("GPU adapter");
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(), required_limits: adapter.limits(), ..Default::default()
    })).expect("GPU device");
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let mut renderer = egui_wgpu::Renderer::new(&device, format, Default::default());
    let mut document = Document::new("Studio");
    let point = Entity::new("sample", Geometry::Point([1., 2., 3.]), None);
    let id = point.id;
    document.entities.insert(id, point.clone());
    let mut app = MatrixApp::new(document).unwrap();
    let archive = app.controller.copy_jobbag_objects_om9(&[id]).unwrap();
    app.project_manager.capture_archive(0, vec![point], app.controller.document(), archive).unwrap();
    let (document, archive) = app.controller.capture_jobbag_document().unwrap();
    app.project_manager.capture_named_version("Master", document, archive).unwrap();
    let context = egui::Context::default();
    context.set_pixels_per_point(scale);
    let mut final_output = None;
    for frame in 0..8 {
        let mut output = context.run_ui(egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1280., 900.))),
            time: Some(frame as f64 / 60.), ..Default::default()
        }, |ui| app.show(ui));
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas { renderer.update_texture(&device, &queue, *id, delta); }
        }
        output.textures_delta.clear();
        final_output = Some(output);
    }
    let output_frame = final_output.unwrap();
    let shapes = context.tessellate(output_frame.shapes, scale);
    let screen = egui_wgpu::ScreenDescriptor { size_in_pixels: [width_px, height_px], pixels_per_point: scale };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("project panel visual"), size: wgpu::Extent3d { width: width_px, height: height_px, depth_or_array_layers: 1 },
        mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2, format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC, view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("project panel readback"), size: u64::from(width_px) * u64::from(height_px) * 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    let callbacks = renderer.update_buffers(&device, &queue, &mut encoder, &shapes, &screen);
    {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("project panel"), color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view, depth_slice: None, resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })], depth_stencil_attachment: None, timestamp_writes: None, occlusion_query_set: None, multiview_mask: None,
        });
        renderer.render(&mut pass.forget_lifetime(), &shapes, &screen);
    }
    encoder.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0,
        origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        wgpu::TexelCopyBufferInfo { buffer: &buffer, layout: wgpu::TexelCopyBufferLayout {
            offset: 0, bytes_per_row: Some(width_px * 4), rows_per_image: Some(height_px) } },
        wgpu::Extent3d { width: width_px, height: height_px, depth_or_array_layers: 1 });
    queue.submit(callbacks.into_iter().chain(std::iter::once(encoder.finish())));
    let slice = buffer.slice(..);
    slice.map_async(wgpu::MapMode::Read, |_| {});
    device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None }).unwrap();
    let bytes = slice.get_mapped_range().unwrap();
    let image = image::RgbaImage::from_raw(width_px, height_px, bytes.to_vec()).unwrap();
    drop(bytes);
    buffer.unmap();
    let top = ((app.project_row_rects[&app.project_manager.active().id].top() - 24.).max(0.) * scale) as u32;
    let bottom = ((app.project_action_rects["named-strip"].bottom() + 18.).min(900.) * scale) as u32;
    let width = ((app.sidebar_content_rect.unwrap().right() + 2.) * scale).ceil() as u32;
    image::imageops::crop_imm(&image, 0, top, width, bottom - top).to_image().save(&output).unwrap();
    println!("offscreen panel: {}", output.display());
}
