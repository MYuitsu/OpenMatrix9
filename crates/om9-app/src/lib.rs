// SPDX-License-Identifier: LGPL-2.1-or-later
#![forbid(unsafe_code)]
mod brep_bridge;
pub mod command_events;
pub mod curve_menu;
pub mod display;
pub mod dock_layout;
pub mod f6;
pub mod file_lifecycle;
pub mod gpu_viewport;
pub mod icons;
pub mod interaction_log;
pub mod layer_command;
pub mod layer_dialog;
pub mod menu_catalog;
mod native_picker;
pub mod option_parser;
pub mod panel_ops;
pub mod pictureframe;
pub mod project_manager;
pub mod report;
pub mod session;
pub mod solid_menu;
pub mod startup;
pub mod status_surface;
pub mod superselect;
pub mod surface_menu;
pub mod three_dm_host;
pub mod transcript;
pub mod ui;
pub mod utilities;
pub mod viewport_menu;
mod window_identity_guard;
pub mod workbench_regions;

/// Admission limits for a document the application opens.
///
/// The default is the product budget. An explicit stress run raises the ceiling through
/// `OM9_MAX_OBJECTS`, `OM9_MAX_BYTES` or `OM9_MAX_POINTS`; the values are clamped so a typo cannot ask
/// for an unbounded document, and the default behaviour is unchanged when none is set.
pub fn document_limits() -> om9_core::budget::Limits {
    fn env_u64(name: &str) -> Option<u64> {
        std::env::var(name).ok()?.trim().parse::<u64>().ok()
    }
    let mut limits = om9_core::budget::Limits::default();
    // Named stress level: 500k objects and the matching point budget. Individual variables still win.
    if std::env::var("OM9_STRESS_LEVEL").is_ok_and(|value| value.eq_ignore_ascii_case("500k")) {
        limits.max_objects = 500_000;
        limits.max_points = 500_000_000;
    }
    if let Some(value) = env_u64("OM9_MAX_OBJECTS") {
        limits.max_objects = value.clamp(1, 2_000_000);
    }
    if let Some(value) = env_u64("OM9_MAX_POINTS") {
        limits.max_points = value.clamp(1, 500_000_000);
    }
    if let Some(value) = env_u64("OM9_MAX_BYTES") {
        limits.max_bytes = value.clamp(1 << 20, 64 << 30);
    }
    limits
}

pub mod camera_rig;
pub mod frame_scheduler;
pub mod navigation;
mod navigation_audit;
