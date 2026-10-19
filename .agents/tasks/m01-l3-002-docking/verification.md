# Verification note: OM9-L3-01-01-002 Workbench Docking Regions

Date: 2026-10-16 · Iteration 1 (no review.json existed) · Windows, PowerShell · Rust only, safe code, no native exception.
No git commit, stage or push was made. No `skills/` file and no `planning_state` in `OM9_Milestones/` was changed.

Full logs (command, stdout, stderr, exit code) are in `implementation/evidence/M01/OM9-L3-01-01-002/`.

## Commands and results

| Command (cwd `standalone` unless noted) | Result |
| --- | --- |
| RED, before code: `cargo test -p om9-app --lib workbench_regions` / `--test workbench_regions` | exit 101; lib 3 passed 18 failed; integration 1 passed 6 failed; assertion failures, no build errors (`-02.red.log`) |
| `cargo fmt --all -- --check` | exit 0 |
| `cargo check -p om9-app --all-targets` / `cargo check --workspace --all-targets` | exit 0 / exit 0 |
| `cargo build --workspace` | exit 0 |
| `cargo test -p om9-app --lib workbench_regions` | 21 passed, 0 failed |
| `cargo test -p om9-app --test workbench_regions` | 8 passed, 0 failed |
| `cargo test -p om9-app --test shell_docking` | 10 passed (existing tests, unmodified) |
| `cargo test -p om9-app` | exit 0, every group ok |
| `cargo test --workspace` | exit 0, no failures |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 (clippy 0.1.98 installed) |
| `python tools/public_source_audit.py` (repo root) | **exit 1, NOT passed.** 11721 errors, all in `ref/`, `manuals/`, `OM9_Milestones/`, `QymCAD/`, `vendor-research/`, `om9-window.png` (private references and untracked checkouts the tool walks). 0 errors in any path this task touched. Not fixed: AGENTS.md forbids touching private references. |
| `python tools/validate_m01_spec.py` | not run, tool absent (exit 2) |
| `git -C vendor-research/QymCAD rev-parse HEAD` / `status --porcelain` | pin `746a17f1...` / empty |
| `git diff --cached --name-only` | empty (nothing staged) |

## Required tests

- (a) valid input: `valid_layout_exposes_left_center_four_views_and_optional_right_dock`, `plans_for_normal_sizes_never_overlap_and_tile_the_center` (1920x1080, 1600x1000, 1280x900, 1024x768, with and without right dock, second call equal), `one_maximized_view_fills_the_center`, `smallest_usable_window_keeps_every_minimum`, fixture replay of `positive.json`.
- (b) invalid or degenerate: `zero_negative_nonfinite_and_huge_windows_are_invalid`, `window_below_the_minimum_returns_window_too_small`, `hidden_left_rail_lowers_the_minimum_width`, `narrow_window_yields_right_dock_with_note`, `hidden_right_dock_is_a_defined_state`, `hidden_command_line_is_kept_with_note`, `view_set_must_be_one_to_four_distinct_views`, `nonfinite_stored_size_is_an_explicit_error`, fixture replay of `negative.json`. Each asserts the layout JSON is unchanged.
- (c) only documented state changes: `compute_regions_changes_nothing_but_its_return_value`, `hosts_are_asked_only_what_the_contract_documents` (stub counters), `too_small_window_reports_visibly_and_changes_nothing_else` (real frame: dock JSON, `Workspace`, entity count unchanged, last good plan kept, recovery clears only the text this module raised), `an_unrelated_failure_is_not_cleared_by_the_region_check`.
- Positive oracle: `valid_layout_exposes_...` and `normal_window_shows_left_rail_four_viewports_and_the_right_dock`, `the_plan_matches_the_regions_egui_actually_lays_out` (plan within 8 px of the real egui panels).
- Negative oracle: `narrow_window_never_overlaps_or_hides_command_input` and the fixture-driven sweep (widths 200..1600, heights 100..1000, step 20, three layouts): every `Ok` plan keeps the command input at least 58 px high, inside the window, overlapping nothing; every other size is an explicit `WindowTooSmall`.

