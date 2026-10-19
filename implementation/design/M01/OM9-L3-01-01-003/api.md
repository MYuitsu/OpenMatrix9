# `OM9-L3-01-01-003` — Reorder Collapse And Float Panels: API contract and test design

> **Superseded where it conflicts by `design.md` (revision 2, wf_9fe91738fb13ff99, 2026-10-16).** The Matrix9
> main-window layout rules (steering `.kiro/steering/matrix9-main-window-layout.md`) changed D6 L5 (audit hook docks
> Snaps, not Projects), the supported size set, and added Part A (layout parity) plus `PanelOp::Close`. The edits this
> file still needs are listed in `design.md` §12 and are applied in L4-02.

Owner: `OM9-L2-01-01` (Febe Application Shell Docking) · Milestone `M01` · Stage `E02` · L4 `OM9-L4-01-01-003-02`
Source revision audited: `746a17f127897e921a57118b6caad08968602bf5` (see `source_crosswalk.json`)
Legal reuse of QymCAD: **HUMAN_APPROVED_2026-10-16 (user: "M00 duyệt")**, recorded in
`.agents/tasks/m01-l3-003-panels/user-decisions.md`. Scope of that approval beyond QymCAD reuse is still being asked, so
only QymCAD reuse is treated as approved. Nothing is copied anyway (ideas only). If code is ever copied, the file must
carry the `QymCAD/LICENSE` and `QymCAD/THIRD-PARTY-NOTICES.md` notices, and OM9 still has no build-time QymCAD dependency.
Manual gate M01-UI: **OPEN**. `OM9-L2-01-02` / `OM9-L2-01-03`: **BLOCKED_DEPENDENCY** (unimplemented modules; this task
uses the existing seams only).

User decisions binding this design (later steps apply them too): `.agents/tasks/m01-l3-003-panels/user-decisions.md`
(1) M00 / QymCAD reuse approved; (2) Layers placement in the left column directly below Info & Settings at every size
(D6).

Fixture location: the step brief listed `positive.json`/`negative.json` under this design folder. The L4-02 `l4.json`
`outputs`, the requirements (§4) and the L3-002 precedent put them under
`implementation/fixtures/M01/OM9-L3-01-01-003/`. They are written there, once, with no duplicate copy here.

## 1. Scope and exclusions

**In scope.** For the seven documented Matrix menu sections (Icon History, Main Menu, Display, Snaps, Info & Settings,
Layers, Projects): roll up / expand a title bar, reorder by drag inside a column, dock across the left column and the
right ~300 px mirror column, float over the model as an owned window, dock back without losing state, a live drop
preview, drag cancel, and keeping every floating window reachable (off-screen recovery). Added scope (user,
2026-10-16): Layers stays in the left column directly below Info & Settings at every supported size (D6).

**Panel and menu state is UI state, not document state.** Every L3-003 operation changes only `DockLayout` section
placement (`dock`, `order`, `home`), `collapsed`, float rectangles, `dock_dirty`, and the status/CMD-history line. It never
reads or writes document objects, IDs, layers, units, selection, the command draft, the controller's running command,
the undo/redo journal, `controller.workspace` (views, active, maximized), the scene or the render cache. Document
Undo/Redo neither records nor reverts panel operations. "Undo where applicable" is **not applicable** to layout state.
The document Undo journal belongs to `OM9-L2-01-02`. "Cancel" here means drag cancel (§2 D5). No operation writes a file.
Writing stays behind View ▸ Matrix Menus ▸ Set as Default (L3-005).

**Out of scope (siblings own them).** Reset Menus / Reload Menus / the default restore (L3-004), layout persistence and
schema migration (L3-005), Builder slot contents and adjacency below Projects (L3-008), active document window state
(L3-006), status-surface rules (L3-007, reused only), command dispatch / Undo (L2-01-02), document / scene (L2-01-03),
the Custom menu (book 1 p.17), separate OS windows (multi-viewport, see D2), and any `planning_state` change or gate
signature.

Positive oracle: *Move, collapse, float permitted dock panel; restore without losing state.*
Negative oracle: *Dragging panel outside screen can be recovered/reset.*

## 2. Resolved decision points

### D1 Off-screen recovery (required)

**Recovery here is clamp/snap-back only; Reset Menus belongs to L3-004.** L3-003 adds no reset command and does not
change `reset_menus`, `reload_menus`, `reset_dock_layout` or `DockLayout::reset_to_default`.

*Visible work area* (`WorkArea`, logical points, origin top-left of the main window content rect):

```
x = FLOAT_MARGIN                                   (8)
y = MENU_BAR_HEIGHT + FLOAT_MARGIN                 (21 + 8 = 29)
w = W - 2 * FLOAT_MARGIN
h = H - MENU_BAR_HEIGHT - STATUS_BAR_HEIGHT - 2 * FLOAT_MARGIN     (H - 57)
```

It is the window below the menu bar and above the status bar, inset by 8 px. Reason: a floating menu must never hide
the menu bar (the documented Matrix Menus commands) or the status line, where L3-007 keeps an unacknowledged failure
visible. Constants come from `workbench_regions::MENU_BAR_HEIGHT` and `dock_layout::{STATUS_BAR_HEIGHT, FLOAT_MARGIN}`,
with no new copies. 1600x1000 gives `[8, 29, 1584, 943]`. 1280x720 gives `[8, 29, 1264, 663]`.

Clamp (`WorkArea::clamp`, the only implementation): `width = w.clamp(FLOAT_MIN_WIDTH, area.w)`,
`height = h.clamp(FLOAT_MIN_HEIGHT, area.h)`, `x = x.clamp(area.x, area.x + area.w - width)`,
`y = y.clamp(area.y, area.y + area.h - height)`. Any change is reported as a `PanelNote::Recovered { from, to }`.

When it runs:
1. **On every float request.** A drop, the ⧉ button, the API, or a drop outside the main window: `PanelOp::Float`.
2. **On floating-window move/resize readback.** `PanelOp::SetFloatRect` runs each frame the egui rect differs from the
   stored one by more than 0.5 px. While the user drags, egui already keeps the window inside the work area (point 4),
   so this normally reports nothing.
3. **On window resize and on load.** `DockLayout::clamp_floats(window)` keeps its name and signature, which
   `load_dock_layout`, `sync_dock_window` and the existing test use. It is re-implemented on `WorkArea::for_window` +
   `WorkArea::clamp`. It covers floating sections and the F6 `CommandSearch` window. Its diagnostics keep the existing
   text `"{title} was moved back inside the window"` and go through `note_dock_diagnostics`.
