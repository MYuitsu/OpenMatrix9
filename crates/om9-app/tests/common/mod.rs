// SPDX-License-Identifier: LGPL-2.1-or-later
//! Shared helpers for the docking tests (plan task OM9-L3-01-01-002).
//!
//! One place decides which section a test puts in the right dock
//! ([`RIGHT_DOCK_FIXTURE`], a neutral section that is never Layers) and what
//! "Layers is where Matrix9 puts it" means.
#![allow(dead_code)]

use eframe::egui::{self, Pos2, Rect, Vec2};
use om9_app::dock_layout::{DockSide, RIGHT_DOCK_FIXTURE, SectionId};
use om9_app::ui::MatrixApp;

/// The documented left column, top to bottom, with the reserved Builder area
/// directly below Projects.
pub const DOCUMENTED_LEFT: [SectionId; 8] = [
    SectionId::IconHistory,
    SectionId::MainMenu,
    SectionId::Display,
    SectionId::Snaps,
    SectionId::InfoSettings,
    SectionId::Layers,
    SectionId::Projects,
    SectionId::BuilderSlot,
];

/// Put the shared neutral fixture section into the right dock, the way a user
/// who drags a panel there would. Returns what `dock_menu_section` returns.
pub fn dock_right_fixture(app: &mut MatrixApp) -> bool {
    app.dock_menu_section(RIGHT_DOCK_FIXTURE, DockSide::Right)
}

/// Run one frame of `app.show` on a `size` point screen at 1.0 scale.
pub fn run_frame(app: &mut MatrixApp, context: &egui::Context, size: Vec2) {
    let mut output = context.run_ui(
        egui::RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, size)),
            ..Default::default()
        },
        |ui| app.show(ui),
    );
    output.textures_delta.clear();
}

/// Run `frames` frames at `size`.
pub fn settle(app: &mut MatrixApp, context: &egui::Context, size: Vec2, frames: usize) {
    for _ in 0..frames {
        run_frame(app, context, size);
    }
}

/// Layers is in the left column, directly below Info & Settings and above
/// Projects, with its title bar inside the sidebar rectangle.
pub fn assert_layers_between_info_and_projects(app: &MatrixApp, label: &str) {
    let info = app
        .section_rect(SectionId::InfoSettings.key())
        .unwrap_or_else(|| panic!("{label}: info-settings title bar missing"));
    let layers = app
        .section_rect(SectionId::Layers.key())
        .unwrap_or_else(|| panic!("{label}: layers title bar missing"));
    let projects = app
        .section_rect(SectionId::Projects.key())
        .unwrap_or_else(|| panic!("{label}: projects title bar missing"));
    assert!(
        info.max.y <= layers.min.y + 0.5,
        "{label}: Layers {layers:?} is not below Info & Settings {info:?}"
    );
    assert!(
        layers.max.y <= projects.min.y + 0.5,
        "{label}: Layers {layers:?} is not above Projects {projects:?}"
    );
    let sidebar = app
        .sidebar_rect
        .unwrap_or_else(|| panic!("{label}: sidebar rect missing"));
    assert!(
        layers.min.x >= sidebar.min.x - 0.5 && layers.max.x <= sidebar.max.x + 0.5,
        "{label}: Layers {layers:?} is outside the sidebar columns {sidebar:?}"
    );
    assert!(
        layers.min.y >= sidebar.min.y - 0.5 && layers.max.y <= sidebar.max.y + 0.5,
        "{label}: Layers {layers:?} is clipped by the sidebar {sidebar:?}"
    );
    assert!(
        app.dock.left_sections().contains(&SectionId::Layers),
        "{label}: Layers is not in the left column"
    );
    assert!(
        !app.dock.right_sections().contains(&SectionId::Layers),
        "{label}: Layers is in the right dock"
    );
    assert!(
        !app.dock.floating_sections().contains(&SectionId::Layers),
        "{label}: Layers floats"
    );
}
