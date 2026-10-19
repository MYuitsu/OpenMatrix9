# M01 OM9-L3-01-01-002 Workbench Docking Regions (2026-10-16)

Scope: standalone Rust OM9, module OM9-L2-01-01. Only what was run is listed. Full logs: `implementation/evidence/M01/OM9-L3-01-01-002/`; verification note: `.agents/tasks/m01-l3-002-docking/verification.md`.

## Implemented

- `standalone/crates/om9-app/src/workbench_regions.rs`: pure region model (left rail, center with four viewports or one, optional right dock, command input, toolbar, status bar), explicit `RegionError`/`RegionNote`, `compute_regions`, `compute_regions_for`, `tile_views`, `RegionPlan::validate`, and two seam traits toward OM9-L2-01-02/03 tested against stubs.
- `ui.rs`: the frame validates a plan each pass, tiles the viewports through `tile_views`, and shows errors and notices; `chrome.rs`: panel-size sync pauses while the check has an error.
- Artefacts: `source_crosswalk.json`, `api.md`, `positive.json`, `negative.json`, red/target/regression logs, `ui-review.json`.

## Measured

- Red run then green: lib 21 and integration 8 new tests pass; `cargo test --workspace`, `cargo check --workspace --all-targets`, `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check` exit 0.
- `om9.exe --ui-audit` at 1600x1000 (right dock shown) and 800x600 (right dock yielded): exit 0, screenshots and receipts hashed in the regression log.

## Not passed / not done

- `python tools/public_source_audit.py` exits 1 on 11721 pre-existing errors in private reference and untracked trees; none in files touched here.
- `tools/validate_m01_spec.py` is absent: not run.
- M01-UI manual gate OPEN, reviewer NOT_SIGNED; real OS DPI, second monitor and live drag not captured (a simulated 150% matrix exists, see Close-out).
- Legal reuse PENDING_HUMAN_M00_APPROVAL; M00 NOT_PASS; OM9-L2-01-02/03 not implemented (BLOCKED_DEPENDENCY).
- No `planning_state` changed; no commit. The skill record was added later by the close-out (see below).

## Close-out (2026-10-16)

Evidence only. Nothing below passes the M01-UI, M00 or legal gates, and nothing was committed or staged by this work.

| Item | Result | Who closes it |
| --- | --- | --- |
| 1 Status-bar text overlap at 800x600 | Fixed in `ui/chrome.rs::status_bar_ui`. RED `closeout-01.red.log` (exit 101, 3 assertion failures), GREEN `closeout-01.green.log`; `tests/status_bar_overlap.rs` 4 passed. Real audit screenshots viewed. | Machine |
| 2 Size matrix and simulated 150% scale | `tests/workbench_regions_sizes.rs` 10 passed; `tools/validate_m01_l3_002_ui_matrix.py` exit 0, 10 of 10 real-window cases exit 0 (first width with the right dock = 915 pt; superseded: the minimum-window rule below moved it to 1200 pt and replaced these cases). SIMULATED/AUTOMATED, does not replace the human sign-off. Real DPI, second monitor and a live drag are NOT done. | Machine (simulated); human (real) |
| 3 Public source audit | Full `python tools/public_source_audit.py`: exit 1, 11721 errors, NOT passed. Scoped mirror run over the touched files: exit 0, 557 source files (details below). Scoping change is a proposal only. | Human decides the proposal |
| 4 Skill record | Section "Vùng workbench docking (OM9-L3-01-01-002)" added to `skills/openmatrix9-workspace-contract/SKILL.md` with the dated consent line; installed copy synced (SHA-256 equal). | Machine |
| 5 OM9-L2-01-02 / OM9-L2-01-03 | Out of scope, stubs kept. Status BLOCKED_DEPENDENCY (see below). | Other modules |
| 6 Legal / M00 | PENDING_HUMAN_M00_APPROVAL unchanged. No QymCAD build dependency, no upstream text copied for this item. | Human |
| 7 `planning_state` | Unchanged (explained below). | Human, if the convention should change |
| 8 Ledger and reports | `docs/openmatrix9-progress.json`, this file, `verification.md`, `final-report-vi.md` updated. | Machine |

### Public source audit (item 3)

Evidence: `implementation/evidence/M01/OM9-L3-01-01-002/closeout-03.scoped-audit.log`.

