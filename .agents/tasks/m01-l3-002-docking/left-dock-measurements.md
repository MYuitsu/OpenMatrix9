# Left-dock measurements (FEAT-001, 2026-10-16)

Measured with a temporary frame test (deleted afterwards) driving the real `MatrixApp::show` for 4 frames, and with the real `om9.exe --ui-audit`.

## Staged baseline

`git diff --cached --name-only` at the start: 5023 lines (4 `tools/*.py` edits plus the staged `OM9_Milestones` renames). The full list is in `staged-baseline.txt` next to this file; SHA-256 `31D6F2D391F56B3FC9E92B22390020E062F51C45B0874827378CE93B8CA0BA35`. After FEAT-001 the same command gives the same hash (nothing was staged, committed, stashed or reset).

## Widths

| window | rail setting | outer (Panel::show rect) | inner (`sidebar_rect`) | icon edge | stored PanelState.size |
|---|---|---|---|---|---|
| 1280x720 | default | 300 | 295 | 22 | 300 |
| 1600x1000 | default | 300 | 295 | 22 | 300 |
| 1280x720 / 1600x1000 | 240 | 240 | 235 | 18 | 240 |
| 1280x720 / 1600x1000 | 460 | 460 | 455 | 34 | 460 |

`LEFT_RAIL_CHROME_PX = 5` (2 px margin each side + 1 px panel stroke). Under the audit zoom 1.25/1.5 (physical-pixel rounding) the outer width is 300 or 300.375. Before the change the stored value was the inner 295 (`ui.max_rect().width()`); no existing test asserted 295, so no test assertion had to change for that.

## Title bars (y, height 14, x 2..297 at rail 300)

| key | 1280x720 | 1600x1000 | rail 240 | rail 460 |
|---|---|---|---|---|
| icon-history | 23 | 23 | - | - |
| main-menu | 63 | 63 | - | - |
| display | 160 | 160 | - | - |
| snaps | 219 | 219 | - | - |
| info-settings | 278 | 278 | - | - |
| layers | 337 | 337 | 377 | 397 |
| projects | 527 | 527 | 567 | 587 |
| builder-slot | 675 | 675 | - | - |

Sidebar rect at 1280x720: y 23..721, so Layers is visible without scrolling at every rail width (also at 460, y 397). Scroll offset is 0. Left stack content height: 821 (rail 300), 989 (240), 893 (460), i.e. taller than the 698 px viewport at 1280x720, so the stack scrolls/clips at the bottom (Builders at 675 still starts inside the viewport at rail 300). All 51 sidebar icon rects stay inside the sidebar x-range at 240, 300 and 460.

## CMD error ("unsupported unknown/unavailable command")

Reproduced in DEFAULT mode (nothing docked right) with `OM9_UI_AUDIT_WINDOW=1366x700 OM9_UI_AUDIT_ZOOM=1.5 OM9_UI_AUDIT_MONITOR=1366x768` (lap150): `cmd` held `missing previous command`, `unsupported unknown/unavailable command` and concatenated drafts, 0 of the 4 checks. lap125 (1366x680, 1.25) passed in default mode in the first run, so the failure is not caused by the right dock.

