# Minimum window 1280x720 logical for OM9-L3-01-01-002

The change makes 1280x720 logical points the supported window minimum. The planner gets a rail-independent floor of 800x400, and the right dock now yields below 1200 points instead of 900. The real window enforces the minimum by sending `ViewportCommand::MinInnerSize(effective_min_size(monitor))` on frame 0 and whenever the monitor or DPI changes. On screens smaller than 1280x720 in logical points (1366x768 at 125% or 150%) the minimum is capped to `monitor - [16, 80]`, so the high-DPI conflict is resolved and not hidden. Evidence is a pure sizing unit test, headless frame tests, a computed layout table, a real-exe matrix, and two screenshots I viewed myself.

Watch for: a maximized start on a real 1366x768 screen at 125%/150% is unproven (recorded honestly). The usable area is estimated from fixed constants, not the Win32 work area (confirmed, documented). Unrelated staged `OM9_Milestones` renames exist in the index from a separate path-shortening task (confirmed not part of this diff).

**Verdict**: APPROVED

## High-level view

Window-level enforcement lives in `MatrixApp::enforce_min_window`, which reads the monitor from the raw viewport info on every frame and sends the min-size command only when the value changes. The pure function `startup::effective_min_size` holds all the arithmetic. The builder gets the planner floor because the monitor is unknown when it runs. A real run reports `source: "viewport"` with the monitor 1440x900 and `sent: [1280, 720]`, so the frame-0 path works against a real winit window.

The planner keeps the same error semantics (`WindowTooSmall`, `Layout:` status text, last good plan kept) and only the numbers move: floor 800x400, rail-independent. `NotEnoughRoom` and `LeftRailShrunk` become unreachable. They are guarded by compile-time assertions and a sweep test, not deleted.

The dock threshold is 1200. The viewport width is `(W - 602) / 2`, so 299 pt at 1200 against roughly 160 pt under the old 915 rule. The computed table and a real-frame sweep (`OM9_SWEEP first_dock_width=1200`) back it.

The skill gained one dated consent line and two bullets. The repo and installed copies have equal SHA-256 (`B1D2305A...8541`, re-compared by me). No gate was claimed. `ui-review.json` keeps `NOT_SIGNED`, `m01_ui: OPEN`, `m00: NOT_SIGNED`, `legal_reuse: PENDING_HUMAN_M00_APPROVAL`, and 5 checklist items with null decisions.

<details>
<summary>Issues (4, none blocking)</summary>

1. **Real small-screen start unproven (confirmed, disclosed)**: no real 1366x768 at 125%/150% run exists; the cases are `OM9_UI_AUDIT_MONITOR`/`ZOOM` simulations labelled SIMULATED. It is on the human checklist; keep it there.
2. **Estimated work area (confirmed, disclosed)**: `SHELL_RESERVE_PT = [16, 80]` is a constant, not the Win32 work area. The doc says changing one constant switches to a taskbar-only reserve. No action beyond the human check.
3. **Stale index state outside this task (confirmed)**: `git diff --cached` now lists thousands of `OM9_Milestones` renames and four `tools/*.py` edits, from the separate `om9-milestones-path-shortening` work (its tools changed at 12:15, progress entry present). `verification.md` says "nothing staged" for this task's own files, which holds, but the user should know the index is not empty.
4. **Audit-only status text in lap125/lap150 screenshots (confirmed)**: both show "Error: unsupported unknown/unavailable command", from scripted command text arriving concatenated. It is not a `Layout:` error (`regions.error` is null). Non-blocking, fix the audit script text separately.

</details>

<details>
<summary>Details</summary>

### Enforcement at the window level, and the 1280x720 vs high-DPI conflict

`enforce_min_window` is called right after `configure_style` in `show`. It reads `i.raw.viewports[viewport_id].monitor_size`, avoiding `i.viewport()` which panics when a backend sends no info. That is a sound choice. With `OM9_UI_AUDIT_MONITOR` set, the monitor comes from the simulated pixels divided by `pixels_per_point`. The result goes through `effective_min_size`, which is `max(floor, min(nominal, monitor - reserve))` per axis, with `None`, non-finite or non-positive input falling back to plain 1280x720.

The conflict is stated in the plan, code docs, validation doc and final report: a hard 1280x720 logical minimum cannot fit a 1366x768 screen at 125% (1093x614) or 150% (911x512). The chosen resolution is the `min(1280x720, work area)` form. Measured results: 1366x768 at 100% gives 1280x688, 125% gives 1076.8x534.4, 150% gives 894.7x432. These match the unit tests and the receipts (`min_window.sent`). The reserve includes the title bar (80 = 48 + 32), a refinement over taskbar-only, and the report says so.

The builder uses `builder_min_window()`, the 800x400 floor, and `AppConfig.min_window` is now the nominal 1280x720. `validate()` refuses windows below it, tested at 1279x720, 1280x719 and 1000x600. The real exe refuses 1000x600 (exit 1, no receipt).

