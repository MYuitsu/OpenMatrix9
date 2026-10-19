# `OM9-L3-01-01-003` Reorder Collapse And Float Panels + Matrix9 main-window parity: technical design (revision 4)

Owner `OM9-L2-01-01` · Milestone `M01` · L4s `OM9-L4-01-01-003-01`, `-02`, `-03`, `-05` · Manual gate M01-UI: **OPEN**
Workflow `wf_9fe91738fb13ff99` (revision of the stopped, unreviewed run `wf_c6ba092eed7e7c12`).
Legal reuse of QymCAD: **HUMAN_APPROVED_2026-10-16 (user: "M00 duyệt")**, QymCAD code reuse only (user, 2026-10-16:
"chỉ tái sử dụng QymCAD trước, còn tới phần nào khác thì hỏi tôi tiếp"). Other M00 parts are not approved.
OM9-L2-01-02 / OM9-L2-01-03: **BLOCKED_DEPENDENCY** (unimplemented modules).

This file is the binding design. Where it conflicts with `api.md`, `positive.json` or `negative.json` from the first
run, this file wins, and §12 lists the exact edits those files need. Revision 3 resolved the twelve findings of the
first review; revision 4 resolves the fourteen findings of the second `design-review.md` (§17.2).

## 0. Inputs, baseline and precedence

Read for this revision (2026-10-16, 14:24 local):

