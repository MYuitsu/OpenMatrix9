# `OM9-L3-01-01-002` — Workbench Docking Regions: API contract

Owner: `OM9-L2-01-01` (Febe Application Shell Docking) · Milestone `M01` · Stage `E02`
Source revision audited: `746a17f127897e921a57118b6caad08968602bf5` (see `source_crosswalk.json`)
Legal reuse of upstream code: **PENDING_HUMAN_M00_APPROVAL** (nothing is copied) · M00 gate: **NOT_PASS** · Manual gate M01-UI: **OPEN**

## 1. Scope and exclusions

**In scope.** The region model of the Matrix9 workbench window: a left rail, a center holding the
four Matrix viewports (or one maximized viewport), an optional right properties dock, plus the three
bars that frame the center (menu bar, command input, toolbar) and the status bar. The model is pure
(no egui, no IO), explicit about failure (a typed error or a visible note, never a silent clamp) and
`serde`-ready.

**Out of scope (siblings own them, nothing here changes their behavior).**
persistence and the layout store (L3-005), restore default / Reset Menus (L3-004), reorder / float of
sections (L3-003), the Builder slot (L3-008), command dispatch / Undo (L2-01-02), document and scene
(L2-01-03), OCCT / geometry, a mechanical-CAD UI set, a second Matrix Main Menu.

Positive oracle: *Show left rail, four viewport center region and optional right properties dock.*
Negative oracle: *Narrow window does not overlap/lose critical command input panel.*

## 2. Typed API (`om9_app::workbench_regions`)

```rust
pub const MENU_BAR_HEIGHT: f32 = 21.;          // the egui "menu" panel in ui.rs
pub const VIEW_GAP: f32 = 2.;                   // gap between viewports in the 2x2 grid
pub const MAX_WINDOW_DIMENSION: f32 = 100_000.; // larger windows are rejected as InvalidWindow

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WindowSize { pub width: f32, pub height: f32 }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RegionRect { pub x: f32, pub y: f32, pub width: f32, pub height: f32 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RegionKind { MenuBar, LeftRail, Center, RightDock, CommandInput, Toolbar, StatusBar }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ViewSlot { pub view: ViewId, pub rect: RegionRect }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum YieldReason { NarrowWindow, NotEnoughRoom }

/// A visible, non-silent fallback record. The UI shows it; tests assert it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RegionNote {
    RightDockYielded { reason: YieldReason },
    LeftRailShrunk { from: f32, to: f32 },
    CommandInputKept,                      // layout said "hidden"; the command input is critical
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RegionPlan {
    pub window: WindowSize,
    pub menu_bar: RegionRect,
    pub left_rail: Option<RegionRect>,     // None when the left rail is hidden
    pub center: RegionRect,
    pub views: Vec<ViewSlot>,              // 1..=4 slots tiling `center`
    pub right_dock: Option<RegionRect>,    // None: hidden or yielded (see notes)
    pub command_input: RegionRect,         // always present, height >= COMMAND_MIN
    pub toolbar: RegionRect,
    pub status_bar: RegionRect,
    pub notes: Vec<RegionNote>,
}
impl RegionPlan { pub fn validate(&self) -> Result<(), RegionError>; }

#[derive(Debug, Clone, PartialEq)]
pub enum RegionError {
    InvalidWindow { width: f32, height: f32 },                       // non-finite, <= 0 or > MAX_WINDOW_DIMENSION
    WindowTooSmall { width: f32, height: f32, min_width: f32, min_height: f32 },
    CenterTooSmall { width: f32, height: f32 },                      // tile_views on a degenerate center
    CommandInputUnavailable,                                         // host seam says not ready
    ViewSetInvalid { count: usize },                                 // 0, > 4 or duplicate views
    Overlap { a: RegionKind, b: RegionKind },                        // validate()
    OutOfBounds { kind: RegionKind },                                // validate()
    NonFinite { kind: RegionKind },                                  // validate()
    CommandInputTooSmall { height: f32 },                            // validate()
    ViewSlotInvalid { view: ViewId },                                // validate()
}
// Display + std::error::Error; From<RegionError> for om9_core::Error (Invalid(label)).

pub fn compute_regions(layout: &DockLayout, window: WindowSize, views: &[ViewId])
    -> Result<RegionPlan, RegionError>;                // pure: &DockLayout, nothing is mutated
pub fn tile_views(center: RegionRect, views: &[ViewId])
    -> Result<Vec<ViewSlot>, RegionError>;             // the 2x2 grid / single view used by the frame

// Seams toward OM9-L2-01-02 (command input) and OM9-L2-01-03 (document / scene).
pub trait CommandInputHost { fn command_input_ready(&self) -> bool; }
pub trait ViewportSceneHost { fn visible_views(&self) -> Vec<ViewId>; }
pub fn compute_regions_for(layout: &DockLayout, window: WindowSize,
    scene: &impl ViewportSceneHost, command: &impl CommandInputHost)
    -> Result<RegionPlan, RegionError>;
```

