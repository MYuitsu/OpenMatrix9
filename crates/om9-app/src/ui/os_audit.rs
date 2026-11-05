// SPDX-License-Identifier: LGPL-2.1-or-later
//! Opt-in OS-mouse acceptance on the real GPU window (`OM9_GUMBALL_OS_AUDIT=<dir>`).
//!
//! The app creates a line and a snap point, selects the line, and writes a ready receipt with the
//! client-space gumball and snap positions. An external process drives the real OS pointer; the
//! app then records the committed placement and a GPU screenshot, then closes. Nothing here runs
//! unless the environment variable is set.
use super::*;
use om9_core::identity::ObjectId;
use serde_json::json;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

const READY: &str = "ready.json";
const RESULT: &str = "result.json";
const SCREENSHOT: &str = "gumball-os-audit.png";
const TIMEOUT: Duration = Duration::from_secs(30);
const SCREENSHOT_WAIT: Duration = Duration::from_secs(4);

pub(super) struct GumballOsAudit {
    directory: PathBuf,
    frames: u32,
    stable: u32,
    last_size: [f32; 2],
    started: Instant,
    line: ObjectId,
    before: [[f64; 3]; 2],
    ready: bool,
    dragging: bool,
    result: bool,
    screenshot_at: Option<Instant>,
    finished: bool,
}

