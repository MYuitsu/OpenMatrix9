// SPDX-License-Identifier: LGPL-2.1-or-later
#![forbid(unsafe_code)]
pub mod addons;
mod brep_bridge;
pub mod command_events;
pub mod command_options;
pub mod curve_menu;
pub mod display;
pub mod dock_layout;
pub mod f6;
pub mod file_lifecycle;
pub mod gpu_viewport;
pub mod gpu_budget;
pub mod headless;
pub mod icons;
pub mod interaction_log;
pub mod layer_command;
pub mod layer_dialog;
pub mod mcp;
pub mod menu_catalog;
mod native_picker;
pub use om9_command_options as option_parser;
pub mod panel_ops;
pub mod pictureframe;
pub mod polyline_options;
pub mod project_manager;
pub mod report;
pub mod session;
pub mod solid_menu;
pub mod startup;
pub mod status_surface;
pub mod superselect;
pub mod surface_menu;
pub mod three_dm_host;
pub mod transform_menu;
pub mod transcript;
pub mod ui;
pub mod ui_replay;
pub mod utilities;
pub mod viewport_menu;
mod window_identity_guard;
pub mod workbench_regions;

/// Admission limits for an interactive document. Structural limits remain
/// independent of the hardware-derived managed byte ceiling.
pub fn document_limits() -> om9_core::budget::Limits {
    document_limits_with_memory_status().0
}

pub(crate) fn document_limits_with_memory_status() -> (om9_core::budget::Limits, bool) {
    fn env_u64(name: &str) -> Option<u64> {
        std::env::var(name).ok()?.trim().parse::<u64>().ok()
    }
    let total = om9_native_owner::physical_memory_bytes().filter(|&bytes| bytes > 0);
    let mut limits = document_limits_for_memory(total, env_u64("OM9_MAX_BYTES"));
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
    (limits, total.is_some())
}

fn document_limits_for_memory(total: Option<u64>, requested_bytes: Option<u64>) -> om9_core::budget::Limits {
    let mut limits = om9_core::budget::Limits::default();
    if let Some(total) = total.filter(|&bytes| bytes > 0) {
        limits.max_bytes = (u128::from(total) * 3 / 5) as u64;
    }
    if let Some(value) = requested_bytes {
        limits.max_bytes = limits.max_bytes.min(value.clamp(1 << 20, 64 << 30));
    }
    limits
}

#[cfg(test)]
mod interactive_limit_tests {
    use super::document_limits_for_memory;
    const MIB: u64 = 1024 * 1024;

    #[test]
    fn managed_limit_uses_physical_ram_with_conservative_fallback_and_lower_override() {
        let limits = document_limits_for_memory(Some(8 * 1024 * MIB), None);
        assert_eq!(limits.max_bytes, 8 * 1024 * MIB * 3 / 5);
        assert_eq!(limits.max_points, om9_core::budget::Limits::default().max_points);
        assert_eq!(document_limits_for_memory(None, None).max_bytes, 256 * MIB);
        assert_eq!(document_limits_for_memory(Some(512 * MIB), None).max_bytes, 512 * MIB * 3 / 5);
        assert_eq!(document_limits_for_memory(Some(8 * 1024 * MIB), Some(512 * MIB)).max_bytes, 512 * MIB);
        assert_eq!(document_limits_for_memory(Some(512 * MIB), Some(1024 * MIB)).max_bytes, 512 * MIB * 3 / 5);
        assert_eq!(document_limits_for_memory(None, Some(1024 * MIB)).max_bytes, 256 * MIB);
    }
}

pub mod camera_rig;
pub mod frame_scheduler;
pub mod navigation;
mod navigation_audit;
