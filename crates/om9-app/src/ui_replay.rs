// SPDX-License-Identifier: LGPL-2.1-or-later
//! Headless input replay through the real `MatrixApp::show` widget tree.
//!
//! No window and no GPU surface: an `egui::Context` runs the same frames the application runs,
//! pointer/key events are fed as `RawInput`, and each step returns a typed receipt (view rects,
//! camera revisions, selection, prompt, undo depth, focus, panels). This is how hover, pick,
//! orbit, pan, zoom, rubber-band and keyboard focus are tested without a screenshot.
use crate::ui::MatrixApp;
use eframe::egui::{
    Context, Event, Key, Modifiers, MouseWheelUnit, PointerButton, Pos2, RawInput, Rect,
    TouchPhase, Vec2,
};
use om9_core::{Error, Result, model::Document};
use serde_json::{Value, json};

/// Runs `steps` and returns one receipt per step, plus the initial state.
pub fn replay(document: Document, size: [f32; 2], dpi: f32, steps: &[Value]) -> Result<Value> {
    if !(size[0] > 0. && size[1] > 0.) {
        return Err(Error::Invalid("ui replay size"));
    }
    let mut app = MatrixApp::new(document)?;
    replay_app(&mut app, size, dpi, steps)
}

/// [`replay`] on an existing application (add-ons attached, objects already selected …).
pub fn replay_app(app: &mut MatrixApp, size: [f32; 2], dpi: f32, steps: &[Value]) -> Result<Value> {
    if !(size[0] > 0. && size[1] > 0.) {
        return Err(Error::Invalid("ui replay size"));
    }
    let app = &mut *app;
    let context = Context::default();
    context.set_pixels_per_point(dpi.clamp(0.5, 4.0));
    let extent = Vec2::new(size[0], size[1]);
    let mut time = 0.0;
    // Two settling frames: the first builds panels, the second lets focus follow them.
    frame(app, &context, extent, &mut time, Vec::new());
    frame(app, &context, extent, &mut time, Vec::new());
    let mut receipts = vec![receipt(app, &context, 0)];
    for (index, step) in steps.iter().enumerate() {
        apply(app, &context, extent, &mut time, step)?;
        for _ in 0..4 {
            frame(app, &context, extent, &mut time, Vec::new());
        }
        receipts.push(receipt(app, &context, index + 1));
    }
    Ok(json!({"size": size, "dpi": dpi, "steps": receipts}))
}

fn frame(app: &mut MatrixApp, context: &Context, size: Vec2, time: &mut f64, events: Vec<Event>) {
    *time += 1. / 30.;
    let raw = RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
        events,
        time: Some(*time),
        focused: false,
        ..Default::default()
    };
    let mut output = context.run_ui(raw, |ui| app.show(ui));
    output.textures_delta.clear();
}

/// The active view follows the pointer, exactly like a user clicking in a viewport.
fn activate_at(app: &mut MatrixApp, point: Pos2) {
    if let Some((id, _)) = app.view_rects.iter().find(|(_, rect)| rect.contains(point)) {
        let _ = app.controller.activate_view(id.title());
    }
}

