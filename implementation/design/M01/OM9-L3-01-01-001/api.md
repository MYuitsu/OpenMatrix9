# `OM9-L3-01-01-001` — Application Startup And Main Window: API contract

Owner: `OM9-L2-01-01` (Febe Application Shell Docking) · Milestone `M01` · Stage `E02`
Source revision audited: `746a17f127897e921a57118b6caad08968602bf5` (see `source_crosswalk.json`)

## 1. Scope and exclusions

**In scope.** Desktop process bootstrap: validate the startup record, create the empty document,
open exactly one window whose title/identity is OpenMatrix9's own, choose a graphics adapter (or
explain the fallback), route every start failure to an owned diagnostic, and hand the running
application to the frame loop.

**Out of scope.** The CAD content of the document (L2-01-03), command dispatch and Undo (L2-01-02),
docking/menu layout (L2-01-01 L3-002…L3-008), OCCT/geometry, 3DM exchange, and any second UI toolkit.

## 2. Typed API (implementation surface)

```rust
// om9_app::startup
pub struct AppConfig {
    pub app_title: String,        // title bar product name, e.g. "OpenMatrix9"
    pub document_name: String,    // empty document created at start
    pub window: [f32; 2],         // initial logical size
    pub min_window: [f32; 2],     // smallest accepted size (dock layout relies on it)
    pub maximized: bool,          // interactive default; audits keep the exact size
    pub audit_dir: Option<PathBuf>,   // own-app audit receipt directory
    pub layout_path: Option<PathBuf>, // app-scoped layout store
    pub startup_dialog: bool,     // may a failed start show the native dialog
}
impl AppConfig {
    pub fn interactive(document_name: &str, audit_dir: Option<PathBuf>) -> Self;
    pub fn window_title(&self) -> &str;
    pub fn validate(&self) -> Result<(), StartupFailure>;   // before any window/device/document
}

pub enum StartupStage { Document, Renderer, Native }
pub struct StartupFailure { pub stage: StartupStage, /* bounded detail */ }
impl StartupFailure {
    pub const MAX_DETAIL_BYTES: usize = 512;
    pub const RECEIPT_FILE: &str = "startup-failure.json";
    pub const DIALOG_ENV: &str = "OM9_STARTUP_DIALOG";
    pub fn new(stage: StartupStage, detail: &str) -> Self;
    pub fn from_error(stage: StartupStage, error: &om9_core::Error) -> Self;
    pub fn detail(&self) -> &str;
    pub fn line(&self) -> String;                 // one user-visible line
    pub fn to_json(&self) -> Result<String>;      // schema "om9.startup.failure.v1"
    pub fn write_receipt(&self, directory: &Path) -> Result<PathBuf>;
    pub fn report(&self) -> Result<Option<PathBuf>>;  // next to the layout store
}

pub fn adapter_note(adapter: Option<&str>) -> (String, Option<String>); // identity + fallback diagnostic
pub fn should_report_dialog(value: Option<&str>) -> bool;               // "0"/"false" opt out

// om9_app::ui
impl MatrixApp {
    pub fn start(config: &AppConfig) -> Result<MatrixApp, StartupFailure>;
}
pub fn launch_classified(document: Document, audit: Option<PathBuf>) -> Result<(), StartupFailure>;
pub fn launch_file_classified(path: &Path) -> Result<(), StartupFailure>;
pub fn report_startup_failure(failure: &StartupFailure) -> bool;  // native owner dialog
pub fn launch(document: Document, audit: Option<PathBuf>) -> Result<()>;   // legacy facade
pub fn launch_file(path: &Path) -> Result<()>;                            // legacy facade
```

Native boundary: `om9_native_owner::report_startup_failure(title, message) -> bool`
(`MessageBoxW`, safe wrapper, 512-character bound, `false` on non-Windows).

## 3. Ownership rules

| Concern | Owner |
| --- | --- |
| Document, scene, undo/redo, transactions, selection | `om9_app::session::Controller` (unchanged) |
| Window title, dirty marker | `Controller::active_file_path` + `MatrixApp::update_file_title` |
| Startup record and failure type | `om9_app::startup` (this module, `OM9-L2-01-01`) |
| Adapter identity and fallback explanation | `startup::adapter_note` + `StatusSurface` |
| Failure receipt bytes | `StartupFailure::write_receipt` (app scope, atomic replace, never the document) |
| Native dialog | `om9-native-owner` (only unsafe owner) |
| Dock/menu layout, status line | `dock_layout`, `status_surface`, `ui::chrome` |

A failed start **never** mutates a document, writes a layout, publishes a scene or leaves a child
process. `AppConfig::validate` runs before any window, device or document exists, so an unusable
record fails with `StartupStage::Renderer` and no side effects.

## 4. Errors

| Stage | Raised when | Observable result |
| --- | --- | --- |
| `Document` | `MatrixApp::from_file`/`Document` admission fails (missing/corrupt archive) | stderr line + `startup-failure.json` with `stage = "document"`, exit 1 |
| `Renderer` | `AppConfig::validate` refuses the record, or window/device creation fails | stderr line + receipt with `stage = "renderer"`, exit 1 |
| `Native` | a native owner call fails during start | stderr line + receipt with `stage = "native"` |

Dialog policy: shown for a real UI start (`--ui`, `--ui-audit`, no arguments) unless
`OM9_STARTUP_DIALOG=0|false`. The console-less Windows build therefore cannot exit silently.

## 5. Oracles and fixtures

Positive (`implementation/fixtures/M01/OM9-L3-01-01-001/positive.json`): one window titled
`OpenMatrix9 — Untitled`, an empty clean `Untitled` document, four viewports, the documented left
menu column, `Ready` status line and a present command line.

Negative (`…/negative.json`): each case is observable without a live reviewer —
`missing-document` (stage + receipt schema), `window-smaller-than-the-minimum` (typed refusal,
no window), `no-hardware-adapter` (adapter `software-fallback` + `GPU adapter unavailable`),
`silent-exit-forbidden` (receipt file + dialog switches).

Driven by `standalone/crates/om9-app/tests/startup_contract.rs`, which reads the fixtures from disk
at run time, so a contract change that contradicts a recorded expectation fails the build.

## 6. Stop conditions (from the plan, recorded honestly)

- `M00 not PASS => BLOCKED_M00`: M00 is **not signed**; this module was implemented under the user's
  explicit instruction for the standalone app. No `OM9_Milestones/` status is promoted by this work.
- `License/fork permission not approved => BLOCKED_APPROVAL`: QymCAD is AGPL-3.0-or-later. **No QymCAD
  code was copied**; only patterns were adapted into OM9-owned safe Rust (see the crosswalk). Copying
  upstream code still requires the G00-06 approval.
- `Missing Rust/OCCT/GPU/Windows => BLOCKED_ENV`: not triggered — Rust, wgpu and Windows are present;
  OCCT is not needed for this L3.