`RegionPlan` derives `Serialize`/`Deserialize` so the later persistence task can store it, but this
task adds no file IO and no schema.

### Geometry (matches the egui panel order in `MatrixApp::show`)

The panels are declared menu, left, command, toolbars, right, status, center, and egui gives each the
space left by the earlier ones. The plan reproduces that order, so the left rail and the right dock
run to the bottom of the window and the status bar sits between them:

```
menu_bar       (0, 0, W, 21)
left_rail      (0, 21, L, H-21)
command_input (L, 21, W-L, C)                 C = command height
toolbar       (L, 21+C, W-L, 67)
right_dock    (W-R, 21+C+67, R, H-21-C-67)
status_bar    (L, H-20, W-L-R, 20)
center        (L, 21+C+67, W-L-R, H-20-21-C-67)
views         2x2 grid in `center`, VIEW_GAP 2, order LookingDown, Perspective, ThroughFinger, SideView
              (one view fills `center`)
```

Widths `L` and `R` come from `DockLayout::resolve` (single source of truth for the constants).
`resolve().canvas[1]` omits the 21 px menu bar, so `RegionPlan.center` is the authoritative canvas.

### Rules

1. `window` not finite, `<= 0` or `> 100000` in either dimension -> `Err(InvalidWindow)`.
2. Minimum usable window: width `240 + 320 = 560` (`320` when the left rail is hidden), height
   `21 + 58 + 67 + 20 + 120 = 286`. Smaller -> `Err(WindowTooSmall)`; the caller keeps its last good plan.
3. Narrow (`width < 900`) or `left + right + 320 > width` -> the right dock is dropped,
   `RegionNote::RightDockYielded`. A hidden right dock (no menu docked right) is a defined state:
   `right_dock: None`, no note.
4. The left rail shrinks toward its minimum before the canvas drops below 320 px: `RegionNote::LeftRailShrunk`.
5. The command input is critical. If the layout marks it hidden, the plan still reserves it
   (`RegionNote::CommandInputKept`). Its height is `size.clamp(58, 400)` reduced only as far as needed to
   keep a 120 px canvas, and is never below 58.
6. `views` has 1..=4 distinct entries; otherwise `Err(ViewSetInvalid { count })`.
7. `validate()` (also run on every `Ok` result) asserts: all rects finite and non-negative; menu bar,
   left rail, command input, toolbar, right dock, status bar and center lie inside the window and do not
   overlap one another; `command_input.height >= 58`; view slots lie inside `center` and do not overlap.

## 3. Ownership

| Concern | Owner |
| --- | --- |
| Region sizes, constants, dock/visibility state | `DockLayout` in `dock_layout.rs` (unchanged) |
| Region geometry, notes, errors, four-view tiling | `workbench_regions.rs` (this task) |
| egui panels, status line, audit receipt | `ui.rs` (reads the plan; no layout logic of its own) |
| Document, transaction, Undo/Redo, scene, render cache | **not owned**: `om9-core` / `om9-scene` |

The docking regions own layout state only. `compute_regions` takes `&DockLayout` and plain values; it
holds no document, transaction, scene or render-cache reference, so an `Err` cannot change them by
construction (a test also drives a real frame and compares the dock layout and workspace before/after).

