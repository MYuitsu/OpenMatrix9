// SPDX-License-Identifier: LGPL-2.1-or-later
//! Typed startup failure surface for the standalone shell.
//!
//! Owner: plan module `OM9-L2-01-01` L3-001. A desktop start that cannot restore
//! the document, create the window or obtain a GPU adapter must produce an
//! explicit diagnostic instead of a silent exit. The failure is Rust-owned data:
//! it is bounded, serialized as a receipt next to the application layout and, on
//! Windows, optionally shown through the native owner.
use om9_core::{Error, Result};
use std::path::{Path, PathBuf};

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
    /// Smallest accepted window size; the dock layout relies on it.
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
            min_window: [720., 480.],
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

/// Map the graphics adapter identity to the value recorded in receipts plus the
/// diagnostic the user must see when the hardware path is unavailable.
pub fn adapter_note(adapter: Option<&str>) -> (String, Option<String>) {
    match adapter {
        Some(info) if !info.trim().is_empty() => (info.to_owned(), None),
        _ => (
            "software-fallback".to_owned(),
            Some(
                "GPU adapter unavailable: running the software fallback; 3D performance is not representative"
                    .to_owned(),
            ),
        ),
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
    fn adapter_note_explains_the_software_fallback() {
        let (adapter, note) = adapter_note(Some("NVIDIA GeForce RTX 4070 (Vulkan)"));
        assert_eq!(adapter, "NVIDIA GeForce RTX 4070 (Vulkan)");
        assert!(note.is_none());

        for missing in [None, Some(""), Some("   ")] {
            let (adapter, note) = adapter_note(missing);
            assert_eq!(adapter, "software-fallback");
            let note = note.expect("fallback diagnostic");
            assert!(note.contains("GPU adapter unavailable"));
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
