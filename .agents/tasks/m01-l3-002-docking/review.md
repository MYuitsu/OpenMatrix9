# Workbench docking regions as a pure, validated region plan (OM9-L3-01-01-002)

The change adds `om9-app/src/workbench_regions.rs`, an egui-free model of the Matrix9 window. It takes `&DockLayout`, a window size and a view set, and returns a `RegionPlan` (left rail, centre with 1 to 4 viewport slots, optional right dock, command input, bars) or an explicit `RegionError`. `ui.rs` now takes its four-viewport tiling from this module and keeps the last good plan when a frame cannot produce a new one. The four L4 artifact sets, the red log and the evidence files exist, and the logs record real commands with exit codes. Legal reuse, M00, the M01-UI manual gate and the reviewer signature are all labelled pending or open. No QymCAD dependency, no staging, no out-of-scope features.

Watch for: the public source audit exits 1 for the repository as a whole (pre-existing, private trees; confirmed honestly recorded as not passed). Three business-rule changes still await user consent and nothing was written to a skill (confirmed). The `CommandInputHost` adapter in `ui.rs` always answers "ready" (confirmed, documented as a seam).

**Verdict**: APPROVED

## High-level view

Geometry that used to sit inline in egui code is now a plain function with typed failures. Constants and widths still come from `DockLayout::resolve`, so `dock_layout.rs` is untouched and the sibling L3 tasks (persistence, reorder, reset, builder slot) are not affected. The command input is treated as critical: it is never shorter than `COMMAND_MIN`, a hidden flag is overridden with a note, and a window that cannot honour the minimums is rejected rather than squeezed.

The seams toward OM9-L2-01-02 and OM9-L2-01-03 are two small traits with stub-driven tests. The real adapters in `ui.rs` claim nothing beyond "CMD frame attached" and "workspace visible views". The blockers are reported as BLOCKED_DEPENDENCY in the logs, not faked.

Evidence is consistent with the code: the red log shows assertion failures with no build errors (18 lib and 6 integration failures), the green logs show 21 lib, 8 integration and 10 unchanged `shell_docking` tests, and two real `--ui-audit` runs at 1600x1000 and 800x600 have screenshots on disk.

<details>
<summary>Issues (4)</summary>

1. **Public source audit not green** — `tools/public_source_audit.py` exits 1 (11721 errors, all in private or untracked trees; 0 in task paths). It is recorded as not passed. Keep it open until the audit excludes `vendor-research/` and the private trees, or a human decides it is expected. Not blocking: the plan's "exit 0" cannot be met without touching private references, which AGENTS.md forbids.
2. **Business-rule consent pending** — the four changes listed in `verification.md` (below-minimum window shows an error instead of clamping silently, hidden command line still reserved, layout sync suspended while the error is active, right-dock and left-rail notices on CMD and status) need the user's decision before any skill is edited. Ask the user.
3. **Constant-true command host adapter** — `AttachedCmdFrame::command_input_ready` always returns true, so `CommandInputUnavailable` can only occur in tests until OM9-L2-01-02 exists. Documented in code and in the logs; revisit when that module lands. Confirmed, non-blocking.
4. **Status text overlap at 800x600** — the pre-existing status line overlaps itself in the narrow capture (L3-007 area). Listed for the human UI reviewer. Not a regions overlap.

</details>

<details>
<summary>Details</summary>

## Command-input protection and the negative oracle

`compute_regions` returns `WindowTooSmall` below `left min + canvas min` wide or `menu + command min + toolbar + status + canvas min` tall. Above that, the command height is `clamp(COMMAND_MIN, COMMAND_MAX).min(height - reserved).max(COMMAND_MIN)`, so it never goes under 58 px. Every plan is passed through `RegionPlan::validate` (finite, inside the window, no pairwise overlap, command at least `COMMAND_MIN`, view slots disjoint and inside the centre) before it is returned, so an inconsistent plan cannot escape.

The negative test is not a stub. `negative_fixture_sweep_never_overlaps_or_hides_the_command_input` sweeps widths and heights over three layouts (default, right-docked main menu, command line hidden). Every `Ok` plan asserts `command.height >= 58` and no overlap with the left rail, centre, right dock, toolbar or status bar. Every other size must be `WindowTooSmall`. It also asserts that both outcomes occurred, so it cannot pass vacuously. A frame-level test drives the real `MatrixApp` at a narrow width and confirms the right dock yields with a note while the command input stays.