4. **egui 0.36.2 interplay.** By default an egui `Area`/`Window` is constrained to `ctx.content_rect()`, the whole window
   including the menu and status bars (`area.rs:140,439,563`). OM9 sets `.constrain_to(work_rect)` on every floating
   menu window and on the F6 window. egui then keeps user moves and resizes inside the same rectangle the model uses,
   and caps the size to it (`window.rs:691-694`). The egui memory position is overridden **only** on the frame after the
   model itself changed a rect (outcome `Floated`, any `Recovered` note, or a `clamp_floats` diagnostic). That frame uses
   `.current_pos(stored.min)` and `.fixed_size(stored.size - chrome)`, where `chrome` is the per-section outer-minus-content
   size measured on the last shown frame (`Vec2::ZERO` before the first show). It is held in
   `MatrixApp::float_resync: BTreeSet<SectionId>` and `float_chrome: BTreeMap<SectionId, Vec2>`, both UI-only state.
   Normal frames never override egui, so a user drag is never fought.
5. **L3-002 rule kept.** While `region_error` is `Some`, no recovery rewrites stored layout. `sync_dock_window` is not
   called (unchanged). Float readback is skipped, as `sync_panel_size` does. `PanelOp::Float` is refused with
   `PanelOpError::LayoutBlocked`.

### D2 Float target (required, recommendation): **Option A — egui `Window` inside the main OS window**

| Criterion | A: egui `Window` in the main viewport (chosen) | B: separate OS viewport (egui multi-viewport) |
| --- | --- | --- |
| Headless testability | Works with the L3-002 single-`Context` harness (`Context::run_ui` + synthetic pointer events); the real path is the tested path | A headless context has no viewport backend; child viewports fall back to embedded windows, so tests would not exercise the shipped path |
| Per-monitor DPI | Inherits the main window's `pixels_per_point`; a monitor with another scale is not handled | Each viewport gets its own native scale (better) |
| wgpu backend | One surface (unchanged) | One surface per OS window; deferred viewports need `Send + Sync + 'static` callbacks, i.e. `MatrixApp` section UI behind shared ownership (large refactor) |
| Lifecycle / focus | Stable egui id `("om9-float-menu", key)`; CMD focus logic (`focus_cmd`) unaffected | Keyboard focus moves to the child OS window, so typing no longer reaches the CMD frame; close-request handling and screenshots are per viewport |
| Off-screen recovery | One coordinate space; pure, testable clamp (D1) | Needs monitor **work areas** to recover across monitors. egui exposes only `monitor_size` (see `.agents/tasks/m01-l3-002-docking/min-window-plan.md`); the real work area needs Win32 FFI/unsafe, which Rust-first rejects |

**Choice: A.** It keeps one tested implementation, safe Rust and the CMD-focus contract. **Known deviation (recorded, not
a silent limit):** the manual (book 1 p.13) says menus can be dragged onto other monitors. OM9 does not support that.
Releasing a title-bar drag outside the main window (`DropTarget::FloatOutside`) floats the menu, recovers it into the
work area and always adds `PanelNote::OutsideMainWindow`. Status (warning) and CMD history then read
`"{title} cannot float outside the main window; placed inside it"`. A floating window dragged by its own title bar
stops at the work-area edge (egui `constrain_to`). Option B is a backlog item for a later, separately authorised task.
It is not part of M01.

### D3 Restore semantics: **remember the home position and keep the roll-up flag**

- New field `SectionState.home: Option<SectionHome>` with `SectionHome { side: ColumnSide, index: u32 }`. It is
  `#[serde(default, skip_serializing_if = "Option::is_none")]`, so schema-2 files load unchanged, older readers ignore it
  (`DockLayout` does not deny unknown fields) and **`SCHEMA_VERSION` stays 2**.
- `home` is set when a section **leaves a docked column** (a float from Left/Right, or a dock to the other column). It
  is set to the column and index it had at that moment. It is cleared (`None`) when the section lands in a column. It
  means "last docked position".
- `Dock { side }` inserts at `min(home.index, len)` when `home.side == side`. Otherwise it inserts at the **documented
  position**: in the left column, directly after the last section already in that column that precedes it in
  `SectionId::ALL` (so Layers lands directly below Info & Settings, see D6). In the right column it goes at the end.
  `DockHome` (floating window close button) uses `home.side` the same way, defaulting to `Left` at the documented
  position. `Place { side, index }` (an explicit drop position) always uses the given index and ignores `home`. The
  helper is `DockLayout::documented_index(id, side) -> usize`.
- `collapsed`, `height`, `visible` and the last `float` rect are **never** changed by move/dock/float/dock-back. The
  current `state.collapsed = false` in `dock_section` (`dock_layout.rs:448`) is removed.
- The floating window is built with `.collapsible(false)`. Roll-up while floating is not offered, the body is always
  shown, and the stored `collapsed` flag round-trips and applies again once docked. This is a minor UI deviation and
  part of the business-rule proposal.

Business-rule change: yes (before: re-dock appends and expands; after: re-dock returns to the remembered index and keeps
roll-up). It goes to the final report for consent (`references/business-rule-updates.md`). No `skills/` edit happens.

### D4 Permitted panels

`SectionId::is_movable(self) -> bool { !self.is_reserved() }` is the single predicate. The 7 documented menus are
permitted. **`BuilderSlot` is not** (L3-008 owns it). Its title bar keeps its text, is drawn with no ± glyph and uses
`Sense::click()` only, so no drag can start. A click runs `PanelOp::ToggleCollapsed { BuilderSlot }`. That returns
`PanelOpError::NotPermitted`, the status shows the warning
`"Builders is reserved for builder tools and cannot be rolled up, moved or floated"` and nothing changes. Other sections
may still be inserted before or after it, and L3-003 does not enforce adjacency to Projects. `PanelId` regions (left rail,
command line, toolbar, right dock) are not L3-003 panels. The F6 window takes part only in recovery (D1).

### D5 Drag cancel

Release over no target is a cancel (`CancelReason::NoTarget`). **Escape also cancels a drag in progress.** The Escape
handler in `MatrixApp::show` (the `consume_key(Modifiers::NONE, Key::Escape)` chain) gets a new **first** branch: when
`self.section_drag.is_some()` it takes the drag, clears `drop_preview` and reports
`PanelOpError::DragCancelled { reason: Escape }`, and it **does not** fall through. So `controller.cancel()`,
`transcript.set_draft("")`, the F6 close and the popup close do not run, and the CMD draft and any running command are
preserved. The later pointer release finds `section_drag == None` and applies nothing. When no drag is in progress, the
Escape chain is unchanged.