- Full run `python tools\public_source_audit.py` at the repo root: exit 1, 11721 errors, 23807 source files scanned. Errors by first path segment: OM9_Milestones 5877, ref 5611, vendor-research 114, QymCAD 114, manuals 4, om9-window.png 1. None outside those private trees. The full audit has NOT passed.
- Scoped run: a copy (never a move) of `Resources/` and the L3-002 touched files was placed in `build/scoped-audit/2026-10-16-l3-002/` (no PNG from the touched set, nothing from a private tree). `python tools\public_source_audit.py --project-root build\scoped-audit\2026-10-16-l3-002`: exit 0, `source_files` 557, `errors` empty.
- Control: with `control.exe` and `control.png` added to the mirror only, the same command exits 1 with exactly those two errors; after deleting them it exits 0 again. So the scoped result is not vacuous.
- A scoped result covers only the mirrored files. It is not a statement about the repository.
- `tools/public_source_audit.py` was not edited.
- Proposal (not implemented, for the user to decide): add `--paths FILE...` (or `--only-tracked` via `git ls-files`) and an optional `--exclude` list for `ref`, `manuals`, `OM9_Milestones`, `QymCAD`, `vendor-research` and root screenshots, with default behaviour unchanged so strict mode stays fail-closed. Trade-off: an exclude list used by default would weaken fail-closed behaviour; `--paths` does not.

### Skill record (item 4)

