# Requirements: OM9-L3-01-01-003 Reorder Collapse And Float Panels

Milestone M01 > OM9-L2-01-01 Febe Application Shell Docking > OM9-L3-01-01-003. Request (2026-10, Vietnamese): "làm xong các việc chưa xong cho xong M01 L2-01-01 L3-003" = finish every unfinished item of this L3. The model for "finished" is the closed-out sibling OM9-L3-01-01-002 (`.agents/tasks/m01-l3-002-docking/final-report-vi.md`). This document says what done means. It does not design or implement.

## 1. Summary

The L3 goal is "draggable/collapsible/floatable panels". Positive oracle: "Move, collapse, float permitted dock panel; restore without losing state". Negative oracle: "Dragging panel outside screen can be recovered/reset". Manual gate: M01-UI.

### 1.1 What exists today (checked by reading code; corrects the brief)

The brief said nothing had been implemented for L3-003. That is half right.

- No L3-003 artifacts exist. `implementation/design/M01/OM9-L3-01-01-003/`, `implementation/fixtures/M01/OM9-L3-01-01-003/` and `implementation/evidence/M01/OM9-L3-01-01-003/` are absent. No L3-003 task notes, validation doc or progress-ledger entry exist. `l3.json` is `SPEC_ONLY_NOT_IMPLEMENTED` and the four `l4.json` are `NOT_STARTED`.
- Behaviour code does exist. It was added in the 2026-10-16 L2-01-01 docking pass (`docs/validation/2026-10-16-m01-l2-01-01-docking-implementation.md`; progress entry `m01-l2-01-01-shell-docking-implementation-2026-10-16`). `dock_layout.rs` is tracked and unmodified in git.
  - `standalone/crates/om9-app/src/dock_layout.rs`: `SectionId` (7 documented menus + reserved `BuilderSlot`), `SectionState {visible, collapsed, dock, order, height, float}`, `toggle_section` / `set_section_collapsed`, `move_section(id, delta) -> Result<(SectionId, usize)>`, `dock_section(id, Left|Right)`, `float_section(id, FloatRect)`, `section_order`, `clamp_floats(window) -> Vec<String>`, `normalize`, `reset_to_default` (L3-004), `load/save` (L3-005).
  - `src/ui/chrome.rs`: `section()` title bar (click = roll-up, `drag_started` = begin drag). `section_bar_controls()` keeps Snaps/Layers controls on a closed bar. `settle_section_drag()` handles drop: over a viewport it floats, over the right dock it docks right, over a left title bar it reorders. `float_section_at()`, `right_dock_ui()` (▲ ▼ ◀ ⧉ buttons) and `float_menu_windows()` (egui `Window` inside the main OS window; closing it re-docks left) are here too.
  - `src/ui.rs`: `move_menu_section`, `dock_menu_section`, `float_menu_section`, `sync_dock_window` (calls `clamp_floats` only when the window size changes, and only when there is no region error), `section_drag: Option<SectionId>`, settle-on-release in the frame loop.
- Existing tests. `dock_layout.rs` unit tests: `reorder_moves_only_the_requested_menu_and_keeps_other_state`, `dock_right_mirrors_the_left_column_and_float_needs_a_rectangle`, `clamp_floats_recovers_menus_dragged_outside_the_window`. `tests/shell_docking.rs`: `rolling_up_a_title_bar_keeps_its_state_and_persists`, `menus_dock_to_the_right_mirror_column_and_back`, `a_floated_menu_keeps_its_rectangle_and_docks_back`, `builder_area_stays_reserved_and_non_interactive`.
- The claim in `implementation/evidence/M01/OM9-L3-01-01-001/OM9-L4-01-01-001-05.ui-review.json` (checklist step "Roll up and expand one menu title bar, then drag one above another", `observed_by_agent: "covered by OM9-L2-01-01 L3-003 tests"`) is only partly true. Roll-up is covered by a real frame click. No test drives the pointer drag path (`section_drag` / `settle_section_drag`) at all. Reorder is covered only through the `move_section` API. "Drag one above another" is not proven.

### 1.2 Gaps between existing code and the L3 oracles (found by reading; to be confirmed in L4-01)