### D6 Layers placement (explicit user instruction 2026-10-16, `user-decisions.md` second section)

User: *"layer cũng phải nằm bên trái chứ phía dưới info & setting, phải giống y chang matrix9 dùng ở độ phân giải nào"*
(Layers must be in the left column, below Info & Settings, exactly like Matrix9, at any resolution). The user's
instruction authorises this behaviour change in code. The skill text still waits for consent.

What was found (read, not assumed): the factory order already has Layers at left index 5, directly after Info &
Settings (`SectionId::ALL`). The workspace reference picture `om9-window.png` shows the same. Layers left the column in
real-window evidence because of the audit-only hook `OM9_UI_AUDIT_RIGHT_DOCK=1`, which runs
`dock_menu_section(SectionId::Layers, DockSide::Right)` (`ui.rs` ~1854). `tools/validate_m01_l3_002_ui_matrix.py` sets
that hook for **every** L3-002 matrix case, so every L3-002 screenshot shows Layers in the right column. A second risk
was `dock_section`'s "append at the end" rule: Layers docked back went below Projects/Builders, at the bottom of the
column.

Rules (owned by `dock_layout.rs`; asserted by tests and receipts):
- **L1 Default.** In the factory layout (`matrix_default`, and therefore after Reset Menus, which is L3-004 and
  unchanged), Layers is `DockSide::Left`, with order = order(Info & Settings) + 1, visible and not collapsed. A unit test
  pins it.
- **L2 Every supported size.** For every window where the region plan is `Ok` (at least `PLAN_FLOOR_WINDOW`, up to
  2560x1440, at zoom 1.0 and SIMULATED 1.5): with the default layout, the Layers title bar is inside `sidebar_rect`
  (x-range within the left rail). Its top is below the bottom of the Info & Settings section block, with no other
  title bar between them. It is fully visible at sidebar scroll offset 0. When the column is shorter than its content,
  the existing vertical `ScrollArea` scrolls. Nothing is reordered, moved to the right dock or to the bottom of the
  window, or reflowed into another column. The sections above Layers (Icon History … Info & Settings) keep their content
  and keep priority. Layers' own body may be clipped below the fold and is reached by scrolling. The left rail width
  rule is L3-002's (it narrows only below `left + CANVAS_MIN_WIDTH`). If a measured supported size breaks L2, that is an
  L4-03 defect to fix before GREEN. The only allowed fix is to reduce vertical spacing in the sections above Layers. It
  is never to reorder or move it.
- **L3 Restore.** Docking Layers back (◀, Dock left, close of its floating window, `DockHome`) without a remembered home
  inserts it directly below Info & Settings (D3 documented position). With a home it returns to that home.
- **L4 User moves stay allowed.** Layers is a permitted panel (D4). Roll-up keeps the closed-bar current-layer label.
  Float and dock follow L3-003. These are explicit user session changes, written to disk only by Set as Default.
  OM9 never moves Layers on its own. A user's own saved store is not rewritten. Reset Menus restores L1.
- **L5 Audit hook.** `OM9_UI_AUDIT_RIGHT_DOCK=1` now docks **Projects** (last documented menu above the Builders area)
  instead of Layers. The right column still appears, so the L3-002 matrix expectation `expect_right_dock` is unchanged,
  and Layers stays left in every real-window screenshot. L4-03 updates only the stale comment in
  `tools/validate_m01_l3_002_ui_matrix.py` ("with Layers docked right") and makes no logic change there. In-process
  L3-002 tests that dock Layers right to exercise region geometry (`workbench_regions*.rs`, `status_bar_overlap.rs`)
  are test-only scenarios and stay unchanged.
