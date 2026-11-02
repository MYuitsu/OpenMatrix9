# Close-out plan: OM9-L3-01-01-002 Workbench Docking Regions

Scope: close the 8 open items left after the APPROVED run (`review.json`). Work in `h:\OpenMatrix9`, cargo in `h:\OpenMatrix9\standalone`, PowerShell with `;` (never `&&`).
No commit, push or staging. No edit, move, delete or staging of `ref\`, `manuals\`, `OM9_Milestones\`, `QymCAD\`, `vendor-research\`, `om9-window.png`. No QymCAD code copied or depended on. Evidence only. No human gate (M01-UI, M00, legal) is ever claimed as passed, and `reviewer.status` stays `NOT_SIGNED`.

## What I read and what it established

- Status-bar code: `standalone/crates/om9-app/src/ui/chrome.rs::status_bar_ui` (line ~994). It draws the message label, then a nested `right_to_left` layout holding the Dismiss button and the `dock_summary()` label, both with no width limit. The summary is roughly 100 characters ("left [Icon History, Main Menu, ...] right [Layers]"), so in a 505 px status bar (800x600, rail 295 px) it extends left over the message. The existing capture `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-docking/narrow/workspace.png` shows exactly that: "Ready: Ready" printed over "...px | left [Icon History, ..." at the bottom. This is a confirmed reviewer finding, reproduced from the image. It is not "already working".
- The status bar is `egui::Panel::bottom("matrix-status")` in `ui.rs` (~line 898), `status_rect` is recorded; the message and summary rects are not recorded anywhere, so nothing can currently fail on this overlap.
- Audit harness: `ui.rs::launch_classified` reads `OM9_UI_AUDIT_WINDOW`; `launch_with` reads `OM9_UI_AUDIT_RIGHT_DOCK`; the receipt has a `regions` block. There is no zoom/DPI override. `AppConfig` has `min_window = [720, 480]` and `validate()` rejects a smaller window, so "below minimum" cannot be a real window; it can only be a headless egui frame (as in `tests/workbench_regions.rs::too_small_window_reports_visibly_and_changes_nothing_else`, which uses `context.run_ui` with `RawInput.screen_rect`).
- Model thresholds (`dock_layout.rs`, `workbench_regions.rs`): `NARROW_WINDOW_WIDTH = 900`, `LEFT_RAIL_MIN 240`, `CANVAS_MIN_WIDTH 320`, `CANVAS_MIN_HEIGHT 120`, `COMMAND_MIN 58`, menu 21, toolbar 67, status 20. Hard minimum 560x286 (320x286 without rail). A right dock at 910 wide still yields (`NotEnoughRoom`), so the width where the dock first appears must be found by a sweep, not hard-coded.
- Display: primary 2560x1440 (work area 2560x1392) and a second monitor 1080x1920 at x=2560. Real windows up to 2560x1300 fit. The second monitor exists but placing a window on it with a different DPI is not machine-proven here, so it stays human.
- `tools/public_source_audit.py` has only `--project-root`. It walks everything below that root; there is no path filter or exclude list. It also reads `Resources/menu/icons.ini` and `source-manifest.json` under the root, so a scoped root must contain `Resources/`.
- Plan-tree vocabulary (counted by script): `l3.json` uses `SPEC_ONLY_NOT_IMPLEMENTED` (586) and `SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED` (206); `l4.json` uses `NOT_STARTED` (1749) and `SPEC_ONLY_NOT_EXECUTED` (340); L2 uses the same two as L3. `tools/merge_l4_overlay.py` only moves `SPEC_ONLY_NOT_IMPLEMENTED` to `SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED`. No value in the convention means "implemented but gated".
- Skill: `skills/openmatrix9-workspace-contract/SKILL.md` (Vietnamese). Its installed copy `C:\Users\nguye\.codex\skills\openmatrix9-workspace-contract\SKILL.md` is currently byte-identical (same SHA-256 `A44B068E...F87B`). The skill itself says "Sync repository and installed skill copies after approved updates." The L3-002 spec path now lives under `OM9_Milestones\M01 - CAD Shell, Viewport\L2-01-01 - Febe Application Shell Docking\L3-002 - Workbench Docking Regions\` (the long M01 folder name was shortened earlier). That tree has `l3.json` and four `l4.json` (L4-01, 02, 03, 05) and is read-only for this task.

## Decomposition

Three features, strictly in order (all share the working tree and cargo build dir). FEAT files are in `task-m01-l3-002-closeout/features/`.

- FEAT-001 (item 1): status-bar overlap, failing test first, then fix, then screenshots.
- FEAT-002 (item 2): size and DPI matrix tests, audit zoom override, matrix runner, screenshots, shorter checklist. Depends on FEAT-001 (the matrix asserts the status text does not overlap).
- FEAT-003 (items 3, 4, 5, 6, 7, 8): scoped audit, skill text, BLOCKED_DEPENDENCY wording, QymCAD check, planning_state explanation, ledger and report. Depends on FEAT-001 and FEAT-002 (the touched-file list and the numbers).

Item closability:

| Item | Machine-closable? | Notes |
| --- | --- | --- |
| 1 status-bar overlap | Yes, fully | red test, fix, green test, images viewed |
| 2 manual checklist | Partly | layout/overlap/command survival at several sizes and a simulated 150% scale are machine-proven and labelled SIMULATED; real DPI, second monitor, live drag stay PENDING_HUMAN |
| 3 public source audit | Partly | scoped result can be recorded; the full audit stays failed; scoping change is a proposal for the user |
| 4 skill text | Yes | consent already given by the user's reply; edit repo skill and sync the installed copy |
| 5 L2-01-02 / L2-01-03 | Yes (documentation only) | stubs stay, state BLOCKED_DEPENDENCY with the follow-up; nothing implemented |
| 6 legal / M00 | No | human only; stays PENDING_HUMAN_M00_APPROVAL; only the no-copy / no-dependency confirmation is machine-checked |
| 7 planning_state | Yes (decision = leave unchanged, explained) | no justified value exists |
| 8 ledger and report | Yes | with the done / human-only table |

---

## Item 1. Status-bar text overlap at 800x600 (MACHINE-CLOSABLE)

Files: `standalone/crates/om9-app/src/ui/chrome.rs` (status_bar_ui), `standalone/crates/om9-app/src/ui.rs` (two new `pub Option<Rect>` fields next to `status_rect`, initialised in `from_controller`, and the `regions` receipt block), new `standalone/crates/om9-app/tests/status_bar_overlap.rs`.

Steps (order matters, failing first):

1. Reproduce. Build (`cargo build -p om9-app`) and run `standalone\target\debug\om9.exe --ui-audit <dir>` with env `OM9_UI_AUDIT_WINDOW=800x600` and `OM9_UI_AUDIT_RIGHT_DOCK=1`. Save the "before" image into the new dated folder `build/standalone-validation/ui-artifacts/<YYYY-MM-DD>-m01-l3-002-closeout/before-narrow/` and view it. Record that the overlap is visible. Use the date from `Get-Date`; do not overwrite the 2026-10-16 folder.
2. Add measurement only, no behaviour change. In `status_bar_ui` store the response rect of the message label in `self.status_message_rect` and of the summary label in `self.status_summary_rect` (reset both to `None` at the top of each frame in the status panel closure). Add both to the `regions` receipt block (`status_message_px`, `status_summary_px`, `status_text_overlap` boolean).
3. Write the regression test `tests/status_bar_overlap.rs` using the helper pattern of `tests/workbench_regions.rs` (`MatrixApp::new`, `egui::Context::default()`, `run_ui` with `RawInput.screen_rect`, two warm-up frames). Cases, each asserting `message.max.x <= summary.min.x + 0.5`, both rects inside `status_rect`, and (when a failure is on screen) the Dismiss button not overlapped:
   - 800x600 with the Layers section docked right (this is the screenshot case);
   - 720x480 with the right dock;
   - 1600x1000 (control: must still pass, summary fully shown);
   - 560x300 narrowest usable width with an Error status (`Layout:` text via a too-small neighbour, or `app.status.error(long text)`), so the long-message branch is covered.
   Also assert the text is not lost: when the summary or message is shortened, the full text is available (hover text / tooltip text equals the full string, tested via the stored string or `on_hover_text` response, whichever the implementer can reach; the full status string is also in the CMD history for layout notices, which the existing test already checks).
4. RED: run `cargo test -p om9-app --test status_bar_overlap`. It must exit non-zero with assertion failures on the 800x600 and 720x480 cases and no build error. Save the log to `implementation/evidence/M01/OM9-L3-01-01-002/closeout-01.red.log` (command, stdout, stderr, exit code). If the test passes before the fix, the test is wrong (headless fonts differ from the GPU run); fix the test, do not proceed.
5. FIX, minimal, in `chrome.rs::status_bar_ui` only. Render each label with `egui::Label::new(rich_text).truncate()` so egui elides it to the width it is given, give the message label priority (it is laid out first and keeps its width), and give the right group (Dismiss + summary) only the remaining width. Attach `.on_hover_text(full_text)` to both so nothing is lost. Do not change `compute_regions`, `RegionPlan`, `dock_layout.rs`, the status text content or `StatusSurface`. No change to status semantics (Escape, Dismiss, acknowledgment). If `truncate` is not enough to stop the nested right-to-left layout from overflowing leftwards, wrap the right group in `ui.allocate_ui_with_layout` with an explicit `max_rect` starting at `message_rect.max.x`; still local to this function.
6. GREEN: `cargo test -p om9-app --test status_bar_overlap` passes. Then `cargo test -p om9-app --test workbench_regions`, `--test shell_docking`, `--lib workbench_regions`, and all tests that touch the status bar (`cargo test -p om9-app`). Log to `closeout-01.green.log`.
7. Re-capture with the real app into `build/standalone-validation/ui-artifacts/<date>-m01-l3-002-closeout/`: `normal/` (1600x1000, `OM9_UI_AUDIT_RIGHT_DOCK=1`) and `narrow/` (800x600, `OM9_UI_AUDIT_RIGHT_DOCK=1`); keep `ui-receipt.json` and `workspace.png`. View both images with the image viewer and write what is seen into the evidence note: the bottom line shows the message and an elided summary with no glyphs on top of each other; the 1600x1000 line still shows the full summary. Confirm the receipt `status_text_overlap` is `false` in both runs and the audit still reports 21 passes with exit 0.
8. Ledger note: record in the validation doc and `progress.json` that this is a minimal status-bar fix inside `chrome.rs` (L3-007 area, a sibling item), that strict L3-007 scope (status-bar content, colours, acknowledgment rules) is untouched, and that region-plan behaviour is unchanged (the `workbench_regions` tests are the proof).

Verify: `cargo fmt --all -- --check`; `cargo check --workspace --all-targets`; `cargo clippy --workspace --all-targets -- -D warnings`; the RED log shows exit 101 with assertion failures; the GREEN run exits 0; both new screenshots viewed.

## Item 2. Automate the manual checklist (PARTLY machine-closable)

Everything produced here is labelled `SIMULATED / AUTOMATED evidence. Does NOT replace the human M01-UI sign-off.`

Files: new `standalone/crates/om9-app/tests/workbench_regions_sizes.rs`; `standalone/crates/om9-app/src/ui.rs` (audit-only `OM9_UI_AUDIT_ZOOM`, receipt fields); new `tools/validate_m01_l3_002_ui_matrix.py`; `implementation/evidence/M01/OM9-L3-01-01-002/OM9-L4-01-01-002-05.ui-review.json`; screenshots under the dated folder.

1. Headless frame matrix test (`tests/workbench_regions_sizes.rs`). Helpers as in `tests/workbench_regions.rs`. A `check_frame(app, size_in_points, label)` asserts for every size: no panic; `plan.validate()` ok; exactly four `view_rects`, pairwise non-overlapping, inside `plan.center` (8 px tolerance, as the existing test), not touching the left rail or right dock; `command_input_rect()` is `Some`, inside `plan.command_input`, intersects no view; `status_message_rect` / `status_summary_rect` do not overlap (needs FEAT-001); all rects finite and inside the screen rect. Sizes (physical px; logical = physical / scale):
   - minimum usable by the plan: 560x286 (expect ok), 559x286 and 560x285 (expect `WindowTooSmall`, status Error starting `Layout:`, last good plan kept, dock JSON, workspace and entity count unchanged);
   - OS minimum 720x480 and just below 719x479 (the model must still plan; the real-window rejection is `AppConfig::validate`, tested in the same file with `AppConfig::interactive`: size below `min_window` returns a `StartupFailure`);
   - 900-wide boundary: find by sweeping 700..1100 step 1 the first width where the right dock is shown (`plan.right_dock.is_some()`); assert dock absent at `first-1`, present at `first`, and assert the note is `NarrowWindow` below 900 and `NotEnoughRoom` between 900 and `first-1`; test at 899, 900 and 901 explicitly as well;
   - ultra-wide 5120x1440 and 2560x600; tall-narrow 720x2000 and 720x1300;
   - simulated 150% DPI: `context.set_pixels_per_point(1.5)` with `screen_rect = physical / 1.5` for 1600x1000 (=1066.7x666.7 pt), 1280x800, 800x600 (=533.3x400 pt), and 1440x960 with the right dock. Assert the same invariants. State honestly in the test doc comment that egui lays out in points, so this proves point-space layout plus rounding at 1.5, not OS per-monitor DPI behaviour.
   - Right-dock requested in each case (`dock_menu_section(SectionId::Layers, DockSide::Right)`), and one run with no right dock.
2. Checklist items that become machine-provable through the same test file:
   - item 4 (hidden command line cannot be lost): in a frame, `app.dock.set_visible(PanelId::CommandLine, false)` and `app.toggle_right_dock()`; assert `command_input_rect()` is still `Some`, the plan carries `CommandInputKept`, the notice is in the CMD history. First grep `ui.rs` and `ui/chrome.rs` for a menu entry that hides the command line (the earlier grep for `CommandLine` in `src/ui/**` found none). If there is no menu entry, say so in the checklist: this is proven by driving the layout API, not the menu.
   - item 5 (command, Undo, Redo, save, reopen leave the regions alone): reuse the controller patterns in `tests/command_sessions.rs` and `tests/cold_reopen.rs`/`file_lifecycle.rs`. Run one Line command, Undo, Redo, save to a temp `.om9`, reopen, and assert `region_plan` is equal before and after at the same window size. If those APIs cannot be driven from a frame without a native dialog, do not force it: leave item 5 as human-only and record why.
3. Audit zoom override. In `launch_with`, when `app.audit.is_some()` and `OM9_UI_AUDIT_ZOOM` is set, parse a finite value in `[0.5, 4.0]`, else fail visibly as `StartupFailure` (same style as `OM9_UI_AUDIT_WINDOW`), and call `cc.egui_ctx.set_zoom_factor(zoom)` inside the `run_native` closure before the first frame. Add unit tests next to `parse_window_size` (valid, zero, NaN, text, out of range). Add `zoom_factor`, `pixels_per_point` and `screen_points` to the receipt `regions` block. `OM9_UI_AUDIT_ZOOM=1.5` with `OM9_UI_AUDIT_WINDOW=1600x1000` simulates a 1600x1000 physical window at 150%: the app sees 1066.7x666.7 points. Check the screenshot size (physical px) in the runner; if eframe returns a different size for the screenshot, record it as is.
4. Real-window matrix runner `tools/validate_m01_l3_002_ui_matrix.py` (precedent: `tools/validate_standalone_ui.py`). Rust-first exception, to be recorded in the evidence: it only launches the real `om9.exe` with environment variables, waits, and reads `ui-receipt.json`; it needs a real GPU window, cannot live inside `cargo test`, and holds no business logic. It takes `--binary` (default `standalone/target/debug/om9.exe`, build first with `cargo build -p om9-app`), `--output`, runs each case in a temp profile under `build/standalone-validation`, requires exit 0 and `regions.error == null` (or the expected error), checks the receipt invariants (4 views, no overlap, command input present, status text overlap false, right dock expectation), writes `matrix-summary.json` with every case, exit code, receipt SHA-256 and screenshot SHA-256. Cases (real windows, primary display 2560x1440, ppp 1.0 expected; record `pixels_per_point` and flag any run where it is not what is requested):
   - `min-720x480` with right dock, `boundary-899x700`, `boundary-<first>x700` (first width from the sweep, `first-1` too), `ultrawide-2560x600`, `tall-narrow-720x1300`, `normal-1600x1000`, `narrow-800x600`, and the SIMULATED DPI cases `dpi150-1600x1000`, `dpi150-1280x800` (`OM9_UI_AUDIT_ZOOM=1.5`).
   - "Just below minimum" and "far too small" have no real-window case (OS minimum / startup validation); the summary lists them as `frame-test-only`.
   - Each screenshot is viewed once (at least minimum, boundary, ultrawide, tall-narrow, dpi150-1600x1000) and the observations written down. Do not claim an image was inspected unless it was opened.
5. Refresh `OM9-L4-01-01-002-05.ui-review.json`: add `automated_simulated_evidence` (label `SIMULATED/AUTOMATED - does not replace human M01-UI sign-off`, list of cases with results and screenshot paths, commands, exit codes). Replace `manual_reviewer_checklist.items` with a shorter list where each entry has `machine_proven` (what the tests and runner proved, with the artifact) and `human_must_verify` (the remainder), and `reviewer_decision: null`. Target list (6 or fewer):
   1. Real Windows at 150% display scaling, and a window moved between the 2560x1440 and the 1080x1920 monitors: left rail, four viewports, command input, status text readable, no overlap, text crisp. (machine: simulated 1.5 point-space layout. human: real per-monitor DPI, second monitor.)
   2. Live drag-resize by a person: right dock yields at about 900 px and returns, no flicker or stuck panel, and the visible "Layout:" message when shrinking toward the minimum (machine: every discrete size and the boundary; human: the drag itself).
   3. Hide the command line / right dock (menu if it exists): command line cannot be lost (machine: layout API frame test; human: the menu path, if any).
   4. Command, Undo, Redo, save, reopen leave regions alone (machine: frame/controller test if achievable; human: confirm on the real window).
   5. Look at the bottom status line at 800x600 and 720x480 after the fix: readable, not overlapping (machine: rect test and image viewed by the agent; human: judge readability).
   Keep `reviewer.status = "NOT_SIGNED"`, `name`/`date` null, `gate.m01_ui = "OPEN"`, `gate.m00 = "NOT_SIGNED"`, `gate.legal_reuse = "PENDING_HUMAN_M00_APPROVAL"`. Update `not_captured` and `remaining`; remove "status text overlaps" from `observed_defect` only as history (keep the old string as `previous_defect_fixed_in_closeout`).

Verify: `cargo test -p om9-app --test workbench_regions_sizes` passes; `cargo test -p om9-app` passes; `python tools\validate_m01_l3_002_ui_matrix.py` exit 0 and `matrix-summary.json` present; JSON files parse (`python -c "import json,sys;json.load(open(sys.argv[1],encoding='utf-8'))" <file>`); `reviewer.status` still `NOT_SIGNED` (checked by the runner or a tiny assertion in the final review).

## Item 3. Public source audit (scoping not supported; PARTLY closable)

Files: none edited in the repo tools. New output `implementation/evidence/M01/OM9-L3-01-01-002/closeout-03.scoped-audit.log`.

Finding: `public_source_audit.py` accepts only `--project-root`; there is no path filter or exclude. I will not change its behaviour.

1. Re-run the full audit once and record the exact command and result: `python tools\public_source_audit.py` (repo root). Expect exit 1 with about 11721 errors in the private trees; record the count and that it is not passed. If the number changed, say so and list the top-level directories of the errors (group error paths by first segment) to prove none are in L3-002 paths.
2. Scoped run using the existing `--project-root` flag on a read-only mirror: create `build\scoped-audit\<date>-l3-002\` (`build` is already an excluded directory name, and the mirror is outside every private tree), copy into it (copy only; never move) `Resources\` plus the L3-002 touched files that the audit can scan, keeping relative paths: the Rust and test files from `final-report-vi.md` and FEAT-001/002 (`standalone/crates/om9-app/src/workbench_regions.rs`, `src/ui.rs`, `src/ui/chrome.rs`, `src/lib.rs`, `tests/workbench_regions.rs`, `tests/status_bar_overlap.rs`, `tests/workbench_regions_sizes.rs`), `tools/validate_m01_l3_002_ui_matrix.py`, `implementation/{design,fixtures,evidence}/M01/OM9-L3-01-01-002/*`, `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md`, `docs/openmatrix9-progress.json`, `skills/openmatrix9-workspace-contract/SKILL.md`, `.agents/tasks/m01-l3-002-docking/*.md|*.json`. Do not copy any PNG into the mirror (the tool flags unreviewed rasters) and do not copy anything from the private trees.
3. Run `python tools\public_source_audit.py --project-root build\scoped-audit\<date>-l3-002`. Record the command, stdout JSON, exit code. Expect exit 0 with `source_files` equal to the number of mirrored files.
4. Non-vacuity control: add a dummy `control.exe` and `control.png` inside the mirror only, re-run, expect exit 1 with exactly those two errors, then delete them and re-run to show exit 0 again. Record all three runs.
5. Final report wording: "scoped mirror run over the touched files, exit 0; the full repository audit still exits 1 with N errors in private trees; the full audit has NOT passed." Never write that the audit passed.
6. Proposal (for the user, not implemented): add `--paths FILE...` (or `--only-tracked` via `git ls-files`) and an `--exclude` list for `ref`, `manuals`, `OM9_Milestones`, `QymCAD`, `vendor-research` and root screenshots, with the default behaviour unchanged so CI keeps the strict mode. Trade-off: an exclude list weakens fail-closed behaviour if used by default; a `--paths` mode does not. Put the proposal in the final report and ask the user to decide.

Verify: the three log entries exist; the full-run exit code is 1 and is reported as such.

## Item 4. Record the business-rule change in the skill (MACHINE-CLOSABLE)

Files: `skills/openmatrix9-workspace-contract/SKILL.md` (repo copy), then `C:\Users\nguye\.codex\skills\openmatrix9-workspace-contract\SKILL.md` (installed copy, sync).

Consent: the user's reply "làm xong các việc chưa xong" to the final report's question is consent to the exact text under "Thay đổi quy tắc nghiệp vụ" in `final-report-vi.md`. Add nothing beyond the four behaviours plus the ~900 px right-dock yield.

1. Insert a new section after "Mặc định giao diện khi khởi động OpenMatrix9" and before "Read only the relevant reference:", written in Vietnamese in the same layout (heading `## Vùng workbench docking (OM9-L3-01-01-002)`, a dated consent line in the same form as the 2026-10-11 line, then bullets):
   - consent line: `Người dùng đồng ý ngày <date from Get-Date>: trả lời "làm xong các việc chưa xong cho xong M01 L2-01-01 L3-002" cho đề xuất nội dung skill trong báo cáo cuối của OM9-L3-01-01-002.`
   - Rail trái, bốn viewport và ô nhập lệnh luôn được giữ; dock phải nhường khi cửa sổ hẹp hơn khoảng 900 px.
   - (a) Cửa sổ nhỏ hơn mức tối thiểu (560x286, hoặc 320x286 khi không có rail trái) hiện lỗi `Layout:` trên thanh trạng thái và giữ kế hoạch tốt gần nhất, không kẹp âm thầm.
   - (b) Dòng lệnh không thể bị mất bằng cách ẩn: vẫn được giữ chỗ, kèm thông báo `CommandInputKept`.
   - (c) Khi đang có lỗi vùng, đồng bộ kích thước panel và kẹp menu nổi không ghi lại layout.
   - (d) Khi dock phải nhường hoặc rail trái bị thu hẹp, ghi thông báo vào lịch sử CMD, và vào thanh trạng thái nếu đang không có lỗi.
   - A sentence that this is a rule for the workbench region plan only; it is not full docking, persistence or M01-UI acceptance, matching the scope sentence of the startup section. Do not add the status-bar truncation as a business rule (it is a presentation fix); do not add anything else.
2. Sync: after the repo edit, copy the same file over the installed copy (`Copy-Item` of that single file), then compare SHA-256 of both (`Get-FileHash`) and record that they match. If the installed directory is missing or differs in other files, report instead of overwriting other files. The repo copy is the source; only SKILL.md is synced. Note the sync requirement in the validation doc.
3. Also update the pending-consent text in `docs/openmatrix9-progress.json` (`behaviour_changes_pending_consent` and its last bullet) and in the validation doc so they say the change is now recorded with the consent date, and are not left saying "awaiting consent".

Verify: `Select-String` shows the new section in both copies; hashes match; the section lists exactly the four behaviours plus the 900 px yield; the file still parses as skill front matter (first lines unchanged).

## Item 5. L2-01-02 and L2-01-03 stay stubs, stated as BLOCKED_DEPENDENCY (MACHINE-CLOSABLE, docs only)

Files: `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md`, `docs/openmatrix9-progress.json` (the `m01-l3-01-01-002-...` entry), `.agents/tasks/m01-l3-002-docking/final-report-vi.md`, and the doc comment of `AttachedCmdFrame` in `ui.rs` only if the follow-up is not already stated there (read it first; add one sentence, no logic change).

1. Do not implement OM9-L2-01-02 (command dispatch / Undo) or OM9-L2-01-03 (document / scene). Keep `CommandInputHost`, `ViewportSceneHost`, and `command_input_ready` always true.
2. State in the ledger and report: status `BLOCKED_DEPENDENCY`; the exact follow-up: "When OM9-L2-01-02 lands, replace the constant-true `AttachedCmdFrame::command_input_ready` adapter in `ui.rs` with the real CMD host query, and add a frame test where the host reports not-ready so that `RegionError::CommandInputUnavailable` is exercised through the real frame; when OM9-L2-01-03 lands, replace the `Workspace` implementation of `ViewportSceneHost` with the real scene host and re-run `tests/workbench_regions.rs`." Make clear that until then `CommandInputUnavailable` is reachable only in unit and fixture tests.

Verify: `Select-String BLOCKED_DEPENDENCY` finds the follow-up in the three documents; `git diff` for `ui.rs` shows only the status-bar rect fields, zoom override and (optionally) one comment.

## Item 6. Legal / M00 approval of QymCAD reuse (HUMAN ONLY; PREPARED-AND-PENDING)

Files: none changed. Evidence added to the validation doc only.

1. Keep `legal_reuse_status = PENDING_HUMAN_M00_APPROVAL` in `source_crosswalk.json`, `ui-review.json` and the ledger. Do not change it.
2. Confirm and record, with the exact commands: `cargo tree -p om9-app` (and `cargo tree --workspace`) piped through `Select-String -Pattern qymcad -SimpleMatch -CaseSensitive:$false` returns no match; `Select-String -Path standalone\**\Cargo.toml,standalone\Cargo.toml -Pattern 'qymcad|QymCAD'` returns no match; `Select-String -Path standalone\crates\**\src\**\*.rs -Pattern 'qymcad'` lists only doc comments (list them); `git -C vendor-research/QymCAD status --porcelain` is empty and `rev-parse HEAD` still equals `746a17f1...` (read-only git commands). State that comments about the pinned SHA are not code reuse and that no upstream text was copied (as already recorded in `workbench_regions.rs`).
3. In the report: "Legal and M00 approval of QymCAD reuse remains a human decision; nothing was approved by the agent."

## Item 7. planning_state values (MACHINE-CLOSABLE; decision: leave unchanged)

Files: none changed (the plan tree is under `OM9_Milestones\`, a protected tree; any edit would also touch the user's staged renames).

1. Re-run the vocabulary count (read-only script over `l3.json`, `l4.json`, `L2.json` under `OM9_Milestones`), record the value sets: L3 `SPEC_ONLY_NOT_IMPLEMENTED` / `SPECIFIED_L4_PLANNED_NOT_IMPLEMENTED`, L4 `NOT_STARTED` / `SPEC_ONLY_NOT_EXECUTED`, L2 same as L3. Also read `execution_policy` in one `l4.json` to see whether another status field is used for execution results.
2. Decision and explanation to write in the report: the L3-002 files stay `SPEC_ONLY_NOT_IMPLEMENTED` and the four `l4.json` stay `NOT_STARTED`. Reasons: (a) the repo convention has no implemented-but-gated value, and `tools/merge_l4_overlay.py` is the only writer and moves values only between the two "not implemented" states; inventing one would change the vocabulary used by many validators (`tools/validate_*`); (b) the execution preconditions declared in the `l4.json` files (M00 pass, owner legal approval, dependencies pass) are not met: M00 NOT_PASS, legal pending, L2-01-02/03 unimplemented; (c) the implementation was done under the user's explicit instruction as a local standalone port, which the crosswalk records. If the user wants a new state name, that is a convention change for the user to decide.
3. Never write M01-UI, M00 or legal gates as passed.

Verify: `git status --short -- "OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L3-002 - Workbench Docking Regions"` shows no change made by this task (the task makes no edit there).

## Item 8. Ledger and reports (MACHINE-CLOSABLE)

Files: `docs/openmatrix9-progress.json`, `docs/validation/2026-10-16-m01-l3-002-workbench-docking-regions.md`, `.agents/tasks/m01-l3-002-docking/final-report-vi.md`, `.agents/tasks/m01-l3-002-docking/verification.md` (append a close-out section; do not rewrite the original run).

1. Reconcile with the real code and test evidence after FEAT-001 to FEAT-003: use the actual test counts from the final runs (the earlier 21 lib / 8 integration, plus the new tests), the actual exit codes and image paths. Do not copy numbers from this plan.
2. `progress.json`: in the `m01-l3-01-01-002-...` entry add `closeout` with date, per-item status, evidence paths, and update `not_yet_done` (remove the status-bar overlap, the "150% DPI not captured" statement becomes "simulated 150% captured; real DPI and second monitor human"), `behaviour_changes_pending_consent` (recorded), `next_action`. Keep the JSON valid (`python -c "import json;json.load(open('docs/openmatrix9-progress.json',encoding='utf-8'))"`). Do not remove history.
3. Validation doc (English, matching the file): add "Close-out" section with the done / human-only table and the commands run.
4. `final-report-vi.md` (Vietnamese): rewrite the "Chưa được xác minh" and "Thay đổi quy tắc nghiệp vụ" parts to the new state, add the table:

   | Việc | Trạng thái | Ai đóng |
   | --- | --- | --- |
   | Chữ thanh trạng thái chồng lên nhau ở 800x600 | Đã sửa, có test đỏ→xanh, ảnh mới đã xem | Máy |
   | Ma trận kích thước + 150% DPI giả lập | Đã chạy, nhãn SIMULATED | Máy (không thay thế ký của người) |
   | DPI thật, màn hình thứ hai, kéo cửa sổ thật | Chưa làm | Người (M01-UI) |
   | Kiểm toán nguồn công khai | Quét thu hẹp exit 0; toàn bộ vẫn exit 1 | Người quyết định đề xuất |
   | Ghi quy tắc vào skill | Đã ghi, đã sync bản cài | Máy |
   | L2-01-02 / L2-01-03 | BLOCKED_DEPENDENCY, thay stub khi module đó xong | Module khác |
   | Pháp lý / M00 | PENDING_HUMAN_M00_APPROVAL | Người |
   | planning_state | Giữ nguyên, có giải thích | Người nếu muốn đổi quy ước |

   Exact next actions for the user, in order: (1) run the shortened M01-UI checklist and sign `OM9-L4-01-01-002-05.ui-review.json`; (2) decide M00 legal approval of QymCAD reuse; (3) decide the audit scoping proposal; then continue with the next sibling, L3-003 (live drag preview).
5. Clean-up: no temporary files left outside `build\`; remove the scoped-audit control files; `git diff --cached --name-only` is empty (nothing staged), and say that nothing was committed.

Verify: JSON parses; the report states the items honestly; `Select-String "PASS"`-style grep is not used as proof.

## Final checks (convergence loop)

Run the full set once more and record: `cargo fmt --all -- --check`; `cargo check --workspace --all-targets`; `cargo build --workspace`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; the matrix runner; the scoped audit; `git diff --cached --name-only` (must be empty); `git status --short` shows no changes inside the protected private trees made by this task. The reviewer writes `closeout-review.json` with `verdict` last.
