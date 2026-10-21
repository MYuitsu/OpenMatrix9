// SPDX-License-Identifier: LGPL-2.1-or-later
//! Native held-audit observations for live resize; inactive in normal sessions.
use super::*;

impl MatrixApp {
    pub(super) fn audit_icon_resize(&self, ctx: &egui::Context) {
        let Some(audit) = self.audit.as_ref().filter(|a| a.hold && a.receipt_written) else {
            return;
        };
        let directory = &audit.directory;
        ctx.input(|input| {
            for event in &input.events {
                if let egui::Event::Screenshot { image, .. } = event {
                    let pixels: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                    let _ = image::save_buffer(
                        directory.join(format!("resize-{}x{}.png", image.size[0], image.size[1])),
                        &pixels,
                        image.size[0] as u32,
                        image.size[1] as u32,
                        image::ColorType::Rgba8,
                    );
                }
            }
        });
        let window = ctx.content_rect().size();
        let path = directory.join(format!("resize-{:.0}x{:.0}.json", window.x, window.y));
        if path.exists() {
            return;
        }
        let slots = |section: SectionId, next: SectionId| {
            let top = self
                .section_rect(section.key())
                .map(|r| r.bottom())
                .unwrap_or(0.);
            let bottom = self.section_rect(next.key()).map(|r| r.top()).unwrap_or(0.);
            self.rail_icon_rects
                .iter()
                .filter(|r| r.top() >= top && r.bottom() <= bottom)
                .map(|r| [r.left(), r.top(), r.width(), r.height()])
                .collect::<Vec<_>>()
        };
        let receipt = serde_json::json!({
            "window_points": [window.x, window.y],
            "pixels_per_point": ctx.pixels_per_point(),
            "screen_icon_scale": self.screen_icon_scale,
            "toolbar_slot_pt": self.toolbar_icon_edge,
            "main": slots(SectionId::MainMenu, SectionId::Display),
            "display": slots(SectionId::Display, SectionId::Snaps),
            "snaps": slots(SectionId::Snaps, SectionId::InfoSettings),
            "info": slots(SectionId::InfoSettings, SectionId::Layers),
            "dock": self.dock_receipt(ctx),
        });
        if let Ok(bytes) = serde_json::to_vec_pretty(&receipt) {
            let _ = std::fs::write(path, bytes);
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(Default::default()));
    }
}
