# Design review: `OM9-L3-01-01-003` design.md (revision 3)

Reviewer: fresh design-review step, workflow `wf_9fe91738fb13ff99`, 2026-10-16.
Reviewed: `implementation/design/M01/OM9-L3-01-01-003/design.md` (77,412 bytes, written 14:12).
Binding sources checked: `.agents/tasks/m01-l3-003-panels/user-decisions.md` (all sections),
`docs/analysis/matrix9-main-window-layout.md`, steering `matrix9-main-window-layout`, the L3-003 `l3.json`,
`requirements.md` (headings and FR/D sections). Code read in `standalone/crates/om9-app` (`dock_layout.rs`, `ui.rs`,
`ui/chrome.rs`, `ui/command.rs`, `startup.rs`, `transcript.rs`, `workbench_regions.rs`), `om9-core` (`model.rs`,
`transaction.rs`), `om9-scene/src/workspace.rs`, the L3-002 left-dock task (`task.json`, FEAT-001..003,
`left-dock-measurements.md`) and the L3-003 fixtures. No build or test was run.

## Verdict

**CHANGES_REQUESTED**: 0 HIGH, 6 MEDIUM, 8 NIT.

Revision 3 is close. It gets every key point right: the left order, Layers directly below Info & Settings, the
Command box at the top of the right workspace, the right dock kept but off, the 1280x720 minimum, the disabled
placeholders, the S1-S28 superseded list, and panel state never touching the document or Undo. The open items are
stale baseline facts (the L3-002 measurements now exist and settle D6 L2), one unreconciled source conflict on the
command-box height, and missing details in the fixture edits, the source-scan test and the CMD-history append path.

## Findings

### 1. MEDIUM: the baseline is already stale, and the new measurements settle facts the design still lists as unknown

Where: §0 baseline table, §2.1, D6 L2, §9 "Capped 894.67x432", §10.3 item 3, A12.

Problem:
- §0 says `left-dock-measurements.md` "does not exist yet". It exists (14:09, before the design was written at
  14:12). FEAT-001 status is now `completed` (`FEAT-001.json`, 14:10).
- The `ui.rs` SHA-256 in the table is `50F74809…8300`. The tree has `6F924EA0…3597`. `dock_layout.rs`, `chrome.rs`
  and the validator still match.
- The measurements (default rail 300, 1280x720 and 1600x1000) give: Layers bar y 337..351, Info & Settings 278,
  Projects 527, sidebar rect y 23..721, scroll 0. At rail 460 Layers is at 397..411. Bar y does not depend on window
  height, because it is identical at 720 and 1000.
- So the design's "unmeasured" status (A12, the `<measured>` placeholders in §10.3 item 3) is out of date. The §9 claim
  "Capped 894.67x432 … Layers below the fold" is wrong. That window's sidebar runs to about y 433, so a 337..351
  (or 397..411) bar is visible at scroll 0.
- Item 3 therefore asserts only reachability-by-scroll at capped sizes, which is weaker than the facts allow. The user
  requires Layers below Info & Settings "dùng ở độ phân giải nào".

Fix:
- §0: mark `left-dock-measurements.md` present and FEAT-001 `completed`. Replace the `ui.rs` hash, or replace the
  per-file hashes with "re-captured at P0" so they cannot go stale.
- D6 L2 / A12: "Measured by FEAT-001 (pre-L3-003): Layers bar 337..351 at rail 300 and 397..411 at rail 460, sidebar
  starts at y 23. L3-003 changes Main Menu/Snaps row heights (G12) and must re-measure in L4-02 (RED) and L4-03."
- §10.3 item 3: at every size in the list, **including the capped sizes**, and at rail 300 and 460: assert the
  Layers bar rect is fully inside `sidebar_rect` at `sidebar_scroll() == 0`. Keep the existing failure rule (report
  the measured y, change no constant, order or collapse state) as the fallback if a measured case fails. Keep the
  scroll-to-reach check for the user-widened, user-reordered or expanded cases only.