- Panel layout stays UI state: no document object or Undo history changes (§1).
- Evidence and limits. `docs/analysis/matrix9-main-window-layout.md` (the user's Matrix 9.0 screenshot, about 1024x608)
  gives the left order ICON HISTORY … INFO & SETTINGS, LAYERS, PROJECTS, builders. The column is taller than the window,
  so it is clipped or scrolled, not shrunk to fit. This matches L2: scroll, never reorder. That screenshot shows **no**
  right dock. The right mirror column used by L3-003 is the existing OM9 feature from the manual text (book 1 p.13,
  "dock … far right … 300-pixel column"). It starts hidden and only appears when the user docks a menu there. L3-003
  does not make it a default and does not put Layers in it. D6 fixes order, adjacency and visibility only. Column width
  and panel-height proportions at other resolutions or DPI are not evidenced by a second capture, so no proportional
  rule is invented here.

## 3. Typed API (`om9_app::panel_ops`, new file `standalone/crates/om9-app/src/panel_ops.rs`)

Pure, egui-free and IO-free, with `#![forbid(unsafe_code)]` inherited from `lib.rs`. `lib.rs` gets `pub mod panel_ops;`.
Rectangles reuse `workbench_regions::RegionRect`, and floats reuse `dock_layout::FloatRect`.

```rust
pub const FLOAT_DROP_SIZE: [f32; 2] = [300., 240.];   // size of a menu floated by a drop (today's float_section_at)
pub const FLOAT_DROP_GRAB: [f32; 2] = [150., 7.];     // pointer offset inside that rect (centre of the title bar)
pub const RIGHT_DROP_STRIP: f32 = 48.;                // dock-right target width while the right column is not shown

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColumnSide { Left, Right }
impl From<ColumnSide> for DockSide { .. }
impl TryFrom<DockSide> for ColumnSide { type Error = PanelOpError; .. }  // Top/Bottom/Floating -> InvalidTarget

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SectionHome { pub side: ColumnSide, pub index: u32 }       // stored in SectionState.home

/// Always valid by construction: fields are private, the only constructor validates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct WorkArea { x: f32, y: f32, width: f32, height: f32 }
impl WorkArea {
    pub fn for_window(window: [f32; 2]) -> Result<Self, PanelOpError>;  // D1 formula
    pub fn rect(&self) -> RegionRect;
    pub fn clamp(&self, rect: FloatRect) -> FloatRect;                  // finite input required (validated by callers)
    pub fn contains(&self, rect: FloatRect) -> bool;
}
pub fn float_drop_rect(pointer: [f32; 2]) -> FloatRect;  // [p.x-150, p.y-7, 300, 240], before clamping

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum FloatOrigin { Drop, OutsideMainWindow, Button, Api }

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum PanelOp {
    ToggleCollapsed { section: SectionId },
    Move { section: SectionId, delta: i32 },                       // ▲ ▼ and move_menu_section
    Place { section: SectionId, side: ColumnSide, index: usize },  // drop at a column position (same or other column)
    Dock { section: SectionId, side: ColumnSide },                 // ◀, "Dock right ▶", dock_menu_section
    DockHome { section: SectionId },                               // floating window closed
    Float { section: SectionId, rect: FloatRect, work: WorkArea, origin: FloatOrigin },
    SetFloatRect { section: SectionId, rect: FloatRect, work: WorkArea },  // user moved/resized the floating window
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum PanelOutcome {
    Collapsed { section: SectionId, collapsed: bool },
    Moved { section: SectionId, side: ColumnSide, index: usize },
    Docked { section: SectionId, side: ColumnSide, index: usize, restored_home: bool },
    Floated { section: SectionId, rect: FloatRect },
    FloatRectSet { section: SectionId, rect: FloatRect },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum PanelNote {
    Recovered { section: SectionId, from: FloatRect, to: FloatRect },
    OutsideMainWindow { section: SectionId },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PanelReport { pub outcome: PanelOutcome, pub notes: Vec<PanelNote> }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CancelReason { NoTarget, Escape, PointerLost, LayoutChanged }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum PanelOpError {
    NotPermitted { section: SectionId },                  // BuilderSlot (D4)
    UnknownSection { key: String },                       // string boundary (audit env); key truncated to MAX_SECTION_BYTES
    Hidden { section: SectionId },                        // state.visible == false
    InvalidTarget { side: DockSide },                     // Top / Bottom / Floating given as a column
    InvalidRect { section: SectionId },                   // any non-finite field, width <= 0 or height <= 0
    InvalidWindow { width: f32, height: f32 },            // non-finite, <= 0, > MAX_WINDOW_DIMENSION, or work area below FLOAT_MIN_*
    NoWorkArea,                                           // no frame has produced a usable work area yet
    LayoutBlocked,                                        // region error active (L3-002 rule)
    NotDocked { section: SectionId },                     // Move/Place-reorder of a floating section
    NotFloating { section: SectionId },                   // SetFloatRect on a docked section
    AlreadyFloating { section: SectionId },
    AlreadyThere { section: SectionId, side: ColumnSide, index: usize },  // no-op: reported, not silent
    DragCancelled { section: SectionId, reason: CancelReason },
}
// impl Display + std::error::Error. Display texts are in §6.

impl SectionId { pub fn is_movable(self) -> bool; }                    // in dock_layout.rs, next to is_reserved
impl DockLayout {
    /// The single L3-003 mutation entry. Validates, then mutates a clone and commits it only on Ok.
    pub fn apply_panel_op(&mut self, op: PanelOp) -> Result<PanelReport, PanelOpError>;
}

// Drop resolution (egui-free; the UI fills DropZones from this frame's live rects)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColumnZone { pub rect: RegionRect, pub content_bottom: f32 }
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DropZones {
    pub window: Option<RegionRect>,                       // main window content rect
    pub left: Option<ColumnZone>,                         // sidebar_rect
    pub right: Option<ColumnZone>,                        // right_dock_rect (None while the column is not shown)
    pub right_strip: Option<RegionRect>,                  // rightmost RIGHT_DROP_STRIP of the views' union, only when
                                                          // `right` is None and window width >= NARROW_WINDOW_WIDTH
    pub bars: Vec<(SectionId, ColumnSide, RegionRect)>,   // title bars rendered this frame
    pub views: Vec<RegionRect>,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum DropTarget {
    Place { side: ColumnSide, index: usize },
    Float { pointer: [f32; 2] },
    FloatOutside { pointer: [f32; 2] },
    Cancel { reason: CancelReason },
}
pub fn resolve_drop(layout: &DockLayout, dragged: SectionId, pointer: Option<[f32; 2]>, zones: &DropZones) -> DropTarget;
pub fn drop_indicator(layout: &DockLayout, dragged: SectionId, target: DropTarget, zones: &DropZones,
    work: Option<WorkArea>) -> Option<RegionRect>;
pub fn drop_op(section: SectionId, target: DropTarget, work: Option<WorkArea>) -> Result<PanelOp, PanelOpError>;
```

`dock_layout.rs` changes: `ColumnSide` and `SectionHome` are **defined in `dock_layout.rs`** (persisted schema types) and
re-exported by `panel_ops`, so the persisted model never depends on the operations module. Also `SectionState.home`
(above) and `SectionId::is_movable`. A new primitive
`place_section(id, side: DockSide, index: Option<usize>) -> Result<usize>` replaces the body of `dock_section`, which
becomes `place_section(id, side, None)`. `None` means home-or-end per D3. It records `home`, keeps `collapsed`, keeps
orders dense and returns the final index. `float_section` records `home` when the section leaves a column. Its
signature `(&mut self, SectionId, FloatRect)` is unchanged because existing tests call it as a statement.
`clamp_floats` uses `WorkArea` (D1). `move_section`, `dock_section`, `float_section`, `toggle_section` and
`set_section_collapsed` stay `pub` as the **unchecked model primitives**, used by tests, `normalize` and
`apply_panel_op`. No UI, menu, mouse or audit code may call them (§8).

## 4. Operation rules (`DockLayout::apply_panel_op`)

Common prefix, in order: `!section.is_movable()` → `NotPermitted`. `!state.visible` → `Hidden`. Then per operation,
always on a clone. On `Err`, `self` is untouched, so `to_json()` is byte-identical **by construction**.

| Op | Preconditions → error | Effect | Outcome |
| --- | --- | --- | --- |
| `ToggleCollapsed` | — | flip `collapsed`; nothing else | `Collapsed` |
| `Move { delta }` | dock `Floating` → `NotDocked`; target index == current (incl. `delta == 0`, top moved up, bottom moved down) → `AlreadyThere` | `target = index.saturating_add_signed(delta).min(len-1)` (saturation kept from today: `-99` → 0) | `Moved` |
| `Place { side, index }` | floating + same-column semantics do not apply (a floating section placed into a column is a dock at `index`); same column and final index == current → `AlreadyThere` | remove from current column, insert at `min(index, len_without_it)`, renumber both columns, home rule D3 | `Moved` (same column) / `Docked { restored_home: false }` (other column or from floating) |
| `Dock { side }` | already docked on `side` → `AlreadyThere { index: current }` | D3 index (home or end) | `Docked { restored_home }` |
| `DockHome` | not floating and already on `home.side or Left` → `AlreadyThere` | `Dock { home.side or Left }` | `Docked` |
| `Float { rect, work, origin }` | dock already `Floating` → `AlreadyFloating`; rect invalid → `InvalidRect` | `to = work.clamp(rect)`; set `dock = Floating`, `float = Some(to)`, home rule D3, dense orders | `Floated { rect: to }`; note `Recovered` if `to != rect`; note `OutsideMainWindow` if origin is `OutsideMainWindow` |
| `SetFloatRect { rect, work }` | not floating → `NotFloating`; rect invalid → `InvalidRect` | `float = Some(work.clamp(rect))` | `FloatRectSet`; note `Recovered` if clamped |

Invariants checked after every `Ok` (debug-assert in code, asserted in tests): every `SectionId` appears exactly once
across `left_sections() ∪ right_sections() ∪ floating_sections()`. Orders in each column are dense `0..n`. Every floating
section has `float: Some(r)` with finite `r`. After `Float`/`SetFloatRect`, `work.contains(r)`.
**Owner of these invariants: `panel_ops.rs`/`dock_layout.rs` (model layer).** The UI cannot violate them because it has no
other mutation path (§8 source-scan test).

## 5. Drop resolution (`resolve_drop`) and preview

Precedence for pointer `p` (the last finite pointer position seen during the drag):

1. `p` is `None` or non-finite → `Cancel { PointerLost }`.
2. `zones.window` is known and does not contain `p` → `FloatOutside { p }` (D2).
3. `right.rect` contains `p` → `Place { Right, index }`. 4. `left.rect` contains `p` → `Place { Left, index }`.
5. `right_strip` contains `p` → `Place { Right, index: right_len_without_dragged }`.
6. any `views` rect contains `p` → `Float { p }`.
7. otherwise (menu bar, command line, toolbar, status bar) → `Cancel { NoTarget }`.

Index rule for column `S`: `others = layout.section_order(S)` without `dragged`. The index is the position in `others` of
the first section whose **rendered** bar centre `y >= p.y`. If there is none, it is `others.len()`. Hidden sections
keep their order slot, and the rule needs no hovered bar, so a drop on a menu body also works.

Indicator (`drop_indicator`, `RegionRect` in window points, `None` for `Cancel`):
- `Place`: a 3 px line across the column at `bar.y - 1.5` of the "insert before" bar, or at `content_bottom - 1.5` for
  the end. Colour `Color32::from_rgb(150, 55, 55)` (the existing ± glyph colour).
- `Float` / `FloatOutside`: `work.clamp(float_drop_rect(p))`, the exact rect the drop will produce. 2 px stroke in that
  colour plus a 20% alpha fill.

UI wiring (`ui/chrome.rs`, `ui.rs`):
- `section()` (left title bar): `if response.drag_started() && id.is_movable()` →
  `self.section_drag = Some(SectionDrag { section: id, last_pointer })`. Right-column titles become
  `ui.add(egui::Button::selectable(!collapsed, text).sense(Sense::click_and_drag()))` with the same click and
  drag_started handling, so right-column menus are draggable too. Each rendered bar is recorded with its side for
  `DropZones.bars`. `sidebar_ui` and `right_dock_ui` record `content_bottom = ui.min_rect().bottom()`.
- After `float_menu_windows` in `MatrixApp::show`: if a drag is active, update `last_pointer` from
  `pointer.latest_pos()` (else `interact_pos()`), build `DropZones` from `sidebar_rect`, `right_dock_rect`,
  `view_rects`, the recorded bars and `ui.max_rect()`, and resolve.
  - Pointer still down: `self.drop_preview = Some(DropPreview { section, target, indicator })`. Paint on
    `LayerId::new(Order::Foreground, Id::new("om9-section-drop-preview"))`. Cursor `Grabbing`, or `NoDrop` for `Cancel`.
  - Pointer released: `section_drag = None`, `drop_preview = None`, then `drop_op(section, target, self.work_area)`
    → `apply_panel_op`. `Cancel` maps to `Err(DragCancelled)`. What is applied is exactly the `target` that was
    previewed on the last pressed frame (same resolver, same zones source).
- Reset paths (`reset_menus`, `reload_menus`, `reset_dock_layout`, `load_dock_layout`) take an active drag, clear
  `drop_preview` and append `DragCancelled { LayoutChanged }` to CMD history only (the reset's own status stays). On each
  frame, a drag whose section became hidden is cancelled the same way.

Public, test-visible state on `MatrixApp`: `pub drop_preview: Option<DropPreview>`, `pub work_area: Option<WorkArea>`
(set each frame from `ui.max_rect().size()`, `None` on `Err`), `pub last_panel_op: Option<PanelOpRecord>`,
`pub closed_bar_controls: BTreeSet<&'static str>` (sections whose closed-bar controls rendered this frame), and
`pub fn section_drag(&self) -> Option<SectionId>`. `section_drag` itself becomes `Option<SectionDrag>`
(`struct SectionDrag { section: SectionId, last_pointer: Option<[f32; 2]> }`) and stays private.

## 6. UI boundary, errors and messages (`MatrixApp`, `ui.rs`)

```rust
pub fn apply_panel_op(&mut self, op: PanelOp) -> Result<PanelReport, PanelOpError>;
// existing identifiers kept as one-line delegations (tests and audits use them):
pub fn move_menu_section(&mut self, id: SectionId, delta: i32) -> bool;          // Move
pub fn dock_menu_section(&mut self, id: SectionId, side: DockSide) -> bool;      // ColumnSide::try_from(side) then Dock
pub fn float_menu_section(&mut self, id: SectionId, rect: FloatRect) -> bool;    // Float { work: self.work_area?, origin: Api }
```

`MatrixApp::apply_panel_op`:
1. `Float`/`SetFloatRect` while `region_error.is_some()` → `Err(LayoutBlocked)`.
2. Call `self.dock.apply_panel_op(op)`.
3. On `Ok`: set `dock_dirty = true`. Insert into `float_resync` for `Floated` or any `Recovered` note. Record
   `last_panel_op`.
4. Report. Every message goes to the status line **unless** `status.kind().is_failure()` (an unacknowledged Error, the
   L3-007/L3-002 rule). In that case it goes to CMD history only.
   - `Ok`, no notes: `status.info(outcome text)`. `FloatRectSet` without notes emits nothing: it is a readback of a
     visible window move.
   - `Ok` with notes: each note → `transcript.append_history`, the last note → `status.warning`, and notes are added to
     `dock_diagnostics` (receipt).
   - `Err`: `transcript.append_history("Menus: {error}")` plus `status.warning(error)`, except
     `DragCancelled { Escape }` → `status.info`.
5. Nothing touches `controller`, `transcript` draft/prompt, `focus_cmd` or `controller.workspace`.

| Outcome / note / error | Text (title = `SectionId::title()`) | Kind |
| --- | --- | --- |
| `Collapsed` | `{title} rolled up` / `{title} expanded` (today's text) | Info |
| `Moved` | `{title} moved to menu position {index+1}` (today's text) | Info |
| `Docked` | `{title} docked to the right menu column` / `{title} docked in the left menu column` (today's text) | Info |
| `Floated` | `{title} floated over the model` (today's text) | Info |
| `Recovered` | `{title} was moved back inside the window` | Warning + history |
| `OutsideMainWindow` | `{title} cannot float outside the main window; placed inside it` | Warning + history |
| `NotPermitted` | `{title} is reserved for builder tools and cannot be rolled up, moved or floated` | Warning |
| `UnknownSection` | `menu section '{key}' is not recognized` | Warning |
| `Hidden` | `{title} is hidden; show it before moving it` | Warning |
| `InvalidTarget` | `menus dock only to the left or right column` | Warning |
| `InvalidRect` | `{title}: the floating rectangle is not valid` | Warning |
| `InvalidWindow` | `the window {w}x{h} is too small or not valid for floating menus` | Warning |
| `NoWorkArea` | `floating menus are not available before the window is shown` | Warning |
| `LayoutBlocked` | `menus cannot float while the layout has an error` | Warning |
| `NotDocked` | `{title} is floating; dock it before reordering` | Warning |
| `NotFloating` | `{title} is not floating` | Warning |
| `AlreadyFloating` | `{title} is already floating` | Warning |
| `AlreadyThere` | `{title} is already at menu position {index+1} on the {side}` | Warning |
| `DragCancelled` | `{title} drag cancelled` + `: not over a menu column or viewport` (NoTarget) / ` (Esc)` / `: pointer lost` / `: menus were reset` | Info for Esc, Warning otherwise |

Recoverability: every error is recoverable, and the layout is unchanged. No operation is fatal and none panics. Logging
is the status line and CMD history above. There is no file log.

Input validation (external inputs):

| Input | Required | Type / limits | On failure |
| --- | --- | --- | --- |
| pointer position | optional per frame | finite `f32` pair; `None` allowed | the last finite position is used; none at release → `Cancel { PointerLost }` |
| window size (work area) | per frame | finite, `> 0`, `<= MAX_WINDOW_DIMENSION`, work area `>= FLOAT_MIN_*` | `InvalidWindow`; `work_area = None`; float ops → `NoWorkArea` |
| `FloatRect` from API / drop / readback | required for float ops | all finite; `width > 0`, `height > 0`; any finite magnitude | `InvalidRect` (layout unchanged); valid but out of area → clamped + `Recovered` |
| `delta` | required for `Move` | any `i32`; saturates at the column ends | result == current → `AlreadyThere` |
| `index` | required for `Place` | any `usize`; above `len` means the end | same position → `AlreadyThere` |
| `DockSide` from the bool wrapper | required | `Left` / `Right` | `InvalidTarget` |
| section key (audit env) | required in that path | known `SectionId::from_key`, `<= MAX_SECTION_BYTES` (truncate for display) | `UnknownSection` |
| `OM9_UI_AUDIT_PANELS` | optional | one of the scenario names in §7 | launch error `OM9_UI_AUDIT_PANELS must be one of …`, non-zero exit (same pattern as `OM9_UI_AUDIT_ZOOM`) |

Floating window builder (`float_menu_windows`): `egui::Window::new(title).id(("om9-float-menu", key))
.constrain_to(work_rect).collapsible(false).default_pos(stored.min).default_size(stored.size)`, plus the D1 resync
override. The "◀ Dock left" button → `Dock { Left }`, "Dock right ▶" → `Dock { Right }`, close (X) → `DockHome`.
Readback → `SetFloatRect` (skipped during a region error). The F6 window gets `.constrain_to(work_rect)`; its readback
stays `DockLayout::set_float` (it is not a permitted panel). Right column: ▲/▼ → `Move ±1`, ◀ → `Dock { Left }`,
⧉ → `Float { rect: FloatRect { min: ui.max_rect().center() + (-40·index, 40), size: FLOAT_DROP_SIZE }, origin: Button }`,
and a title click → `ToggleCollapsed`. The left title click → `ToggleCollapsed`. **Menu, mouse, buttons, API and audit
all reach `apply_panel_op`, with no second path.**

## 7. Audit receipt and real-window replay (FR-14)

`receipt["panels"]` (new, always written):

```json
{ "sections": { "<key>": { "dock": "left|right|floating", "order": 0, "visible": true, "collapsed": false,
                           "height": 120.0, "float": [x, y, w, h] | null, "home": {"side": "left", "index": 3} | null } },
  "work_area": [x, y, w, h] | null,
  "last_op": { "op": <PanelOp>, "result": "ok" | "error", "report": <PanelReport> | null, "error": <PanelOpError> | null } | null,
  "drag_in_progress": false, "drop_preview": <DropPreview> | null,
  "scenario": "<name>" | null, "replay": [ … ],
  "layers_placement": { "side": "left", "index": 5, "info_settings_index": 4, "directly_below": true,
                        "layers_bar_px": [x, y, w, h] | null, "info_settings_block_bottom_px": 0.0 | null,
                        "sidebar_px": [x, y, w, h] | null, "sidebar_scroll": 0.0,
                        "layers_bar_visible": true } }
```

`directly_below` is true when the left order has Layers at Info & Settings + 1 and no rendered title bar lies between
the Info & Settings bar and the Layers bar. `layers_bar_visible` is true when the Layers bar rect is fully inside
`sidebar_px`. `sidebar_ui` records the Info & Settings block bottom as `ui.min_rect().bottom()` after that section's
body.

**Layers size matrix (D6 L2).** The launcher runs the **default layout** with no `OM9_UI_AUDIT_RIGHT_DOCK` and a fresh
`OM9_UI_LAYOUT` temp path, so no user store leaks in. Cases: the real-window cases of
`tools/validate_m01_l3_002_ui_matrix.py` as they stand when L4-05 runs (re-read the file; the list is being edited by the
concurrent L3-002 close-out), plus `normal-1920x1080`, `qhd-2560x1440`, and SIMULATED `dpi150-1920x1080`
(`OM9_UI_AUDIT_ZOOM=1.5`) next to the existing SIMULATED 150% cases. If the host display is smaller than the request
(the reference screenshot host is 2576x1408), the OS clamps the window. The receipt's actual window is recorded and
checked, and the clamp is reported, not hidden. Per case: exit 0, `layers_placement.side == "left"`,
`directly_below == true` and `layers_bar_visible == true` at `sidebar_scroll == 0`. The screenshot is opened and the
observation written down, and never claimed unless it was inspected. The frame-level sweep covers what a real window
cannot (see §10).

`OM9_UI_AUDIT_PANELS=<scenario>` (audit-only, opt-in, `--ui-audit` required) replays **real pointer events** through
`raw_input_hook` (new `ui/panel_audit.rs`, same pattern as `ui/viewport_audit.rs`), using live rects. Then it takes a
screenshot and writes the receipt. Scenarios: `rollup` (click the Projects bar), `reorder` (drag Projects onto the upper
half of the Display bar), `dock-right` (drag Snaps into the right strip, or into the right column if it is shown),
`float` (drag Display into the LookingDown view centre), `float-offscreen` (drag Display and release 100 px beyond the
right edge of the window), `shrink` (float Display at the bottom-right, then
`ViewportCommand::InnerSize(SUPPORTED_MIN_WINDOW)` and wait for the size change), `refuse-builders` (click the Builders
bar), `escape-cancel` (start a drag, press Escape, release over a view). Each scenario pushes its check strings into
`audit.checks`. The optional launcher `tools/validate_m01_l3_003_panels.py` is a recorded Rust-first exception, like
`tools/validate_m01_l3_002_ui_matrix.py`: it only launches `om9.exe --ui-audit` with these variables and reads
receipts. It holds no business logic. Output goes to
`build/standalone-validation/ui-artifacts/<date>-m01-l3-003-panels/<case>/`.

## 8. Ownership and single-name rule

| Concern | Owner |
| --- | --- |
| Permitted rule, validation, clamp, home semantics, layout invariants, drop resolution | `panel_ops.rs` + `dock_layout.rs` (model layer, pure) |
| Status/CMD history mapping, `dock_dirty`, drag lifecycle, Escape precedence, egui window building, preview painting, receipt | `ui.rs`, `ui/chrome.rs`, `ui/panel_audit.rs` |
| Region plan and the region-error flag (read only) | `workbench_regions.rs` (L3-002, unchanged) |
| Reset/Reload Menus, persistence | L3-004 / L3-005 (unchanged; only the drag-cancel hook is added) |
| Document, transactions, Undo/Redo, scene, render cache | **not owned**: `om9-core` / `om9-scene` |

Single name: `DockLayout::apply_panel_op` is the one model entry, and `MatrixApp::apply_panel_op` is the one UI
boundary. The three existing `*_menu_section` functions are delegations, not second implementations. A source-scan test
(idea ADAPTED from `qymcad-shell` `the_shell_names_no_workbench`) reads `include_str!` of `src/ui.rs`,
`src/ui/chrome.rs` and `src/ui/panel_audit.rs`, strips comment lines and asserts that none contains
`.toggle_section(`, `.move_section(`, `.dock_section(`, `.float_section(`, `.place_section(` or
`.set_section_collapsed(`.

## 9. REUSE / ADAPT / ADD

| Piece | Decision | Local path / evidence |
| --- | --- | --- |
| Section model, primitives, constants, clamp entry, load/normalize | REUSE_IN_PLACE + EXTEND | `standalone/crates/om9-app/src/dock_layout.rs` |
| Title bars, right column, float windows, settle | EXTEND | `standalone/crates/om9-app/src/ui/chrome.rs` |
| Wrappers, Escape chain, frame loop, receipt | EXTEND | `standalone/crates/om9-app/src/ui.rs` |
| Home overlay, one door, typed drop request, source-scan test | ADAPT (ideas only, nothing copied) | upstream `qymcad-shell/src/lib.rs:104-152,198-213`; `qymcad/src/gui.rs:407,1855-1867` @ `746a17f` |
| Typed ops/errors, work area, drop resolver, preview, cancel, real-window replay | ADD | `panel_ops.rs`, `ui/panel_audit.rs` |

There are no new dependencies and no native code. This is Rust only, safe code, no native exception. The only non-Rust
file is the optional launcher (§7).

## 10. Test design

Unit tests (`panel_ops.rs`, `dock_layout.rs`), all pure:
- `WorkArea::for_window`: 1600x1000 → `[8, 29, 1584, 943]`; 1280x720 → `[8, 29, 1264, 663]`; `0`, negative, NaN, inf,
  `1e6` and a work area below the float minimum (e.g. `150x150`) → `InvalidWindow`.
- The clamp table and every `negative.json` float case (exact numbers).
- Every operation row in §4, ok and error. On every `Err`, `to_json()` before == after.
- Home semantics (`positive.json` P5/P6/P7). `Move` saturation and `AlreadyThere`.
- `resolve_drop` precedence with synthetic `DropZones` (one case per rule 1–7, plus hidden-section index).
- Invariant sweep: seeded xorshift (no new crate), 2,000 random ops including invalid sections, rects (NaN/inf/0/huge),
  deltas (`i32::MIN/MAX`) and windows. Asserts no panic and the §4 invariants after each step.
- Schema: a schema-2 JSON without `home` loads with no diagnostic; a layout with `home` round-trips equal.

Frame tests (new `standalone/crates/om9-app/tests/panels_reorder_collapse_float.rs`, real `MatrixApp::show`, synthetic
pointer events: press, move +10 px past the drag threshold, move to target, release; 1600x1000):
- AC-3: drag Projects onto the upper half of the Display bar. Only Projects' order changes, and all other
  `SectionState`s are equal.
- AC-4: click Projects twice, `SectionState` equal to the original; Snaps collapsed → `closed_bar_controls` contains
  `snaps`.
- AC-5: dock right by drag, then ◀, field-by-field restore; float by drag, then close (DockHome), field-by-field restore.
- AC-6: each `negative.json` drop or shrink case. The text appears in the status line and CMD history. During a region
  error (resize to 500x250) the stored rect is unchanged.
- AC-7: drop on the toolbar, Escape, Builders click, `move_menu_section` of a floating section, re-float, `dock_menu_section(_, Top)`.
  Each one: typed error in `last_panel_op`, visible status, `to_json()` unchanged.
- AC-8: `drop_preview` is `Some` on pressed frames, with `target` equal to the applied result, and `None` after release
  or Escape.
- AC-9: a snapshot of entity count, entity ids, current layer, undo/redo length, draft, `controller.is_running()` and
  `controller.workspace` (views, active, maximized) is equal before and after every scenario. With a temp
  `set_dock_layout_path`, the file is still absent afterwards (FR-13).
- AC-10: a drag left in progress across `reset_menus`, `reload_menus` and a hidden section ends with
  `section_drag() == None`, no panic, and a release that applies nothing.
- Source-scan test (§8).
- D6 Layers: `matrix_default` pins L1. A frame sweep runs the default layout over widths
  `{800, 1024, 1280, 1366, 1600, 1920, 2560}` × heights `{400, 432, 480, 600, 720, 768, 900, 1080, 1440}` (only pairs the
  region plan accepts) at zoom 1.0, plus SIMULATED zoom 1.5 for `1280x800`, `1600x1000`, `1920x1080`. Each frame asserts
  L2 from `section_rects` and `sidebar_rect`. A drag of Layers into a view followed by a close (`DockHome`) returns it
  directly below Info & Settings (L3, no home: dock right from the API on a fresh layout, then dock left). The audit hook
  test asserts `OM9_UI_AUDIT_RIGHT_DOCK` docks Projects and leaves Layers left (L5): the hook body is a small
  `pub(crate) fn apply_audit_right_dock(&mut self)` that the test calls directly.

RED plan (L4-02, before the implementing diff): add `panel_ops.rs` with all types. `WorkArea` is complete, and
`apply_panel_op` delegates to the **current** primitives (old semantics: no home, no work-area clamp on the float path,
Builders allowed). `resolve_drop` returns `Cancel { NoTarget }`, `drop_indicator` returns `None`, there is no Escape
branch, and `drop_preview` stays `None`. Use no `todo!`/`unimplemented!`, so tests fail on assertions, not panics. Run
`cargo test -p om9-app --lib panel_ops` and `--test panels_reorder_collapse_float`. Capture exit ≠ 0 with assertion
failures and no compile error into `implementation/evidence/M01/OM9-L3-01-01-003/OM9-L4-01-01-003-02.red.log`.

Existing tests: the assertions are expected to stay valid. `shell_docking::menus_dock_to_the_right_mirror_column_and_back`
keeps its asserts; the stale comment "Re-docking appends the menu" becomes "Re-docking restores the menu's home
position". `clamp_floats_recovers_menus_dragged_outside_the_window` still holds under the tighter work area.
`a_floated_menu_keeps_its_rectangle_and_docks_back` floats at `(600, 300, 320, 240)` inside the 1280x900 work area, so it
is unchanged. Any assertion L4-03 still has to change is listed with its reason in `OM9-L4-01-01-003-03.diffstat.txt`.

## 11. Edge cases

- **Narrow window.** Below `NARROW_WINDOW_WIDTH` (now 1200 after the concurrent L3-002 close-out edit; read the
  constant, never a literal), no right strip is offered. A "Dock right ▶" there is allowed and the L3-002
  `RightDockYielded` note explains why the column is not drawn. The section is kept in the layout.
- **Layers docked right by the user, then the window narrows.** The L3-002 yield applies. The right column and Layers
  are not drawn, and the `RightDockYielded` note says so. Layers is not moved automatically (D6 L4). Dock left, or Reset
  Menus, brings it back below Info & Settings.
- **A float dragged onto a second monitor** → D2 deviation message. A float window dragged by its title bar stops at the
  work-area edge.
- **First frame.** `work_area` is `None` → float ops return `NoWorkArea`. Tests always run two frames first.
- **Re-float after dock.** egui memory still holds the old Area. The resync override (D1 point 4) moves and sizes it to
  the new rect on the next frame.
- **Readback outer rect.** `FloatRect` stores the outer window rect (as today). After one frame the stored size is within
  24 px of a requested content size. The fixtures state this tolerance.
- **High DPI / zoom.** All values are logical points. `OM9_UI_AUDIT_ZOOM` cases are SIMULATED, not real per-monitor DPI.
- **Concurrent working-tree edits.** `ui.rs`, `ui/chrome.rs`, `dock_layout.rs` and `workbench_regions.rs` carry
  uncommitted L3-002 close-out changes (minimum window 1280x720, `PLAN_FLOOR_WINDOW` 800x400). L4-03 re-reads them and
  must not revert them (NFR-4).

## 12. Absent tooling and unverified items

- `tools/validate_m01_spec.py`, `validate_m00_execution.py`, `next_m01_task.py`, `m01_capture.py`,
  `m01_init_evidence.py`, `validate_m01_execution.py` do not exist: **not run**.
- `om9.m01.oracle.v1.5` has no local definition and is carried as a field only.
- `OpenMatrix9_Codex_Spec_v2/` (including MATRIX9_UI_CONTRACT) is absent locally. The behaviour follows the L2 reference
  "Changing the Menu Layout" (book 1 p.11-13), `dock_layout.rs` and the workspace-contract skill.
- egui behaviour cited from the 0.36.2 registry source, not observed in a real window yet: `constrain_to`,
  `fixed_size`/`current_pos` resync, and pointer events delivered outside the OS window during a drag (winit capture).
  The last one is covered by `last_pointer` and `PointerLost`. All of these go into the M01-UI human checklist.
- Business-rule changes (D1 work area, D2 deviation, D3 home + roll-up kept + documented re-dock position, D4 Builders
  refused, D5 Escape, preview, cancel messages, non-collapsible float window, D6 Layers placement and the audit hook
  docking Projects) need the user's consent at the end, before any skill text changes. D6 behaviour itself is authorised
  by the user's 2026-10-16 instruction. No skill file is edited.
- D6 L2 is only verified at sizes the frame sweep and the real-window matrix actually run. Real 150% DPI and a second
  monitor stay on the M01-UI human checklist.
