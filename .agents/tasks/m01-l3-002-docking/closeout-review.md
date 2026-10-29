# Review: OM9-L3-01-01-002 close-out (8 items)

Reviewer pass over the whole close-out change set. Read-only: no source was modified by the review.

**Verdict**: APPROVED

Every one of the 8 items is either closed with machine evidence or honestly labelled human-only/pending. The overlap at 800x600 is gone in the new screenshots, the RED/GREEN pair is real, the simulated matrix is labelled as such, and no human gate, and not the full public source audit, is claimed as passed. Three advisory findings below are not blocking.

## Findings (advisory, none blocking)

1. Runner check is vacuous if rects are missing (confidence: confirmed). `h:\OpenMatrix9\standalone\crates\om9-app\src\ui.rs` receipt computes `status_text_overlap` as `false` when either rect is `None`, and `h:\OpenMatrix9\tools\validate_m01_l3_002_ui_matrix.py` (lines 104, 125) only checks that boolean, not that `status_message_px` / `status_summary_px` are present. The frame tests (`status_bar_overlap.rs`, lines 40-42) `expect` both rects, so the overlap is genuinely proven there. Suggest the runner also require both px fields to be non-null.
2. Fix is local but larger than a one-liner (confidence: confirmed). `chrome.rs::status_bar_ui` was rewritten with three scoped sub-layouts (about 60 lines). It stays inside that one function, and `workbench_regions.rs` and `dock_layout.rs` are untouched in `git status`, so region-plan behaviour is unchanged. Accepted.
3. Known defects recorded, not fixed (confidence: confirmed from reports). Saving a document with a Line fails `Invalid('archive encoding verification')` in `om9-archive`, and the command panel leaves a 28 px gap below about 311 px height (not reachable, since the OS minimum is 480). Both are disclosed in `final-report-vi.md`. They are follow-ups outside this item.

## Item-by-item

1. Status-bar overlap: closed.
   - `closeout-01.red.log` shows exit 101 and 3 of 4 tests failing on the assertion at `status_bar_overlap.rs:44`, with no build error. `closeout-01.green.log` shows 4/4 passing.
   - I re-ran `cargo test -j 1 -p om9-app --test status_bar_overlap` (4 passed) and `--test workbench_regions_sizes` (10 passed).
   - I viewed `before-narrow/workspace.png`: "Ready: Ready" is printed over the summary text, so the defect is visible. I viewed `narrow/workspace.png` (800x600) and `matrix/min-720x480/workspace.png`: the message and the elided summary are separate and nothing is stacked. I viewed `normal/workspace.png` (1600x1000): the full summary is shown.
   - The `ui.rs` diff holds only the status rect fields, the audit-only zoom override, receipt fields and one BLOCKED_DEPENDENCY comment.
2. Matrix and checklist: closed for the machine part.
   - `matrix-summary.json` carries the SIMULATED/AUTOMATED label, `all_passed` true and 10 of 10 cases. `frame_test_only` lists the cases that have no real window.
   - `ui-review.json` parses. `reviewer.status` is `NOT_SIGNED` with null name and date. The gates are `m01_ui` OPEN, `m00` NOT_SIGNED and `legal_reuse` PENDING_HUMAN_M00_APPROVAL.
   - The checklist is cut to 5 items, each with `machine_proven`, `human_must_verify` and `reviewer_decision` null.
3. Public source audit: closed, honestly.
   - `tools/public_source_audit.py` has no git change.
   - `closeout-03.scoped-audit.log` records the full run as exit 1 with 11721 errors, all in private trees (0 elsewhere), and "NOT PASSED".
   - The scoped mirror run exits 0 with 557 files. The control run exits 1 with exactly `control.exe` and `control.png`. The re-run exits 0. The proposal is written and left for the user.
   - I re-ran only the scoped logic indirectly, via the log. I did not re-run the full audit.
4. Skill: closed. `SKILL.md` has a Vietnamese section "Vùng workbench docking (OM9-L3-01-01-002)" with a dated consent line (2026-10-16), the 900 px yield, the four behaviours and a scope sentence. SHA-256 of the repo copy and the installed copy are equal (`453705A0...F69F`), which I computed myself.
5. L2-01-02/03: closed (docs only). BLOCKED_DEPENDENCY follow-up is present in the validation doc, the final report, `progress.json` and the `AttachedCmdFrame` doc comment. No implementation was added.
6. Legal/M00: PENDING_HUMAN_M00_APPROVAL unchanged. The report states that no QymCAD dependency was found and no code was copied (the `cargo tree` and Cargo.toml claims are taken from the logs, see "Not verified").
7. planning_state: left unchanged, with explanation in the report and in `progress.json`. `git status` for the M01 milestone folder shows nothing, and there is no unstaged diff in `OM9_Milestones`.
8. Ledger and reports: closed. `progress.json`, `ui-review.json` and `matrix-summary.json` parse as JSON. `final-report-vi.md` has the done / human-only table and the three ordered next actions (sign M01-UI, decide QymCAD legal/M00, decide audit scoping). The validation doc has a Close-out section.

## Repository hygiene

- `git status` for `ref`, `manuals`, `QymCAD`, `vendor-research` and `om9-window.png` is empty.
- `git diff --cached` has 5023 entries. All are the user's `OM9_Milestones` renames plus four unrelated `tools/*.py` (`apply_implementation_samples`, `record_m00_l2_01_status`, `sync_qymcad_revision`, `verify_m08_l4_merge`). None of this task's files is staged. HEAD is `b5a15dae`, which matches origin, so there is no new commit.
- Grep of the final report, the validation doc and the close-out verification finds no claim that the human gates or the full audit passed.
- `implementation/tools/om9_shorten_milestone_paths.py` and `record_path_shortening_progress.py` are modified but are not from this task (as `closeout-verification.md` also states).

## Not verified

- I did not re-run the whole cargo suite (581 passed, 24 ignored is taken from the logs), the matrix runner, the full public audit, `cargo tree`/QymCAD greps, or the planning_state counts.
- I did not open the screenshots for boundary-899, boundary-914/915, ultrawide, tall-narrow or the dpi150 cases. The FEAT notes say the agent opened several of them.
- I cannot tell from git alone who staged the four `tools/*.py` files. I inferred they are not from this task because they are unrelated and match the earlier note.
- Real OS DPI, the second monitor and live drag: human-only, not machine-verified, and labelled so.