### Dependency seams (honest status)

`OM9-L2-01-02` (command dispatch / Undo) and `OM9-L2-01-03` (document / scene) are
`SPEC_ONLY_NOT_IMPLEMENTED` and their L4 acceptance is not PASS (`BLOCKED_DEPENDENCY`). This task does
**not** implement them and does not claim them satisfied. It defines the two minimal traits above in the
docking crate and tests against local stubs. In `ui.rs` thin adapters read what already exists: the
workspace's `visible_views()` and the always-attached CMD frame (`Transcript`). The adapter for
`CommandInputHost` reports the CMD frame as attached; it makes no claim about the command journal.

## 4. REUSE / ADAPT / ADD decision

| Piece | Decision | Local path / evidence |
| --- | --- | --- |
| Constants, widths, dock state | REUSE_IN_PLACE | `standalone/crates/om9-app/src/dock_layout.rs:32-48,628` |
| egui panel arrangement | EXTEND | `standalone/crates/om9-app/src/ui.rs:725-848` |
| Place kinds (left/right/centre/top/bottom), host seam idea | ADAPT (idea only, nothing copied) | upstream `crates/qymcad-shell/src/lib.rs:21,92` @ `746a17f` |
| Four-viewport tiling, narrow-window rule, command-input guarantee, typed errors | ADD | `standalone/crates/om9-app/src/workbench_regions.rs` |

No QymCAD crate or path is a build dependency. Rust only, safe code (`#![forbid(unsafe_code)]`), no
native exception. If upstream text is ever copied, the QymCAD `LICENSE` / `THIRD-PARTY-NOTICES.md`
notices must be carried into that file first.

## 5. Single-name rule

`compute_regions` / `tile_views` exist once, in `om9-app`. The frame calls `tile_views` for the actual
central rectangle and `compute_regions` for validation; there is no second backend/UI function with the
same meaning.

## 6. Test design

| Required test | Where | Oracle |
| --- | --- | --- |
| (a) valid input, no panic, no stale reference | `workbench_regions.rs` unit tests; `positive.json` | hand-computed rects for 1600x1000, 1280x900, maximized view, 560x286; table over 1920x1080, 1600x1000, 1280x900, 1024x768 with finite, non-overlapping, tiling invariants; a second call equals the first |
| (b) invalid / empty / degenerate | unit tests; `negative.json` | zero, negative, NaN, inf, huge windows -> `InvalidWindow`; below minimum -> `WindowTooSmall`; hidden right dock -> defined state; hidden command line -> `CommandInputKept`; 0 / 5 / duplicate views -> `ViewSetInvalid`; the layout serializes identically before and after |
| (c) each operation changes only its documented state | unit tests with stub hosts; `tests/workbench_regions.rs` | `compute_regions` leaves `DockLayout` equal; stub counters show only the two documented host calls; a real frame with an `Err` leaves dock and workspace unchanged |
| Positive oracle | both | left rail + four viewports + optional right dock at 1600x1000 |
| Negative oracle | sweep test | widths 200..1600 and heights 100..1000, step 20, three layouts: every `Ok` plan keeps the command input >= 58 px high, inside the window and overlapping nothing; every other size is an explicit error |

## 7. Absent tooling and unverified items

- `tools/validate_m01_spec.py`, `validate_m00_execution.py`, `next_m01_task.py`, `m01_capture.py`,
  `m01_init_evidence.py`, `validate_m01_execution.py` do not exist in this workspace: **not run**.
- `om9.m01.oracle.v1.5` has no local definition; the string is carried as a field in the fixtures only.
- `OpenMatrix9_Codex_Spec_v2/` (including MATRIX9_UI_CONTRACT) is absent locally; the layout follows the
  root `AGENTS.md` summary, the manual-derived constants already in `dock_layout.rs` and the previous
  L2-01-01 pass. This is an unverified constraint.
- The actual egui panels stay user-resizable and are not clamped by the plan; the plan validates the
  frame and tiles the viewports. DPI scaling and multi-monitor behavior are not verified (manual gate).