## Runtime evidence

The standalone egui app launched in this environment (NVIDIA GeForce RTX 3060, Vulkan, debug build). `om9.exe --ui-audit <dir>` ran twice, exit 0 both times, 21 passes, screenshots saved:

- 1600x1000 with the right dock: `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-docking/normal/workspace.png`. Left rail, four viewports and the right Layers dock visible. Viewport rects (x,y,w,h): Looking Down 300,178,499,401; Perspective 801,178,499,401; Through Finger 300,581,499,401; Side View 801,581,499,401.
- 800x600: `.../narrow/workspace.png`. Right dock yielded (receipt note `RightDockYielded/NarrowWindow`), left rail, command input and four viewports kept, no overlap. Viewport rects: 300,178,249,201 / 551,178,249,201 / 300,381,249,201 / 551,381,249,201.

To get these two sizes I added two audit-only environment variables (`OM9_UI_AUDIT_WINDOW=WxH`, `OM9_UI_AUDIT_RIGHT_DOCK=1`), read only when an audit directory is given. The existing flags had no way to set a size.

Observed, not changed: at 800x600 the text on the bottom status line overlaps itself (pre-existing L3-007 behavior). Not captured: 150% DPI, a second monitor, a live drag by a person, and a window below the 720x480 OS minimum (covered by frame tests only).

## L4-05 manual checklist: PENDING HUMAN VERIFICATION (not passed)

Reviewer decisions are all `null` in `OM9-L4-01-01-002-05.ui-review.json`; `reviewer.status = NOT_SIGNED`; gate `m01_ui = OPEN`.

(Shortened 2026-10-16 to the supported range, minimum 1280x720 logical, smallest device 1366x768. The earlier 7-item list, with its "about 900 px" drag step, is superseded; live drag preview stays with L3-003.)

1. Real 1366x768 device at 100%, 125% and 150% Windows DPI: the window starts fully on screen, the status bar and command line are above the taskbar, left rail, four viewports and command panel are visible and not overlapped.
2. Window-minimum enforcement by dragging an edge: it stops at 1280x720 logical (or the capped value on a small screen) and cannot be shrunk further.
3. Second monitor with a different DPI: move the window; the minimum follows and nothing overlaps.
4. The right dock shows from 1200 logical points and yields below with a CMD/status notice; the command line cannot be lost.
5. Status line readable at the minimum (1280x720 and the capped 911x480 case).

## Honest status

- Legal reuse: PENDING_HUMAN_M00_APPROVAL. M00 gate: NOT_PASS. Nothing copied from QymCAD; no QymCAD dependency in any `Cargo.toml`.
- OM9-L2-01-02 and OM9-L2-01-03 are not implemented. `CommandInputHost` and `ViewportSceneHost` are minimal traits tested against stubs. The `ui.rs` adapter for the command input only reports the CMD frame as attached; it says nothing about dispatch or the Undo journal. L4-03 dependencies are therefore BLOCKED_DEPENDENCY.
- Absent: `OpenMatrix9_Codex_Spec_v2/` (so MATRIX9_UI_CONTRACT was not read), the `tools/validate_m01_*` family, and any definition of `om9.m01.oracle.v1.5`. The per-task `*.evidence.json` files named in the runbooks were not written because their capture tools are absent; the declared `outputs` were.
- The egui panels remain user-resizable and are not clamped by the plan. The plan validates each frame and tiles the viewports.
- Business-rule changes awaiting consent (nothing recorded in any skill): (1) a window below 560x286 (320x286 without the left rail) shows a "Layout:" error and keeps the last plan, where it used to clamp silently; (2) a command line marked hidden is still reserved; (3) while that error is active, panel-size sync and floating-menu clamping leave the stored layout alone; (4) right-dock/left-rail notices go to the CMD history and, if no failure is on screen, the status line.