- §9: delete "Layers below the fold" for 894.67x432 and state the expected bar range.

### 2. MEDIUM: "two-line Command box" (steering and screenshot) vs "two history rows + one input row" is not reconciled

Where: §0 precedence, §4.3, §15 A6, BR15.

Problem: The design's own precedence puts the steering (2026-10-16) above other sources. The steering and the
analysis say "green two-line `Command:` box", "about 30 px tall" in a capture the design itself treats as downscaled
(214 → 300, factor about 0.71, so about 42 logical points). The design adopts the 2026-10-06 approved rule
"two history rows plus one live input row". That gives C ≈ 56, three text rows. It calls this consistent with "the
two-line box in the capture" without explaining the different line count. A reviewer cannot tell whether "two-line"
means two rows in total or two history rows.

Fix: Add assumption A15 and use it in §4.3 and BR15. "The steering's 'two-line' is read as two history rows above
the pinned input row, per the approved `command-and-curve.md` rule (2026-10-06). The capture estimate (~30 px
downscaled, about 42 pt) is smaller than C (measured, about 56 pt). The rule wins because it is an explicit approved
business rule, and the capture height is an estimate." The final report must show the measured C next to the capture
estimate and ask the user to confirm. Do not tune C toward 42 without that answer.

### 3. MEDIUM: §12 and the re-grep do not cover L3-003's own fixtures or the in-crate drop-zone tests that move with C

Where: §10.1 re-grep, §12 `positive.json` / `negative.json`, S28.

Problem:
- `positive.json` holds `derived_geometry_1600x1000_no_right_dock` with `center [300,178,1300,802]`, `toolbar
  [300,111,1300,67]` and view rects. All of these move with `C`.
- `positive.json` also has `real_window_matrix.env.OM9_UI_AUDIT_RIGHT_DOCK: "unset"` and receipt keys
  `panels.layers_placement.{side, directly_below, layers_bar_visible}`. These conflict with the §5.6 keys
  (`below_info_settings_block`, `bar_between`, …) and with the renamed env var.
- `negative.json` N5 names `SideView [951, 580, 649, 400]`, which moves to y about 563 with C = 56. §12 lists only the
  N5 strip.
- The required re-grep searches `tests/`, `workbench_regions.rs` and the L3-002 fixtures only. It misses
  `implementation/fixtures/M01/OM9-L3-01-01-003`, the in-crate tests in `src/ui/chrome.rs` (the FEAT-001 drop-zone
  tests use canvas coordinates), `src/ui.rs` tests and the L3-002 validator (after FEAT-003 it will hold case geometry
  and the `rail240` case).

Fix:
- §12 `positive.json`: recompute `derived_geometry_1600x1000_no_right_dock` with Δ = C − 90 (center y `88 + C`,
  height `892 − C`; toolbar y `21 + C`; view rows per S23); rename `layers_placement` keys to the §5.6 names; replace
  the env entry with `OM9_UI_AUDIT_LAYOUT_MODE: "default"`.
- §12 `negative.json`: N5 SideView rect per S23. Keep the pointer only if it stays inside after recomputation.
- Extend the re-grep paths to
  `implementation/fixtures/M01/OM9-L3-01-01-003 standalone/crates/om9-app/src tools/validate_m01_l3_002_ui_matrix.py`,
  with the same rule (every hit is command/rail-independent or gets an S-row).

### 4. MEDIUM: the source-scan test ("no UI code outside `apply_panel_op` mutates sections") has no defined banned set or allowlist

Where: §5.2 Owner, §8, §10.3 (`panels_reorder_collapse_float.rs`).

Problem: Today `ui.rs` / `chrome.rs` call `dock.reset_to_default` (reset, reload, load), `dock.clamp_floats` (resize),
`dock.set_float(PanelId::CommandSearch, …)` (F6 readback), `dock.set_size` (`sync_panel_size`, audit rail width),
`dock.set_visible(PanelId::RightDock, …)` (`toggle_right_dock`, kept public) and the audit layout-mode hook (docks
`RIGHT_DOCK_FIXTURE`). The test is described only in words. As written it either fails on these legitimate callers
or is too loose to prove anything.