- G1. There is no live drag preview or drop indicator. The 2026-10-16 ledger lists this as the open part of L3-003. The manual gate requires the reviewer to "observe preview/feedback".
- G2. Silent no-ops. A drop outside every target (sidebar, right dock, viewports) does nothing and says nothing. `float_menu_section` returns `false` with no message when the section already floats. `float_section` with a non-finite rect silently substitutes the default rect. This conflicts with "never silently do nothing".
- G3. Errors are stringly typed (`om9_core::Error::Invalid("floating menu order")`, `"menu dock side"`). There is no typed panel-operation error enum, unlike `RegionError` in L3-002. L4-03 S01 requires "typed IDs và error enum".
- G4. "Restore without losing state". `dock_section` resets `collapsed = false` and appends the section at the end of the target column. Float, then dock back, does not return a menu to its previous index or roll-up state. The existing test asserts "Re-docking appends the menu".
- G5. Off-screen recovery runs only on window-size change and on load. A float dropped by `float_section_at` is clamped only at `max(0)` on x/y, so it can sit past the right or bottom edge until the next resize. Whether egui 0.36 `Window` constraining hides this at runtime is not verified.
- G6. There is no drag cancel. Escape during a drag is not handled for the drag. Escape is owned by CMD cancel (`ui.rs` ~775).
- G7. There is no permitted-panel rule. Every `SectionId`, including the reserved `BuilderSlot` (owned by L3-008), can be dragged, reordered, docked and floated. `PanelId` regions (left rail, command line, toolbar, right dock) are not draggable panels. The F6 `CommandSearch` window floats and is already covered by `clamp_floats`.
- G8. No test proves that L3-003 operations leave the document, undo journal and scene unchanged. Only the Builder-slot click test checks entity and undo counts.

## 2. Functional Requirements

### Operations in scope (one owner: `OM9-L2-01-01`, Rust, `om9-app`)

- FR-1 Collapse/expand (roll-up). Clicking a permitted section's title bar toggles `collapsed`. Order, dock side, height and float rect stay unchanged. Snaps and Layers keep their closed-bar controls. Body content state (for example the current layer or snap toggles, which live in the controller) is unaffected.
- FR-2 Reorder by drag. Dragging a permitted section's title bar above or below another section in the same docked column (left, or right mirror) moves it to the indicated index. All other sections keep their relative order and state. The ▲/▼ buttons in the right column and `move_menu_section` share one implementation with the drag path (menu/mouse/API use one code path).
- FR-3 Dock across columns. Dropping on the right dock area docks right. Dropping on the left column docks left at the drop index. ◀ / "Dock left/right" buttons use the same operation.
- FR-4 Float. Dropping over the viewports, or pressing ⧉, floats the section as an owned window at a rect derived from the drop point. The rect is layout state. Moving or resizing the floating window updates it.
- FR-5 Restore without losing state. Docking a floated or right-docked section back, and expanding a rolled-up one, must not lose the section's state. The exact restore semantics (home index and roll-up flag) are decision point D3.
- FR-6 Live drag feedback. While a drag is in progress, a visible indicator shows the pending result: insert position in a column, the dock-right target, or a float target. Releasing applies exactly what was indicated.
- FR-7 Drag cancel. Releasing over no valid target, or the cancel input chosen in D5, ends the drag with no layout change and a visible status ("drag cancelled" or equivalent). This is "cancel where applicable".
- FR-8 Off-screen recovery (negative oracle). See D1. A floating section or the F6 window must always end up reachable inside the visible work area: on drop, on floating-window move or resize end, and on window resize. Recovery is reported visibly (status and CMD history, matching the existing `note_dock_diagnostics` pattern). The L3-002 rule stays: while a region error is active, clamping must not rewrite stored layout.
- FR-9 Permitted panels. A single typed predicate decides which panels L3-003 may move, collapse or float (D4). A request on a non-permitted panel returns a typed error, shows a visible warning and changes nothing.
- FR-10 Typed results. Each operation returns `Result<Outcome, PanelOpError>` (names fixed by the design). The error enum covers at least: not permitted, unknown or invalid target, invalid or non-finite rect, no-op (already in requested state, reported, not silent) and operation not valid in the current dock state (for example ordering a floating menu). Errors map to a visible status message at the UI boundary.
- FR-11 Explicit invalid input. Non-finite, NaN, infinite, zero or negative rects and window sizes, out-of-range deltas or indices, and unknown section keys each give an explicit error or a documented, reported normalisation. None is silently swallowed.
- FR-12 Scope isolation. L3-003 operations change only `DockLayout` section/panel placement, `collapsed`, float rects, `dock_dirty` and the status/transcript line. They never touch document objects, IDs, layers, units, selection, the command draft, the undo/redo journal, the scene or the render cache. Undo/redo of document history does not record or revert panel operations. Panel/menu state is UI state, not document state.
- FR-13 No implicit persistence. L3-003 operations mark the layout dirty but write no file. Writing stays behind View ▸ Matrix Menus ▸ Set as Default (L3-005).
- FR-14 Audit receipt. `om9.exe --ui-audit` receipts expose what is needed to prove L3-003 on a real window: per-section dock, order, collapsed and float rect, the last panel-operation result, and any recovery diagnostic. If a scripted drag needs audit-only environment hooks, they follow the `OM9_UI_AUDIT_*` style and validation of L3-002 (`OM9_UI_AUDIT_WINDOW`, `OM9_UI_AUDIT_ZOOM`).

