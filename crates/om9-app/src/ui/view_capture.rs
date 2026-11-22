// SPDX-License-Identifier: LGPL-2.1-or-later
use super::{MatrixApp, paint_axis_triad};
use crate::{
    gpu_viewport::{self, GpuViewport, SharedResourcePool, ViewportRenderRequest},
    viewport_capture::{self, Session, Stage},
};
use eframe::{egui, egui_wgpu, wgpu};
use om9_core::{
    Error, Result,
    budget::memory::MemoryClass,
    transaction::{Cancellation, ExternalMemoryReservation},
};
use om9_scene::camera::Viewport;
use std::{
    sync::{Arc, mpsc},
    time::Duration,
};

pub(super) struct CaptureJob {
    id: uuid::Uuid,
    state: egui_wgpu::RenderState,
    port: GpuViewport,
    pool: SharedResourcePool,
    ctx: egui::Context,
    size: [u32; 2],
    frame: u64,
    dpi: f32,
    reservation: Option<ExternalMemoryReservation>,
    writing: Option<mpsc::Receiver<Result<()>>>,
    cancel: Cancellation,
}
impl Drop for CaptureJob {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
impl CaptureJob {
    fn new(app: &MatrixApp, capture: &Session, dpi: f32) -> Result<Self> {
        let gpu = app
            .gpu_state
            .as_ref()
            .ok_or(Error::Missing("viewport GPU"))?;
        let rect = app
            .view_rects
            .get(&capture.view_id)
            .ok_or(Error::Missing("viewport dimensions"))?;
        let size = capture.settings.size(
            [
                (rect.width() * dpi).round() as u32,
                (rect.height() * dpi).round() as u32,
            ],
            capture.scripted,
        )?;
        let [w, h] = size;
        let limits = gpu.device.limits();
        if w.max(h) > limits.max_texture_dimension_2d
            || u64::from(w).div_ceil(64) * 256 * u64::from(h) > limits.max_buffer_size
        {
            return Err(Error::Budget(
                "capture exceeds GPU texture or transfer limit",
            ));
        }
        let bytes = u64::from(w) * u64::from(h) * 32 + u64::from(h) * 256;
        let reservation = app
            .controller
            .coordinator()
            .reserve_external_memory_with_held(
                MemoryClass::Export,
                bytes,
                om9_native_owner::physical_memory().map(|s| (s.total_bytes, s.available_bytes)),
                app.controller.scene_cache_usage()?,
            )?;
        // The private painter owns its font atlas and viewport texture IDs. The
        // live window keeps its own registrations on the same device and queue.
        let mut state = gpu.clone();
        state.target_format = wgpu::TextureFormat::Rgba8Unorm;
        state.renderer = Arc::new(egui::mutex::RwLock::new(egui_wgpu::Renderer::new(
            &state.device,
            state.target_format,
            Default::default(),
        )));
        let pool = SharedResourcePool::default();
        let mut port = GpuViewport::new(&state, pool.clone());
        let used =
            gpu_viewport::window_target_diagnostics(&app.shared_scene, app.gpu_viewports.values())
                .and_then(|d| {
                    d.unique_bytes
                        .checked_add(gpu_viewport::window_known_buffer_bytes(
                            &app.shared_scene,
                            app.gpu_viewports.values(),
                        )?)
                });
        port.set_window_known_gpu_bytes(used);
        port.set_display_mode(capture.view.mode);
        port.set_grid_visibility(crate::display::GridVisibility {
            lines: capture.settings.grid,
            axes: capture.settings.cplane_axes,
        });
        port.set_transparent_background(capture.settings.transparent);
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(dpi);
        Ok(Self {
            id: capture.id,
            state,
            port,
            pool,
            ctx,
            size,
            dpi,
            frame: 0,
            reservation: Some(reservation),
            writing: None,
            cancel: capture.cancel.clone(),
        })
    }