- `design-review.json` / `design-review.md` (second review: CHANGES_REQUESTED, 0 HIGH, 6 MEDIUM, 8 NIT; the first
  review's 3 HIGH / 4 MEDIUM / 5 NIT are answered in §17.1).
- `.agents/tasks/m01-l3-002-docking/left-dock-measurements.md` (FEAT-001 and FEAT-002 sections) and
  `standalone/crates/om9-app/tests/left_stack_layers.rs`, `tests/common/mod.rs` (FEAT-002).
- `.agents/tasks/m01-l3-003-panels/user-decisions.md` (all four sections), `requirements.md` (G1-G8, D1-D5).
- `docs/analysis/matrix9-main-window-layout.md`, steering `.kiro/steering/matrix9-main-window-layout.md`.
- The concurrent task `.agents/tasks/m01-l3-002-docking/task-m01-l3-002-left-dock-icons/` (`task.json`,
  FEAT-001..003) and its plan `.agents/tasks/m01-l3-002-docking/plan.md`.
- Working tree of `standalone/crates/om9-app` (`dock_layout.rs`, `workbench_regions.rs`, `ui.rs`, `ui/chrome.rs`,
  `ui/command.rs`, `tests/*.rs`), `tools/validate_m01_l3_002_ui_matrix.py`, L3-002 and L3-003 fixtures,
  `skills/openmatrix9-workspace-contract/references/command-and-curve.md`, egui 0.36.2 `containers/panel.rs`.

**Tree observed while writing revision 4** (14:24 local). The concurrent L3-002 task keeps editing the same files, so
these values are informative only. **The binding baseline is re-captured at P0** (§2.1): L4-03 records `HEAD`, the
SHA-256 of `git diff -- standalone/crates/om9-app implementation/fixtures/M01 tools/validate_m01_l3_002_ui_matrix.py`
and the per-file SHA-256 of every file in §6 in its diffstat note before the first edit.

| Item | Observed value |
| --- | --- |
| `HEAD` | `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a` |
| `dock_layout.rs` / `ui.rs` / `ui/chrome.rs` file SHA-256 | `5E15FD51…D510` / `6F924EA0…3597` / `51D7B4C0…8957` |
| `tests/left_stack_layers.rs` (new, FEAT-002) SHA-256 | `621BD7D0…EB75` |
| `tools/validate_m01_l3_002_ui_matrix.py` (untracked) SHA-256 | `8D97D7AC…E06D` (still sets `OM9_UI_AUDIT_RIGHT_DOCK=1`, no `--mode`; FEAT-003 rewrites it) |
| `task-m01-l3-002-left-dock-icons` | `task.json` `pending`; FEAT-001 `completed` (production code: constants, audit modes, `dock_receipt`, drop zone, outer rail width); FEAT-002 `completed` (test migration to `dock_right_fixture`, `tests/left_stack_layers.rs`, `left_rail_max_justification_table`); FEAT-003 (validator `--mode`, `--self-test`) `pending`; `left-dock-measurements.md` present (FEAT-001 and FEAT-002 sections) |

Precedence when sources disagree: the newest explicit user decision first (steering, then `user-decisions.md`),
then the screenshot analysis, then the step brief, then the first-run artifacts.

**Left column width (finding 5).** Three sources:

- The capture reads about 214 px of a ~1024x608 image. The analysis itself says the pixel values are "evidence for
  proportions at that capture size only". A 1024x608 Matrix 9.0 main window is below any practical working size, so
  the image is very likely downscaled. 214 px is therefore **not used** as a logical width. This stays an assumption
  (A1) until a second capture exists.
- The Matrix8 manual (book 1 p.13, `OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell
  Docking/reference/text/m8book1__p11-13__a-tour-of-the-interface.md`, lines 104-108) says a menu dragged to the far
  right of the screen docks in a 300-pixel-wide menu that "mirrors its counterpart" on the left. That sentence is
  about the **right** column. The left value of 300 is **inferred** from "mirrors its counterpart", not stated.
  (Only the short phrase is quoted; the manual text is not copied into the repo.)
- The **user decisions** that settle it are the steering lines: "Left column: default width 300 px, user can widen it
  (resizable) so SVG icons can be larger; icon size should follow the widened column" and "User decision
  (2026-10-16): keep the right dock, but it is hidden by default and appears only after the user drags panels to it".
  `user-decisions.md` §4 confirms that the steering is authoritative.

**Decision:** 300 logical points by default, never proportional to the window, scaled only by DPI, user-widenable.
The final report lists the widenable column and the user-docked right dock as **OM9 additions the user decided**,
quoted as above, and does not ask again. The manual sentence also shows that drag-to-the-far-right docking is
documented Matrix behaviour, which supports keeping the right-dock code.

Other steering lines this design follows: right-dock fixtures use **Snaps** and are labelled user-docked; Layers (or
any panel) is never docked right in audits, fixtures or parity screenshots; parity is required across ≥ 1280x720
logical plus 1366x768 capped by the work area; no proportional compression.

## 1. Overview

One change set, two parts, one crate (`om9-app`), safe Rust, egui/eframe 0.36.2.

Part A, **main-window parity**, makes the existing window match the Matrix9 capture where it still differs. Region
stacking is already right: menu bar, a full-height left column, and a right workspace holding the green command box at
the top, the toolbar strip, the icon row, the viewport tabs and the 2x2 grid. Part A fixes the left-column minimum,
the command-box height (two history rows plus one input row), title-bar style, column scrolling (G9), text-row
clipping, the toolbar `>>` overflow, the viewport `+` tab and the right-dock View-menu item. It consumes, and does not
redefine, the rail/icon/audit work of the concurrent L3-002 task (§2.1).

Part B, **L3-003 mechanics**, adds one typed, egui-free operation module (`panel_ops.rs`): roll-up, drag reorder,
dock left/right, float, dock back with home restore, live drop preview, drag cancel and off-screen recovery, each one
path shared by mouse, buttons, API and audit.

Panel layout is UI state only. Nothing here reads or writes document objects, IDs, layers, units, selection, the
command draft, the Undo/Redo journal, the scene or the render cache. No backend-owned crate or file is edited (§13).

## 2. What exists today (read, not assumed)

Already right, and kept:

- Region order. `ui.rs::show` declares `Panel::top("menu")` (21), `Panel::left("matrix-sidebar")`,
  `Panel::top("command")` (green `rgb(130,180,140)`, inner margin 3, resizable `COMMAND_MIN`..`COMMAND_MAX`, default
  `COMMAND_DEFAULT` 90), `Panel::top("matrix-toolbars")` (exact 67), the optional `Panel::right("matrix-right-dock")`,
  `Panel::bottom("matrix-status")` (20) and `CentralPanel` (2x2 grid). The command box is at the top of the right
  workspace.
- Grid order TL Looking Down, TR Perspective, BL Through Finger, BR Side View; view-tab order Perspective, Through
  Finger, Side View, Looking Down.
- Left order `SectionId::ALL` = IconHistory, MainMenu, Display, Snaps, InfoSettings, **Layers**, Projects,
  BuilderSlot, all `DockSide::Left`, order = index, so Layers (5) = InfoSettings (4) + 1 and Projects = 6. No section
  needs a placeholder. `bar_title()` is UPPERCASE.
- 1280x720 minimum (L3-002 close-out): `SUPPORTED_MIN_WINDOW` 1280x720, `PLAN_FLOOR_WINDOW` 800x400,
  `startup::effective_min_size`, `AppConfig::validate`, `enforce_min_window`, `tests/window_minimum.rs`. No `560`/`286`
  rule remains. `NARROW_WINDOW_WIDTH` = 1200.
- Done by `task-m01-l3-002-left-dock-icons` FEAT-001 (in the tree, consumed here, not redefined):
  `LEFT_RAIL_MAX = 460` (compile-time `1200 >= 460 + 420 + 320`), outer rail width stored
  (`left_rail_outer_px`, `LEFT_RAIL_CHROME_PX = 5`), `ICON_BASE_EDGE 22` / `ICON_MIN_EDGE 18` / `ICON_MAX_EDGE 34`
  with `rail_icon_edge(outer)` (rounded, clamped) and `column_icon_edge` used by `icon()` in the sidebar and right
  dock only, `RIGHT_DOCK_FIXTURE = SectionId::Snaps`, `RIGHT_DROP_ZONE_WIDTH = 28` with `compute_right_drop_zone` /
  `right_drop_zone_ui` and the release branch in `settle_section_drag` (plus "Right dock needs a window at least 1200
  points wide" below 1200), `AuditLayoutMode` / `OM9_UI_AUDIT_LAYOUT_MODE=default|user-docked-right` (the old
  `OM9_UI_AUDIT_RIGHT_DOCK` now fails the launch), `OM9_UI_AUDIT_RAIL_WIDTH` (bounded by `LEFT_RAIL_MIN..=MAX`), and
  `dock_receipt()` (`left_sections`, `right_sections`, `right_dock_visible`, `right_dock_sections`,
  `left_rail_outer_px`, `left_rail_inner_px`, `left_icon_edge_px`, `left_icon_rects_in_sidebar`,
  `left_stack_content_px`, `sidebar_scroll_offset`, `section_title_bars_px`, `layers_rect_px`, …).
- Done by the same task's FEAT-002 (in the tree, consumed here): every integration-test right-dock fixture uses
  `tests/common/mod.rs::dock_right_fixture` (Snaps; S6-S9 done), shared `DOCUMENTED_LEFT`, `run_frame`, `settle`,
  `assert_layers_between_info_and_projects`; new `tests/left_stack_layers.rs` (8 tests, among them
  `layers_title_bar_is_visible_without_scrolling_at_1280x720` at rail 300 and 460 and
  `icon_edge_follows_the_rail_width`); `tests/workbench_regions_sizes.rs::left_rail_max_justification_table`.
- **Measured left-stack geometry** (`left-dock-measurements.md`, before any L3-003 change; title bar height 14):
  sidebar inner rect y 23..718 at 1280x720 (FEAT-002 value; FEAT-001 wrote 721, the same frame with the 3 pt
  bottom margin), scroll 0. Bar y at rail 300: Icon History 23, Main Menu 63, Display 160, Snaps 219, Info & Settings
  278, **Layers 337..351**, Projects 527, Builders 675. Rail 460: **Layers 397..411**, Projects 587. The bar y values
  are identical at 1280x720 and 1600x1000, so they do not depend on window height. Stack content height 821 (rail 300)
  and 893 (rail 460), taller than the column, so the bottom is clipped/scrolled.
- Right dock hidden by default: `PanelId::RightDock` defaults to `visible: false`; `resolve` shows the column only
  when a visible section is docked right and `width >= 1200`.

Gaps. G1-G8 (`requirements.md` §1.2) stay valid: no drag preview, silent no-ops, stringly typed errors, re-dock
appends and expands, recovery only on resize, no drag cancel, no permitted-panel rule, no UI-state-only proof. New:

- **G9, the left column cannot scroll (defect, still present).** `chrome.rs::sidebar_ui` does
  `let scroll = std::mem::take(&mut self.sidebar_scroll)`, passes `.vertical_scroll_offset(scroll)` every frame, then
  sets `self.sidebar_scroll` from temp key `"matrix-sidebar-sections-offset"`, which nothing writes. The offset is
  forced to 0 every frame, so wheel/drag scrolling is undone and Projects/Builders below the fold are unreachable.
- **G10, title-bar style.** `section()` paints a light gradient `rgb(199,199,186)`→`rgb(151,151,141)` with black text
  and a `+`/`−` glyph. Matrix9: dark bar, UPPERCASE light title, small close box at the right.
- **G11, the left minimum is not Matrix9.** `LEFT_RAIL_MIN = 240` lets the column go narrower than the Matrix default.
  Steering allows only widening.
- **G12, text rows wrap.** Main Menu sub-menu/category rows and the Snaps chip row use `horizontal_wrapped`; Matrix9
  clips them. The sub-menu row lacks `Reset`.
- **G13, no toolbar `>>` overflow and no viewport `+` tab.**
- **G14, View ▸ "Open/Close Right Menu Column"** calls `toggle_right_dock`, which flips `PanelId::RightDock.visible`;
  `resolve` ignores that flag, so the item reports "opened" and shows nothing. The remaining non-neutral right-dock
  fixtures are listed in §10.1.
- **G16, command box too tall.** The approved rule (`command-and-curve.md`, approved 2026-10-06) is "Default height is
  two history rows plus one live input row, based on font metrics". `command.rs` uses
  `row = monospace_row + 2` and `history = area - row - 2`; at `COMMAND_DEFAULT = 90` that is about four history rows.
  The code is out of line with the rule and with the two-line box in the capture.
- (G15 of rev 2, "icons do not follow the column", is closed by the L3-002 task and removed from this design.)

### 2.1 Ownership split with `task-m01-l3-002-left-dock-icons` (finding 1)

| Area | Owner | L3-003 does |
| --- | --- | --- |
| Outer rail width storage, `LEFT_RAIL_MAX` 460, `LEFT_RAIL_CHROME_PX`, `rail_icon_edge` + `ICON_*_EDGE`, `column_icon_edge` | L3-002 left-dock task | consume only |
| `RIGHT_DOCK_FIXTURE`, `tests/common/mod.rs::dock_right_fixture`, migration of every **Layers**-right test fixture, `tests/left_stack_layers.rs` | L3-002 left-dock task (FEAT-002, completed) | verify only (§10.1 rows marked L3-002); S20/S29 edit two `left_stack_layers.rs` / `workbench_regions_sizes.rs` rows because of `LEFT_RAIL_MIN` 300 |
| `OM9_UI_AUDIT_LAYOUT_MODE`, `OM9_UI_AUDIT_RAIL_WIDTH`, `dock_receipt()` | L3-002 left-dock task | add new keys in a separate `layout_parity` / `panels` object; never rename or remove `dock` keys |
| `RIGHT_DROP_ZONE_WIDTH` 28 and its "no zone below 1200 pt" rule, `compute_right_drop_zone`, `right_drop_zone_ui` | L3-002 left-dock task | feed into `DropZones` (§5.4); `settle_section_drag` is replaced by the resolver with the same zone width and the same below-1200 message |
| `tools/validate_m01_l3_002_ui_matrix.py` (`--mode`, `--self-test`, cases) | L3-002 left-dock task | call it as a subprocess (§10.4); one case removal (S21) after P0 |
| `LEFT_RAIL_MIN` | shared constant | **L3-003 changes 240 → 300** after P0 (decision below) |
| Non-Layers right-dock fixtures (Projects, Main Menu), `COMMAND_DEFAULT`/`COMMAND_MIN`, command sync, G9-G14, Part B | L3-003 | implement |

**Sequencing precondition P0 (binding).** L4-01 (artifact edits) and L4-02 (new RED test files only:
`tests/panels_reorder_collapse_float.rs`, `tests/main_window_parity.rs`, `src/panel_ops.rs` tests) may start now.
L4-03 must not edit `dock_layout.rs`, `workbench_regions.rs`, `ui.rs`, `ui/chrome.rs`, existing tests, L3-002 fixtures
or the L3-002 validator until `task-m01-l3-002-left-dock-icons/task.json` shows FEAT-001..003 done (at 14:24 FEAT-001
and FEAT-002 are `completed`, FEAT-003 is `pending`, so P0 is not met yet). If it is not done, L4-03 stops and reports
to the orchestrator (no partial edits). When P0 holds, L4-03 first re-runs the §17 spot check and the §10.1 re-grep
against the new tree, records the baseline of §0 (HEAD, diff SHA-256, per-file SHA-256) in its diffstat note, and only
then edits.

**`LEFT_RAIL_MIN` decision: option (b), 300.** Steering says "default width 300 px, user can widen it"; nothing
permits narrowing. `LEFT_RAIL_MIN = LEFT_RAIL_DEFAULT = 300`. This supersedes these L3-002 assertions (listed with the
reason in §10.1 S20-S22): the validator case `rail240-1280x720`, the `left_stack_layers.rs` icon row at rail 240, and
the shell-docking bounds test value, the `icon_edge_follows_the_rail_width` loop/anchor and the rail-240 rows of
`left_rail_max_justification_table` (S29). `rail_icon_edge` itself is **not** edited (it depends only on
`LEFT_RAIL_DEFAULT`, not on `LEFT_RAIL_MIN`): its pure-function anchor `rail_icon_edge(240.) == 18` (written with the
literal, not `LEFT_RAIL_MIN`) and `ICON_MIN_EDGE` stay as tested math, now unreachable from the UI because the rail
cannot go below 300 (UI icon range becomes 22..34). The final report states (b) as an **agent choice for the user to
confirm**.

**Flag for the orchestrator, not acted on.** FEAT-003 of the L3-002 task says "legal PENDING_HUMAN_M00_APPROVAL
unchanged" for L3-002, but `user-decisions.md` §1 records the user's M00 approval of QymCAD reuse for sibling L3-002
too. L3-003 does not copy the PENDING status.

## 3. Technology stack (locked)

- Rust 2024 crate `standalone/crates/om9-app`, `#![forbid(unsafe_code)]`, egui/eframe **0.36.2** (wgpu), `serde` /
  `serde_json`. No new crate or feature.
- New modules: `src/panel_ops.rs` (pure, egui-free, IO-free) and `src/ui/panel_audit.rs` (audit-only pointer replay,
  same pattern as `ui/viewport_audit.rs`).
- No C++, no FFI, no Win32 call. Python only for the **required** real-window launcher
  `tools/validate_m01_l3_003_panels.py` (§10.4). Rust-first exception, recorded: it drives a real GPU window and
  external processes, cannot run inside `cargo test`, and holds no business logic (same justification as the L3-002
  launcher).
- No QymCAD build-time dependency. Ideas only from `vendor-research/QymCAD` @ `746a17f` (home overlay, one door,
  source-scan test). Any copied line carries the `QymCAD/LICENSE` and `QymCAD/THIRD-PARTY-NOTICES.md` notices.

## 4. Main-window layout model (Part A)

All sizes are **egui logical points**; physical = points × `ctx.pixels_per_point()`. DPI is the only scaling. Panel
sizes in egui 0.36.2 are **outer** sizes (`Panel::min_size` writes `outer_size_range`, `containers/panel.rs:381`).

### 4.1 Region stacking

`compute_regions` logic and the panel declaration order are unchanged. Only the command height `C`
(`COMMAND_DEFAULT`, §4.3) moves the rects below it. At 1280x720, default layout: menu `[0,0,1280,21]`, left
`[0,21,300,699]`, command `[300,21,980,C]`, toolbar `[300,21+C,980,67]`, center `[300,88+C,980,612−C]`, status
`[300,700,980,20]`, views `489 x (610−C)/2`. With the expected `C = 56` (§4.3): toolbar y 77, center y 144 / height
556, views 489x277. No right dock.

### 4.2 Left column

**Placement.** One `Panel::left("matrix-sidebar")` holding one vertical `ScrollArea`. Docked-left sections are drawn in
`DockLayout::left_sections()` order; hidden sections are skipped but keep their order slot. Factory order is
`SectionId::ALL`. D6 (§5.3) protects Layers at index 5, directly below Info & Settings and above Projects; Builders is
last.

**Width.**

- `LEFT_RAIL_DEFAULT = 300.`, `LEFT_RAIL_MIN = 300.` (was 240, §2.1), `LEFT_RAIL_MAX = 460.` (L3-002 task). The user
  widens the column by dragging its right edge (`resizable(true)` kept) and cannot make it narrower than 300.
- The stored width is the outer width (L3-002 task, already in the tree).
- `DockLayout::normalize`: inside the existing `PanelId::ALL` loop, the `LeftRail` check runs **before** the generic
  `size_bounds` clamp: a finite stored `size < LEFT_RAIL_MIN` pushes the diagnostic
  `"left menu column width {w} is below the Matrix default 300; widened to 300"` and then the clamp applies. A
  non-finite size keeps today's reset-to-default path. The diagnostic reaches status and CMD history through
  `note_dock_diagnostics`. `set_size` keeps clamping silently (model primitive; its only UI caller is
  `sync_panel_size`, which egui already bounds by `min_size`).
- `parse_audit_rail_width` keeps reading `LEFT_RAIL_MIN..=LEFT_RAIL_MAX`, so `OM9_UI_AUDIT_RAIL_WIDTH=240` becomes a
  launch error (unit-tested, S21).
- `800 >= LEFT_RAIL_MAX + CANVAS_MIN_WIDTH = 780` still holds, so `resolve` never narrows the rail;
  `RegionNote::LeftRailShrunk` stays an unreachable, tested dead note.

**Icons.** No new icon rule. Sidebar and right-dock icons use the L3-002 task's `rail_icon_edge(outer)`; with the 300
minimum the UI range is 22 (300) .. 34 (460). Toolbars, floating windows, F6 and viewport chrome stay at 22.

**Scroll and clip (fix G9).** In `sidebar_ui`:

- Remove `std::mem::take(&mut self.sidebar_scroll)` and the dead `"matrix-sidebar-sections-offset"` read.
- Keep `ScrollArea::vertical().id_salt("matrix-sidebar-sections").auto_shrink([false, false])`. Apply
  `.vertical_scroll_offset(v)` only when a new private `pending_sidebar_scroll: Option<f32>` is `Some(v)`, then clear
  it.
- After `show`, set `self.sidebar_scroll = scrolled.state.offset.y` (the existing field keeps feeding
  `dock.sidebar_scroll_offset`) and keep `self.sidebar_content_size = Some(scrolled.content_size)`.
- `pub fn request_sidebar_scroll(&mut self, y: f32) -> bool` (finding 8): non-finite `y` → `false`, nothing stored;
  otherwise stores `Some(y.max(0.))` and returns `true`. egui clamps to the content on the next frame.
- `pub fn sidebar_scroll(&self) -> f32` returns the field.
- Sections are never compressed. The stack runs past the window bottom and is scrolled (wheel, scroll bar) or clipped.

**Title bar (fix G10).** `section_bar(&mut self, ui, id, side: ColumnSide) -> bool` replaces `section()` and the
right column's `selectable_label`, so both columns draw the same bar:

- Height 14 (unchanged), full available width.
- Fill: vertical gradient `Color32::from_gray(64)` → `from_gray(38)`, 1 px bottom line `from_gray(20)`.
- Title `id.bar_title()` at `left_center + (4, 0)`, `FontId::proportional(11.)`, `from_gray(235)`, clipped to the bar
  minus the close box. No `+`/`−` glyph. Snaps and Layers keep their closed-bar controls (`section_bar_controls`).
- Close box (the seven menus only; the BUILDERS bar has **no** close box, because the slot is not a menu and has no
  close semantics): 10x10 rect centred at `right_center − (8, 0)`, stroke and `×` glyph `from_gray(150)`, its own
  `ui.interact(close_rect, Id::new(("om9-bar-close", key)), Sense::click())`, hover
  `"Closing a menu is not available yet; click the title to roll it up"`. A click runs `PanelOp::Close { section }`,
  which always returns `Err(CloseUnsupported)` (A5). `section_bar` draws it only when `id.is_movable()`.
- Bar sense: `Sense::click_and_drag()` when `id.is_movable()`, else `Sense::click()`. A bar click inside `close_rect` is
  ignored by the bar.
- Rects recorded: `section_rects[key] = bar_rect` (existing, feeds `dock.section_title_bars_px`), new
  `section_bar_sides[key] = side`, and new `section_blocks[key] = (bar.top, body_bottom)` (window-space y, from
  `ui.min_rect()` after the body).

**Text rows (fix G12).** Helper `fn clipped_row(ui, height, add)`:

```text
let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::hover());
ui.scope_builder(UiBuilder::new().max_rect(rect).layout(Layout::left_to_right(Align::Center)), |ui| {
    ui.set_clip_rect(rect.intersect(ui.clip_rect()));
    add(ui);
});
```

The parent allocation is exactly the column width, so `sidebar_wrapping_and_toolbar_scrolling_keep_a_240px_allocation`
still holds (S12). Rows: the Main Menu sub-menu row (height 16; `File, View, Utilities, Measure, Custom` disabled,
plus a disabled `Reset` with hover `"Reset (Panic) belongs to OM9-L3-01-01-004; not available yet"`), the two Main
Menu category rows (height 14 each), the Snaps value-chip row (height 18). Icon rows keep `horizontal_wrapped`.

**Builders.** The BUILDERS slot stays last, same dark bar, not movable (D4), body unchanged (L3-008).

### 4.3 Right workspace

**Command box (fix G16, finding 3).** Position, colour, transcript `ScrollArea`, `TextEdit` and the pinned input row
are unchanged.

- `COMMAND_DEFAULT` becomes the font-metric value for two history rows plus one input row, as an **outer** panel
  height: `C = ceil(3 * (r + 2) + 2 + 2 * 3)`, where `r` is `fonts.row_height(&FontId::monospace(12.))` (the style
  `chrome.rs` installs) at `pixels_per_point = 1.0`, `+2` is `command.rs`'s row padding, `2` its separator gap above
  the input row and `2 * 3` the frame inner margin. L4-02 measures `r` once in a frame test and writes `C` as a
  literal constant. Expected `r ≈ 14`, `C ≈ 56`; the measured value is authoritative.
- `COMMAND_MIN = min(58, C)`. With the expected value that is `C`, so the box can never show fewer than two history
  rows. User resizing up to `COMMAND_MAX` 400 stays (approved).
- **"Two-line" reading (A15).** The steering and the capture describe a "green two-line `Command:` box", about 30 px
  tall in a capture that is downscaled by about 0.71 (214 px → 300 pt), i.e. about 42 pt. This design reads
  "two-line" as **two history rows above the pinned live input row**, per the approved `command-and-curve.md` rule
  (2026-10-06, "two history rows plus one live input row, based on font metrics"). The rule wins because it is an
  explicit approved business rule; the capture height is an estimate from a downscaled image. Expected C ≈ 56 is
  larger than the ≈ 42 estimate. The final report shows the measured `C` next to the capture estimate and asks the
  user to confirm the reading. **C is not tuned toward 42 without that answer.**
- **Command size sync (unconditional).** Today `ui.rs` (`Panel::top("command")`, ~line 1019) syncs
  `ui.max_rect().height()` inside a frame with `inner_margin(3)`, so the stored size is always outer − 6; with
  `COMMAND_MIN = C` that inner value is then clamped up to `C` by `set_size`, so stored and drawn sizes would agree
  only by accident. L4-03 therefore always keeps the response,
  `let command_panel = egui::Panel::top("command")…show(ui, |ui| { ui.set_min_height(ui.available_height()); self.command_ui(ui); });`,
  removes the inner `sync_panel_size` call and, after `show`, calls
  `self.sync_panel_size(PanelId::CommandLine, command_panel.response.rect.height())`, the same fix the L3-002 task
  made for the rail. Stored, planned and drawn heights then agree within 0.5 (S26 becomes exact).
- **Saved sizes.** A stored `CommandLine.size` (e.g. 90 from a Set-as-Default file) is inside
  `[COMMAND_MIN, COMMAND_MAX]`, so `normalize` keeps it. It is user state and is not rewritten; that user still sees
  about four history rows. Reset Menus (L3-004 path, `reset_to_default`) gives `C`.
- Guard test `command_default_matches_font_metrics` recomputes `C` from the live font and asserts equality, so a font
  change cannot silently break the rule.
- **Row metric.** `command_history_rows() = floor((history_height + 0.5) / (r + 2))`, where `history_height` is the
  `history_height` value `command.rs` computes in the last frame (`area.height() − row_height − 2`) and `r` is the
  font row height it computes on the same line (`ui.fonts_mut(|f| f.row_height(&TextStyle::Monospace.resolve(ui.style())))`,
  i.e. its `row_height − 2`) at the live zoom. `command.rs` stores both in new private fields
  `command_history_height` / `command_font_row` (no layout change). The `+ 0.5` absorbs physical-pixel rounding (a 2·(r+2) − 0.01 height still counts as 2). It is a
  proxy for the visible row count (the galley rows are `r` high; `+2` is `command.rs`'s row padding). Assertions:
  `== 2` at zoom 1.0 with the default height; `>= 2` at 1.25 and 1.5. A shortfall at 1.25/1.5 is reported to the
  user; the constant is not tuned per DPI.
- Every moved rect and fixture is listed in §10.1 (S23-S28).

**Toolbar tab strip with `>>` (fix G13).**

- `toolbars_ui` lays the strip out in `ui.horizontal(..)`: the existing `ScrollArea::horizontal` with
  `.max_width(ui.available_width() - OVERFLOW_W)` (`OVERFLOW_W = 22.`), then `ui.menu_button(">>", ..)`, drawn only
  when `self.toolbar_tabs_overflow`.
- `toolbar_tabs_overflow = output.content_size.x > output.inner_rect.width() + 0.5`, set after the `ScrollArea`, so it
  applies on the next frame (no layout cycle).
- The menu lists every tab whose recorded rect (`toolbar_tab_rects[i]`) is not fully inside `output.inner_rect`.
  Clicking one sets `toolbar_tab = i`, `focus_cmd = true`, `toolbar_tab_scroll_to = Some(i)`; next frame that tab's
  response calls `scroll_to_me(Some(Align::Max))` and clears the request.
- TABS content and order unchanged. Default tab stays `Standard` (A10).

**Icon row.** One row for the active tab, 22-point icons, unchanged.

**Viewport tabs + `+` (fix G13).** After the four tabs, `egui::Button::new("+").min_size(vec2(17., 17.))` with the
inactive fill and weak text. Click → `status.warning("New viewport tabs are not available yet")`, the same text in
CMD history, focus back to CMD, no layout or workspace change. Same hover text.

**2x2 grid.** Unchanged (L3-002 / L2-03-01).

### 4.4 Right dock: kept, hidden by default, user-docked only

- Code kept: `Panel::right("matrix-right-dock")`, `right_dock_ui`, `RIGHT_DOCK_*`, `NARROW_WINDOW_WIDTH = 1200`,
  `RegionNote::RightDockYielded`. The default layout has no right section, so no right dock is drawn at any size.
- **Entry points (finding 12)**, both explicit user actions, both through `apply_panel_op`:
  1. dragging a menu title bar and releasing it in the right drop zone (`RIGHT_DROP_ZONE_WIDTH` 28, painted only during
     a drag while the dock is hidden and the window is ≥ 1200 points, L3-002 task) or over the shown right column;
  2. the existing "Dock right ▶" button of a floating menu window.
- View menu (G14): when `right_sections()` is empty, the item reads `"Right Menu Column"`, disabled, hover
  `"Drag a menu title bar to the right edge of the viewports to open the right menu column"`. Otherwise it reads
  `"Close Right Menu Column"` and calls `pub fn close_right_menu_column(&mut self) -> usize`, which applies
  `PanelOp::Dock { section, side: Left }` to each right section in order (home or documented position, D3), reports
  `"Right menu column closed; {n} menus docked left"` (Info) and returns `n`. `toggle_right_dock` stays public for its
  existing test caller; the menu no longer calls it.
- The yield below 1200 matters only for a user-docked column on capped high-DPI screens. Unchanged.
- Audit: `OM9_UI_AUDIT_LAYOUT_MODE=user-docked-right` (L3-002 task) docks `RIGHT_DOCK_FIXTURE` (Snaps). L3-003 adds
  `receipt["layout_parity"]["fixture"]` = `"default"`, `"user-docked-right:snaps"`, or
  `"scenario:<name>"` for `OM9_UI_AUDIT_PANELS` runs (the `dock-right` scenario docks Snaps by a replayed drag, label
  `"scenario:dock-right (user-docked snaps)"`), so no such screenshot is presented as the Matrix9 default.

### 4.5 Minimum window

No new code. Supported range: ≥ 1280x720, plus 1366x768 capped by the work area (`effective_min_size`: 1280x688 at
100%, 1076.8x534.4 at 125%, 894.67x432 at 150%). `startup.rs` and `tests/window_minimum.rs` are kept as they are.

### 4.6 Visible, disabled placeholders (complete list)

| Item | Where | Behaviour |
| --- | --- | --- |
| `Reset` (Main Menu sub-menu row) | Main Menu body | disabled, hover names L3-004 |
| Title-bar close box | every menu bar (seven menus; none on BUILDERS) | click → `CloseUnsupported` warning |
| `+` viewport tab | view-tab strip | weak look; click → warning, no change |
| BUILDERS slot | bottom of left column | unchanged reserved slot (L3-008) |

Panels missing from OM9: none. Not added (§15): menu-bar `Rhino 5.0`, `Blend`, `Clayoo` (A7); Icon History
previous/next arrows (A9); a `Shaded` Display dropdown (A8).

## 5. L3-003 mechanics (Part B)

### 5.1 Module and types (`src/panel_ops.rs`, new; `lib.rs` adds `pub mod panel_ops;`)

The first run's api.md §3 holds with these revisions:

```rust
// dock_layout.rs (persisted schema types live with the schema)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ColumnSide { Left, Right }
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionHome { pub side: ColumnSide, pub index: u32 }
// SectionState gains: #[serde(default, skip_serializing_if = "Option::is_none")] pub home: Option<SectionHome>
impl SectionId { pub fn is_movable(self) -> bool { !self.is_reserved() } }
impl DockLayout {
    pub fn documented_index(&self, id: SectionId, side: ColumnSide) -> usize;   // D3
    pub fn place_section(&mut self, id: SectionId, side: DockSide, index: Option<usize>) -> Result<usize>; // primitive
    pub fn apply_panel_op(&mut self, op: PanelOp) -> Result<PanelReport, PanelOpError>; // the one model door
}

// panel_ops.rs (re-exports ColumnSide, SectionHome)
pub const FLOAT_DROP_SIZE: [f32; 2] = [300., 240.];
pub const FLOAT_DROP_GRAB: [f32; 2] = [150., 7.];
// No strip constant here: the drop zone width is dock_layout::RIGHT_DROP_ZONE_WIDTH (L3-002 task).
pub struct WorkArea { x: f32, y: f32, width: f32, height: f32 }   // private fields, validated constructor
impl WorkArea { pub fn for_window(window: [f32; 2]) -> Result<Self, PanelOpError>;
                pub fn rect(&self) -> RegionRect; pub fn clamp(&self, r: FloatRect) -> FloatRect;
                pub fn contains(&self, r: FloatRect) -> bool; }
pub fn float_drop_rect(pointer: [f32; 2]) -> FloatRect;
pub enum FloatOrigin { Drop, OutsideMainWindow, Button, Api }
pub enum PanelOp {
    ToggleCollapsed { section: SectionId },
    Move { section: SectionId, delta: i32 },
    Place { section: SectionId, side: ColumnSide, index: usize },
    Dock { section: SectionId, side: ColumnSide },
    DockHome { section: SectionId },
    Float { section: SectionId, rect: FloatRect, work: WorkArea, origin: FloatOrigin },
    SetFloatRect { section: SectionId, rect: FloatRect, work: WorkArea },
    Close { section: SectionId },                                   // title-bar close box (always refused)
}
pub enum PanelOutcome { Collapsed{..}, Moved{..}, Docked{.., restored_home: bool}, Floated{..}, FloatRectSet{..} }
pub enum PanelNote { Recovered { section, from, to }, OutsideMainWindow { section } }
pub struct PanelReport { pub outcome: PanelOutcome, pub notes: Vec<PanelNote> }
pub enum CancelReason { NoTarget, Escape, PointerLost, LayoutChanged, RightDockTooNarrow }
pub enum PanelOpError {
    NotPermitted { section }, UnknownSection { key: String }, Hidden { section },
    InvalidTarget { side: DockSide }, InvalidRect { section }, InvalidWindow { width: f32, height: f32 },
    NoWorkArea, LayoutBlocked, NotDocked { section }, NotFloating { section }, AlreadyFloating { section },
    AlreadyThere { section, side: ColumnSide, index: usize }, DragCancelled { section, reason: CancelReason },
    CloseUnsupported { section },
}
pub struct ColumnZone { pub rect: RegionRect, pub content_bottom: f32 }
pub struct DropZones { pub window: Option<RegionRect>, pub left: Option<ColumnZone>, pub right: Option<ColumnZone>,
                       pub right_drop_zone: Option<RegionRect>,        // from compute_right_drop_zone()
                       pub right_too_narrow_band: Option<RegionRect>,  // window < 1200, dock hidden: rightmost
                                                                       // RIGHT_DROP_ZONE_WIDTH of the window
                       pub bars: Vec<(SectionId, ColumnSide, RegionRect)>, pub views: Vec<RegionRect> }
pub enum DropTarget { Place { side: ColumnSide, index: usize }, Float { pointer: [f32; 2] },
                      FloatOutside { pointer: [f32; 2] }, Cancel { reason: CancelReason } }
pub fn resolve_drop(layout: &DockLayout, dragged: SectionId, pointer: Option<[f32; 2]>, zones: &DropZones) -> DropTarget;
pub fn drop_indicator(layout: &DockLayout, dragged: SectionId, target: DropTarget, zones: &DropZones,
                      work: Option<WorkArea>) -> Option<RegionRect>;
pub fn drop_op(section: SectionId, target: DropTarget, work: Option<WorkArea>) -> Result<PanelOp, PanelOpError>;
```

All enums/structs derive `Debug, Clone, (Copy where possible), PartialEq, Serialize`. `PanelOpError` implements
`Display` and `std::error::Error`; texts are api.md §6 plus `CloseUnsupported` →
`"Closing {title} is not available yet; click its title to roll it up"` and `DragCancelled { RightDockTooNarrow }` →
`"Right dock needs a window at least 1200 points wide"` (the L3-002 task's existing text, now Info through the one
door).

`SCHEMA_VERSION` stays **2**. `home` is optional with `serde(default)`; `DockLayout` does not deny unknown fields, so
old and new files load in both directions.

### 5.2 Operation rules (`DockLayout::apply_panel_op`)

The api.md §4 table is kept. Every op runs on a clone committed only on `Ok`, so on `Err` `to_json()` is
byte-identical by construction. Common prefix: `!is_movable` → `NotPermitted`, `!visible` → `Hidden`. New row:

| Op | Preconditions → error | Effect |
| --- | --- | --- |
| `Close` | always `CloseUnsupported` (`BuilderSlot` gets `NotPermitted` from the prefix first; API/audit only, since the BUILDERS bar draws no close box) | none |

Invariants after every `Ok` (`debug_assert!` in code, asserted in tests): each `SectionId` in exactly one of
`left ∪ right ∪ floating`; each column's orders dense `0..n`; every floating section has a finite `float`; after
`Float`/`SetFloatRect`, `work.contains(rect)`.

**Owner: the model layer (`dock_layout.rs` + `panel_ops.rs`).** The UI has no other mutation path for **sections**;
the source-scan test below enforces it. Region panel sizes (`PanelId`) are not section ops and keep their own,
allowlisted setters.

**Source-scan test `ui_mutates_sections_only_through_apply_panel_op`** (in `tests/panels_reorder_collapse_float.rs`,
`std::fs` only, no new crate):

- Scanned files: a recursive `std::fs::read_dir` walk of `concat!(env!("CARGO_MANIFEST_DIR"), "/src")`, every `*.rs`,
  except `src/dock_layout.rs` and `src/panel_ops.rs` (the model) and `src/workbench_regions.rs` (pure planner; it
  mutates only its own local `DockLayout` values inside `#[cfg(test)]` helpers, never `MatrixApp::dock`). Because the
  walk is live, a new source file is scanned automatically. The test asserts the walk found `ui.rs` and
  `ui/chrome.rs` (guards against a wrong root).
- Pre-processing: strip `//` and `/* */` comments, string literals (`"…"`, raw `r#"…"#`) and char literals, so a
  token in a doc comment or a message does not count.
- **Banned** (zero occurrences allowed in any scanned file, any receiver): `.dock_section(`, `.place_section(`,
  `.move_section(`, `.float_section(`, `.toggle_section(`, `.set_section_collapsed(`, `.set_section_visible(`,
  `.set_section_height(`, and `.section_mut(` (direct field writes; it stays private to `dock_layout.rs`). After L4-03 the
  UI reaches these only through `MatrixApp::apply_panel_op` → `DockLayout::apply_panel_op`.
- **Allowlisted** other mutators, asserted as an exact `(file, token, count)` table so that any new call site fails
  and any removed one forces a deliberate table edit:
  - `src/ui.rs`: `.reset_to_default(` (reset layout, `reset_menus`, `reload_menus` without a store; L3-004 paths),
    `.clamp_floats(` (window resize), `.set_float(PanelId::CommandSearch` (F6 readback), `.set_size(PanelId::LeftRail`
    (audit rail width), `.set_visible(PanelId::RightDock` (`toggle_right_dock`, kept public), `self.dock = ` (layout
    load in `load_dock_layout`).
  - `src/ui/chrome.rs`: `.set_size(` inside `sync_panel_size` (region sync).
  - L4-03 fills in the counts from the post-P0 tree and records them in the diffstat note.
- The audit layout-mode hook already docks `RIGHT_DOCK_FIXTURE` with `app.dock_menu_section(..)` (`ui.rs` ~2059);
  after L4-03 that method is a one-line delegation to `apply_panel_op(PanelOp::Dock { section, side })`, so the hook
  goes through the one door without its own edit. The existing UI calls that move to `apply_panel_op` are:
  `chrome.rs` title-bar `toggle_section` (~270) and right-dock `toggle_section` (~1004) → `ToggleCollapsed`;
  `float_section_at`'s `float_section` (~455, deleted) → the resolver; the floating-window readback `float_section`
  (~1065) → `SetFloatRect`; `ui.rs` `move_menu_section` / `dock_menu_section` / `float_menu_section` → delegations.
- RED evidence: the test is written in L4-02 and fails on today's tree (it finds `toggle_section` in `chrome.rs`); a
  mutation run in L4-03 re-adds one banned call and records the failure.

### 5.3 Decisions D1-D6

- **D1 Off-screen recovery: clamp and snap-back only (Reset Menus is L3-004).** Work area
  `x = 8, y = 21 + 8, w = W − 16, h = H − 21 − 20 − 16` (1280x720 → `[8,29,1264,663]`, 1600x1000 →
  `[8,29,1584,943]`). Recovery on every float request, on floating-window readback (`SetFloatRect`), and on resize and
  load (`clamp_floats` keeps its name/signature, re-implemented on `WorkArea`, F6 window included). Every floating menu
  window and the F6 window get `.constrain_to(work.rect())`. egui memory (`current_pos`, `fixed_size`) is overridden
  only on the frame after the model changed a rect (`float_resync`). While `region_error` is `Some`, recovery rewrites
  nothing, readback is skipped and `Float` returns `LayoutBlocked`.
- **D2 Float target: Option A, an egui `Window` inside the main OS window.** Testable with the single-context harness,
  one surface, keeps the CMD-focus contract, no Win32 work area. Known deviation: no floating onto another monitor; a
  release outside the main window floats inside the work area with `"{title} cannot float outside the main window;
  placed inside it"`. Multi-viewport (Option B) is backlog, outside M01.
- **D3 Restore without losing state.** `home` is set when a section leaves a docked column, cleared when it lands in
  one. `Dock { side }` uses `home.index` when `home.side == side`, else `documented_index` (left: directly after the
  last left section that precedes it in `SectionId::ALL`; right: at the end). `collapsed`, `height`, `visible` and the
  last `float` are never changed by dock/float; the `state.collapsed = false` line in `dock_section` is removed.
  Floating windows use `.collapsible(false)`.
- **D4 Permitted panels.** The 7 documented menus are permitted. `BuilderSlot` is not: `Sense::click()`, a click
  returns `NotPermitted` with `"Builders is reserved for builder tools and cannot be rolled up, moved or floated"`,
  never draggable. `PanelId` regions are not panels; the F6 window only takes part in recovery.
- **D5 Drag cancel.** Release over no target → `Cancel { NoTarget }` (Warning). Escape during a drag is a new first
  branch of the Escape chain in `MatrixApp::show`: it takes the drag, clears the preview, reports
  `DragCancelled { Escape }` (Info) and does not fall through, so the CMD draft, a running command, F6 and popups are
  untouched.
- **D6 Layers placement (user instruction 2026-10-16, steering).**
  - **L1 Default.** `matrix_default()`: Layers `Left`, order = order(InfoSettings) + 1 = 5; Projects 6.
  - **L2 Supported sizes.** At every supported size (§4.5), default layout: Layers is in the left column, below the
    Info & Settings block, with no title bar between them, and the Projects bar below it; nothing is reordered, moved
    right or compressed. **Visibility at scroll 0 (measured).** FEAT-001/FEAT-002 measured, before L3-003: Layers
    bar y 337..351 at rail 300 and 397..411 at rail 460, sidebar starting at y 23, identical at window heights 720
    and 1000 (§2). The smallest supported column is the capped 894.67x432 window: sidebar bottom ≈ 430, so both bar
    ranges fit with ≥ 19 pt to spare. Therefore at **every** size of §10.3 (capped sizes included) and at rail 300
    and 460, default layout, scroll 0, the Layers bar rect is asserted **fully inside `sidebar_rect`**. L3-003
    changes the Main Menu and Snaps row heights (G12, `clipped_row`) and the title bars, so L4-02 re-measures the bar
    y (RED run, before the change) and L4-03 re-measures it (GREEN run, after), both recorded in the evidence.
    **Fallback if a measured case fails** after L4-03: the run reports the measured y and the size to the user, the
    failing case is asserted for order + adjacency + reachability-by-scroll only, and **no layout constant, body
    height, collapse state or order is changed to make it fit**. Reachability-by-scroll (without the scroll-0 claim)
    is asserted only for user-changed layouts: a user-reordered stack, expanded optional bodies, or a widened column
    other than 300/460.
  - **L3 Restore.** Dock-back without a home (◀, Dock left, closing a floating window, `close_right_menu_column`)
    inserts Layers directly below Info & Settings.
  - **L4 User moves** are allowed and are session state. OM9 never moves Layers by itself. A saved store is not
    rewritten. Reset Menus restores L1.
  - **L5 Audit and fixtures** dock Snaps (`RIGHT_DOCK_FIXTURE`), never Layers.
  - **L6 Roll-up and short columns.** Rolling up Layers or Info & Settings changes only `collapsed`; the order slot is
    kept. When the column is shorter than the stack, no section has priority: every section keeps its full body
    height and the stack scrolls/clips.
  - **L7 Hidden neighbour.** If Info & Settings is hidden (stored user state), it keeps its slot; Layers is drawn
    directly below the previous visible section and returns below Info & Settings when it is shown again. OM9 never
    hides Info & Settings itself; the close box does not hide sections (A5).

### 5.4 Drop resolution, preview, cancel

Precedence for pointer `p` (replaces `settle_section_drag`; same zone width and the same below-1200 behaviour):

1. no finite `p` → `Cancel { PointerLost }`
2. outside the window → `FloatOutside`
3. inside `right_drop_zone` (exists only while dragging, dock hidden, window ≥ 1200) → `Place { Right, 0 }`
4. inside `right_too_narrow_band` (window < 1200, dock hidden) → `Cancel { RightDockTooNarrow }` (Info, no change)
5. inside the shown right column → `Place { Right, idx }`
6. inside the left column → `Place { Left, idx }`
7. inside a view → `Float`
8. anything else (menu bar, command box, toolbar, status bar) → `Cancel { NoTarget }`

The two right-edge areas cover different y-ranges **on purpose**, as in L3-002 today: the drop zone (rule 3,
window ≥ 1200) is the last `RIGHT_DROP_ZONE_WIDTH` points of the **union of `view_rects`** (`compute_right_drop_zone`),
so it never covers the command box or toolbar; the narrow band (rule 4, window < 1200) is the rightmost
`RIGHT_DROP_ZONE_WIDTH` points of the **whole window height** (menu bar to status bar), so any release near the right
edge of a too-narrow window explains why nothing docked instead of silently cancelling. Rule 4 is checked before
rule 8, so a release on the command box's right edge below 1200 gives `RightDockTooNarrow`, not `NoTarget`.

Index rule: the position, among the column's other sections, of the first whose rendered bar centre y is `>= p.y`;
none → column length. Works for drops on a body and for bars scrolled out of view.

Preview: `drop_preview: Option<DropPreview { section, target, indicator }>` painted on
`LayerId::new(Order::Foreground, Id::new("om9-section-drop-preview"))`, colour `rgb(150, 55, 55)`. `Place` → a 3 px
insertion line; float → the exact `work.clamp(float_drop_rect(p))` rect with a 2 px stroke and 20% fill. The right
drop zone keeps the L3-002 task's own paint (`right_drop_zone_ui`). Cursor `Grabbing`, or `NoDrop` for a `Cancel`. On
release the target previewed on the last pressed frame is applied through `drop_op` → `apply_panel_op`.

Drag state: `section_drag: Option<SectionDrag { section, last_pointer }>` (the existing `Option<SectionId>` field is
widened). Reset, Reload, Load and a section becoming hidden cancel with `LayoutChanged` (CMD history only). The
settle step stays where it is in `show` (after `float_menu_windows`) and calls the resolver; `settle_section_drag` and
`float_section_at` are deleted.

### 5.5 UI boundary (`MatrixApp`, `ui.rs`)

- `pub fn apply_panel_op(&mut self, op: PanelOp) -> Result<PanelReport, PanelOpError>` is the only UI door.
- `move_menu_section`, `dock_menu_section`, `float_menu_section` stay as one-line delegations returning `bool`
  (`tests/common/mod.rs::dock_right_fixture` keeps working).
- Messages (api.md §6): Info → status line; notes → status Warning + CMD history; errors → status Warning + CMD
  history as `"Menus: {error}"`. Nothing replaces an unacknowledged Error (L3-007): then the message goes to CMD
  history only.
- **CMD cursor after a history append (stale-cursor guard).** `left-dock-measurements.md` proves that
  `Transcript::append_history` moves the protected history past the stored CMD cursor, so `Transcript::insert`
  refuses the next typed text (index < `protected_chars`): the first keystroke after the append is silently lost.
  All L3-003 history appends go through one private helper `fn panel_message(&mut self, kind, text)` used by
  `apply_panel_op`'s message path, the drag-cancel path, `close_right_menu_column` and the `+` / `>>` handlers. After
  any `transcript.append_history` it sets `self.focus_cmd = true` (the existing mechanism the CMD frame uses to put
  the cursor at the transcript end, as the About/Help items in `chrome.rs` already do). The pre-existing
  `note_dock_diagnostics` path (resize/load diagnostics) gets the same one-line `focus_cmd = true`. The general fix
  inside `Transcript::append_history` (shift the stored cursor by the appended char count) is **out of scope** and
  stays a flagged caveat in the final report.
- `dock_dirty = true` on every `Ok`. No file is written (L3-005 keeps Set as Default).
- New test-visible state: `drop_preview`, `work_area`, `last_panel_op`, `closed_bar_controls`, `section_drag()`,
  `sidebar_scroll()`, `section_blocks`, `toolbar_tabs_overflow`, `command_history_rows()`.

### 5.6 Receipts and audit replay

The `dock` object from `dock_receipt()` (L3-002 task) is not changed. L3-003 adds two objects:

- `receipt["panels"]` (api.md §7) with `last_panel_op`, `drop_preview`, `float_rects`, `homes` and
  `"layers_placement": { "info_settings_block_px", "layers_bar_px", "projects_bar_px", "below_info_settings_block":
  bool, "bar_between": bool }`. Scroll is read from `dock.sidebar_scroll_offset`, not duplicated.
- `receipt["layout_parity"]`: `fixture` (§4.4), `sidebar_rect_px`, `bar_style: "dark"`, `toolbar_rect_px`,
  `command_above_toolbar: bool`, `command_history_rows` (`floor((history_height + 0.5) / (r + 2))`, §4.3),
  `command_font_row_px` (`r`), `command_panel_px` (outer drawn height), `command_default_px` (`C`),
  `command_capture_estimate_px: 42` (constant label for the final-report comparison, A15), `toolbar_tabs_overflow`, `view_tabs: ["Perspective","Through Finger","Side View","Looking Down","+"]`. Right
  dock state is read from `dock.right_dock_visible` / `dock.right_dock_sections`, not duplicated.

`OM9_UI_AUDIT_PANELS=<scenario>` replays real pointer events: `rollup`, `reorder`, `dock-right` (Snaps dragged into the
right drop zone), `float`, `float-offscreen`, `shrink` (to the effective minimum), `refuse-builders`, `escape-cancel`,
`scroll-column` (wheel over the column; receipt shows `dock.sidebar_scroll_offset > 0` with the Builders bar visible).
Scenarios run in layout mode `default`; setting both `OM9_UI_AUDIT_PANELS` and
`OM9_UI_AUDIT_LAYOUT_MODE=user-docked-right` is a launch error. An unknown scenario is a launch error with non-zero
exit, like `OM9_UI_AUDIT_ZOOM`.

## 6. File-by-file change map

All edits to existing files happen in L4-03 after P0 (§2.1).

| File | Change |
| --- | --- |
| `src/panel_ops.rs` (new) | §5.1 types, `WorkArea`, `resolve_drop`, `drop_indicator`, `drop_op`, `Display` texts; unit tests |
| `src/lib.rs` | `pub mod panel_ops;` |
| `src/dock_layout.rs` | `LEFT_RAIL_MIN = 300`; `COMMAND_DEFAULT = C`, `COMMAND_MIN = min(58, C)`; `ColumnSide`, `SectionHome`, `SectionState.home`; `SectionId::is_movable`; `documented_index`, `place_section` (new body of `dock_section`, drops `collapsed = false`); `float_section` records `home`; `clamp_floats` on `WorkArea`; `normalize` LeftRail diagnostic before the generic clamp; `apply_panel_op`; Projects-right unit tests → `RIGHT_DOCK_FIXTURE` (S3, S4). Not touched: `rail_icon_edge`, `ICON_*_EDGE`, `RIGHT_DOCK_FIXTURE`, `RIGHT_DROP_ZONE_WIDTH`, `LEFT_RAIL_MAX`, `LEFT_RAIL_CHROME_PX` |
| `src/workbench_regions.rs` | Logic unchanged. Module doc: "optional right properties dock" → "optional user-docked right menu column (hidden by default; Matrix9 shows none)". Helper `layout_with_right_dock` docks `RIGHT_DOCK_FIXTURE` (S5). Hard-coded command-dependent rects → §10.1 S23 |
| `src/ui.rs` | Command panel: unconditional outer-height sync after `show` (§4.3). `panel_message` helper and `focus_cmd = true` after history appends, also in `note_dock_diagnostics` (§5.5). Escape-chain first branch (D5). Drag lifecycle and resolver after `float_menu_windows`. `apply_panel_op` + delegations. `close_right_menu_column`. `request_sidebar_scroll -> bool`, `sidebar_scroll`, `command_history_rows`. Fields `pending_sidebar_scroll`, `section_bar_sides`, `section_blocks`, `drop_preview`, `work_area`, `last_panel_op`, `float_resync`, `float_chrome`, `closed_bar_controls`, `toolbar_tab_rects`, `toolbar_tabs_overflow`, `toolbar_tab_scroll_to`; `section_drag` widened. `OM9_UI_AUDIT_PANELS` parsing (and its conflict with user-docked-right). Receipt `panels` and `layout_parity`. F6 window `.constrain_to`. Audit-mode code and `dock_receipt` unchanged |
| `src/ui/command.rs` | Record `command_history_height` and `command_font_row` for `command_history_rows`; no layout change |
| `src/ui/chrome.rs` | `section_bar` replaces `section`; `right_dock_ui` uses `section_bar`; `clipped_row`, Main Menu `Reset`, Snaps chip row; `sidebar_ui` scroll fix (G9); `toolbars_ui` `>>` and `+`; View-menu right-column item; `float_menu_windows` per D1/D3 (buttons → `apply_panel_op`); build `DropZones` from `compute_right_drop_zone` and the narrow band; delete `settle_section_drag`, `float_section_at`. `icon()`, `compute_right_drop_zone`, `right_drop_zone_ui` unchanged |
| `src/ui/panel_audit.rs` (new) | `OM9_UI_AUDIT_PANELS` replay through `raw_input_hook` |
| `src/startup.rs` | No change |
| `tests/panels_reorder_collapse_float.rs` (new) | frame tests (§10.3) |
| `tests/main_window_parity.rs` (new) | layout parity frame tests (§10.3) |
| existing `tests/*.rs`, in-crate tests in `src/ui.rs` / `src/ui/chrome.rs`, L3-002 and L3-003 fixtures | superseded assertions per §10.1 (L3-003 rows only) |
| `tools/validate_m01_l3_002_ui_matrix.py` | only S21 (remove `rail240-1280x720`) and the S15 wording if the L3-002 task left it |
| `tools/validate_m01_l3_003_panels.py` (new, **required**) | §10.4 launcher with `--self-test` |
| L3-003 `api.md`, `positive.json`, `negative.json`, `source_crosswalk.json` | §12 |

Not touched: `om9-core`, `om9-scene`, `om9-archive`, `om9-exchange`, `om9-native-owner`;
`implementation/*/M01/OM9-L2-01-0{8,2,3}*`; `.agents/tasks/m01-l2-01-02-03-backend/`; `OM9_Milestones/`; `skills/`;
`QymCAD/`; `vendor-research/`; `ref/`; `src/ui/layers.rs` (L2-03-14). Seams `CommandInputHost`,
`ViewportSceneHost`, `AttachedCmdFrame::command_input_ready` unchanged.

## 7. Error handling and input validation

Every L3-003 failure is **recoverable**, leaves the layout unchanged, never panics, and is reported on the status line
and in CMD history (§5.5). No file log. api.md §6 tables are kept. Additions:

| Operation / input | Failure condition | Caller receives | Visible as |
| --- | --- | --- | --- |
| close box click | always (BUILDERS has no close box; `PanelOp::Close { BuilderSlot }` from the API → `NotPermitted`) | `Err(CloseUnsupported)` | Warning + history, then `focus_cmd = true` (§5.5) |
| `+` view tab click | always | no `Result`; UI only | Warning + history |
| `>>` menu entry | index out of range (guarded with `get`) | ignored, `debug_assert` | none |
| `request_sidebar_scroll(y)` | non-finite `y` | `false`, nothing stored | none (test/audit API) |
| release in the narrow band (< 1200) | dock hidden, no room | `DragCancelled { RightDockTooNarrow }` | Info + history |
| `normalize` left width | finite `< 300` | widened to 300 + diagnostic (before the clamp) | Warning + history |
| `normalize` left width | non-finite | default 300 (existing path) | existing diagnostic |
| `close_right_menu_column` | no right sections | `0`, no change | item disabled, unreachable from the UI |
| `OM9_UI_AUDIT_PANELS` | unknown value, or combined with `user-docked-right` | launch error, non-zero exit | stderr + startup failure receipt |
| `OM9_UI_AUDIT_RAIL_WIDTH` | `< 300` (now) or `> 460` | launch error (existing parser) | stderr |
| `command_default_matches_font_metrics` | font row height changed | test failure | CI |

## 8. Invariants and owners

| Invariant | Owner | Why there |
| --- | --- | --- |
| Section membership, dense order, finite floats, work-area containment | `dock_layout.rs` + `panel_ops.rs` | pure model, the only mutation door |
| Default order, Layers at InfoSettings + 1 | `SectionId::ALL` / `matrix_default` (order owned by OM9-L3-03-07-001; L3-003 keeps it on restore) | one constant, used by Reset Menus |
| Left width ∈ [300, 460] | `dock_layout.rs` constants + `size_bounds` + `normalize` + egui `min/max_size` | stored and live panel agree |
| Column icon edge = `rail_icon_edge(outer)`, UI range [22, 34] | L3-002 task (`dock_layout.rs`, `ui.rs`) | consumed, not redefined |
| Command box ≥ two history rows + one input row | `COMMAND_DEFAULT`/`COMMAND_MIN` + `command_default_matches_font_metrics` | approved font-metric rule |
| Region rects, command input kept, no overlap | `workbench_regions.rs` (L3-002) | validated plan |
| No panel op touches document, layers, units, Undo, scene or draft | `ui.rs` boundary (no controller call in `apply_panel_op`) + snapshot test (§10.3 item 9) + source scan | the only place that could leak |
| Right dock appears only through a user panel op | `resolve` (needs a right section) + `apply_panel_op` as the only door + receipt `fixture` label | default layout has no right section |

## 9. Edge cases

- **Widened column at 1280x720.** At rail 460 icons grow to 34 and the stack gets taller (893), but the Layers bar
  stays at y 397..411, inside the column at scroll 0 (measured, D6 L2). Lower sections (Projects, Builders) move
  below the fold and are reached by scrolling; nothing is reordered or compressed.
- **Capped 894.67x432 (1366x768 at 150%).** Sidebar y 23..≈430. The Layers bar (337..351 at rail 300, 397..411 at
  rail 460, before the G12 re-measure) is visible at scroll 0; Projects (527 / 587) and Builders are below the fold
  and reachable by scrolling. A user-docked right column yields (L3-002 note).
- **History append while the user types.** A panel message or a resize/load diagnostic appended to CMD history sets
  `focus_cmd`, so the next keystroke lands at the transcript end (§5.5). Side effect, accepted: a resize diagnostic
  also returns keyboard focus to CMD, which is the CMD-focus contract's default owner.
- **Smaller command box.** The canvas gains `90 − C` points; the left column is full height and does not change.
- **`>>` state flips on resize** one frame late; the strip stays wheel-scrollable meanwhile.
- **Drag over a scrolled column.** Bars out of view keep window-space rects; the index rule handles them; the preview
  line is clipped to the column.
- **Drag starting while `viewport_menu_owns_frame`.** The column is disabled; no drag starts (existing).
- **Floating Layers.** The closed-bar current-layer label applies only when docked; in a float the body always shows.
- **Concurrent edits.** P0 (§2.1) guards the shared files. A backend workflow runs in the same checkout; no backend
  file is in the change map.

## 10. Tests

### 10.1 Superseded L3-002 assertions (change deliberately, with reason, in the L4-03 diffstat note)

Already gone (verified earlier by grep): every `560x286`, `320x286`, `559x286`, `560x285`, `719x479`, `900`, `915`
assertion.

Owner column: **L3-002** = done or planned by `task-m01-l3-002-left-dock-icons`, L3-003 only verifies after P0;
**L3-003** = changed by this design in L4-03.

| # | File · test / helper | Today | Change | Reason | Owner |
| --- | --- | --- | --- | --- | --- |
| S1 | `src/dock_layout.rs` · `narrow_window_yields_the_right_column_and_keeps_command_line` | `RIGHT_DOCK_FIXTURE` (done) | verify only | steering | L3-002 |
| S2 | `src/dock_layout.rs` · `right_dock_threshold_boundary_is_1200_logical_points` | `RIGHT_DOCK_FIXTURE` (done) | verify only | steering | L3-002 |
| S3 | `src/dock_layout.rs` · `dock_right_mirrors_the_left_column_and_float_needs_a_rectangle` (~954) | Projects right | `RIGHT_DOCK_FIXTURE`; float part unchanged | neutral fixture | L3-003 |
| S4 | `src/dock_layout.rs` · `round_trip_persists_menu_identity_visibility_and_order` (~1135) | Projects right | `RIGHT_DOCK_FIXTURE` right **and** collapsed; Info & Settings hidden unchanged; add `home` round-trip | neutral fixture; proves D3 field | L3-003 |
| S5 | `src/workbench_regions.rs` · helper `layout_with_right_dock` (~606) | Main Menu right | `RIGHT_DOCK_FIXTURE`; expected rects unchanged except S23 | neutral fixture | L3-003 |
| S6 | `tests/workbench_regions.rs` · four tests docking Layers right (~374, ~406, ~433, ~486) | Layers right | `dock_right_fixture` | steering | L3-002 |
| S7 | `tests/workbench_regions_sizes.rs` · `settle(.., dock)`, `dock_threshold_sweep_1150_to_1250`, `pure_plan`, the `OM9_SWEEP` line | Layers right | `dock_right_fixture` | steering | L3-002 |
| S8 | `tests/status_bar_overlap.rs` · `settle` | Layers right | `dock_right_fixture` | steering | L3-002 |
| S9 | `tests/shell_docking.rs` · `narrow_window_yields_the_right_column_and_keeps_command_line_and_canvas` | Layers right | `dock_right_fixture` | steering | L3-002 |
| S10 | `tests/shell_docking.rs` · `menus_dock_to_the_right_mirror_column_and_back` | comment "Re-docking appends the menu" | comment → "Re-docking restores the menu's home position"; add `assert_eq!(left, DOCUMENTED_LEFT)` | D3 | L3-003 |
| S11 | `tests/shell_docking.rs` · `the_left_rail_and_command_line_keep_their_documented_bounds` | `LEFT_RAIL_MIN` = 240, `COMMAND_MIN` = 58 | same symbolic assertions (now 300 and `min(58, C)`); add `assert_eq!(LEFT_RAIL_MIN, LEFT_RAIL_DEFAULT)` | G11, G16 | L3-003 |
| S12 | `src/ui/chrome.rs` · `sidebar_wrapping_and_toolbar_scrolling_keep_a_240px_allocation` | runs at 240 | code unchanged; doc: "240 is below the 300 minimum: proves clipped rows never escape a narrower allocation" | G12 | L3-003 |
| S13 | L3-002 `positive.json` · `left-rail-four-views-right-dock-1600x1000` | `dock_right: ["main-menu"]` | `["snaps"]`, `"label": "user-docked case (not the Matrix9 default)"`; rects per S23 | neutral fixture | L3-003 |
| S14 | `src/ui.rs` · audit hook | replaced by `OM9_UI_AUDIT_LAYOUT_MODE` (done) | verify only | steering | L3-002 |
| S15 | `tools/validate_m01_l3_002_ui_matrix.py` comment ~39-42 "with Layers docked right" | (FEAT-003 rewrites it) | verify; if still present, "with Snaps docked right (user-docked case)" | steering | L3-002 |
| S16 | `tests/shell_docking.rs` · `reset_and_reload_menus_follow_the_documented_commands` (~275) | Projects right, asserts `right_sections() == [Projects]` | `RIGHT_DOCK_FIXTURE`, assert `== [Snaps]` | neutral fixture | L3-003 |
| S17 | L3-002 `negative.json` · `narrow-800x600-with-right-dock-yields-and-keeps-command-input` | `dock_right: ["main-menu"]` | `["snaps"]`, `"label": "plan-only, below the supported range (user-docked case)"`; rects per S23 | neutral fixture; 800x600 < 1280x720 | L3-003 |
| S18 | L3-002 `negative.json` sweep `layouts` + `tests/workbench_regions.rs` loader arm (~289) | `"right-dock-main-menu"` docks `MainMenu` | rename `"right-dock-user-docked-snaps"`, arm docks `RIGHT_DOCK_FIXTURE` | neutral fixture | L3-003 |
| S19 | L3-002 `negative.json` `constants.command_min_height` 58, sweep text `command_input.height >= 58`; `tests/workbench_regions.rs` (~310, ~457) and `tests/workbench_regions_sizes.rs` (~232) literal `58.` | 58 | `COMMAND_MIN` symbol in Rust; the JSON value = `min(58, C)` | G16 | L3-003 |
| S20 | `tests/left_stack_layers.rs` (L3-002 FEAT-002) · `icon_edge_follows_the_rail_width` | loop `[LEFT_RAIL_MIN, LEFT_RAIL_DEFAULT, 380., LEFT_RAIL_MAX]` with strict `edge > previous`; anchor `rail_icon_edge(LEFT_RAIL_MIN) == 18.` | loop `[LEFT_RAIL_DEFAULT, 380., LEFT_RAIL_MAX]` (with MIN == DEFAULT the old loop would fail `22 > 22`); anchor → `rail_icon_edge(240.) == 18.` (literal: pure math, below the UI range); add `assert_eq!(LEFT_RAIL_MIN, LEFT_RAIL_DEFAULT)`; other anchors unchanged | `LEFT_RAIL_MIN` 300 | L3-003 |
| S21 | L3-002 validator case `rail240-1280x720`; `ui.rs` `parse_audit_rail_width` test | runs at 240 | remove the case; add `assert_eq!(parse_audit_rail_width("240"), None)` and `Some(300.)` for `"300"` | `LEFT_RAIL_MIN` 300 | L3-003 |
| S22 | any L3-002 test that sets `LeftRail` to 240 and expects 240 (re-grep `240` in `tests/` after P0) | 240 | expect 300, or the test is about allocation (S12) and stays | `LEFT_RAIL_MIN` 300 | L3-003 |
| S23 | `src/workbench_regions.rs` tests (~655-735, ~870, ~885, ~973), L3-002 `positive.json` (cases 1600x1000, 1280x900, single-view, 800x400) and `negative.json` (800x600, hidden-left 800x400) | command height 90, toolbar y 111, center y 178, right dock y 178 | with `Δ = C − 90`: command height `C`; toolbar y `21 + C`; center / right dock / top-row views y `88 + C`, heights `− Δ`; bottom-row view y `+ Δ/2`; both view-row heights `− Δ/2`; x and widths unchanged (C = 56 example: 1280x720 toolbar y 77, center `[300,144,980,556]`, views 489x277) | G16 | L3-003 |
| S24 | `tests/workbench_regions_sizes.rs` · `layout_evidence_table` rows (canvas height, view height) and the L3-002 FEAT-002 rows added there | 522/260 at 720 etc. | canvas height `− Δ`, view height `− Δ/2`; regenerate `implementation/evidence/M01/OM9-L3-01-01-002/left-dock-layouts.json` with a note "command box set to two history rows by OM9-L3-01-01-003" | G16 | L3-003 |
| S25 | `src/dock_layout.rs` tests using `COMMAND_DEFAULT` symbolically (~1048, ~1056) | symbolic | no change; re-run | G16 | L3-003 |
| S26 | `tests/workbench_regions_sizes.rs` comment ~220-225 (plan-versus-frame command gap) | "recorded, not fixed" | the outer sync is unconditional (§4.3): the comment states the gap is closed and the slack assertion becomes exact (`<= 0.5`) | G16 | L3-003 |
| S27 | L3-002 validation doc `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md` rect tables | 90-based numbers | correction note pointing to this design (no rewrite of historical numbers) | G16 | L3-003 |
| S28 | L3-003 `negative.json` N5 precondition strip `[1552, 178, 48, 802]` | 48-wide strip, y 178 | `[1572, 88 + C, 28, 892 − C]`: the last 28 pt of the **union of `view_rects`** (`compute_right_drop_zone`), not the whole canvas/window (1600x1000) | `RIGHT_DROP_ZONE_WIDTH`, G16 | L3-003 |
| S29 | `tests/workbench_regions_sizes.rs` · `left_rail_max_justification_table` (L3-002 FEAT-002) | loop `for rail in [240., 300., 460.]`; doc "the rail at 240 / 300 / 460" | loop `[LEFT_RAIL_MIN, LEFT_RAIL_MAX]` (300, 460); doc "the rail at 300 / 460 (the minimum is the Matrix default)"; regenerate `left-dock-layouts.json/.log` together with S24 | `LEFT_RAIL_MIN` 300 (a 240 row would silently be clamped to 300 and mislabel the evidence) | L3-003 |
| S30 | `src/ui/chrome.rs` in-crate tests `dragging_a_menu_to_the_right_edge_docks_it_and_shows_the_hidden_dock`, `a_window_under_1200_points_offers_no_drop_strip_and_docks_nothing` (L3-002 FEAT-001) | `app.section_drag = Some(RIGHT_DOCK_FIXTURE)` | `app.section_drag = Some(SectionDrag::new(RIGHT_DOCK_FIXTURE))` (field widened, §5.4); pointers already come from the live `view_rects`, so no coordinate changes with `C`; every assertion kept (the narrow test's `"1200"` status text is the `RightDockTooNarrow` text) | §5.4 drag state | L3-003 |
| S31 | L3-003 `positive.json` `derived_geometry_1600x1000_no_right_dock`, `real_window_matrix`; `negative.json` N5 SideView, N31, N32 | 90-based rects, `OM9_UI_AUDIT_RIGHT_DOCK`, old receipt keys | per §12 | G16, §5.6 | L3-003 |

**Re-grep before L4-03, required** (paths include L3-003's own fixtures, the whole crate `src/` with its in-crate
tests, and the L3-002 validator, which after FEAT-003 holds case geometry):
`rg -n "DockSide::Right|dock_right|right-dock-|section_drag" standalone/crates/om9-app implementation/fixtures/M01 tools/validate_m01_l3_002_ui_matrix.py` and
`rg -n "\b(58|90|111|178|240|580|802)\b|LEFT_RAIL_MIN|COMMAND_(MIN|DEFAULT)" standalone/crates/om9-app/src standalone/crates/om9-app/tests implementation/fixtures/M01/OM9-L3-01-01-002 implementation/fixtures/M01/OM9-L3-01-01-003 tools/validate_m01_l3_002_ui_matrix.py`.
Every right-dock hit must use Snaps / `RIGHT_DOCK_FIXTURE` / `dock_right_fixture` or be a row above; every numeric or
symbolic hit must be command/rail-independent or a row above. Any other hit is added as a new S-row in the diffstat
note before editing.

Kept: `dead_notes_stay_unreachable` and every compile-time `const _` assertion (true with min 300, max 460 and a
smaller `COMMAND_MIN`), `window_minimum.rs`, the `startup.rs` tests, `hidden_right_dock_is_a_defined_state`,
`a_floated_menu_keeps_its_rectangle_and_docks_back`, the pure `rail_icon_edge` tests.

### 10.2 New unit tests (pure)

- `panel_ops`: `WorkArea::for_window` table (1280x720 → `[8,29,1264,663]`, 1600x1000 → `[8,29,1584,943]`; 0,
  negative, NaN, inf, 1e6 → `InvalidWindow`); the clamp table and every `negative.json` float case; each §5.2 row ok
  and error with `to_json()` unchanged on `Err`; `Close` → `CloseUnsupported`, Builders → `NotPermitted`;
  `resolve_drop` rules 1-8 (drop zone only when the dock is hidden and width ≥ 1200; narrow band only below 1200);
  D3 home and documented-index cases; a 2,000-step seeded xorshift sweep (no new crate) asserting no panic and the
  invariants.
- `dock_layout`: `LEFT_RAIL_MIN == LEFT_RAIL_DEFAULT == 300`; `normalize` widens a stored 240 to 300 **with** the
  diagnostic (proves the order before the clamp); L1 (Layers == InfoSettings + 1, Projects == Layers + 1); a schema-2
  JSON without `home` loads with no diagnostic; L6 (`ToggleCollapsed` on Layers / Info & Settings leaves
  `left_sections()` unchanged); L7 (Info & Settings hidden → visible order `…, Snaps, Layers, Projects, …`; shown
  again → Layers at 5); `COMMAND_MIN <= COMMAND_DEFAULT`.
- `ui`: `parse_audit_rail_width` 240 → `None`, 300 → `Some(300.)` (S21); `OM9_UI_AUDIT_PANELS` parser (known names,
  unknown → error, combined with user-docked-right → error).

### 10.3 New frame tests (real `MatrixApp::show`, synthetic events)

`tests/panels_reorder_collapse_float.rs` keeps the api.md §10 list (AC-3 … AC-10, the source-scan test that no UI
code outside `apply_panel_op` mutates sections, D6 L3 and L5) with L5 expecting **Snaps** in the right column and the
sizes below.

`tests/main_window_parity.rs` (new). Default layout, `OM9_UI_LAYOUT` unset. Sizes `{1280x720, 1366x768, 1440x900,
1600x1000, 1920x1080, 2560x1440}` at zoom 1.0, capped `{1280x688, 1076.8x534.4, 894.67x432}`, and SIMULATED zoom 1.5
for physical 1920x1080 and 2560x1440 (`startup_scaled` helper from `workbench_regions_sizes.rs`). Each size asserts:

1. `right_dock_rect.is_none()`, `dock.right_sections().is_empty()`; left outer width 300 ± 0.5.
2. Left order == `SectionId::ALL`; Layers bar top ≥ Info & Settings block bottom − 0.5 with no bar between; Projects
   bar below Layers; Builders last; Layers bar x-range inside `sidebar_rect`.
3. Layers visibility per D6 L2: at **every** size in the list above, **capped sizes included**, and at rail 300 and
   rail 460 (set before the first frame, as `left_stack_layers.rs::fresh` does), with `sidebar_scroll() == 0`: the
   Layers bar rect is fully inside `sidebar_rect` (`sidebar_rect.contains_rect(bar)` with 0.5 tolerance). The test
   prints `OM9_SWEEP layers_bar size=… rail=… y=…..… sidebar=…..…` for every case; L4-02 records the RED values
   (before G12, expected 337..351 / 397..411) and L4-03 the GREEN values. Fallback per D6 L2 if a case fails after
   L4-03: report the measured y, assert order + adjacency + reachability for that case only, change no constant,
   order or collapse state. Reachability-by-scroll is checked separately, only for user-changed layouts (Layers
   moved to the end of the column by `PanelOp::Place`, and rail 380): `request_sidebar_scroll(y)` with
   `y = section_blocks["layers"].0 − sidebar_rect.top + sidebar_scroll()` (content-space offset), then **one more
   frame**, then the Layers bar is inside `sidebar_rect`.
4. `cmd_rect.max.y <= toolbar_rect.min.y`; toolbar above all `view_rects`; grid TL/TR/BL/BR = LookingDown /
   Perspective / ThroughFinger / SideView.
5. Command box: `command_history_rows()` (`floor((history_height + 0.5) / (r + 2))`, §4.3) `== 2` at zoom 1.0
   default height; `>= 2` at 1.25 and 1.5; the stored `CommandLine.size` equals the outer command panel height
   (`command_panel.response.rect.height()`) within 0.5 at every size; `command_default_matches_font_metrics`; the
   printed line `OM9_SWEEP command C=… r=… history=… capture_estimate=42` feeds the final-report comparison (A15).
6. View-tab labels end with `+`. `toolbar_tabs_overflow == (measured content width > visible strip width)`, both
   printed; the `>>` button is present exactly when true. No fixed true/false per size is assumed.
7. A wheel event over the column gives `sidebar_scroll() > 0`, still > 0 on the next frame (G9 regression).
8. Widening the column to 460 (panel resize drag) stores 460 and section-body icon rects are 34x34 (L3-002 rule
   consumed); dragging the edge below 300 stores 300.
9. **UI-state-only snapshot (finding 7)**, compared before and after every interaction in this file and in
   `panels_reorder_collapse_float.rs` (roll-up of Layers, drag of Layers to a view and back, Escape-cancel, close box,
   `+`, `>>`, column widen, scroll, `close_right_menu_column`):
   ```rust
   struct DocSnapshot {
       document: serde_json::Value,   // serde_json::to_value(app.controller.document()): entity ids and attributes,
                                      // layers (id, name, color, visible, locked, …), layer_order,
                                      // current_layer, page_unit
       undo_len: usize, redo_len: usize,          // app.controller.coordinator()
       draft: String,                              // transcript draft
       workspace: om9_scene::Workspace,            // app.controller.workspace.clone(); Clone + PartialEq
   }
   ```
   om9-core exposes no undo-label getter and is backend-owned (not edited), so "undo top label" is covered by the
   unchanged document value plus unchanged `undo_len`/`redo_len`; recorded as a limit.
10. A close-box click warns and `to_json()` is unchanged; a `+` click warns and the workspace is unchanged; Main Menu
    `Reset` is disabled; the BUILDERS bar has no close-box rect (no `"om9-bar-close"` interaction for its key).
11. **No keystroke lost after a panel message** (§5.5): (a) close-box click on Snaps (history append), then
    `Event::Text("L")` in the **same** raw-input batch as the click and, in a separate run, in the **next** frame; the
    CMD draft equals `"L"` in both runs. The same-frame case works because the sidebar is drawn before the command
    panel and `command.rs` applies `focus_cmd` (cursor to the transcript end) before `TextEdit::show`. (b) The same
    two runs after a `+` click and after a drag released in the narrow band (1100x720, `RightDockTooNarrow`); these
    appends happen after the command panel in the frame, so the same-frame `Text` is consumed before the append and
    the next-frame run proves the repair. (c) Mutation evidence: with the `focus_cmd = true` line in
    `panel_message` removed, the next-frame run of (a) fails (recorded in L4-03, then restored byte-for-byte).

### 10.4 Real-window evidence (L4-05), required runner (finding 2)

Runner: **`tools/validate_m01_l3_003_panels.py`** (required). Preconditions: P0 holds and
`tools/validate_m01_l3_002_ui_matrix.py --self-test` exits 0. Output:
`build/standalone-validation/ui-artifacts/<date>-m01-l3-003-panels/`.

Steps:

1. Run `python tools/validate_m01_l3_002_ui_matrix.py --mode default --output <out>/l3-002-default` as a subprocess
   (reuses its cases and checks, no import). Require exit 0. Then apply the L3-003 PASS conditions below to every
   receipt it wrote.
2. Run the L3-003 cases (each with its own `OM9_UI_LAYOUT` in a temp profile, layout mode `default`):

   | Case | Window (physical) | Env |
   | --- | --- | --- |
   | `big-1920x1080` | 1920x1080 | — |
   | `big-2560x1440` | 2560x1440 | — |
   | `dpi150-1280x720` | 1920x1080 | `OM9_UI_AUDIT_ZOOM=1.5` (SIMULATED) |
   | `widened-460-1280x720` | 1280x720 | `OM9_UI_AUDIT_RAIL_WIDTH=460` |
   | `scenario-<name>-1280x720` for each §5.6 scenario | 1280x720 | `OM9_UI_AUDIT_PANELS=<name>` |

   SIMULATED 150% real-window evidence is also `lap150-1366x700` from step 1.
3. Case result ∈ {`PASS`, `FAIL`, `CLAMPED_FRAME_TEST_ONLY`}. A receipt whose `regions.window` is smaller than the
   requested point size by > 1 in either axis is `CLAMPED_FRAME_TEST_ONLY` (the test host display is 1440x900, so the
   `big-*` and `dpi150-*` cases are expected to clamp there); it names the covering test
   (`main_window_parity::*`) and is **never counted as PASS**. The PASS conditions are still checked on what was drawn;
   any violation makes it `FAIL`.
4. Exit 0 iff no `FAIL`. The summary JSON lists PASS / FAIL / CLAMPED counts separately.

PASS conditions per receipt (default-mode and default-layout scenario receipts):

- `dock.right_dock_visible == false` and `dock.right_dock_sections == []` (except `scenario-dock-right`, which must have
  `== ["snaps"]` and `layout_parity.fixture` naming the user-docked scenario);
- `dock.left_sections` == the keys of `SectionId::ALL` in order (for `scenario-reorder`, the scenario's expected order,
  still with Layers directly after Info & Settings unless the scenario moved Layers itself);
- `dock.section_title_bars_px.layers.column == "left"`, its y between the `info-settings` and `projects` bars with no
  other bar between, `panels.layers_placement.below_info_settings_block == true` and `bar_between == false`;
- the Layers bar x-range inside `layout_parity.sidebar_rect_px`; vertically inside it at non-capped sizes per D6 L2;
- `layout_parity.command_above_toolbar == true`, `layout_parity.command_history_rows >= 2`;
- `layout_parity.bar_style == "dark"`, `layout_parity.view_tabs[-1] == "+"`.

`--self-test` rejects doctored receipts: right dock shown in default mode; Layers in `right_sections`; Layers missing
from the left; Layers before Info & Settings; a bar between them; Layers bar outside the sidebar; command below the
toolbar; `command_history_rows == 1`; a clamped case reported as PASS; a scenario receipt without `panels`.

PNGs viewed (with the read tool) and the observation written into
`implementation/evidence/M01/OM9-L3-01-01-003/OM9-L4-01-01-003-05.ui-review.json` as **agent viewing, not a human
sign-off**: `l3-002-default/min-1280x720`, `lap125-1366x680`, `lap150-1366x700`, `big-1920x1080` (or its clamp),
`widened-460-1280x720`, `scenario-dock-right-1280x720` (labelled user-docked), `scenario-scroll-column-1280x720`.
For each: Layers in the left column below Info & Settings, no right dock (except dock-right), dark bars, two-line
green command box above the toolbar, `+` tab, and `>>` when shown.

### 10.5 Regression gates

As `requirements.md` AC-11…AC-17: targeted tests, `cargo test -p om9-app`, `cargo test --workspace` (`-j 1` if os
error 1455 returns), `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`,
`cargo fmt --all -- --check`, `cargo tree --workspace | grep -i qymcad` empty, `python tools/validate_m01_l3_003_panels.py
--self-test`, and `tools/public_source_audit.py`. The full audit's real exit code is recorded (1 expected from private
trees, never reported as passed); the scoped mirror must exit 0 and its control run 1.

## 11. Testability split

- Pure unit tests: `panel_ops.rs`, `normalize`, `documented_index`, D3 home rules, parsers.
- Headless frame tests (egui `Context::run_ui` with events): drag, drop, preview, Escape, scroll, header clicks, `>>`,
  `+`, widths, command rows, order at sizes, the UI-state-only snapshot.
- Real window only: OS minimum enforcement, real DPI, second monitor, and the look against Matrix9. These go on the
  M01-UI human checklist (at most 6 items) and stay `NOT_SIGNED`.

## 12. Revisions required to the first-run artifacts (L4-01/L4-02, before RED)

- `api.md`: banner already present ("superseded where it conflicts by design.md"; update to rev 3). D6 L5: Projects →
  **Snaps**. §1: Part A scope incl. the command box. §3: `PanelOp::Close`, `PanelOpError::CloseUnsupported`,
  `CancelReason::RightDockTooNarrow`, `DropZones.right_drop_zone` / `right_too_narrow_band`, `ColumnSide` /
  `SectionHome` in `dock_layout.rs`; remove `RIGHT_DROP_STRIP` and `MENU_ICON_*`. §5: drop precedence of §5.4. §7:
  `panels` and `layout_parity` objects of §5.6, `OM9_UI_AUDIT_LAYOUT_MODE` instead of `OM9_UI_AUDIT_RIGHT_DOCK`, the
  `scroll-column` scenario, §10.3 sizes. §10: sizes and §10.1 table. §11 "Narrow window": reword for capped screens.
- `positive.json`: header text "right drop strip 48" → "right drop zone 28 (`RIGHT_DROP_ZONE_WIDTH`)"; P11 sweep →
  §10.3 sizes (drop `widths [800, 1024, …]`, `heights [400, …]`); add P14 `scroll-column`, P15
  `widen-column-icons-follow` (460 → 34), P16 `right-workspace-order` (command rows 2, `+` tab), P17
  `command-box-two-history-rows`.
- `negative.json`: N5 strip per S28; N31 → Snaps, labelled user-docked; N32 → Snaps docked right by the user,
  `resize_to` capped `1076.8x534.4`, expect `RightDockYielded`, Snaps still right, `dock_left` restores index 3; add
  N35 `close-box-refused`, N36 `plus-view-tab-refused`, N37 `left-width-below-300-normalized-with-diagnostic`, N38
  `release-in-narrow-band-below-1200-cancels`, N39 `audit-rail-width-240-rejected`.
- `source_crosswalk.json`: OM9 REUSE-IN-PLACE rows for `Panel::left` + `ScrollArea`, `toolbars_ui`, the L3-002
  audit-mode hook, `compute_right_drop_zone`, `rail_icon_edge`; G9-G14 and G16; `legal_reuse_status` =
  `HUMAN_APPROVED_2026-10-16 (user: "M00 duyệt")` (QymCAD reuse only); `m00_gate` stays `NOT_PASS` for the
  unapproved M00 parts.

## 13. Environment and spec findings

- `OpenMatrix9_Codex_Spec_v2/` is **absent** in this checkout, so RCORE-00, CODE_GUIDE, BACKEND_POLICY and
  MATRIX9_UI_CONTRACT could not be read. Binding sources are the local `OM9_Milestones` l3/l4 JSON (read only), the
  screenshot analysis, the steering and the approved skill references.
- `tools/validate_m01_spec.py` and other `test_commands_preview` tools are absent ("absent, not run").
  `om9.m01.oracle.v1.5` has no local definition.
- QymCAD: `vendor-research/QymCAD` HEAD `746a17f127897e921a57118b6caad08968602bf5`; notices exist; nothing copied.
- The left stack order is owned by OM9-L3-03-07-001 (L2-03-07); L3-003 keeps it on every restore.

## 14. Business-rule changes (before → after; skill text needs the user's consent, no `skills/` edit)

| # | Rule | Before | After |
| --- | --- | --- | --- |
| BR1 | Left column width | default 300, resizable 240-460 (outer, L3-002 task) | default 300 outer, **minimum 300** (agent choice, option (b), for the user to confirm), user-widenable to 460; a stored width < 300 is widened with a diagnostic |
| BR2 | Menu icon size | follows the column by `rail_icon_edge` (L3-002 task, 18-34) | unchanged rule; UI range becomes 22-34 because the column cannot go below 300 |
| BR3 | Section title bar | light gradient, black text, ± glyph | dark bar, light UPPERCASE title, close box (not available yet, warns) |
| BR4 | Column overflow | scroll pinned to 0 (defect) | scrolls/clips like Matrix9; never compressed |
| BR5 | Main Menu / Snaps text rows | wrap | clipped at the column edge; Main Menu gets a disabled `Reset` |
| BR6 | Toolbar tabs | hidden scroll only | `>>` overflow menu |
| BR7 | Viewport tabs | 4 tabs | 4 tabs + `+` (not available yet, warns) |
| BR8 | Right dock | View toggle with no visible effect; drop zone (L3-002 task) | hidden by default; appears only when the user docks a menu there: **drag to the right drop zone (or onto the shown column), or the existing "Dock right ▶" button of a floating menu**; View item closes it (docks all back) or is disabled; audits/fixtures use Snaps, labelled user-docked |
| BR9 | Re-dock | append at the end, expand | home index or documented position; roll-up kept |
| BR10 | Builders slot | movable like a menu | fixed; visible refusal |
| BR11 | Escape | cancels the CMD draft | during a menu drag, cancels only the drag |
| BR12 | Drag feedback and recovery | none; clamp only on resize | live preview, visible cancel, clamp to the work area on every drop/move/resize, no other monitor (warning) |
| BR13 | Layers placement | default only, not protected on restore | L1-L7 (D6) |
| BR14 | Floating menu window | collapsible egui window | not collapsible; roll-up flag kept for re-dock |
| BR15 | Command box height | default 90 (≈ 4 history rows), min 58 | two history rows + one input row from font metrics (`C`), min `min(58, C)`; brings code in line with the already approved `command-and-curve.md` rule, so no new skill text for this row |

Proposed text for `skills/openmatrix9-workspace-contract/SKILL.md`, new section "Bảng menu và bố cục cửa sổ chính
(OM9-L3-01-01-003)", shown in the final report and added only after the user agrees:

- Cột trái rộng mặc định 300 điểm logic (không co theo cửa sổ, chỉ theo DPI), tối thiểu 300, người dùng kéo rộng tới 460; icon menu lớn theo cột (22 đến 34).
- Thứ tự cột trái: ICON HISTORY, MAIN MENU, DISPLAY, SNAPS, INFO & SETTINGS, LAYERS, PROJECTS, rồi BUILDERS; Layers luôn ngay dưới Info & Settings ở mọi kích thước được hỗ trợ; cột dài hơn cửa sổ thì cuộn/cắt, không nén.
- Thanh tiêu đề tối, chữ HOA, ô đóng bên phải (chưa hỗ trợ, báo cảnh báo); bấm tiêu đề để thu gọn, kéo để đổi chỗ, thả vào viewport để nổi.
- Dock phải ẩn mặc định, chỉ hiện khi người dùng kéo menu sang mép phải (hoặc bấm "Dock right ▶" của menu nổi); audit dùng Snaps và ghi rõ là ca người dùng dock.
- Dock lại trả menu về vị trí cũ (hoặc vị trí Matrix), giữ trạng thái thu gọn; Esc khi đang kéo chỉ huỷ việc kéo; menu nổi luôn được kéo về trong vùng làm việc.
- The L3-002 line "Khi dock phải nhường hoặc rail trái bị thu hẹp …" becomes "Khi dock phải (do người dùng dock) nhường …". The left-rail-shrink case can no longer happen.

## 15. Open assumptions

- **A1.** 300 pt default from the steering (user decision), consistent with the manual's "mirrors its counterpart"
  (inference). 214 px in the capture is treated as downscaled. Pending a second Matrix9 capture.
- **A2.** Minimum 300 (option (b)); agent choice for the user to confirm.
- **A3.** Icon scaling is the L3-002 task's `rail_icon_edge` (linear, rounded, clamped); no curve is invented here.
- **A4.** Bar height 14 and the dark greys are chosen, not measured (the bitmap is not in the repo).
- **A5.** The close box is drawn but not functional; Matrix9 close semantics and reopen path are not evidenced.
- **A6.** `C` is measured from the egui monospace 12 pt row height at zoom 1.0. "Two history rows" in points may
  round differently at 1.25/1.5; a shortfall is reported, not tuned away.
- **A7.** Menu-bar plugin menus (Rhino 5.0, Blend, Clayoo) not added: menu-bar scope, third-party names.
- **A8.** The Display dropdown stays `Wireframe ▾` (user-approved grid/viewports reference), not `Shaded`.
- **A9.** Icon History keeps its Undo/Redo icons, not previous/next arrows.
- **A10.** Default toolbar tab stays Standard; Transform in the capture is session state.
- **A11.** The bottom status bar (L3-007) stays; it does not appear in the capture.
- **A12.** "Layers visible at scroll 0" is expected at 1280x720/width 300 but unmeasured; D6 L2 defines the failure
  rule.
- **A13.** egui 0.36.2 behaviours read from source, not yet observed: `constrain_to`, `current_pos`/`fixed_size`
  resync, `ScrollAreaOutput.state.offset`/`content_size`/`inner_rect`, `scroll_to_me` in a horizontal `ScrollArea`,
  clip-limited interaction. L4-02 RED and L4-03 frame tests confirm them; a failure is a design defect to report.
- **A14.** M00 approval covers QymCAD reuse only (user's scope answer).

## 16. Out of scope

L3-004 Reset/Reload Menus and the Main Menu Reset (Panic) function (placeholder only); L3-005 persistence (no new
write path; schema stays 2); L3-008 builder contents; L3-006/L3-007 rules (reused); OM9-L2-01-02/03 backends;
menu-bar contents; Layers panel content (L2-03-14); multi-monitor floating (D2 Option B); a second Matrix9 capture;
the L3-002 task's rail/icon/audit-mode/validator work (§2.1); any `planning_state` change, gate signature or skill
edit.

## 17. Revision notes (rev 3) and responses to `design-review.md`

Spot check against the tree (14:01): `LEFT_RAIL_MIN 240`, `LEFT_RAIL_MAX 460`, `LEFT_RAIL_CHROME_PX 5`,
`ICON_BASE/MIN/MAX_EDGE 22/18/34`, `RIGHT_DOCK_FIXTURE Snaps`, `RIGHT_DROP_ZONE_WIDTH 28`, `COMMAND_MIN 58`,
`COMMAND_DEFAULT 90`, `OM9_UI_AUDIT_RIGHT_DOCK` rejected at launch, G9 still present (`chrome.rs` ~538-561), the L3-002
validator still on `OM9_UI_AUDIT_RIGHT_DOCK=1`. L4-03 repeats this after P0.

| # | Finding | Response |
| --- | --- | --- |
| 1 HIGH | Baseline changed; overlapping L3-002 task | Addressed: §0 baseline table, §2.1 ownership split + P0 sequencing, consume `LEFT_RAIL_MAX` 460 / `rail_icon_edge` / `LEFT_RAIL_CHROME_PX` / `RIGHT_DOCK_FIXTURE` / `RIGHT_DROP_ZONE_WIDTH` / `OM9_UI_AUDIT_LAYOUT_MODE` / `dock_receipt`; deleted `MENU_ICON_*`, `menu_icon_size`, `RIGHT_DROP_STRIP`, `apply_audit_right_dock`; `LEFT_RAIL_MIN` option (b) 300 with S20-S22; M00 status conflict flagged to the orchestrator |
| 2 HIGH | No default-mode matrix producer | Addressed: §10.4 required `validate_m01_l3_003_panels.py`, reusing the L3-002 validator `--mode default` by subprocess, own case table, result classes, PASS conditions, self-test, named PNGs |
| 3 HIGH | Command box height vs two-line rule | Addressed: §4.3 font-metric `C`, `COMMAND_MIN = min(58, C)`, outer sync if drift, guard test, S19/S23-S27; A6 rewritten; removed from §16 |
| 4 MEDIUM | Superseded list incomplete/stale | Addressed: S1/S2/S14 marked done (verify), S16-S18 added, Owner column, required re-grep |
| 5 MEDIUM | Width evidence quoted loosely | Addressed: §0 cites the manual sentence as an inference, the steering lines as the user decisions, and why 214 px is not used; final report lists the additions as user-decided |
| 6 MEDIUM | Layers at scroll 0 unmeasured | Addressed: D6 L2 failure rule, measurement source, §10.3 item 3 |
| 7 MEDIUM | Snapshot misses layer flags/units | Addressed: §10.3 item 9 snapshots the whole serialized document (layers with all flags, `layer_order`, `current_layer`, `page_unit`, entities) plus undo/redo lengths, draft, workspace; undo-label limit recorded |
| 8 NIT | `request_sidebar_scroll` signature | Addressed: `-> bool` (§4.2, §7) |
| 9 NIT | Scroll coordinate space | Addressed: §10.3 item 3 formula + one more frame |
| 10 NIT | `normalize` order | Addressed: LeftRail diagnostic before the generic clamp (§4.2, §10.2) |
| 11 NIT | Header lists L4-04 | Addressed: header lists `-01`, `-02`, `-03`, `-05` |
| 12 NIT | Second right-dock entry | Addressed: kept; named in §4.4 and BR8 |