## Failure containment in ui.rs

On `Err`, `refresh_region_plan` stores the error, keeps the previous plan and puts a `Layout:` error on the status line through `status_surface`. Only text raised by this module is cleared on recovery, and a test confirms an unrelated failure is not cleared. Panel-size sync and floating-menu clamping are skipped while the error is active, so a squeezed window does not rewrite the stored layout. `too_small_window_reports_visibly_and_changes_nothing_else` checks dock JSON, `Workspace` and entity count unchanged. The module imports only `dock_layout`, `ViewId` and serde, so it cannot reach the scene or a render cache.

## Seams toward OM9-L2-01-02 and OM9-L2-01-03

`CommandInputHost` and `ViewportSceneHost` are minimal traits. Tests use stubs with call counters to show only the documented queries are made. `ViewportSceneHost` is implemented for the existing `Workspace`. The adapter for the command input is a unit struct that always reports ready (see issue 3). The logs and `api.md` state that those two L2 modules are `SPEC_ONLY_NOT_IMPLEMENTED` and the L4-03 dependencies are BLOCKED_DEPENDENCY. Nothing claims they are satisfied.

## QymCAD, legal state and gates

No `Cargo.toml` in `standalone/` references QymCAD. The only mentions are doc comments. `workbench_regions.rs` says it copies no upstream text and that legal reuse is pending the human M00 review. The crosswalk records the pinned SHA, `verified_head` equal to it, a clean checkout (I re-checked `rev-parse HEAD` and `status --porcelain` on `vendor-research/QymCAD`), four anchors, and `legal_reuse_status` PENDING_HUMAN_M00_APPROVAL with `m00_gate` NOT_PASS. `ui-review.json` has all reviewer decisions `null`, `reviewer.status` NOT_SIGNED, `m01_ui` OPEN and the checklist labelled "PENDING HUMAN VERIFICATION (not passed)". The `.gitignore` addition for `/vendor-research/QymCAD/` fits the private-reference policy.

## Scope and unrelated changes

`dock_layout.rs` is untracked from the earlier L2-01-01 pass and is not modified by this task per the verification note. No persistence, restore-default, reorder, float, builder slot, second menu or mechanical-CAD UI work appears in the new module. The two audit-only environment variables (`OM9_UI_AUDIT_WINDOW`, `OM9_UI_AUDIT_RIGHT_DOCK`) are read only when an audit directory is given, and an unusable value is a visible startup failure. The working tree also holds earlier L3-001 and L2-01-01 changes (startup, status surface, native-owner startup dialog). They predate this task, and nothing is staged (`git diff --cached` is empty). The only `unsafe` in the diff is the earlier L3-001 `MessageBoxW` wrapper inside `om9-native-owner`, which has a SAFETY comment. No new non-Rust code was added.

## Not verified by me

I did not re-run the cargo suites, as instructed; I relied on the recorded logs. I did not open `MATRIX9_UI_CONTRACT` (absent locally, recorded as unverified). The per-task `*.evidence.json` files named in the runbooks are not written because their capture tools are absent; the declared `outputs` are present. DPI, multi-monitor and live-drag behaviour remain for the human reviewer.

</details>

<details>
<summary>File map</summary>

- `standalone/crates/om9-app/src/workbench_regions.rs` — new pure region model, errors, tiling, host seams, 21 unit tests
- `standalone/crates/om9-app/tests/workbench_regions.rs` — fixture replay, sweep, egui frame tests (8)
- `standalone/crates/om9-app/src/ui.rs` — plan refresh, tiling via `tile_views`, error to status line, audit-only env vars
- `standalone/crates/om9-app/src/lib.rs` — `pub mod workbench_regions`
- `implementation/design/M01/OM9-L3-01-01-002/{api.md,source_crosswalk.json}` — L4-01 and L4-02 design outputs
- `implementation/fixtures/M01/OM9-L3-01-01-002/{positive,negative}.json` — L4-02 fixtures
- `implementation/evidence/M01/OM9-L3-01-01-002/*` — red log, diffstat, target-test log, regression log, ui-review.json
- `docs/openmatrix9-progress.json`, `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md` — ledger and report
- `.gitignore` — ignore the local vendor-research checkout

Full diff: `git diff` plus the untracked files listed in `git status`.

</details>