Fix: Specify the test exactly.
- Banned outside `dock_layout.rs`, `panel_ops.rs` and `MatrixApp::apply_panel_op`: the tokens
  `.dock_section(`, `.place_section(`, `.move_section(`, `.float_section(`, `.toggle_section(`,
  `.set_section_collapsed(`, `.set_section_visible(`, `.set_section_height(` on a `dock` receiver.
- Allowed: `reset_to_default` (L3-004 paths), `clamp_floats`, `set_float(PanelId::CommandSearch, …)`, `set_size`
  (region sync and audit rail width), `set_visible(PanelId::…)` (region panels). The audit layout-mode hook must call
  `apply_panel_op(PanelOp::Dock { section: RIGHT_DOCK_FIXTURE, side: Right })` (one-line change) rather than
  `dock_section`.
- The scan strips comments and string literals, and the test lists the allowlisted call sites by file so that any new
  call fails.

### 5. MEDIUM: new CMD-history appends from panel ops re-open the known stale-cursor keystroke drop

Where: §5.5 Messages, §7 (every "Warning + history" row), D5.

Problem: `left-dock-measurements.md` proves that `Transcript::append_history` moves protected history past the
stored CMD cursor, so `Transcript::insert` refuses the next typed text (index < `protected_chars`). FEAT-001 fixed only
the audit script and recorded the product caveat. L3-003 adds many new history appends: every panel note or error,
`Menus: {error}`, the close box, `+`, `RightDockTooNarrow`, the drag cancel and `close_right_menu_column`. The first
keystroke after such a click can be silently lost. That breaks the l3.json manual rule ("never silently do nothing")
and the CMD-focus contract. It is not a document change, but it is lost user input.

Fix: In the single message path of `apply_panel_op` (and the `+` / `>>` handlers), after any
`transcript.append_history`, set `self.focus_cmd = true`. This is the existing mechanism that puts the cursor at the
transcript end. Chrome clicks already return focus to CMD under the CMD-focus contract. Add a frame test: close-box
click (history append), then `Event::Text("L")` in the same and the next frame. The draft must equal `"L"`. Record that
the general fix in `Transcript::append_history` (shift the stored cursor by the appended char count) is out of scope
and stays a flagged caveat.

### 6. MEDIUM: wiring for `command_history_rows()` and the outer command-height sync is conditional where the code already answers the question

Where: §4.3 "Command size sync", §10.3 item 5, §5.6 `command_history_rows`.