fn apply(
    app: &mut MatrixApp,
    context: &Context,
    size: Vec2,
    time: &mut f64,
    step: &Value,
) -> Result<()> {
    let button = match step.get("button").and_then(Value::as_str).unwrap_or("left") {
        "left" | "primary" => PointerButton::Primary,
        "right" | "secondary" => PointerButton::Secondary,
        "middle" => PointerButton::Middle,
        other => {
            return Err(Error::Io(format!(
                "ui replay: unknown mouse button `{other}`"
            )));
        }
    };
    if let Some(wait) = step.get("wait").and_then(Value::as_u64) {
        for _ in 0..wait.min(600) {
            frame(app, context, size, time, Vec::new());
        }
        return Ok(());
    }
    if let Some(point) = step.get("move").and_then(position) {
        activate_at(app, point);
        frame(app, context, size, time, vec![Event::PointerMoved(point)]);
        return Ok(());
    }
    if let Some(point) = step.get("click").and_then(position) {
        activate_at(app, point);
        frame(
            app,
            context,
            size,
            time,
            vec![
                Event::PointerMoved(point),
                Event::PointerButton {
                    pos: point,
                    button,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        frame(
            app,
            context,
            size,
            time,
            vec![Event::PointerButton {
                pos: point,
                button,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        return Ok(());
    }
    if let Some(drag) = step.get("drag") {
        let from = drag
            .get("from")
            .and_then(position)
            .ok_or(Error::Invalid("ui replay drag from"))?;
        let to = drag
            .get("to")
            .and_then(position)
            .ok_or(Error::Invalid("ui replay drag to"))?;
        if let Some(view) = drag.get("view").and_then(Value::as_str) {
            app.controller
                .activate_view(view)
                .map_err(|error| Error::Io(error.to_string()))?;
        } else {
            activate_at(app, from);
        }
        let count = drag
            .get("steps")
            .and_then(Value::as_u64)
            .unwrap_or(6)
            .clamp(1, 120);
        frame(
            app,
            context,
            size,
            time,
            vec![
                Event::PointerMoved(from),
                Event::PointerButton {
                    pos: from,
                    button,
                    pressed: true,
                    modifiers: Modifiers::NONE,
                },
            ],
        );
        for slot in 1..=count {
            let t = slot as f32 / count as f32;
            let point = Pos2::new(from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t);
            frame(app, context, size, time, vec![Event::PointerMoved(point)]);
        }
        frame(
            app,
            context,
            size,
            time,
            vec![Event::PointerButton {
                pos: to,
                button,
                pressed: false,
                modifiers: Modifiers::NONE,
            }],
        );
        return Ok(());
    }
    if let Some(delta) = step.get("wheel").and_then(Value::as_f64) {
        let at = step.get("at").and_then(position);
        let mut events = Vec::new();
        if let Some(point) = at {
            activate_at(app, point);
            events.push(Event::PointerMoved(point));
        }
        events.push(Event::MouseWheel {
            unit: MouseWheelUnit::Point,
            delta: Vec2::new(0., delta as f32),
            modifiers: Modifiers::NONE,
            phase: TouchPhase::Move,
        });
        frame(app, context, size, time, events);
        return Ok(());
    }
    if let Some(name) = step.get("key").and_then(Value::as_str) {
        let key = match name {
            "Escape" | "Esc" => Key::Escape,
            "Enter" | "Return" => Key::Enter,
            "Tab" => Key::Tab,
            "Backspace" => Key::Backspace,
            "Delete" => Key::Delete,
            "ArrowLeft" => Key::ArrowLeft,
            "ArrowRight" => Key::ArrowRight,
            "ArrowUp" => Key::ArrowUp,
            "ArrowDown" => Key::ArrowDown,
            other => return Err(Error::Io(format!("ui replay: unknown key `{other}`"))),
        };
        frame(
            app,
            context,
            size,
            time,
            vec![Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        frame(
            app,
            context,
            size,
            time,
            vec![Event::Key {
                key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Modifiers::NONE,
            }],
        );
        return Ok(());
    }
    if let Some(text) = step.get("text").and_then(Value::as_str) {
        frame(app, context, size, time, vec![Event::Text(text.to_owned())]);
        return Ok(());
    }
    Err(Error::Invalid("ui replay step"))
}

fn position(value: &Value) -> Option<Pos2> {
    let pair = value.as_array()?;
    if pair.len() != 2 {
        return None;
    }
    Some(Pos2::new(
        pair[0].as_f64()? as f32,
        pair[1].as_f64()? as f32,
    ))
}

fn receipt(app: &MatrixApp, context: &Context, step: usize) -> Value {
    let document = app.controller.document();
    let selection: Vec<Value> = app
        .controller
        .selected_ordered()
        .iter()
        .map(|id| {
            json!({
                "id": id.to_string(),
                "label": document.entities.get(id).map(|entity| entity.label.as_str()).unwrap_or(""),
            })
        })
        .collect();
    let cameras: Vec<Value> = app
        .controller
        .view_cameras()
        .into_iter()
        .map(|(title, camera, _)| json!({"view": title, "revision": camera.revision}))
        .collect();
    let views: Vec<Value> = app
        .view_rects
        .iter()
        .map(|(id, rect)| {
            json!({
                "view": id.title(),
                "rect": [rect.min.x, rect.min.y, rect.width(), rect.height()],
            })
        })
        .collect();
    let panels: Vec<Value> = app
        .dock_panel_rects
        .iter()
        .map(|(panel, rect)| {
            json!({
                "panel": panel,
                "rect": [rect.min.x, rect.min.y, rect.width(), rect.height()],
            })
        })
        .collect();
    json!({
        "step": step,
        "focus": context.memory(|memory| memory.focused()).map(|id| format!("{id:?}")),
        "active_view": app.controller.active_view_title(),
        "running_feature": app.controller.running_feature(),
        "prompt": app.controller.prompt_with_options(),
        "selection": selection,
        "undo": app.controller.coordinator().undo_len(),
        "redo": app.controller.coordinator().redo_len(),
        "objects": document.entities.len(),
        "revision": document.revision,
        "views": views,
        "cameras": cameras,
        "panels": panels,
        "navigation": app.navigation_probe(),
        "command": app.transcript.text(),
        "dialog": app.controller.addon_dialog().map(|d| json!({
            "title": d.title,
            "ready": d.ready,
            "rows": d.rows.iter().map(|r| json!({"name": r.name, "label": r.label, "value": r.value})).collect::<Vec<_>>(),
        })),
        "controls": app
            .cmd_token_rects
            .iter()
            .chain(app.session_dialog.rects.iter())
            .map(|(name, rect)| (name.clone(), json!([rect.center().x, rect.center().y])))
            .collect::<serde_json::Map<String, Value>>(),
    })
}

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

/// An offscreen GPU (any adapter, the software one included) for [`screenshot`].
pub(crate) fn offscreen_gpu() -> Result<eframe::egui_wgpu::RenderState> {
    use eframe::{egui_wgpu, wgpu};
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
    .map_err(|e| Error::Io(format!("screenshot: no GPU adapter: {e}")))?;
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::empty(),
        required_limits: adapter.limits(),
        ..Default::default()
    }))
    .map_err(|e| Error::Io(format!("screenshot: GPU device: {e}")))?;
    let format = wgpu::TextureFormat::Rgba8Unorm;
    let renderer = egui_wgpu::Renderer::new(&device, format, Default::default());
    Ok(egui_wgpu::RenderState {
        adapter: adapter.clone(),
        available_adapters: vec![adapter],
        instance,
        device,
        queue,
        target_format: format,
        renderer: std::sync::Arc::new(eframe::egui::mutex::RwLock::new(renderer)),
        surface_config: egui_wgpu::SurfaceConfig::LOW_LATENCY,
    })
}

/// The whole window as the user sees it (menus, viewports, dialogs) after `frames` frames,
/// written as a PNG. Uses the application's GPU state, or an offscreen one.
pub fn screenshot(app: &mut MatrixApp, size: [f32; 2], dpi: f32, frames: usize, out: &std::path::Path) -> Result<()> {
    screenshot_at_pointer(app, size, dpi, frames, None, out)
}

/// Render pointer-dependent previews using the same logical coordinates as the replay.
pub fn screenshot_at_pointer(
    app: &mut MatrixApp, size: [f32; 2], dpi: f32, frames: usize,
    pointer: Option<Pos2>, out: &std::path::Path,
) -> Result<()> {
    use eframe::wgpu;
    if app.gpu_state.is_none() {
        app.gpu_state = Some(offscreen_gpu()?);
    }
    let state = app.gpu_state.clone().expect("gpu state");
    let ppp = dpi.clamp(0.5, 4.0);
    let context = Context::default();
    // A new context: the application sets its fonts and style on it again.
    app.style_ready = false;
    context.set_pixels_per_point(ppp);
    let extent = Vec2::new(size[0], size[1]);
    let mut last = None;
    for _ in 0..frames.max(2) {
        std::thread::sleep(std::time::Duration::from_millis(30));
        let raw = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, extent)),
            events: pointer.map(eframe::egui::Event::PointerMoved).into_iter().collect(),
            ..Default::default()
        };
        let mut output = context.run_ui(raw, |ui| app.show(ui));
        {
            let mut r = state.renderer.write();
            for (id, deltas) in &output.textures_delta.set {
                for delta in deltas.iter() {
                    r.update_texture(&state.device, &state.queue, *id, delta);
                }
            }
        }
        output.textures_delta.clear();
        let _ = state.device.poll(wgpu::PollType::wait_indefinitely());
        last = Some(output);
    }
    let output = last.expect("a frame");
    let (w, h) = ((size[0] * ppp).round() as u32, (size[1] * ppp).round() as u32);
    let pixels = paint_frame_pixels(&context, &state, output, [w,h], ppp, wgpu::Color::BLACK)?;
    image::save_buffer(out, &pixels, w, h, image::ColorType::Rgba8).map_err(|e| Error::Io(e.to_string()))
}

/// Composite a widget draw list into RGBA pixels, including native viewport textures.
pub(crate) fn paint_frame_pixels(context: &Context, state: &eframe::egui_wgpu::RenderState,
    output: eframe::egui::FullOutput, [w,h]: [u32;2], ppp:f32, clear:eframe::wgpu::Color) -> Result<Vec<u8>> {
    use eframe::{egui_wgpu,wgpu};
    if w == 0 || h == 0 || w.max(h) > state.device.limits().max_texture_dimension_2d
        || u64::from(w).div_ceil(64)*256*u64::from(h) > state.device.limits().max_buffer_size {
        return Err(Error::Budget("capture dimensions exceed GPU limits"));
    }
    let prims = context.tessellate(output.shapes, ppp);
    let texture = state.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("om9 screenshot"),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: state.target_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let desc = egui_wgpu::ScreenDescriptor { size_in_pixels: [w, h], pixels_per_point: ppp };
    let mut encoder = state.device.create_command_encoder(&Default::default());
    let extra = state
        .renderer
        .write()
        .update_buffers(&state.device, &state.queue, &mut encoder, &prims, &desc);
    {
        let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("om9 screenshot"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(clear), store: wgpu::StoreOp::Store },
            })],
            ..Default::default()
        });
        let mut pass = pass.forget_lifetime();
        state.renderer.read().render(&mut pass, &prims, &desc);
    }
    let row = w * 4;
    let padded = row.div_ceil(256) * 256;
    let buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("om9 screenshot readback"),
        size: u64::from(padded) * u64::from(h),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(padded), rows_per_image: Some(h) },
        },
        wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
    );
    state.queue.submit(extra.into_iter().chain([encoder.finish()]));
    let (tx, rx) = std::sync::mpsc::channel();
    buffer.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    state.device.poll(wgpu::PollType::wait_indefinitely()).map_err(|e| Error::Io(e.to_string()))?;
    rx.recv()
        .map_err(|e| Error::Io(e.to_string()))?
        .map_err(|e| Error::Io(e.to_string()))?;
    let mapped = buffer
        .slice(..)
        .get_mapped_range()
        .map_err(|e| Error::Io(format!("{e:?}")))?;
    let mut pixels = Vec::with_capacity(row as usize * h as usize);
    for line in mapped.chunks_exact(padded as usize) {
        pixels.extend_from_slice(&line[..row as usize]);
    }
    drop(mapped);
    buffer.unmap();
    Ok(pixels)
}
