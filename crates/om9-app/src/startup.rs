// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed startup failure surface for the standalone shell.
//!
//! Owner: plan module `OM9-L2-01-01` L3-001. A desktop start that cannot restore
//! the document, create the window or obtain a GPU adapter must produce an
//! explicit diagnostic instead of a silent exit. The failure is Rust-owned data:
//! it is bounded, serialized as a receipt next to the application layout and, on
//! Windows, optionally shown through the native owner.
use crate::workbench_regions::{PLAN_FLOOR_WINDOW, SUPPORTED_MIN_WINDOW};
use om9_core::{Error, Result};
use std::path::{Path, PathBuf};

/// Part of the monitor that the window frame, title bar and taskbar can take,
/// in logical points: `[16, 80]` = 16 for the frame width, 80 = 48 taskbar +
/// 32 title bar. Conservative constants, not measured: the real work area needs
/// a Win32 call (unsafe or FFI), which this safe-Rust crate does not make.
/// Change this one constant to reserve the taskbar only.
pub const SHELL_RESERVE_PT: [f32; 2] = [16., 80.];

/// Effective window minimum in logical points for a screen of `monitor_logical`
/// points (egui `ViewportInfo::monitor_size`, already divided by the zoom and
/// native scale).
///
/// Per axis: `max(floor, min(1280x720, monitor - SHELL_RESERVE_PT))`. A screen
/// that is large enough keeps the supported minimum; a smaller one (for example
/// 1366x768 at 125% or 150% DPI) caps the minimum to the usable area instead
/// of forcing the window off-screen. An unknown, non-finite or non-positive
/// monitor gives the plain supported minimum. The result is always finite and
/// within `[floor, supported minimum]` on both axes.
pub fn effective_min_size(monitor_logical: Option<[f32; 2]>) -> [f32; 2] {
    let nominal = [SUPPORTED_MIN_WINDOW.width, SUPPORTED_MIN_WINDOW.height];
    let floor = [PLAN_FLOOR_WINDOW.width, PLAN_FLOOR_WINDOW.height];
    let Some(monitor) = monitor_logical else {
        return nominal;
    };
    if !monitor.iter().all(|value| value.is_finite() && *value > 0.) {
        return nominal;
    }
    let capped =
        |axis: usize| (monitor[axis] - SHELL_RESERVE_PT[axis]).clamp(floor[axis], nominal[axis]);
    [capped(0), capped(1)]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupStage {
    /// Document restore or archive admission failed before any window existed.
    Document,
    /// Window creation or the graphics device/adapter failed.
    Renderer,
    /// A native owner call (dialog, process containment) failed at start.
    Native,
}

impl StartupStage {
    pub fn label(self) -> &'static str {
        match self {
            StartupStage::Document => "document",
            StartupStage::Renderer => "renderer",
            StartupStage::Native => "native",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupFailure {
    pub stage: StartupStage,
    detail: String,
}

/// Typed startup input owned by this module. The process bootstrap takes exactly
/// this record; nothing else decides the window title, size or error policy.
///
/// Adapted from the pinned QymCAD bootstrap (`crates/qymcad/src/gui/startup.rs`
/// at 746a17f1: `launch()`, `NativeOptions`, `ViewportBuilder::with_title`,
/// `with_inner_size`, `run_native`), rewritten with OM9 names and safe Rust
/// ownership. See `implementation/design/M01/OM9-L3-01-01-001/source_crosswalk.json`.
#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    /// Product name shown in the title bar, e.g. `OpenMatrix9`.
    pub app_title: String,
    /// Name of the empty document created when no file is opened.
    pub document_name: String,
    /// Initial window size in logical pixels.
    pub window: [f32; 2],
    /// Nominal supported minimum window size in logical points (1280x720). A
    /// start below it is refused. The running window may lower it to the usable
    /// screen on screens smaller than that in logical points; see
    /// [`effective_min_size`].
    pub min_window: [f32; 2],
    /// Open maximized (the interactive default; audits keep the exact size).
    pub maximized: bool,
    /// Own-app audit directory; `None` for a normal interactive start.
    pub audit_dir: Option<PathBuf>,
    /// Application-scoped layout store; `None` keeps the layout in memory only.
    pub layout_path: Option<PathBuf>,
    /// Whether a failed start may show the native diagnostic dialog.
    pub startup_dialog: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            app_title: "OpenMatrix9".into(),
            document_name: "Untitled".into(),
            window: [1600., 1000.],
            min_window: [SUPPORTED_MIN_WINDOW.width, SUPPORTED_MIN_WINDOW.height],
            maximized: true,
            audit_dir: None,
            layout_path: None,
            startup_dialog: true,
        }
    }
}