Problem:
- `ui.rs` line ~1019 syncs `ui.max_rect().height()` inside a frame with `inner_margin(3)`. The stored size is
  therefore always outer − 6. The design makes the outer sync conditional on a measurement ("if the two differ by more
  than 0.5"), but the drift is certain.
- With `COMMAND_MIN = C`, the inner value (C − 6) is clamped up to C by `set_size`. Stored and drawn sizes then agree
  only by accident, and a later `COMMAND_MIN` change would break that silently.
- The row metric `floor(history_height / (r + 2))` has no epsilon. With C chosen so that
  `history_height == 2·(r + 2)` exactly, a physical-pixel rounding of −0.01 gives 1 and fails `== 2`. The TextEdit
  galley rows are `r` high (no +2), so the metric is a proxy, not the visible row count.

Fix:
- Make the outer sync unconditional. Keep the `Panel::top("command").show(..)` response and call
  `sync_panel_size(PanelId::CommandLine, response.response.rect.height())` after `show`. Make S26's slack assertion
  exact (`<= 0.5`).
- Define `command_history_rows = floor((history_height + 0.5) / (r + 2))` with `r` from
  `fonts.row_height(TextStyle::Monospace.resolve(style))` at the live zoom. Assert `== 2` at zoom 1.0 and `>= 2` at
  1.25 and 1.5.

### 7. NIT: Builders bar and the close box

Where: §4.2 Title bar, §4.6, D4.

The §4.6 table says "every section bar" has a close box. A Builders close click then shows the D4 text "cannot be
rolled up, moved or floated", which does not mention closing. Fix: either draw no close box on BUILDERS, or give
`Close { BuilderSlot }` its own text: "Builders is reserved for builder tools and cannot be closed". Pick the first,
since the slot is not a menu.

### 8. NIT: `DocSnapshot.workspace` type is a placeholder

Where: §10.3 item 9. Write `workspace: om9_scene::Workspace` (cloned from `app.controller.workspace`; it derives
`Clone, PartialEq`).

### 9. NIT: the narrow band and the drop zone cover different y-ranges

Where: §5.4 rules 3-4, `DropZones`. The drop zone (≥ 1200) is the last 28 pt of the union of `view_rects`. The narrow
band (< 1200) is the rightmost 28 pt of the whole window, including the menu bar, command box and toolbar. This
matches today's `settle_section_drag`, but the design should say it on purpose: "the narrow band spans the full window
height, as in L3-002; the drop zone spans the views only". S28 should also say "union of `view_rects`
(`compute_right_drop_zone`)", not "canvas".

### 10. NIT: A1 should name the DPI-only rule itself as the assumption

Where: §15 A1. `user-decisions.md` §3 asks to "record [a fixed logical column width scaled only by DPI] as an
assumption pending a second Matrix9 capture". A1 records the 300 value but not the scaling rule. Fix: "A1. Fixed
logical width (default 300 pt), scaled only by DPI, never by window size, is an assumption pending a second capture."

### 11. NIT: list known visible differences from the capture in the final report

Where: §4.6, §15 A7-A9, A11. The design does not add the Icon History arrows, the `Coarse Mesh` / `Shaded` dropdowns,
the menu-bar plugin menus, or the status-bar removal. Leaving panel contents to their owners is the right scope. The
final report must still list them as "differences from the Matrix9 capture, not added (owner / reason)", so the user's
"giống y chang" requirement is answered openly.

### 12. NIT: a saved layout keeps the old command height

Where: §4.3. `normalize` keeps a stored `CommandLine.size = 90` (inside `[COMMAND_MIN, COMMAND_MAX]`), so a user with
a Set-as-Default file still sees about four history rows. State this: "a saved size is user state and is not
rewritten; Reset Menus gives C".

### 13. NIT: PNG paths may not match the L3-002 validator's output layout

Where: §10.4 PNG list. FEAT-003 plans `default/` and `user-docked-right/` subfolders under the output directory, so
`l3-002-default/min-1280x720` may be `l3-002-default/default/min-1280x720`. Fix: "resolve each PNG from the receipt
paths the validator writes, not from a fixed path".

### 14. NIT: the 1440x900 host claim is environment-dependent

Where: §10.4 step 3. In this session the process sees one 1440x900 screen (remote display). The clamp
classification is computed per receipt, which is correct. Reword "the test host display is 1440x900" to "on the
current host (observed 1440x900)" so a different session does not read it as a constant.

## Verified assumptions

- `SectionId::ALL` = IconHistory, MainMenu, Display, Snaps, InfoSettings, Layers, Projects, BuilderSlot
  (`dock_layout.rs` ~198). Layers index 5 = InfoSettings + 1, so no panel is missing.
- `SUPPORTED_MIN_WINDOW` 1280x720 and `PLAN_FLOOR_WINDOW` 800x400 (`workbench_regions.rs`). `effective_min_size`
  gives 1280x688, 1076.8x534.4 and 894.67x432 (`startup.rs` tests). No `560` / `286` remains in `om9-app`, the M01
  fixtures or the L3-002 validator.
- `LEFT_RAIL_MIN` 240, `LEFT_RAIL_MAX` 460, `LEFT_RAIL_DEFAULT` 300, `LEFT_RAIL_CHROME_PX` 5, `ICON_*_EDGE` 22/18/34,
  `RIGHT_DOCK_FIXTURE = Snaps`, `RIGHT_DROP_ZONE_WIDTH` 28, `COMMAND_MIN` 58, `COMMAND_DEFAULT` 90, `NARROW_WINDOW_WIDTH`
  1200, `SCHEMA_VERSION` 2.
- `Panel::left("matrix-sidebar")` is declared before `Panel::top("command")`, so the left column is full height and
  the Command box sits at the top of the right workspace. `min_size(LEFT_RAIL_MIN)`, `max_size(LEFT_RAIL_MAX)` and
  `resizable(true)`.
- `resolve` shows the right dock only when a visible section is docked right and width ≥ 1200. It ignores
  `PanelId::RightDock.visible`, so G14 is real (View item "Open Right Menu Column" calls `toggle_right_dock`).
- G9 is real: `std::mem::take(&mut self.sidebar_scroll)` feeds `.vertical_scroll_offset` every frame and the read-back
  key `"matrix-sidebar-sections-offset"` is never written.
- G10 is real: the light gradient `rgb(199,199,186)`→`rgb(151,151,141)`, black text, `+`/`−` glyph. G12 is real:
  `horizontal_wrapped` in the Main Menu and Snaps rows.
- `dock_section` sets `collapsed = false` and appends at the column end (D3 change is needed).
- `SectionState` and `DockLayout` do not deny unknown fields, so the optional `home` field is compatible both ways.
- The command font is `TextStyle::Monospace` = `FontId::monospace(12.)` (`chrome.rs` style). `command.rs` uses
  `row = row_height + 2`, `history = area − row − 2`, frame inner margin 3. The §4.3 formula for C matches this code.
- The approved rule "two history rows plus one live input row, based on font metrics" exists in
  `skills/openmatrix9-workspace-contract/references/command-and-curve.md`.
- `Document` derives `Serialize`. `TransactionCoordinator::undo_len()` / `redo_len()` exist, and
  `Controller::document()` / `coordinator()` are public. The §10.3 item 9 snapshot can be built without editing
  om9-core.
- The Escape chain exists at the top of `MatrixApp::show` (file op, layer dialog, popups, then CMD cancel). A new first
  branch for an active drag is feasible.
- The L3-002 task's FEAT-001 is completed with the names the design consumes (`parse_audit_rail_width` 240..=460,
  `AuditLayoutMode`, `dock_receipt`, drop-zone functions). FEAT-002 and FEAT-003 are `pending`, so P0 is not met yet.