## Closeout item 1: status-bar text overlap (2026-10-16)

- Before: `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-closeout/before-narrow/workspace.png` (800x600, right dock requested). Viewed: the dock summary ran left over "Ready: Ready" at the bottom line. Confirmed reviewer finding.
- Measurement only first: `MatrixApp.status_message_rect`, `status_summary_rect`, `status_dismiss_rect` (ui.rs), receipt `regions.status_message_px`, `status_summary_px`, `status_text_overlap`.
- Test: `standalone/crates/om9-app/tests/status_bar_overlap.rs` (4 cases: 800x600, 720x480, 1600x1000 control, 560x300 long Error). RED log `implementation/evidence/M01/OM9-L3-01-01-002/closeout-01.red.log`: exit 101, 3 assertion failures (control passed), no build errors.
- Fix: `ui/chrome.rs::status_bar_ui` only. Dismiss (failure only) is placed at the right edge, message gets the width left of it minus a 48 px summary reserve, summary is elided to the remainder; both use `Label::truncate()` and `on_hover_text(full text)`. Status text content, StatusSurface semantics, Dismiss/Escape behaviour unchanged.
- GREEN log `closeout-01.green.log`: status_bar_overlap 4/4, whole `cargo test -j 1 -p om9-app` exit 0 (lib 61, workbench_regions 8, shell_docking 10). A first run without `-j 1` hit os error 1455 (paging file exhausted while linking); environmental, not a code failure.
- After (real audit, exit 0, passes=21, region error null, `status_text_overlap` false): `.../2026-10-16-m01-l3-002-closeout/narrow/` (800x600: message readable, summary elided with an ellipsis, no overlapping glyphs; plan still RightDockYielded/NarrowWindow with 4 views) and `.../normal/` (1600x1000: full summary shown). Both workspace.png viewed.
- Scope: this is a minimal fix inside `chrome.rs` (L3-007 sibling area). Strict L3-007 scope (status content, colours, acknowledgment rules) is untouched. `workbench_regions.rs` and `dock_layout.rs` unchanged, so region-plan behaviour is unchanged. Not a M01-UI, M00 or legal gate pass.

