// SPDX-License-Identifier: LGPL-2.1-or-later
//! Floating windows of the halo/cluster/baguette builders (`OM9-L2-04-11`).
//!
//! The window is a view of the session only: every control edits the preview and the single
//! Apply button commits one transaction through `Controller::gem_cluster_apply`. Cancel drops
//! the session, so no phantom gem ever exists.
use eframe::egui;

use super::MatrixApp;
use crate::session::gem_cluster::{ClusterTool, LayoutKind};
use om9_core::gem_cluster::ClusterStart;
use om9_scene::camera::Viewport;

impl MatrixApp {
    /// A live wireframe of every preview accent, painted after the committed scene.
    pub(super) fn gem_cluster_preview_ui(
        &self,
        ui: &mut egui::Ui,
        view: &om9_scene::workspace::ViewState,
        viewport: Viewport,
        rect: egui::Rect,
    ) {
        let preview = self.controller.gem_cluster_world_preview();
        if preview.is_empty() {
            return;
        }
        let painter = ui.painter().with_clip_rect(rect);
        let color = egui::Color32::from_rgb(255, 205, 105);
        for (mesh, placement) in &preview {
            for [a, b] in
                super::gem::gem_preview_segments(&mesh.mesh, placement, &view.camera, viewport)
            {
                painter.line_segment([a, b], egui::Stroke::new(1.1, color));
            }
            if let Ok(world) = placement.point([0.0, 0.0, 0.0])
                && let Ok(Some(point)) = view.camera.project(world, viewport)
            {
                painter.circle_filled(
                    egui::pos2(point.screen[0] as f32, point.screen[1] as f32),
                    1.6,
                    color,
                );
            }
        }
    }