Root cause (proven by a per-pass debug trace, removed): the lap cases make the window smaller than the stored F6 float, so the first frames append `Command Search was moved back inside the window` to the CMD history (`note_dock_diagnostics` -> `Transcript::append_history`). The CMD cursor stored by `focus_cmd` then points inside the protected history, and `Transcript::insert` refuses the scripted `Text("Line")` (index < protected_chars). The Enter that follows runs on an empty draft ("missing previous command"), and every later draft is never submitted, so they concatenate. Focus itself was already true at that point, so a pure "has focus" gate (the plan's first idea) does not fix it.

Fix: the default audit script now waits (max 30 consecutive passes) until the CMD field has focus AND its cursor is at the end of the transcript, setting `focus_cmd` so the CMD frame repairs the cursor; then it records the check `audit CMD focus ready` once. After the fix lap125 and lap150 passed 5 times each in default mode and in user-docked-right mode, with all four existing checks plus `audit CMD focus ready` and no `unsupported` text. Product note: the same stale-cursor drop can affect a real user who types in the one frame where a diagnostic line is appended; `Transcript::append_history` is outside FEAT-001 and was not changed.

## Assertions changed (with reason)

- `workbench_regions.rs` compile-time: `NARROW_WINDOW_WIDTH > LEFT_RAIL_MAX + RIGHT_DOCK_MAX + CANVAS_MIN_WIDTH` -> `>=` (460 + 420 + 320 = 1200 exactly; `resolve` yields only when the sum is strictly greater).
- `workbench_regions.rs::dead_notes_stay_unreachable`: NOT weakened, extended: also sweeps the widest rail 460 + dock 420 and the widths 1199, 1200, 1201.
- `dock_layout.rs` unit tests `narrow_window_yields_the_right_column_and_keeps_command_line` and `right_dock_threshold_boundary_is_1200_logical_points`: right-dock fixture changed from Layers to `RIGHT_DOCK_FIXTURE` (Snaps).
- Integration tests under `crates/om9-app/tests` still dock Layers right (they are FEAT-002 scope); all pass unchanged.

## FEAT-002 test migration (2026-10-16): every changed assertion, with reason
No numeric or structural assertion was changed or weakened; every existing test passes with its original expected numbers. Only the right-dock fixture moved from Layers to the shared `RIGHT_DOCK_FIXTURE` (Snaps) via `tests/common/mod.rs::dock_right_fixture`. The pure-planner numbers do not depend on which section is docked right, so the hand-computed rows are untouched.
| file | old | new | reason |
|---|---|---|---|
| `tests/shell_docking.rs` | local `const DOCUMENTED_LEFT` | `common::DOCUMENTED_LEFT` (same 8 ids) | one shared copy |
| `tests/shell_docking.rs::narrow_window_yields_the_right_column_and_keeps_command_line_and_canvas` | `app.dock_menu_section(SectionId::Layers, DockSide::Right)` | `dock_right_fixture(&mut app)` | Layers is never docked right in tests (user decision 2026-10-16) |
| `tests/status_bar_overlap.rs::settle` (4 tests) | docks `SectionId::Layers` right; imports `DockSide, SectionId` | `dock_right_fixture(app)`; imports removed; doc comment "Dock Layers right" -> "Dock the shared neutral fixture right" | same |
| `tests/workbench_regions.rs` (4 tests: `normal_window_shows_left_rail_four_viewports_and_the_right_dock`, `the_plan_matches_the_regions_egui_actually_lays_out`, `narrow_window_yields_the_right_dock_and_keeps_the_command_input`, `too_small_window_reports_visibly_and_changes_nothing_else`) | Layers docked right | `dock_right_fixture(&mut app)` | same |
| `tests/workbench_regions_sizes.rs::settle_scaled` | docks Layers right; doc "Dock Layers on the right" | `common::dock_right_fixture(app)`; doc names Snaps | same |
| `tests/workbench_regions_sizes.rs::dock_threshold_sweep_1150_to_1250` | docks Layers right; sweep print `layers_docked_right=true` | `common::dock_right_fixture`; print `fixture_docked_right=snaps` | same; the first dock width is still asserted equal to 1200 |
| `tests/workbench_regions_sizes.rs::pure_plan` (feeds `layout_evidence_table`, `capped_high_dpi_case_911x480`, the sweep) | `.dock_section(SectionId::Layers, DockSide::Right)` / `.expect("Layers docks right")` | `.dock_section(RIGHT_DOCK_FIXTURE, DockSide::Right)` / `.expect("the fixture docks right")` | same; centre/view numbers (680/522/339/260 and the other 7 rows) unchanged and still asserted exactly |
| `tests/workbench_regions_sizes.rs` imports | `SectionId` | `CANVAS_MIN_WIDTH, LEFT_RAIL_*, RIGHT_DOCK_FIXTURE, RIGHT_DOCK_MAX` (for the new justification test) | `SectionId` no longer used there |

## FEAT-002 new tests
- `tests/left_stack_layers.rs` (8): `layers_sits_between_info_and_projects_at_supported_frames` (1280x720, 1280x688, 1093x585, 911x480, 1600x1000), `layers_stays_in_the_left_column_after_reset_menus_from_a_perturbed_layout`, `layers_stays_in_the_left_column_after_a_save_and_reload_round_trip` (default and Snaps-right), `layers_title_bar_is_visible_without_scrolling_at_1280x720` (rail 300 and 460), `default_frame_has_no_right_dock`, `rail_outer_width_is_300_and_inner_is_chrome_less`, `icon_edge_follows_the_rail_width` (240/300/380/460), `max_rail_keeps_the_canvas_minimum` (rail 460 at 1280x720 and 911x480; 1200 with rail 460 + dock 420; 1199).
- `tests/workbench_regions_sizes.rs::left_rail_max_justification_table` (1): 5 windows x rail 240/300/460 x with/without the Snaps dock, plus 1200/1199/1201 at rail 460 + dock 420; writes `left-dock-layouts.json/.log` when `OM9_WRITE_LEFT_DOCK_EVIDENCE=<dir>` is set.
- Shared helpers: `tests/common/mod.rs` (`DOCUMENTED_LEFT`, `dock_right_fixture`, `run_frame`, `settle`, `assert_layers_between_info_and_projects`).
- Optional fixture `left-stack.json` (plan 6e) was not added.

## FEAT-002 measured results
- Layers title bar at 1280x720 (sidebar y 23..718, content height 821 at rail 300 and 893 at rail 460): rail 300 layers y 337 (projects 527), rail 460 layers y 397 (projects 587): visible without scrolling in both.
- LEFT_RAIL_MAX 460 justification (`left-dock-layouts.json`): with rail 460 and the user-docked Snaps dock of 420, 1200x720 gives centre = canvas = 320 exactly and the dock shown; 1201 gives 321; 1199 yields the dock (centre 739, `NarrowWindow` note, no `NotEnoughRoom`). Rail 460 at 911x480 leaves a 451 pt canvas. Frame centre width equals the pure plan in every row (33 rows, all with a valid frame contract and a visible Layers bar).
- Mutation (`RIGHT_DOCK_FIXTURE = Layers`): `cargo test -p om9-app --test left_stack_layers -j 1` exit 101, 2 of 8 failed by assertion (`layers_stays_in_the_left_column_after_a_save_and_reload_round_trip`, `max_rail_keeps_the_canvas_minimum`); restored byte-for-byte (SHA-256 `5E15FD51...D510`), exit 0, 8 passed. Logs: `left-dock-mutation.red.log` / `.green.log`.
- Full gates: `cargo fmt --all -- --check` 0, `cargo clippy --workspace --all-targets -- -D warnings` 0, `cargo test --workspace -j 1` 0 with 612 passed, 0 failed, 24 ignored. Staged index unchanged (5023 lines), `layers.rs`/`session` diff empty.