The user replied "làm xong các việc chưa xong cho xong M01 L2-01-01 L3-002" to the proposal in `final-report-vi.md`. The four behaviours plus the roughly 900 px right-dock yield were recorded as stated there (the 900 px figure and the 560x286 / 320x286 minimums were later replaced, see "Minimum window 1280x720 logical"), with a dated consent line. The sync requirement of the skill was met: only `SKILL.md` was copied to `C:\Users\nguye\.codex\skills\openmatrix9-workspace-contract\` and both files hash to `453705A0A7EC1791D25BFFD176F778329BE13F3C44B97DF922C4476ED251F69F`. The status-bar truncation is a presentation fix and was not recorded as a rule.

### BLOCKED_DEPENDENCY (item 5)

OM9-L2-01-02 and OM9-L2-01-03 are out of scope here and were not implemented. `CommandInputHost`, `ViewportSceneHost` and `AttachedCmdFrame::command_input_ready` (always true) stay stubs. Status: BLOCKED_DEPENDENCY. Follow-up: when OM9-L2-01-02 lands, replace the constant-true `AttachedCmdFrame::command_input_ready` adapter in `ui.rs` with the real CMD host query and add a frame test where the host is not ready so `RegionError::CommandInputUnavailable` is exercised; when OM9-L2-01-03 lands, replace the `Workspace` impl of `ViewportSceneHost` and re-run `tests/workbench_regions.rs`. Until then `CommandInputUnavailable` is reachable only in unit and fixture tests. The only `ui.rs` comment change for this item is one sentence on `AttachedCmdFrame`; no logic change.

### QymCAD no-copy / no-dependency check (item 6)

- `cargo tree --workspace` (450 lines) and `cargo tree --workspace --edges all --all-targets` (4006 lines), both filtered for `qymcad` (case-insensitive substring): 0 matches.
- `Select-String` over the 7 `Cargo.toml` files below `standalone/` (excluding `target/`) and `standalone/Cargo.lock` for `qymcad`: 0 matches.
- Mentions of `qymcad` in `standalone/crates/**/*.rs` are comments only: `startup.rs:41`, `ui.rs:968`, `window_identity_guard.rs:5`, `workbench_regions.rs:23-24`. `startup.rs` and `window_identity_guard.rs` say "Adapted from" and belong to earlier work, not to OM9-L3-01-01-002; their reuse status is part of the same pending human legal decision. For this item `workbench_regions.rs` states that no upstream text is copied.
- `git -C vendor-research/QymCAD rev-parse HEAD` = `746a17f127897e921a57118b6caad08968602bf5`; `git -C vendor-research/QymCAD status --porcelain` is empty.
- `legal_reuse_status` stays PENDING_HUMAN_M00_APPROVAL. Legal and M00 approval of QymCAD reuse is a human decision; the agent approved nothing.

### planning_state (item 7)

Read-only counts under `OM9_Milestones`: `l3.json` (792 files) SPEC_ONLY_NOT_IMPLEMENTED 586 / SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED 206; `L2.json` (103) 83 / 20; `l4.json` (2611) NOT_STARTED 1749, SPEC_ONLY_NOT_EXECUTED 340, no `planning_state` 522. The L3-002 `l3.json` stays SPEC_ONLY_NOT_IMPLEMENTED and its four `l4.json` stay NOT_STARTED. Reasons: the vocabulary has no implemented-but-gated value; `tools/merge_l4_overlay.py` is the only writer and only moves SPEC_ONLY_NOT_IMPLEMENTED to SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED (both not-implemented); the `execution_policy` in the `l4.json` files declares `status NOT_RUN`, requires M00 sign-off first and says `must_not_fake_pass`, and M00 is NOT_PASS, legal is pending and the L2-01-02/03 dependencies are not implemented; the tree is protected. `git status --short` scoped to the L3-002 folder printed nothing, so this work changed nothing there. A new state name would be a convention change for the user to decide.

### Final runs (standalone, `-j 1` for tests because a parallel link hit os error 1455)

- `cargo fmt --all -- --check`: exit 0. `cargo check --workspace --all-targets`: exit 0. `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test --workspace -j 1`: exit 0; 581 passed, 0 failed, 24 ignored (native-gated tests). `om9-app`: lib 62, `workbench_regions` 8, `status_bar_overlap` 4, `workbench_regions_sizes` 10, `shell_docking` 10.
- `python tools/validate_m01_l3_002_ui_matrix.py`: exit 0, 10 of 10 cases PASS.
- Log: `build/feat003-final-cargo.log` (generated, not checked in).

### Known defects found, not fixed

- Saving a document that contains a Line fails with `Invalid('archive encoding verification')` in `om9-archive`; the region test therefore saves a Point document.
- Below about 311 px height the plan clamps the command panel but egui draws the stored taller panel (28 px gap at 560x286; the OS minimum of 480 prevents it in a real window). Now out of the supported range (the planner floor is 400 px high and the window minimum 720 px or the capped screen value): recorded, no fix.

## Minimum window 1280x720 logical (2026-10-16)

Evidence only. Nothing below passes the M01-UI, M00 or legal gates; nothing was committed or staged. This section supersedes the older numbers above (560x286, 320x286 without the left rail, the roughly 900 px / measured 915 pt dock rule, and the 10-case matrix).

### Rule (user decisions)

1. The smallest supported device screen is 1366x768. The window must not shrink below 1280x720 logical points (egui points).
2. The right-dock yield threshold was delegated: chosen as **1200 logical points**.

### Known conflict and the choice made

A 1366x768 laptop commonly runs at 125% or 150% DPI, i.e. 1093x614 or 911x512 logical points, and the taskbar and title bar take about 80 points more. A hard 1280x720 logical minimum cannot fit those screens: the window would hang off-screen.

Choice implemented (`startup::effective_min_size`, pure, safe Rust): effective minimum per axis = `max(800x400 floor, min(1280x720, monitor_logical - (16, 80)))`. On a normal screen the 1280x720 floor holds. On a small logical screen the minimum is capped to the usable area instead of forcing the window off-screen. Unknown or non-finite monitor information gives plain 1280x720. The reserve (16 for the frame, 80 = 48 taskbar + 32 title bar) is one constant, `SHELL_RESERVE_PT`; the title bar is included because the 720 pt minimum is the client height. Switching to a taskbar-only reserve is a one-constant change (user decision).

| Screen (logical points) | Effective minimum |
| --- | --- |
| 2560x1440, 1920x1080 | 1280x720 |
| 1366x768 at 100% | 1280x688 |
| 1366x768 at 125% (1092.8x614.4) | 1076.8x534.4 |
| 1366x768 at 150% (910.7x512) | 894.7x432 |
| 640x480 or smaller | 800x400 (floor) |
| unknown / NaN / zero | 1280x720 |

How it is enforced (egui/eframe 0.36.2, winit 0.30.13 from `Cargo.lock`): the monitor is unknown when the window builder runs, so the builder gets the planner floor 800x400. `MatrixApp::show` reads `ViewportInfo::monitor_size` (the full monitor in logical points, refreshed every frame, so a move to another monitor or DPI is seen) and sends `ViewportCommand::MinInnerSize(effective)` on the first frame and whenever the value changes. The real work area is NOT exposed by egui/winit; reading it needs a Win32 call (unsafe or FFI), which was rejected (Rust-first, safe Rust). The usable area is therefore an estimate. `AppConfig.min_window` is now the nominal 1280x720 and `validate()` refuses a start below it. Audit-only `OM9_UI_AUDIT_MONITOR=WxH` simulates a monitor (physical pixels) so the capped laptop cases can be shown on the 1440x900 test display; a normal start never reads it.

### Planner

`PLAN_FLOOR_WINDOW` 800x400 replaces 560x286 (320x286 without the left rail); hiding the left rail no longer lowers it. The floor is below every size the window level can produce (lowest cap 895x432). Below it: unchanged behaviour (`WindowTooSmall`, `Layout:` status error, last good plan, nothing else changes). `YieldReason::NotEnoughRoom` and `RegionNote::LeftRailShrunk` can no longer be produced at any plannable size (compile-time assertions and the `dead_notes_stay_unreachable` test); the variants stay for API and fixture stability.

### Right-dock threshold: 1200 logical points

With the default rails (left 300, right 300, gap 2) the pure planner gives each viewport `(W - 602) / 2`: 156.5 pt at the old 915 first-dock width, 299 pt at 1200. A viewport needs roughly 300 pt to stay useful, so 1200 is the smallest round width that keeps four useful viewports next to both rails. It sits below the supported minimum 1280, so the dock shows at every supported size on a normal screen, and yields on the capped 125%/150% laptop screens (1093 and 911 pt) where viewports stay 395 and 305 pt wide. A rule derived from the viewport width for any rail width was rejected: it changes `DockLayout::resolve` semantics and 420+420 rails could still squeeze viewports; recorded as a follow-up for L3-003 (live resize). Boundary tests: pure planner `right_dock_threshold_boundary` (1199.0 and 1199.99 yield with `NarrowWindow`; 1200.0 shows a 300 pt dock, no note, viewport 299), `dock_layout::right_dock_threshold_boundary_is_1200_logical_points`, and the full-frame sweep `dock_threshold_sweep_1150_to_1250` (`OM9_SWEEP first_dock_width=1200`; the frame viewport is about 304 pt because egui rails measure 295 pt after margins).

### Computed layout evidence (logical points; Layers docked right)

Pure planner numbers are hand-computed and asserted exactly in `layout_evidence_table`; the real frame must agree on the dock decision and stay within 8 pt. Data: `implementation/evidence/M01/OM9-L3-01-01-002/min-window-layouts.json` (log: `min-window-layouts.log`).

| Window | Right dock | Centre (w x h) | Each viewport (w x h) |
| --- | --- | --- | --- |
| 1280x720 | shown | 680x522 | 339x260 |
| 1366x728 (1366x768 at 100%, client area) | shown | 766x530 | 382x264 |
| 1093x585 (1366x768 at 125%, SIMULATED) | yielded | 793x387 | 395.5x192.5 |
| 911x480 (1366x768 at 150%, SIMULATED) | yielded | 611x282 | 304.5x140 |
| 1920x1080 | shown | 1320x882 | 659x440 |
| 5120x1440 (ultra-wide) | shown | 4520x1242 | 2259x620 |
| 1200x720 (threshold) | shown | 600x522 | 299x260 |
| 1199x720 (just below) | yielded | 899x522 | 448.5x260 |

### Assertions changed (and why)

| Where | Old | New | Why |
| --- | --- | --- | --- |
| `workbench_regions.rs` `smallest_usable_window_keeps_every_minimum` | 560x286 plan, rail shrunk to 240, `LeftRailShrunk` note | renamed `floor_800x400_keeps_every_minimum`: 800x400, rail 300, centre 500x202, four 249x100 viewports, no notes | 560x286 is not a supported size; the floor is 800x400 |
| `window_below_the_minimum_returns_window_too_small` | sizes (300,150) (559,286) (560,285) (1,1); min 560x286 | (300,150) (799,400) (800,399) (1,1); min 800x400; 800x400 is Ok | floor moved |
| `hidden_left_rail_lowers_the_minimum_width` (319x286 error, 320x286 ok) | | replaced by `hidden_left_rail_does_not_change_the_floor` (799x400 error, 800x400 ok, command 800 wide) | the floor is rail-independent |
| `narrow_window_yields_right_dock_with_note` 910x700 part | `NotEnoughRoom` | `NarrowWindow` | 910 < 1200 is simply narrow |
| `errors_are_visible_text_...` | "below the minimum 560x286" | text built from `PLAN_FLOOR_WINDOW` | floor moved |
| `narrow_window_never_overlaps_or_hides_command_input` error branch | `< 560 or < 286` | `< 800 or < 400` | floor moved |
| fixtures `positive.json` / `negative.json` | `smallest-usable-window-560x286`; constants 560x286, 320x286, 900; cases 559x286, 560x285, 319x286, 320x286 | `floor-800x400`; constants supported 1280x720, floor 800x400, narrow 1200; cases 799x400, 800x399, 799x400 rail hidden, 800x400 rail hidden | same |
| `tests/workbench_regions_sizes.rs` | 560x286 valid, 720x480/719x479, 700..=1100 sweep, 899/900/901, 720x2000/720x1300, 150% of 800x600 below minimum | supported-size set, capped 911x480, 1150..=1250 sweep, evidence table, DPI sims 1.25/1.5, ONE below-minimum case `unsupported_size_error_path`, startup refusal below 1280x720 | those sizes are no longer supported targets; the error path is kept once |
| `tests/status_bar_overlap.rs` | `minimum_720x480...`, `long_error_at_560x300...` | `capped_911x480...`, `long_error_at_floor_800x400...` | same; overlap behaviour untouched |
| `startup.rs` / `ui.rs` | `min_window` 720x480 | 1280x720 | the user decision |

No assertion was weakened to pass: each was replaced by an equal-strength assertion at the new numbers. New tests: `effective_min_size` unit tests (large, 1366x768 at 100/125/150%, degenerate and a 1..5000 sweep), `window_minimum.rs` (5 frame tests on the `MinInnerSize` command), audit helper unit test, two threshold boundary tests, `dead_notes_stay_unreachable`, compile-time floor/minimum/threshold assertions.

### Real-window matrix (om9.exe `--ui-audit`)

`python tools/validate_m01_l3_002_ui_matrix.py --output build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-min-window/matrix`: exit 0, 9 of 9 cases PASS (`min-1280x720`, `laptop-1366x728`, `wide-1440x720`, `display-1440x900`, `tall-1280x900`, `lap100-1366x690`, `lap125-1366x680`, `lap150-1366x700`, and `unsupported-1000x600-startup-refused`, label "unsupported size, error path": exit 1, no receipt). The validator now also requires `status_message_px` and `status_summary_px` to be non-null in addition to `status_text_overlap == false`, checks `min_window.sent` against the expected effective minimum and `min_window.source`, and fails a case whose window was clamped by the display.

Finding: in this run the primary display reported 1440x900 logical points at 100% (receipt `min_window.monitor_points`), not the larger display assumed in the plan (the earlier closeout matrix opened a 1600x1000 window, so the display configuration differs between runs). The host clamps a window larger than the monitor, so real windows at 1600x1000, 1920x1080 and 2560x1000 were not possible in this run; an unnoticed clamp would have made a case measure a different size than its name, so the validator now fails such a case. Those sizes (and 1280x720 at 150% zoom = 1920x1080 px) are covered by the frame tests and the computed table. The `lap*` cases use the simulated monitor and zoom (labelled SIMULATED).

Screenshots viewed by the agent: `min-1280x720`, `laptop-1366x728`, `lap125-1366x680`, `lap150-1366x700`. Left rail, four viewports, the command panel and the status line are visible with no overlap; the dock shows at 1280 and 1366 and is yielded at the capped 125%/150% sizes. In the lap125/lap150 shots the status line shows "Error: unsupported unknown/unavailable command" from the audit's scripted command replay (command text arrived concatenated); it is not a `Layout:` error and `regions.error` is null in every receipt.

### Honest limits

SIMULATED/AUTOMATED only. Not proven: a maximized start on a real 1366x768 device at 125%/150% DPI, real OS per-monitor DPI, a second monitor, enforcement by dragging an edge, how it looks to a person. The usable area is estimated (monitor minus a fixed reserve), not the real work area. Fallback if a maximized start overflows on a real small screen: a one-time maximize nudge after the first cap (not implemented).

### Skill record

Section "Vùng workbench docking (OM9-L3-01-01-002)" updated with the user's consent of 2026-10-16 for exactly this change: supported minimum 1280x720 logical (window level, capped to the usable area on smaller logical screens), planner floor 800x400, dock threshold 1200. Repo and installed copies both hash to `B1D2305AEA2FC69BE5698C3261D7A1F0A6DCEA5B57D0CF225E13968616A58541`.

### Defects (unchanged)

- The roughly 311 px command-panel mismatch is out of the supported range: recorded, no fix.
- Saving a document containing a Line still fails with `Invalid('archive encoding verification')` in `om9-archive`: unrelated, unfixed.

### Final runs (standalone, `-j 1` for tests)

`cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, `cargo build --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`: exit 0. `cargo test --workspace -j 1`: exit 0, 596 passed, 0 failed, 24 ignored (native-gated). `om9-app`: lib 73, `window_minimum` 5, `workbench_regions` 8, `workbench_regions_sizes` 9, `status_bar_overlap` 4, `shell_docking` 10, `startup_contract` 3. Log: `build/min-window-final-cargo.log` (generated, not checked in). The full `public_source_audit.py` still exits 1 on pre-existing private-tree errors and is NOT claimed as passed; see `verification.md` for the scoped mirror.
