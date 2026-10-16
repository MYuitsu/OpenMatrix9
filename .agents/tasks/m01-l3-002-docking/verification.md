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

1. Launch on Windows at a normal width: left rail, four viewports and (after docking a menu right) the right dock are visible and do not overlap.
2. Drag narrower than about 900 px: the right dock yields, the command input stays visible and uncovered, left rail and viewports stay usable.
3. Shrink toward the minimum: a visible message appears when the layout cannot be honored; nothing overlaps.
4. Hide the right dock and the command line from the menu: the command line cannot be lost.
5. Run one command, Undo, Redo, save and reopen: regions and document are unaffected.
6. Repeat at 150% scaling and with a second monitor (not done by the agent).
7. Judge the overlapping status text at 800x600.

## Honest status

- Legal reuse: PENDING_HUMAN_M00_APPROVAL. M00 gate: NOT_PASS. Nothing copied from QymCAD; no QymCAD dependency in any `Cargo.toml`.
- OM9-L2-01-02 and OM9-L2-01-03 are not implemented. `CommandInputHost` and `ViewportSceneHost` are minimal traits tested against stubs. The `ui.rs` adapter for the command input only reports the CMD frame as attached; it says nothing about dispatch or the Undo journal. L4-03 dependencies are therefore BLOCKED_DEPENDENCY.
- Absent: `OpenMatrix9_Codex_Spec_v2/` (so MATRIX9_UI_CONTRACT was not read), the `tools/validate_m01_*` family, and any definition of `om9.m01.oracle.v1.5`. The per-task `*.evidence.json` files named in the runbooks were not written because their capture tools are absent; the declared `outputs` were.
- The egui panels remain user-resizable and are not clamped by the plan. The plan validates each frame and tiles the viewports.
- Business-rule changes awaiting consent (nothing recorded in any skill): (1) a window below 560x286 (320x286 without the left rail) shows a "Layout:" error and keeps the last plan, where it used to clamp silently; (2) a command line marked hidden is still reserved; (3) while that error is active, panel-size sync and floating-menu clamping leave the stored layout alone; (4) right-dock/left-rail notices go to the CMD history and, if no failure is on screen, the status line.