    fn step(&mut self, app: &MatrixApp, capture: &Session) -> Result<bool> {
        if let Some(rx) = &self.writing {
            return match rx.try_recv() {
                Ok(r) => r.map(|_| true),
                Err(mpsc::TryRecvError::Empty) => Ok(false),
                Err(_) => Err(Error::Io("Viewport image writer stopped".into())),
            };
        }
        let c = &app.controller;
        if capture.stamp != (c.document().id, c.document().revision)
            || capture.selection != c.selection_state().revision
        {
            return Err(Error::Stale(
                "document or selection changed during viewport capture",
            ));
        }
        self.reservation.as_ref().unwrap().recheck_pending()?;
        self.frame += 1;
        self.port.set_render_request(ViewportRenderRequest {
            frame_id: self.frame,
            ..Default::default()
        });
        let [w, h] = self.size;
        let rect = egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(w as f32 / self.dpi, h as f32 / self.dpi),
        );
        let camera = &capture.view.camera;
        let mut output = self.ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(rect),
                ..Default::default()
            },
            |ui| {
                self.port.show(
                    ui,
                    rect,
                    &self.state,
                    c,
                    gpu_viewport::scene_camera(camera),
                    camera.revision,
                    camera,
                    c.scene(),
                    capture.view.cplane,
                    c.selected(),
                    c.selection_state().revision,
                    c.preview_revision(),
                    c.solid_boolean_two_hidden_sources(),
                    false,
                );
                if capture.settings.world_axes && !capture.settings.transparent {
                    paint_axis_triad(
                        ui.painter(),
                        camera,
                        Viewport {
                            origin: [0.; 2],
                            size: [rect.width() as f64, rect.height() as f64],
                        },
                        rect,
                    );
                }
            },
        );
        gpu_viewport::submit_window(&self.pool, &self.state);
        self.port.window_submitted(&self.state.queue);
        for (id, deltas) in &output.textures_delta.set {
            for delta in deltas.iter() {
                self.state.renderer.write().update_texture(
                    &self.state.device,
                    &self.state.queue,
                    *id,
                    delta,
                );
            }
        }
        let free = std::mem::take(&mut output.textures_delta.free);
        output.textures_delta.set.clear();
        if let Some(error) = self.port.preparation_error() {
            return Err(Error::Io(error.to_owned()));
        }
        if !self.port.final_complete() || !c.scene().is_some_and(|s| self.port.displays_scene(s)) {
            for id in free {
                self.state.renderer.write().free_texture(&id);
            }
            return Ok(false);
        }
        let transparent = capture.settings.transparent;
        let clear = if transparent {
            wgpu::Color::TRANSPARENT
        } else {
            wgpu::Color::BLACK
        };
        let pixels = crate::ui_replay::paint_frame_pixels(
            &self.ctx,
            &self.state,
            output,
            self.size,
            self.dpi,
            clear,
        )?;
        for id in free {
            self.state.renderer.write().free_texture(&id);
        }
        let path = capture
            .path
            .clone()
            .ok_or(Error::Missing("viewport image file name"))?;
        let (tx, rx) = mpsc::channel();
        let size = self.size;
        let cancel = self.cancel.clone();
        let held = self.reservation.take().unwrap();
        std::thread::Builder::new()
            .name("om9-viewport-image".into())
            .spawn(move || {
                let _held = held;
                let _ = tx.send(viewport_capture::save(
                    &path,
                    size,
                    pixels,
                    transparent,
                    &cancel,
                ));
            })
            .map_err(|e| Error::Io(e.to_string()))?;
        self.writing = Some(rx);
        Ok(false)
    }
}
impl MatrixApp {
    pub(super) fn service_view_capture_picker(
        &mut self,
        ctx: &egui::Context,
        frame: &eframe::Frame,
    ) {
        let Some(capture) = self
            .controller
            .view_capture
            .as_ref()
            .filter(|s| s.stage == Stage::Dialog)
        else {
            return;
        };
        let mut alpha = capture.settings.transparent;
        let picked = crate::native_picker::pick_view_image(
            frame,
            self.controller.active_file_path(),
            &mut alpha,
        );
        if let Err(error) =
            picked.and_then(|path| self.controller.accept_view_capture_file(path, alpha))
        {
            self.capture_finished(Err(error));
        }
        self.focus_cmd = true;
        ctx.request_repaint();
    }
    pub(super) fn service_view_capture(&mut self, ctx: &egui::Context) {
        if let Some(capture) = self.controller.view_capture.as_mut()
            && capture.scripted
            && let Some(rect) = self.view_rects.get(&capture.view_id)
        {
            let dpi = ctx.pixels_per_point();
            capture
                .settings
                .width
                .get_or_insert((rect.width() * dpi).round().max(1.) as u32);
            capture
                .settings
                .height
                .get_or_insert((rect.height() * dpi).round().max(1.) as u32);
        }
        let Some(capture) = self
            .controller
            .view_capture
            .as_ref()
            .filter(|s| s.stage == Stage::Render)
            .cloned()
        else {
            self.capture_job = None;
            return;
        };
        let result = (|| {
            let mut job = match self.capture_job.take() {
                Some(job) if job.id == capture.id => job,
                _ => CaptureJob::new(self, &capture, ctx.pixels_per_point())?,
            };
            let complete = job.step(self, &capture)?;
            if !complete {
                self.capture_job = Some(job);
            }
            Ok(complete)
        })();
        match result {
            Ok(false) => ctx.request_repaint_after(Duration::from_millis(16)),
            Ok(true) => self.capture_finished(Ok(())),
            Err(e) => self.capture_finished(Err(e)),
        }
    }
    fn capture_finished(&mut self, result: Result<()>) {
        let Some(session) = self.controller.view_capture.take() else {
            return;
        };
        self.capture_job = None;
        match result {
            Ok(()) => {
                self.controller.capture_settings = session.settings;
                let message = format!("Viewport saved to {}", session.path.unwrap().display());
                self.transcript.append_history(&message);
                self.status.info(&message);
            }
            Err(e) => {
                let message = e.to_string();
                self.transcript.append_history(&message);
                self.status.error(&message);
            }
        }
    }
}