impl GumballOsAudit {
    pub(super) fn from_env() -> Option<Self> {
        let directory = std::env::var_os("OM9_GUMBALL_OS_AUDIT")?;
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).ok()?;
        Some(Self {
            directory,
            frames: 0,
            stable: 0,
            last_size: [0.0, 0.0],
            started: Instant::now(),
            line: ObjectId(uuid::Uuid::nil()),
            before: [[0.0; 3]; 2],
            ready: false,
            dragging: false,
            result: false,
            screenshot_at: None,
            finished: false,
        })
    }

    fn write(&self, name: &str, value: serde_json::Value) {
        let path = self.directory.join(name);
        let _ = std::fs::write(path, value.to_string());
    }

    fn finish(&mut self, ctx: &egui::Context, status: &str, detail: &str) {
        if self.finished {
            return;
        }
        self.finished = true;
        self.write(
            "done.json",
            json!({"status": status, "detail": detail, "elapsed_ms": self.started.elapsed().as_millis()}),
        );
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

impl MatrixApp {
    /// Build the deterministic acceptance document: a 10 mm line, a point snap 5 mm above its
    /// centre, the line selected, Snappy on. The line geometry stays absolute; the move commits
    /// through its placement.
    pub(super) fn os_audit_setup(&mut self) -> Result<()> {
        self.controller.submit("Line")?;
        self.controller.submit("0,0,0")?;
        self.controller.submit("10,0,0")?;
        let line = self
            .controller
            .document()
            .entities
            .values()
            .find(|entity| entity.label == "Line")
            .map(|entity| entity.id)
            .ok_or(Error::Missing("os audit line"))?;
        self.controller.submit("Point")?;
        self.controller.submit("5,5,0")?;
        self.controller.select_objects(&[line])?;
        self.controller.set_gumball_snappy(true);
        let before = line_endpoints(self.controller.document(), line)?;
        if let Some(audit) = self.os_audit.as_mut() {
            audit.line = line;
            audit.before = before;
        }
        Ok(())
    }

    pub(super) fn os_audit_frame(&mut self, ctx: &egui::Context) {
        let Some(mut audit) = self.os_audit.take() else {
            return;
        };
        if audit.finished {
            return;
        }
        audit.frames = audit.frames.saturating_add(1);
        if audit.started.elapsed() > TIMEOUT {
            audit.finish(ctx, "timeout", "no completed drag before the deadline");
            return;
        }
        if !audit.ready {
            // The window starts maximized/resizes after its first layout; project only
            // after the client size has held still, otherwise the receipt names stale
            // screen positions.
            let size = client_rect(ctx).size();
            let size = [size.x, size.y];
            if size != audit.last_size {
                audit.last_size = size;
                audit.stable = 0;
            } else {
                audit.stable = audit.stable.saturating_add(1);
            }
            if audit.frames < 20 || audit.stable < 12 {
                self.os_audit = Some(audit);
                return;
            }
            if let Err(error) = self.write_ready(ctx, &mut audit) {
                audit.finish(ctx, "error", &error.to_string());
                return;
            }
            self.os_audit = Some(audit);
            return;
        }
        if audit.frames % 10 == 0 {
            audit.write(
                "state.json",
                json!({
                    "frames": audit.frames,
                    "session": self.gumball_session.is_some(),
                    "dragging": audit.dragging,
                    "result": audit.result,
                    "focused": ctx.input(|input| input.focused),
                    "down": ctx.input(|input| input.pointer.any_down()),
                    "hover": self.hover,
                }),
            );
        }
        if self.gumball_session.is_some() {
            audit.dragging = true;
            self.os_audit = Some(audit);
            return;
        }
        if audit.dragging && !audit.result {
            let after = match line_endpoints(self.controller.document(), audit.line) {
                Ok(after) => after,
                Err(error) => {
                    audit.finish(ctx, "error", &error.to_string());
                    return;
                }
            };
            audit.write(
                RESULT,
                json!({
                    "before": audit.before,
                    "after": after,
                    "moved": after != audit.before,
                    "drag_started": audit.dragging,
                }),
            );
            audit.write(
                "state.json",
                json!({
                    "frames": audit.frames,
                    "session": false,
                    "dragging": true,
                    "result": true,
                    "focused": ctx.input(|input| input.focused),
                    "hover": self.hover,
                }),
            );
            audit.result = true;
            audit.screenshot_at = Some(Instant::now());
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
            self.os_audit = Some(audit);
            return;
        }
        if audit.result && !audit.finished {
            if let Some(image) = ctx.input(|input| {
                input.events.iter().find_map(|event| match event {
                    egui::Event::Screenshot { image, .. } => Some(image.clone()),
                    _ => None,
                })
            }) {
                let pixels: Vec<u8> = image
                    .pixels
                    .iter()
                    .flat_map(|color| color.to_array())
                    .collect();
                let path = audit.directory.join(SCREENSHOT);
                let _ = image::save_buffer(
                    &path,
                    &pixels,
                    image.size[0] as u32,
                    image.size[1] as u32,
                    image::ColorType::Rgba8,
                );
                audit.finish(ctx, "ok", "drag committed and screenshot saved");
                return;
            }
            if audit
                .screenshot_at
                .is_some_and(|at| at.elapsed() > SCREENSHOT_WAIT)
            {
                audit.finish(ctx, "ok", "drag committed; screenshot event not received");
                return;
            }
        }
        self.os_audit = Some(audit);
    }

    fn write_ready(&mut self, ctx: &egui::Context, audit: &mut GumballOsAudit) -> Result<()> {
        let view = self.controller.workspace.active;
        let rect = self
            .view_rects
            .get(&view)
            .copied()
            .ok_or(Error::Missing("os audit viewport"))?;
        let camera = &self.controller.workspace.views[&view].camera;
        let viewport = om9_scene::camera::Viewport {
            origin: [rect.min.x as f64, rect.min.y as f64],
            size: [rect.width() as f64, rect.height() as f64],
        };
        let pivot = self
            .controller
            .gumball_pivot()
            .ok_or(Error::Missing("os audit gumball"))?
            .origin;
        let project = |point: [f64; 3]| -> Result<[f64; 2]> {
            camera
                .project(point, viewport)
                .ok()
                .flatten()
                .map(|projected| projected.screen)
                .ok_or(Error::Invalid("os audit projection"))
        };
        let pivot_screen = project(pivot)?;
        let target_screen = project([5.0, 5.0, 0.0])?;
        let client = {
            let rect = client_rect(ctx);
            [rect.width(), rect.height()]
        };
        audit.write(
            READY,
            json!({
                "title": self.window_title.clone(),
                "pixels_per_point": ctx.pixels_per_point(),
                "view": view.title(),
                "pivot_world": pivot,
                "target_world": [5.0, 5.0, 0.0],
                "pivot_client": pivot_screen,
                "target_client": target_screen,
                "client": client,
            }),
        );
        audit.ready = true;
        Ok(())
    }
}

fn client_rect(ctx: &egui::Context) -> egui::Rect {
    ctx.input(|input| input.viewport().inner_rect)
        .unwrap_or_else(|| {
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(1600.0, 1000.0))
        })
}

fn line_endpoints(document: &om9_core::model::Document, line: ObjectId) -> Result<[[f64; 3]; 2]> {
    let entity = document
        .entities
        .get(&line)
        .ok_or(Error::Missing("os audit line entity"))?;
    let om9_core::geometry::Geometry::Curve(om9_core::geometry::Curve::Line { start, end, .. }) =
        &entity.geometry
    else {
        return Err(Error::Invalid("os audit line geometry"));
    };
    Ok([
        entity.placement.point(*start)?,
        entity.placement.point(*end)?,
    ])
}
