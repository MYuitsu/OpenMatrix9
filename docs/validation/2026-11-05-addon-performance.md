# Add-on command performance — 2026-11-05

Branch `codex/curves-next` (base: main `dc2d4dfd`). Linux x86-64 cloud container, release
build, headless controller, in-process add-on through the same JSON session boundary as the
DLL. Medians of 40 samples per case, built-in and add-on runs interleaved; every sample
checks one new object and exactly one Undo entry, then undoes it (not timed). Harness:
`cargo test --release -p om9-app --no-default-features --features eframe/x11 --test addon_bench -- --ignored --nocapture`.
The document is pre-filled with `n` lines when opened. Absolute times differ from the
admin's Windows figures (0.308 / 2.371 ms); compare ratios.

## Command from start to commit (ms)

| Document | Command | Built-in | Add-on before | Add-on after |
|---|---|---|---|---|
| empty | Line 10 mm | 0.093–0.129 | 2.106 | 0.106 |
| empty | Circle r5 | 0.083–0.123 | 2.148 | 0.101 |
| 5,000 lines | Line 10 mm | 45.4–48.1 | 55.3 | 45.1 |
| 5,000 lines | Circle r5 | 45.6–50.5 | 53.6 | 50.3 |
| 20,000 lines | Line 10 mm | 220–241 | 268.7 | 223.3 |
| 20,000 lines | Circle r5 | 225–256 | 260.1 | 251.9 |

Pointer move during an add-on command (one hover step): 0.40–0.46 ms before, 0.017–0.021 ms
after, independent of the document size.

## Causes found and fixed

1. Every session step resolved and copied the whole settings table of the package (453 specs
   in OM9 Matrix) and then kept the command's own few: ~0.38 of the 0.41 ms per step.
   `Host::command_settings(id, command)` and `command_info` now look only at the specs that
   apply to the command (same resolution: stored value if valid, else default).
2. `session_selection` and `transformed_selection_lines` walked every scene object even with
   an empty selection or no transform preview; they return at once now.
3. A result with one new object was committed by rebuilding the candidate scene of the whole
   document; it now uses `commit_incremental_scene`, as OM9's own Line/Circle do.

## Object snaps (per pointer move, `crates/om9-scene/tests/osnap_bench.rs`)

| Scene | Before | After |
|---|---|---|
| 20,000 lines | 581 ms | 19.8 ms (same 1,119 candidates) |
| 4,000 cubic NURBS | no candidates at all | 10.8 ms |

The Intersection test paired every segment of the file on each move; only segments passing
within the snap radius of the pointer can form a candidate, so only those are paired
(lossless). NURBS curves — most curves the Curve add-on makes — had no snaps; they now give
End (ends, every polyline vertex), Mid, Near, Perpendicular and Intersection, with a control-hull
broad phase.

## Grip drag (UI tests that failed on main)

- A press on a grip no longer starts the object gumball whose handle crosses it.
- The release frame updates the grip to the released pointer/snap (snaps lag one frame).
- Near/Perpendicular/Tangent/Surface snaps on the edited object itself are ignored during a
  grip drag; point snaps on it (End, Mid …) still apply, e.g. to close a curve.

`ui::chrome::chrome_tests` grip tests: 4 failing → all pass; new test
`control_grip_drag_snaps_to_an_end_of_its_own_curve`.

## Linux RAM probe

`om9_native_owner::physical_memory` returned `None` off Windows, so `Controller::new` failed
on Linux (every app test). It now reads `MemTotal`/`MemAvailable` from `/proc/meminfo`.

## Remaining (core, shared by every command)

Adding one object in a large file still costs ~45–50 ms at 5,000 and ~220–255 ms at 20,000
objects, built-in and add-on alike. Measured split of that commit: `register_prepared_document`
~46 %, `prepare_with_held_usage` ~18 %, `coordinator.commit` ~17 %,
`reserve_prepared_held_usage` ~10 %, `transfer_memory_allowance` ~5 %, scene update ~3 %.

## Tests run

om9-app: add-on command/session/smoke, `addon_bench` (2 + ignored bench), `grip_selection`,
library 381 passed / 5 failing as on main without these changes (LOCALAPPDATA, no OCCT on
Linux, typed-command catalog). om9-scene all, om9-addons all (new
`command_settings_match_the_full_table`), om9-native-owner, om9-addon-curves 41,
om9-addon-matrix, `tools/public_source_audit.py` 0 errors.

## Follow-up: per-command settings index and picked-object lookup

- The host keeps, per installed add-on, the positions of the settings that apply to each
  command (made once when the manifest is read). Looking up one command's settings no longer
  depends on how many settings the package has; `matrix_command_settings_match_the_full_table`
  checks every Matrix command against the full-table filter (same values, same order).
- Add-on steps look up the picked objects through a table "object → scene positions" made
  once per published scene (rebuilt when the scene changes; a stale entry is never trusted),
  instead of walking every scene object on each click and pointer move. Test:
  `addon_steps_see_the_current_selection_after_scene_changes`.

Offset (`CurveOffsetCurve`) on one picked line, release, medians of 20, one at a time:

| Document | Pointer move before | Pointer move after | Whole command after |
|---|---|---|---|
| 1 object | 0.041 ms | 0.051 ms | 1.25 ms |
| 5,000 objects | 0.066 ms | 0.056 ms | 54.9 ms |
| 20,000 objects | 0.228 ms | 0.060 ms | 301 ms |

(The container ran ~30 % slower during the "after" runs; built-in Line went 0.093 → 0.124 ms.)
The whole Offset command in a large file is almost all the core commit (start 0.5 ms, typed
distance 0.3–0.5 ms, click + commit 200–300 ms at 20,000 objects).