### Seams (serde-ready only; no behaviour)

- FR-15 Any new state (home position, permitted set, last-op record) derives `Serialize`/`Deserialize` and fits the existing `DockLayout` schema. Whether that needs a `SCHEMA_VERSION` bump is a design question. It must not break `load` of schema-2 files, which L3-005 owns.
- FR-16 Keep the L3-002 seams (`CommandInputHost`, `ViewportSceneHost`, `AttachedCmdFrame::command_input_ready` constant true) unchanged. Add no L2-01-02 or L2-01-03 implementation.

## 3. Non-Functional Requirements

- NFR-1 Rust first (`skills/openmatrix9-workflow/references/rust-first.md`). Use safe Rust in `om9-app` with `#![forbid(unsafe_code)]`. Add no C++. Python is allowed only for a real-window runner, recorded as a Rust-first exception with its reason, as `tools/validate_m01_l3_002_ui_matrix.py` was.
- NFR-2 No panics, `unwrap`/`expect` or indexing on input-derived values in the new paths. No leaked egui ids or stale `section_rects`/`section_drag` across frames, including a drag in progress when its section is reset, re-docked or hidden.
- NFR-3 Do not regress the L3-002 contract (`skills/openmatrix9-workspace-contract/SKILL.md`, section "Vùng workbench docking (OM9-L3-01-01-002)"): left rail, four viewports and command input always kept; right dock yields at about 900 px; below-minimum error behaviour; command line cannot be lost; no layout rewrite during a region error; notices go to CMD history. All existing `workbench_regions*`, `status_bar_overlap` and `shell_docking` tests stay green.
- NFR-4 Build on the current working tree. `ui.rs` and `ui/chrome.rs` carry uncommitted L3-002 close-out edits. Do not revert them.
- NFR-5 Keep Matrix names and behaviour. Title texts, the documented left order, Builders below Projects, ~300 px right mirror column. QymCAD supplies samples only. Do not add a QymCAD crate or path dependency.

## 4. Deliverables required to match the L3-002 standard

Paths are repo-relative. Every log records the real command, stdout, stderr and exit code. No number is copied from a plan.