- §4.1 rect arithmetic (center `612 − C`, views `(610 − C)/2`, toolbar `21 + C`) and S23/S28 at 1600x1000 are
  consistent with today's 90-based numbers.

## Unverified or wrong assumptions

- Wrong: "`left-dock-measurements.md` does not exist yet" (§0). It exists (finding 1).
- Wrong: the `ui.rs` baseline hash (finding 1).
- Wrong: "Capped 894.67x432 … Layers below the fold" (§9). The measured bar y 337..351 is inside a sidebar ending at
  about 433 (finding 1).
- Stale: A12 / D6 L2 "unmeasured" (finding 1).
- Unreconciled: "two-line" Command box vs the 3-row rule (finding 2).
- Conditional where the code already answers: command outer vs inner drift (finding 6).
- Unverified (accepted as A13, to be confirmed by RED tests): egui 0.36.2 `constrain_to`, `current_pos`/`fixed_size`
  resync, `ScrollAreaOutput.state.offset`, `scroll_to_me` in a horizontal `ScrollArea`, clip-limited interaction.
- Unverified: the L3-002 validator `--mode default`, `--self-test` and its output layout. FEAT-003 has not been built
  yet (finding 13). The P0 precondition covers this.
- Not checked here: the api.md body. The design declares itself binding over api.md and §12 lists the edits.