    /// The builder window. Apply writes back the edited session first, so the transaction uses
    /// exactly the parameters the window shows.
    pub(super) fn gem_cluster_window_ui(&mut self, ctx: &egui::Context) {
        let Some(session) = self.controller.gem_cluster_session().cloned() else {
            return;
        };
        let mut session = session;
        let mut open = true;
        let mut apply = false;
        let mut cancel = false;
        let mut dirty = false;
        egui::Window::new(session.tool.title())
            .id(egui::Id::new(("om9-gem-cluster", session.tool.command())))
            .open(&mut open)
            .default_width(380.0)
            .resizable(false)
            .show(ctx, |ui| {
                match &mut session.kind {
                    LayoutKind::Halo(params) => {
                        dirty |= halo_ui(ui, params, session.tool);
                    }
                    LayoutKind::Cluster(params) => {
                        dirty |= cluster_ui(ui, params, session.tool);
                    }
                    LayoutKind::Channel(params) => {
                        dirty |= channel_ui(ui, params, session.tool);
                    }
                }
                ui.separator();
                ui.label(session.summary());
                if session.edit.is_some() {
                    ui.label("Apply edits the existing layout and keeps the child GemIds.");
                }
                ui.horizontal(|ui| {
                    if ui.button("Apply").clicked() {
                        apply = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if !open && !apply {
            cancel = true;
        }
        if cancel {
            self.controller.gem_cluster_cancel();
            self.status.info("Gem layout canceled");
            return;
        }
        if dirty && let Err(error) = self.controller.gem_cluster_set(session.clone()) {
            self.status.error(&error.to_string());
        }
        if apply {
            if let Err(error) = self.controller.gem_cluster_set(session) {
                self.status.error(&error.to_string());
                return;
            }
            match self.controller.gem_cluster_apply() {
                Ok(()) => {
                    self.status.info("Gem layout committed");
                }
                Err(error) => {
                    self.status.error(&error.to_string());
                }
            }
        }
    }
}

fn accent_ui(ui: &mut egui::Ui, accent: &mut om9_core::gem::GemSpec) -> bool {
    let mut dirty = false;
    let mut length = accent.length_mm;
    let mut width = accent.width_mm;
    let mut depth = accent.depth_mm();
    let mut girdle = accent.girdle_mm;
    ui.horizontal(|ui| {
        ui.label("Accent L");
        if ui
            .add(egui::DragValue::new(&mut length).speed(0.05).suffix(" mm"))
            .changed()
        {
            dirty = true;
        }
        ui.label("W");
        if ui
            .add(egui::DragValue::new(&mut width).speed(0.05).suffix(" mm"))
            .changed()
        {
            dirty = true;
        }
        ui.label("Depth");
        if ui
            .add(egui::DragValue::new(&mut depth).speed(0.05).suffix(" mm"))
            .changed()
        {
            dirty = true;
        }
        ui.label("Girdle");
        if ui
            .add(egui::DragValue::new(&mut girdle).speed(0.01).suffix(" mm"))
            .changed()
        {
            dirty = true;
        }
    });
    if dirty {
        let length = length.max(0.3);
        let width = width.clamp(0.2, length);
        if let Ok(spec) = accent.clone().set_girdle_mm(length, width)
            && let Ok(spec) = spec.set_girdle_mm_thickness(girdle.max(0.01))
            && let Ok(spec) = spec.set_depth_mm(depth.max(girdle + 0.05))
        {
            *accent = spec;
        }
    }
    dirty
}

fn halo_ui(
    ui: &mut egui::Ui,
    params: &mut om9_core::gem_cluster::HaloParams,
    tool: ClusterTool,
) -> bool {
    let mut dirty = accent_ui(ui, &mut params.accent);
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.spacing_mm, 0.0..=3.0)
                .text("Spacing")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.spacing_from_center_mm, 0.0..=5.0)
                .text("Spacing from center")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.angle_deg, -60.0..=60.0)
                .text("Angle")
                .suffix("°"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.rotation_deg, -180.0..=180.0)
                .text("Rotation")
                .suffix("°"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.height_offset_mm, -3.0..=3.0)
                .text("Gems height offset")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.placement_mm, -3.0..=3.0)
                .text("Placement")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.inner_wall_mm, 0.0..=2.0)
                .text("Inner wall")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.outer_wall_mm, 0.0..=2.0)
                .text("Outer wall")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.profile_width_mm, 0.0..=4.0)
                .text("Profile width")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.profile_height_mm, 0.0..=4.0)
                .text("Profile height")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.channel_spacing_mm, 0.0..=2.0)
                .text("Channel spacing")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.channel_wall_height_mm, 0.0..=4.0)
                .text("Channel wall height")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.channel_blend_mm, 0.0..=2.0)
                .text("Channel blend")
                .suffix(" mm"),
        )
        .changed();
    ui.collapsing("Prongs (metal owner)", |ui| {
        dirty |= ui
            .add(
                egui::Slider::new(&mut params.prong_height_offset_mm, 0.0..=3.0)
                    .text("Height offset")
                    .suffix(" mm"),
            )
            .changed();
        dirty |= ui
            .add(egui::Slider::new(&mut params.prong_dome, 0.0..=1.0).text("Dome amount"))
            .changed();
        dirty |= ui
            .add(
                egui::Slider::new(&mut params.edge_nudge_mm, 0.0..=2.0)
                    .text("Edge nudge")
                    .suffix(" mm"),
            )
            .changed();
        dirty |= ui
            .add(
                egui::Slider::new(&mut params.gem_nudge_mm, 0.0..=2.0)
                    .text("Gem nudge")
                    .suffix(" mm"),
            )
            .changed();
        dirty |= ui
            .add(egui::Slider::new(&mut params.prong_taper, -1.0..=1.0).text("Prong taper"))
            .changed();
        dirty |= ui
            .add(
                egui::Slider::new(&mut params.prong_vertical_mm, -3.0..=3.0)
                    .text("Prong vertical")
                    .suffix(" mm"),
            )
            .changed();
        if ui
            .checkbox(&mut params.prongs_meshed, "Prong meshed")
            .changed()
        {
            dirty = true;
        }
    });
    ui.horizontal(|ui| {
        if ui.checkbox(&mut params.even, "Even").changed() {
            dirty = true;
        }
        if ui
            .checkbox(&mut params.fix_position, "Fix position")
            .changed()
        {
            dirty = true;
        }
        if ui
            .checkbox(&mut params.match_shape, "Match shape")
            .changed()
        {
            dirty = true;
        }
    });
    ui.horizontal(|ui| {
        ui.label("Count");
        let mut explicit = params.count.is_some();
        if ui.checkbox(&mut explicit, "explicit").changed() {
            params.count = explicit.then_some(12);
            dirty = true;
        }
        if let Some(count) = &mut params.count
            && ui.add(egui::DragValue::new(count).range(3..=120)).changed()
        {
            dirty = true;
        }
    });
    ui.label(format!("{} loaded accents", tool.title()));
    dirty
}