| L4 | Deliverable | Notes |
| --- | --- | --- |
| L4-01 `OM9-L4-01-01-003-01` | `implementation/design/M01/OM9-L3-01-01-003/source_crosswalk.json` | Schema `om9.m01.source_crosswalk.v1`, same fields as L3-002. Verify `vendor-research/QymCAD` HEAD = `746a17f127897e921a57118b6caad08968602bf5` (confirmed today) and `status --porcelain` clean. Read the 4 anchors (`crates/qymcad/src/gui.rs`, `crates/qymcad/Cargo.toml`, `crates/qymcad-shell/src/lib.rs`, `crates/qymcad-ui-state/src/lib.rs`) with bounded greps. Record real symbols or "not found -> ADD". Record OM9 REUSE-IN-PLACE (`dock_layout.rs`, `ui.rs`, `ui/chrome.rs` with line numbers) and gaps G1–G8. `legal_reuse_status: PENDING_HUMAN_M00_APPROVAL`, `m00_gate: NOT_PASS`. |
| L4-02 `OM9-L4-01-01-003-02` | `implementation/design/M01/OM9-L3-01-01-003/api.md`; `implementation/fixtures/M01/OM9-L3-01-01-003/positive.json`, `negative.json`; `implementation/evidence/M01/OM9-L3-01-01-003/OM9-L4-01-01-003-02.red.log` | `api.md`: typed API, error enum, ownership (UI state only), REUSE/ADAPT/ADD decision, one name per operation (no duplicate backend/UI API), out-of-scope list, resolved decision points, absent-tool statement. Fixtures use `om9.m01.fixture.v1` plus `oracle_schema: om9.m01.oracle.v1.5`, with independent hand-written expectations. Note: L3-002 fixtures live under `implementation/fixtures/`, not `design/`. RED before code: tests compile and fail on assertions, exit non-zero, no build error. |
| L4-03 `OM9-L4-01-01-003-03` | Code; `OM9-L4-01-01-003-03.diffstat.txt`; `OM9-L4-01-01-003-03.target-test.log` | Diffstat header: date, open blockers (BLOCKED_DEPENDENCY on `OM9-L4-01-02-001-05` and `OM9-L4-01-03-001-05`; `OM9-L4-01-01-002-05` exists but its human gate is NOT_SIGNED) and "Rust only, no native exception" (or the recorded exception). GREEN log of the targeted tests. |
| L4-05 `OM9-L4-01-01-003-05` | `OM9-L4-01-01-003-05.regression.log`; `OM9-L4-01-01-003-05.ui-review.json` | Regression: targeted tests, `cargo test -p om9-app`, `cargo test --workspace` (use `-j 1` if os error 1455 recurs), `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check`, and the public source audit (full plus scoped mirror with non-vacuity control, as in L3-002 item 3). Include a SHA-256 digest table of the evidence files. `ui-review.json`: schema `om9.m01.ui_review.v1`, L3-002 shape. `reviewer.status = "NOT_SIGNED"`, name/date null; `gate = {m01_ui: OPEN, m00: NOT_SIGNED, legal_reuse: PENDING_HUMAN_M00_APPROVAL}`; `automated_simulated_evidence` labelled "SIMULATED/AUTOMATED - does not replace human M01-UI sign-off"; a short checklist (6 items or fewer) with `machine_proven` / `human_must_verify` / `reviewer_decision: null`. |
| UI audit receipts | `build/standalone-validation/ui-artifacts/<date>-m01-l3-003-panels/<case>/{ui-receipt.json,workspace.png}` | Real `om9.exe --ui-audit` runs (build output, not committed). Minimum cases: default; one section rolled up; reordered; docked right; floated; float recovered after an off-screen drop or a shrink. Exit 0, hashes recorded. Each image opened and its observation written down. Never claim an image was inspected unless it was. A runner script is optional (NFR-1). |
| Validation doc | `docs/validation/<date>-m01-l3-003-reorder-collapse-float-panels.md` | English, L3-002 section layout: Implemented / Measured / Not passed / Known defects. |
| Ledger | `docs/openmatrix9-progress.json` new entry `m01-l3-01-01-003-...-<date>` | Evidence paths, commands with exit codes, uncertainties, blockers, `next_action`. `milestone_status_promoted: false`. Valid JSON. Also correct the L3-001 ui-review claim (§1.1) in the ledger/validation doc as history; do not rewrite the L3-001 evidence file unless the design decides to annotate it. |
| Task notes | `.agents/tasks/m01-l3-003-panels/{plan.md, verification.md, review.json, final-report-vi.md}` | `review.json` = `{verdict, findings}` written last by an independent reviewer from real output. The final report is in Vietnamese with the L3-002 table "Việc / Trạng thái / Ai đóng". |
| Business-rule proposal | In the final report only | New behaviour (FR-6 to FR-9 and D3 semantics) is a business-rule change. No consent exists yet for L3-003 text, so do not edit any `skills/` file. Present before/after plus proposed skill text for `openmatrix9-workspace-contract` and ask (`references/business-rule-updates.md`). |