Pure-sizing tests cover large screens, the three 1366x768 modes, tiny and degenerate monitors (NaN, inf, 0, negative), and a sweep over 1..5000 asserting finite results inside `[floor, nominal]`. `tests/window_minimum.rs` has 5 frame tests: frame 0 at 2560x1440, at 100%, small logical screens, an unknown monitor, and send-once-then-resend-on-change. RED (stub) and GREEN logs are recorded.

### Planner floor, threshold, and changed assertions

Every old assertion at 560x286, 320x286, 559/560, 900 or 915 was replaced by an assertion at the new numbers with the same strength. For example `floor_800x400_keeps_every_minimum` asserts full hand-computed rects, and `right_dock_threshold_boundary` asserts 1199, 1199.99 and 1200 including the 299 pt viewport width. Nothing was loosened. The rail-hidden minimum test was replaced because the floor no longer depends on the rail, and the doc says why. I found no weakening.

Computed layout evidence exists for 1280x720, 1366x728, 1093x585, 911x480, 1920x1080, 5120x1440, 1200x720 and 1199x720 (`min-window-layouts.log`, with both the pure plan and real egui frame rects). The hand-computed values match (for example 1280x720 views 339x260, 911x480 views 304.5x140, 1200 views 299 wide, 1199 dock yielded).

Compile-time assertions check floor >= rail max + canvas minimum, the supported minimum >= the threshold, and the dead-note inequality. `dead_notes_stay_unreachable` sweeps widths. The unreachable enum variants are kept for serde and fixture stability and documented as such.

### Screenshots and the real-exe matrix

Matrix: 9 of 9 pass. Each case requires non-null `status_message_px` and `status_summary_px` plus no overlap and `min_window.sent` equal to the expectation. The refused case uses `OM9_STARTUP_DIALOG=0`. The primary display was 1440x900, not the planned 2560x1440, so larger real windows were swapped for display-size cases and the validator fails a clamped window. The coder disclosed this deviation.

I viewed `min-1280x720` and `lap150-1366x700`. Both show the menu bar, the left rail, the green `Command:` panel with its input line, the toolbar, four viewports with no overlap, and a status line without overlapping glyphs. The Layers dock appears at 1280 (via the audit-only `OM9_UI_AUDIT_RIGHT_DOCK`) and is absent at the capped 150% size. That variable and the right dock are existing audit behaviour, not new in this diff. The coder says all four named shots were viewed.

### Skill and installed copy

The skill diff versus HEAD shows the whole "Vùng workbench docking" section. The earlier closeout added it with consent, while this round added one dated consent line (a second line) and rewrote the dock threshold and minimum bullets. Other bullets (`CommandInputKept`, sync suspended during error, CMD/status notices, scope sentence) are untouched. SHA-256 comparison of repo vs `C:\Users\nguye\.codex\skills\openmatrix9-workspace-contract\SKILL.md` returned `True`.

### Private trees and gates

`git status`/`git diff HEAD` over `ref`, `manuals`, `QymCAD`, `vendor-research`, `om9-window.png` is empty. The staged `OM9_Milestones` renames are from the separate path-shortening task (tools modified at 12:15, own progress entry), and no `planning_state` edits belong to this change. The full `public_source_audit.py` is reported as exit 1, NOT passed. The scoped-mirror result is labelled as covering only mirrored files. No human, M00 or legal gate is claimed.

</details>

<details>
<summary>File map</summary>

- `standalone/crates/om9-app/src/startup.rs`: `SHELL_RESERVE_PT`, `effective_min_size`, `builder_min_window`, nominal min 1280x720, sizing tests
- `standalone/crates/om9-app/src/workbench_regions.rs`: `SUPPORTED_MIN_WINDOW`, `PLAN_FLOOR_WINDOW`, rail-independent floor, updated and new tests
- `standalone/crates/om9-app/src/dock_layout.rs`: `NARROW_WINDOW_WIDTH` 900 to 1200, boundary test
- `standalone/crates/om9-app/src/ui.rs`: `enforce_min_window`, audit monitor/zoom env, receipt fields, builder floor
- `standalone/crates/om9-app/src/ui/chrome.rs`: status-bar overlap fix (closeout item)
- `standalone/crates/om9-app/tests/window_minimum.rs`, `workbench_regions_sizes.rs`, `status_bar_overlap.rs`: frame tests, size matrix, overlap
- `tools/validate_m01_l3_002_ui_matrix.py`: real-exe matrix
- `implementation/fixtures/M01/OM9-L3-01-01-002/*.json`, `implementation/evidence/M01/OM9-L3-01-01-002/*`: fixtures, logs, layout table, checklist
- `skills/openmatrix9-workspace-contract/SKILL.md` and the installed copy: consent line plus two bullets
- `docs/validation/...docking-regions.md`, `docs/openmatrix9-progress.json`, `.agents/tasks/m01-l3-002-docking/*`: docs and reports

Full diff: `git diff` in `h:\OpenMatrix9`.

</details>