fn cluster_ui(
    ui: &mut egui::Ui,
    params: &mut om9_core::gem_cluster::ClusterParams,
    _tool: ClusterTool,
) -> bool {
    let mut dirty = accent_ui(ui, &mut params.accent);
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.xy_offset_mm, 0.0..=5.0)
                .text("XY offset")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.z_offset_mm, -3.0..=3.0)
                .text("Z offset")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.degree_deg, -180.0..=180.0)
                .text("Degree")
                .suffix("°"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.ring_spacing_mm, 0.0..=3.0)
                .text("Ring spacing")
                .suffix(" mm"),
        )
        .changed();
    ui.horizontal(|ui| {
        ui.label("Start");
        for (label, start) in [
            ("Bottom", ClusterStart::Bottom),
            ("Top", ClusterStart::Top),
            ("Left", ClusterStart::Left),
            ("Right", ClusterStart::Right),
        ] {
            if ui.selectable_label(params.start == start, label).clicked() {
                params.start = start;
                dirty = true;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Rings");
        if ui
            .add(egui::DragValue::new(&mut params.rings).range(1..=2))
            .changed()
        {
            dirty = true;
        }
    });
    ui.horizontal(|ui| {
        ui.label("Count");
        let mut explicit = params.count.is_some();
        if ui.checkbox(&mut explicit, "explicit").changed() {
            params.count = explicit.then_some(12);
            dirty = true;
        }
        if let Some(count) = &mut params.count
            && ui.add(egui::DragValue::new(count).range(3..=120)).changed()
        {
            dirty = true;
        }
    });
    dirty
}

fn channel_ui(
    ui: &mut egui::Ui,
    params: &mut om9_core::gem_cluster::ChannelParams,
    tool: ClusterTool,
) -> bool {
    let mut dirty = accent_ui(ui, &mut params.section);
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.spacing_mm, 0.0..=2.0)
                .text("Spacing")
                .suffix(" mm"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.taper, 0.05..=1.0)
                .text("Taper")
                .suffix(" ×"),
        )
        .changed();
    dirty |= ui
        .add(
            egui::Slider::new(&mut params.banking_deg, 0.0..=90.0)
                .text("Banking")
                .suffix("°"),
        )
        .changed();
    if tool == ClusterTool::Tapered {
        dirty |= ui
            .add(egui::Slider::new(&mut params.point_a, 0.0..=1.0).text("Point A"))
            .changed();
        dirty |= ui
            .add(egui::Slider::new(&mut params.point_b, 0.0..=1.0).text("Point B"))
            .changed();
    }
    ui.horizontal(|ui| {
        if ui.checkbox(&mut params.flip, "Flip").changed() {
            dirty = true;
        }
        if ui.checkbox(&mut params.middle, "Middle").changed() {
            dirty = true;
        }
        if ui.checkbox(&mut params.straight, "Straight").changed() {
            dirty = true;
        }
    });
    ui.horizontal(|ui| {
        if ui.checkbox(&mut params.gem_top, "Gem top").changed() {
            dirty = true;
        }
        if ui
            .checkbox(&mut params.auto_spacing, "Auto spacing")
            .changed()
        {
            dirty = true;
        }
    });
    dirty
}