## 5. Acceptance Criteria (numbered, testable)

Automated, run from `h:\OpenMatrix9\standalone` unless noted:

1. All files in §4 exist at the exact paths, and the JSON files parse.
2. The RED log shows exit ≠ 0 with assertion failures and no compile error, and was captured before the implementing diff.
3. Positive oracle frame test. A real `MatrixApp::show` frame with synthetic pointer events (press on a title bar, move, release) reorders one section above another. Only that section's index changes and other sections' `SectionState` is unchanged. This closes the L3-001 claim.
4. Collapse then expand returns a `SectionState` equal to the original. Snaps and Layers keep their closed-bar controls while collapsed.
5. Float then dock back, and dock right then dock back, end in the state D3 defines. A test asserts that state field by field (order, collapsed, height, visible).
6. Negative oracle. A floating section dropped or moved at or beyond each window edge, a huge rect and a non-finite rect each end fully reachable inside the work area (D1). Recovery is reported in the status line and CMD history. A follow-up window shrink also recovers it. During a region error the stored layout is not rewritten.
7. A drop outside every target, a drag cancel, a non-permitted panel, ordering a floating section, re-floating a floating section and an unknown or invalid target each return the typed error or no-op outcome, show a visible status, and leave `DockLayout` byte-for-byte unchanged (`to_json()` before = after).
8. A drag-preview indicator is present in the frame output while dragging and absent after release or cancel. Its target equals the applied result.
9. For every L3-003 operation, document entity count, entity IDs, current layer, undo length, redo length, command draft and `controller.workspace` view state are unchanged. No layout file is written (FR-13).
10. No panic across a randomized or swept sequence of operations, including invalid inputs and a drag left in progress across reset, re-dock and hide. `section_drag` is `None` after release, cancel or reset.
11. Existing tests pass unchanged in intent: `--lib dock_layout`, `--test shell_docking`, `--test workbench_regions`, `--test workbench_regions_sizes`, `--test status_bar_overlap`. Any changed assertion (for example "Re-docking appends the menu" if D3 changes it) is listed with its reason in the diffstat note.
12. `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` exit 0.
13. `cargo tree --workspace` has no `qymcad`, and no `Cargo.toml` references QymCAD. `vendor-research/QymCAD` is unchanged. There is no `unsafe` in new code.
14. Real-window receipts (§4) exist with exit 0, and their recorded section state matches each case.
15. Public source audit. The full run is recorded with its real exit code. Expect 1 because of private trees, and never report it as passed. The scoped mirror over L3-003 touched files exits 0, and its control run exits 1.
16. `ui-review.json`: `reviewer.status == "NOT_SIGNED"`, `gate.m01_ui == "OPEN"`.
17. No `planning_state` changed under `OM9_Milestones/`. Nothing committed, staged or pushed. No edits in `ref/`, `manuals/`, `OM9_Milestones/`, `QymCAD/`, `vendor-research/`, `om9-window.png` or `skills/`.

Manual (stays open, human only, gate M01-UI):

18. On real Windows: roll up/expand, drag a title bar above another, dock right/back, float/dock back, drag a float to the screen edge and off the window, resize the window. Observe the preview/feedback and the visible warnings. Repeat at real 150% DPI and on the second monitor (1080x1920). Then sign `OM9-L4-01-01-003-05.ui-review.json`.

## 6. Decision points for the design (must be resolved in `api.md`; not left to the implementer)