impl AppConfig {
    /// Config for one interactive start of an empty document.
    pub fn interactive(document_name: &str, audit_dir: Option<PathBuf>) -> Self {
        let audit = audit_dir.is_some();
        Self {
            document_name: document_name.into(),
            maximized: !audit,
            audit_dir,
            layout_path: crate::dock_layout::DockLayout::default_store_path(),
            startup_dialog: should_report_dialog(
                std::env::var(StartupFailure::DIALOG_ENV).ok().as_deref(),
            ),
            ..Self::default()
        }
    }

    pub fn window_title(&self) -> &str {
        &self.app_title
    }

    /// Minimum inner size handed to the window builder. The monitor is not
    /// known when the builder runs, so it gets the safe planner floor; the
    /// running app then sends [`effective_min_size`] on the first frame.
    pub fn builder_min_window(&self) -> [f32; 2] {
        [PLAN_FLOOR_WINDOW.width, PLAN_FLOOR_WINDOW.height]
    }

    /// Reject a configuration that cannot open a usable window before any
    /// window, device or document is created.
    pub fn validate(&self) -> std::result::Result<(), StartupFailure> {
        let finite = |size: [f32; 2]| size.into_iter().all(f32::is_finite);
        if !finite(self.window) || !finite(self.min_window) {
            return Err(StartupFailure::new(
                StartupStage::Renderer,
                "window size must be finite",
            ));
        }
        if self.window[0] < self.min_window[0] || self.window[1] < self.min_window[1] {
            return Err(StartupFailure::new(
                StartupStage::Renderer,
                "window size is below the minimum window size",
            ));
        }
        if self.app_title.trim().is_empty() {
            return Err(StartupFailure::new(
                StartupStage::Renderer,
                "the window title must not be empty",
            ));
        }
        Ok(())
    }
}

impl StartupFailure {
    pub const MAX_DETAIL_BYTES: usize = 512;
    pub const RECEIPT_FILE: &'static str = "startup-failure.json";
    pub const DIALOG_ENV: &'static str = "OM9_STARTUP_DIALOG";

    pub fn new(stage: StartupStage, detail: &str) -> Self {
        Self {
            stage,
            detail: bounded(detail),
        }
    }

    pub fn from_error(stage: StartupStage, error: &Error) -> Self {
        Self::new(stage, &error.to_string())
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }

    /// One user-visible line, e.g. `OpenMatrix9 could not start (renderer): I/O: ...`.
    pub fn line(&self) -> String {
        format!(
            "OpenMatrix9 could not start ({}): {}",
            self.stage.label(),
            self.detail
        )
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&serde_json::json!({
            "schema": "om9.startup.failure.v1",
            "stage": self.stage.label(),
            "detail": self.detail,
        }))
        .map_err(|_| Error::Invalid("startup failure serialization"))
    }

    /// Write the bounded receipt into the application scope. A stale receipt from
    /// an earlier failure is replaced so it cannot be mistaken for the current
    /// start; the previous bytes are never left partially overwritten.
    pub fn write_receipt(&self, directory: &Path) -> Result<PathBuf> {
        let bytes = self.to_json()?;
        std::fs::create_dir_all(directory).map_err(|error| Error::Io(error.to_string()))?;
        let path = directory.join(Self::RECEIPT_FILE);
        let staging = directory.join(format!(
            ".{}.{}.tmp",
            Self::RECEIPT_FILE,
            uuid::Uuid::new_v4()
        ));
        std::fs::write(&staging, bytes.as_bytes()).map_err(|error| Error::Io(error.to_string()))?;
        if std::fs::rename(&staging, &path).is_err() {
            let copied = std::fs::copy(&staging, &path);
            let _ = std::fs::remove_file(&staging);
            copied.map_err(|error| Error::Io(error.to_string()))?;
        }
        Ok(path)
    }

    /// Write the receipt into the directory owning the application layout, when
    /// one is defined. Returns the receipt path that was written.
    pub fn report(&self) -> Result<Option<PathBuf>> {
        let Some(layout) = crate::dock_layout::DockLayout::default_store_path() else {
            return Ok(None);
        };
        let directory = layout
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        self.write_receipt(&directory).map(Some)
    }
}

/// A desktop viewport requires a hardware GPU. Preserve its actual identity in receipts.
pub fn gpu_adapter_identity(adapter: Option<(&str, eframe::wgpu::DeviceType)>) -> Result<String> {
    use eframe::wgpu::DeviceType;
    match adapter {
        Some((
            info,
            DeviceType::DiscreteGpu | DeviceType::IntegratedGpu | DeviceType::VirtualGpu,
        )) if !info.trim().is_empty() => Ok(info.to_owned()),
        _ => Err(Error::Unsupported(
            "hardware GPU adapter required; CPU/software rendering is disabled",
        )),
    }
}