## Closeout items 2 to 8 (2026-10-16)
Evidence only. No M01-UI, M00 or legal gate is claimed. Nothing was committed or staged.
- Item 2 (SIMULATED/AUTOMATED, does not replace the human sign-off): `standalone/crates/om9-app/tests/workbench_regions_sizes.rs` 10 passed (first width with the right dock = 915 pt; 914 NotEnoughRoom; 900..914 NotEnoughRoom; below 900 NarrowWindow). `python tools/validate_m01_l3_002_ui_matrix.py` exit 0, 10 of 10 real-window cases exit 0 with `status_text_overlap` false: `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-closeout/matrix/matrix-summary.json`. `OM9_UI_AUDIT_ZOOM` (audit only) simulates 1.5 in point space, not OS per-monitor DPI. Checklist in `ui-review.json` shortened to 5 items with `machine_proven` and `human_must_verify`; reviewer NOT_SIGNED.
- Item 3: full `python tools\public_source_audit.py` exit 1, 11721 errors (OM9_Milestones 5877, ref 5611, vendor-research 114, QymCAD 114, manuals 4, om9-window.png 1), not passed. Scoped mirror `build/scoped-audit/2026-10-16-l3-002`: exit 0, 557 source files. Control (`control.exe`, `control.png` added to the mirror only): exit 1 with exactly those two; deleted; re-run exit 0. Log: `implementation/evidence/M01/OM9-L3-01-01-002/closeout-03.scoped-audit.log`. `tools/public_source_audit.py` unchanged. Scoping proposal left to the user.
- Item 4: skill section "Vùng workbench docking (OM9-L3-01-01-002)" added and `SKILL.md` synced to `C:\Users\nguye\.codex\skills\openmatrix9-workspace-contract\`; SHA-256 equal on both (`453705A0...F69F`).
- Item 5: BLOCKED_DEPENDENCY for OM9-L2-01-02/03 stated with the follow-up in the ledger, the validation doc, the final report and one doc-comment sentence on `AttachedCmdFrame` in `ui.rs`. No logic change.
- Item 6: no `qymcad` in `cargo tree --workspace` (also `--edges all --all-targets`), none in the 7 standalone `Cargo.toml` files or `Cargo.lock`; `.rs` mentions are comments only; `vendor-research/QymCAD` HEAD `746a17f127897e921a57118b6caad08968602bf5`, `status --porcelain` empty. `legal_reuse_status` stays PENDING_HUMAN_M00_APPROVAL.
- Item 7: `planning_state` left unchanged on purpose (reasons in the validation doc). `git status --short` scoped to the L3-002 folder under `OM9_Milestones` printed nothing.
- Defects recorded, not fixed: saving a document with a Line fails `Invalid('archive encoding verification')` in `om9-archive`; below ~311 px height the plan clamps the command panel to 58 while egui draws the stored taller panel.
- Final runs in `standalone` (`build/feat003-final-cargo.log`): `cargo fmt --all -- --check` exit 0; `cargo check --workspace --all-targets` exit 0; `cargo clippy --workspace --all-targets -- -D warnings` exit 0; `cargo test --workspace -j 1` exit 0, 581 passed, 0 failed, 24 ignored (om9-app: lib 62, workbench_regions 8, status_bar_overlap 4, workbench_regions_sizes 10, shell_docking 10). The matrix runner was re-run after these: exit 0, 10/10.- Supersedes the earlier 'Honest status' bullets: the business-rule changes are no longer awaiting consent (recorded in the skill, item 4); the status-bar overlap and the 'not captured 150% DPI' statement are replaced by the closeout results above (simulated 150% captured; real DPI and second monitor remain human).

## Minimum window 1280x720 (2026-10-16)

Evidence only. No M01-UI, M00 or legal gate passed (`reviewer.status = NOT_SIGNED`, M01-UI OPEN, `PENDING_HUMAN_M00_APPROVAL`). Nothing committed, pushed or staged by this work. Real exit codes and counts below; the full logs are `build/min-window-final-cargo.log` (generated, not checked in) and the files under `implementation/evidence/M01/OM9-L3-01-01-002/`.

### Commands (cwd `h:\OpenMatrix9\standalone`, the cargo workspace root)

| Command | Exit | Result |
| --- | --- | --- |
| `cargo fmt --all -- --check` | 0 | clean |
| `cargo check --workspace --all-targets` | 0 | |
| `cargo build --workspace` | 0 | |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 | |
| `cargo test --workspace -j 1` | 0 | 596 passed, 0 failed, 24 ignored (native-gated) across 92 test targets |

Targets of this item: `om9-app` lib 73 (includes `startup` sizing tests, `workbench_regions`, `dock_layout` boundary, audit helper), `window_minimum` 5, `workbench_regions` 8, `workbench_regions_sizes` 9, `status_bar_overlap` 4, `shell_docking` 10, `startup_contract` 3.

RED then GREEN of the pure sizing function: `min-window-01.red.log` (stub returning 1280x720 for every screen: 3 assertion failures, no build error) and `min-window-01.green.log` (exit 0).

Note on the cwd: there is no `Cargo.toml` or `Cargo.lock` at `h:\OpenMatrix9`; the workspace and `Cargo.lock` (eframe/egui/egui-winit 0.36.2, winit 0.30.13) are in `standalone/`, so every cargo command ran there.

### Real-window matrix

`python tools\validate_m01_l3_002_ui_matrix.py --output build\standalone-validation\ui-artifacts\2026-10-16-m01-l3-002-min-window\matrix`: exit 0, 9 of 9 PASS. New dated folder: `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-min-window/` (generated, not checked in). Cases: `min-1280x720`, `laptop-1366x728`, `wide-1440x720`, `display-1440x900`, `tall-1280x900`, SIMULATED `lap100-1366x690` (min 1280x688), `lap125-1366x680` (min 1076.8x534.4, dock yielded), `lap150-1366x700` (min 894.7x432, dock yielded), and "unsupported size, error path" `unsupported-1000x600-startup-refused` (exit 1, no receipt, refusal text on stderr). The validator requires `status_message_px` and `status_summary_px` non-null in addition to `status_text_overlap == false`, checks `min_window.sent` and `.source`, and fails a clamped window.

Plan deviation, recorded: the primary display reported 1440x900 at 100% in this run (the plan assumed 2560x1440; the earlier closeout opened 1600x1000). The host clamps a larger window, so real windows at 1600x1000, 1920x1080 and 2560x1000 are impossible here and were replaced by the display-size cases; the validator now fails a clamped case. Those sizes stay covered by frame tests and the computed layout table.

Screenshots viewed (agent): `min-1280x720`, `laptop-1366x728`, `lap125-1366x680`, `lap150-1366x700`. Seen: menu bar, left rail with its panels, green command panel with the `Command:` line, toolbar, four viewports, and a status line without overlapping text; the Layers dock appears at 1280 and 1366 and is absent at the capped 125%/150% sizes. In the lap125/lap150 shots the status line shows "Error: unsupported unknown/unavailable command" (the audit's scripted command text arrived concatenated); it is a command error, not a `Layout:` error, and `regions.error` is null in every receipt.

### Public source audit

Evidence: `implementation/evidence/M01/OM9-L3-01-01-002/min-window.scoped-audit.log`.

- Full `python tools\public_source_audit.py`: **exit 1, NOT passed**, 11721 errors, 23827 source files; by first path segment OM9_Milestones 5877, ref 5611, vendor-research 114, QymCAD 114, manuals 4, om9-window.png 1 (all private trees; pre-existing).
- Scoped mirror (copy only) `build/scoped-audit/2026-10-16-l3-002`, refreshed with this item's 17 touched files (no PNG, no private tree): exit 0, 565 source files, `errors` empty. Control with `control.exe` and `control.png`: exit 1 with exactly those two errors; after deleting them exit 0. A scoped result covers only the mirrored files. `tools/public_source_audit.py` was not edited.

### Skill

`skills/openmatrix9-workspace-contract/SKILL.md` and `C:\Users\nguye\.codex\skills\openmatrix9-workspace-contract\SKILL.md`: SHA-256 both `B1D2305AEA2FC69BE5698C3261D7A1F0A6DCEA5B57D0CF225E13968616A58541` (equal; previous hash `453705A0A7EC1791...`). Only the consent line and the two bullets (dock threshold, minimum numbers) changed.

### Assertions changed

See the table "Assertions changed (and why)" in `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md`. Summary: every assertion that used 560x286, 320x286, 719x479, 559x286, 560x285, 900 or 915 was replaced by an equal-strength assertion at 800x400 (planner floor), 1280x720 (nominal minimum) or 1200 (threshold); the tiny-size matrix cases were dropped except ONE kept below-minimum case, `unsupported_size_error_path`; nothing was loosened to pass.

### Unchanged / out of range

- The ~311 px command-panel mismatch is out of the supported range: recorded, no fix.
- Saving a document with a Line fails `archive encoding verification` in `om9-archive`: unrelated, unfixed, listed.
- L2-01-02/03 stubs stay BLOCKED_DEPENDENCY; no `planning_state` changed.
- Temporary files created during verification (python helper scripts) were removed; generated build logs and artifact folders under `build/` remain (ignored, not checked in).