- D1 (required) Minimal in-L3 off-screen recovery versus L3-004. L3-003 owns clamp/snap-back: on drop, on float move or resize end, and on window resize. It moves the rect fully inside the visible work area with a margin and shrinks it if larger, with a visible diagnostic. L3-004 owns Reset Menus and default restore, and L3-003 must not add or change it. The design must define the "visible work area" (whole window minus margin, as `clamp_floats` does today, or the area below the menu bar and above the status bar), when the egui `Window` memory position is overridden, and how it relates to egui 0.36 window constraining. It must state the boundary in `api.md`: "Recovery here is clamp/snap-back only; Reset Menus belongs to L3-004."
- D2 (required, recommendation needed) Float target. Option A: an egui floating `Window` inside the main OS window (the current code). Option B: a separate OS viewport (egui 0.36 multi-viewport), which the manual's "drag menus off onto other monitors" suggests. The design must recommend and choose, with trade-offs: per-monitor DPI, headless testability (the L3-002 harness is single-context), the wgpu backend, lifecycle and focus, and off-screen recovery across monitors. If A is chosen, dragging onto another monitor is explicitly unsupported. The design must define what happens on such a drag (recovered back with a visible message) and record it as a known deviation, not a silent limit.
- D3 Restore semantics for "restore without losing state". Choose between keeping the current behaviour (re-dock appends at the end and expands) and remembering a home column, index and roll-up flag. Either way the choice is a business-rule change requiring consent at the end.
- D4 Permitted-panel set. Recommended: the 7 documented menu sections are permitted. Decide whether `BuilderSlot` (L3-008) stays movable as today, or is fixed below Projects with a visible refusal. `PanelId` regions are not L3-003 panels. The F6 window takes part only in off-screen recovery.
- D5 Drag cancel input. Release-over-nothing is the required cancel. Decide whether Escape also cancels a drag in progress, and if so, how it avoids also cancelling the CMD draft (`ui.rs` Escape handling).

## 7. Out of Scope (siblings own them; serde-ready seams only)

- L3-004 restore default / Reset Menus / Reload Menus, and the Main Menu "Reset" (Panic) button.
- L3-005 layout persistence and layout store (Set as Default, file load/save, schema migration).
- L3-008 Builder slot contents and builder docking.
- L3-006 active document window state. L3-007 status-surface content and acknowledgement rules (only reuse its API).
- OM9-L2-01-02 command dispatch and Undo/Redo. OM9-L2-01-03 document and scene.
- The Custom menu ("click and drag" icons into a Custom menu, book 1 p.17), which is a different feature.
- Promoting `planning_state`, signing any human gate, or changing `tools/public_source_audit.py`.

## 8. Gates that stay open after this work

- M01-UI human gate: OPEN. `reviewer.status = NOT_SIGNED`. Machine evidence is labelled SIMULATED/AUTOMATED.
- M00 / legal reuse of QymCAD: PENDING_HUMAN_M00_APPROVAL; M00 NOT_PASS. The agent approves nothing.
- OM9-L2-01-02 and OM9-L2-01-03: BLOCKED_DEPENDENCY (L4-03 declares `OM9-L4-01-02-001-05` and `OM9-L4-01-03-001-05`). Implement against the existing seams, as L3-002 did.
- Full public source audit: still exits 1 because of private trees. The scoping proposal from L3-002 is still the user's decision.

## 9. Assumptions and environment facts

- `OpenMatrix9_Codex_Spec_v2/` is absent from the workspace (`Test-Path` = False). AGENTS.md routing to it cannot be followed. This is recorded, not invented. The local `OM9_Milestones` l3/l4 JSON and the reference excerpts are the spec.
- `tools/validate_m01_spec.py` (named in every l4 `test_commands_preview`) is absent. Record it as "absent, not run". `om9.m01.oracle.v1.5` has no local definition and is carried as a field only.
- Behavioural reference: the L2 reference `m8book1__p11-13__a-tour-of-the-interface.md` ("Changing the Menu Layout": roll-up, drag reorder, float into viewports, dock right in the 300 px mirror, drag to other monitors). The L3-003 `reference/` holds the Main Menu section (p.14-18), which is background only. Do not copy manual text or images into the repo.
- egui/eframe 0.36.2 (from `standalone/Cargo.lock`).
- "Undo where applicable" is read as not applicable to UI layout state, because document Undo belongs to L2-01-02. "Cancel" means drag cancel (FR-7). The review may challenge this.