/// Whether the process should show the native startup dialog. Automation and
/// audits opt out with `OM9_STARTUP_DIALOG=0`; anything else keeps the default
/// user-visible behavior for a console-less executable.
pub fn should_report_dialog(value: Option<&str>) -> bool {
    !matches!(value.map(str::trim), Some("0") | Some("false"))
}

fn bounded(detail: &str) -> String {
    let mut bounded = String::with_capacity(detail.len().min(StartupFailure::MAX_DETAIL_BYTES));
    for ch in detail.chars() {
        if bounded.len() + ch.len_utf8() > StartupFailure::MAX_DETAIL_BYTES {
            break;
        }
        if ch == '\n' || ch == '\r' || ch == '\t' {
            bounded.push(' ');
        } else {
            bounded.push(ch);
        }
    }
    if bounded.trim().is_empty() {
        "unknown startup failure".into()
    } else {
        bounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn failure_line_names_the_stage_and_keeps_the_detail() {
        let failure = StartupFailure::from_error(
            StartupStage::Renderer,
            &Error::Io("wgpu: no adapter".into()),
        );
        assert_eq!(failure.stage, StartupStage::Renderer);
        assert!(failure.line().contains("renderer"));
        assert!(failure.line().contains("wgpu: no adapter"));
    }

    #[test]
    fn detail_is_bounded_on_char_boundaries_and_newlines_are_folded() {
        let failure = StartupFailure::new(StartupStage::Document, &"ẩ".repeat(1000));
        assert!(failure.detail().len() <= StartupFailure::MAX_DETAIL_BYTES);
        assert!(failure.detail().chars().all(|ch| ch == 'ẩ'));
        let folded = StartupFailure::new(StartupStage::Native, "first\nsecond\r\nthird\ttab");
        assert_eq!(folded.detail(), "first second  third tab");
        let empty = StartupFailure::new(StartupStage::Native, "   ");
        assert_eq!(empty.detail(), "unknown startup failure");
    }

    #[test]
    fn receipt_is_bounded_json_that_replaces_the_previous_failure() {
        let directory = std::env::temp_dir().join(format!("om9-startup-{}", uuid::Uuid::new_v4()));
        let first = StartupFailure::new(StartupStage::Document, "missing archive module");
        let path = first.write_receipt(&directory).unwrap();
        assert_eq!(path.file_name().unwrap(), StartupFailure::RECEIPT_FILE);
        let parsed: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(parsed["schema"], "om9.startup.failure.v1");
        assert_eq!(parsed["stage"], "document");
        assert_eq!(parsed["detail"], "missing archive module");

        let second = StartupFailure::new(StartupStage::Renderer, "no GPU adapter");
        second.write_receipt(&directory).unwrap();
        let parsed: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(parsed["stage"], "renderer");
        assert_eq!(parsed["detail"], "no GPU adapter");
        let staging: Vec<_> = std::fs::read_dir(&directory)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_name().to_string_lossy().ends_with(".tmp"))
            .collect();
        assert!(staging.is_empty(), "staging files must not survive");
        std::fs::remove_dir_all(&directory).unwrap();
    }

    #[test]
    fn hardware_gpu_is_required_without_a_software_fallback() {
        use eframe::wgpu::DeviceType;
        for kind in [
            DeviceType::DiscreteGpu,
            DeviceType::IntegratedGpu,
            DeviceType::VirtualGpu,
        ] {
            assert_eq!(
                gpu_adapter_identity(Some(("hardware", kind))).unwrap(),
                "hardware"
            );
        }
        for missing in [
            None,
            Some(("", DeviceType::DiscreteGpu)),
            Some(("  ", DeviceType::IntegratedGpu)),
            Some(("WARP", DeviceType::Cpu)),
            Some(("unknown", DeviceType::Other)),
        ] {
            let error = gpu_adapter_identity(missing).unwrap_err();
            let failure = StartupFailure::from_error(StartupStage::Renderer, &error);
            assert_eq!(failure.stage, StartupStage::Renderer);
            assert!(failure.detail().contains("hardware GPU adapter required"));
        }
    }

    #[test]
    fn dialog_is_opt_out_for_automation_only() {
        assert!(should_report_dialog(None));
        assert!(should_report_dialog(Some("1")));
        assert!(should_report_dialog(Some("true")));
        assert!(!should_report_dialog(Some("0")));
        assert!(!should_report_dialog(Some(" false ")));
    }

    fn assert_size(actual: [f32; 2], expected: [f32; 2], label: &str) {
        assert!(
            (actual[0] - expected[0]).abs() < 0.05 && (actual[1] - expected[1]).abs() < 0.05,
            "{label}: got {actual:?}, expected {expected:?}"
        );
    }

    #[test]
    fn effective_min_size_keeps_1280x720_on_large_screens() {
        assert_eq!(effective_min_size(Some([2560., 1440.])), [1280., 720.]);
        assert_eq!(effective_min_size(Some([1920., 1080.])), [1280., 720.]);
        // 5120x1440 ultra-wide: width and height both exceed the nominal.
        assert_eq!(effective_min_size(Some([5120., 1440.])), [1280., 720.]);
    }

    #[test]
    fn effective_min_size_fits_a_1366x768_screen_at_100_percent() {
        // 1366x768 minus the shell reserve: the width still holds 1280, the
        // height is capped to 688 so the window fits above the taskbar.
        assert_size(
            effective_min_size(Some([1366., 768.])),
            [1280., 688.],
            "1366x768 @100%",
        );
    }

    #[test]
    fn effective_min_size_is_capped_to_the_work_area_at_125_and_150_percent() {
        assert_size(
            effective_min_size(Some([1366. / 1.25, 768. / 1.25])),
            [1076.8, 534.4],
            "1366x768 @125%",
        );
        assert_size(
            effective_min_size(Some([1366. / 1.5, 768. / 1.5])),
            [894.67, 432.],
            "1366x768 @150%",
        );
    }

    #[test]
    fn effective_min_size_falls_back_safely_for_degenerate_input() {
        // Tiny monitors are held at the planner floor, never zero or negative.
        assert_eq!(effective_min_size(Some([640., 480.])), [800., 400.]);
        assert_eq!(effective_min_size(Some([100., 100.])), [800., 400.]);
        assert_eq!(effective_min_size(Some([1., 1.])), [800., 400.]);
        // Unknown or unusable monitor information: the plain supported minimum.
        for monitor in [
            None,
            Some([0., 0.]),
            Some([-1., 5.]),
            Some([1366., 0.]),
            Some([f32::NAN, 768.]),
            Some([1366., f32::NAN]),
            Some([f32::INFINITY, 768.]),
            Some([1366., f32::NEG_INFINITY]),
        ] {
            assert_eq!(effective_min_size(monitor), [1280., 720.], "{monitor:?}");
        }
    }

    #[test]
    fn effective_min_size_is_always_finite_and_between_the_floor_and_the_minimum() {
        for width in (1..=5000).step_by(37) {
            for height in (1..=5000).step_by(37) {
                let size = effective_min_size(Some([width as f32, height as f32]));
                assert!(
                    size.iter().all(|value| value.is_finite()),
                    "{width}x{height}"
                );
                assert!(
                    size[0] >= PLAN_FLOOR_WINDOW.width && size[0] <= SUPPORTED_MIN_WINDOW.width,
                    "{width}x{height}: {size:?}"
                );
                assert!(
                    size[1] >= PLAN_FLOOR_WINDOW.height && size[1] <= SUPPORTED_MIN_WINDOW.height,
                    "{width}x{height}: {size:?}"
                );
            }
        }
    }

    #[test]
    fn nominal_minimum_is_1280x720_and_the_builder_gets_the_planner_floor() {
        let config = AppConfig::default();
        assert_eq!(config.min_window, [1280., 720.]);
        assert_eq!(config.builder_min_window(), [800., 400.]);
        for window in [[1279., 720.], [1280., 719.], [1000., 600.]] {
            let below = AppConfig {
                window,
                ..AppConfig::default()
            };
            let failure = below.validate().expect_err("below 1280x720 is refused");
            assert!(failure.detail().contains("below the minimum window size"));
        }
        let at_minimum = AppConfig {
            window: [1280., 720.],
            ..AppConfig::default()
        };
        assert!(at_minimum.validate().is_ok());
    }

    #[test]
    fn app_config_refuses_an_unusable_start_before_any_window_exists() {
        let config = AppConfig::default();
        assert_eq!(config.window_title(), "OpenMatrix9");
        assert!(config.validate().is_ok());

        let below_minimum = AppConfig {
            window: [200., 200.],
            ..AppConfig::default()
        };
        let failure = below_minimum.validate().expect_err("small window refused");
        assert_eq!(failure.stage, StartupStage::Renderer);
        assert!(failure.detail().contains("below the minimum window size"));

        let not_finite = AppConfig {
            window: [f32::NAN, 1000.],
            ..AppConfig::default()
        };
        assert!(not_finite.validate().is_err());

        let unnamed = AppConfig {
            app_title: "   ".into(),
            ..AppConfig::default()
        };
        assert!(unnamed.validate().is_err());

        let interactive = AppConfig::interactive("Untitled", None);
        assert!(
            interactive.maximized,
            "an interactive start opens maximized"
        );
        assert!(interactive.validate().is_ok());
        let audited = AppConfig::interactive("Untitled", Some(PathBuf::from("audit")));
        assert!(!audited.maximized, "an audit keeps the exact window size");
        assert_eq!(audited.audit_dir.as_deref(), Some(Path::new("audit")));
    }
}
