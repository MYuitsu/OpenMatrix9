# Close-out verification, iteration 1: OM9-L3-01-01-002

Evidence only. This does not claim M01-UI, M00 or legal gates passed. `reviewer.status` stays `NOT_SIGNED`. The full `public_source_audit.py` has NOT passed. Nothing was committed or staged by this task.

## Cross-FEAT cargo set (cwd `h:\OpenMatrix9\standalone`)

| Command | Exit |
| --- | --- |
| `cargo fmt --all -- --check` | 0 |
| `cargo check --workspace --all-targets` | 0 |
| `cargo build --workspace` | 0 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 0 |
| `cargo test --workspace` | 0 |

`cargo test --workspace`: 581 passed, 0 failed, 24 ignored (the ignored ones are native-gated tests in `om9-exchange`: retained_native_import 8, retained_native_inputs 4, retained_native_write 4, staged_export 8). Targets of this item: `om9-app` lib 62, `workbench_regions` 8, `status_bar_overlap` 4 (new), `workbench_regions_sizes` 10 (new), `shell_docking` 10.

No seam fix was needed. No source edit was made in this iteration. The three FEATs compose: `ui.rs` holds the three status rects (message, summary, dismiss), the `OM9_UI_AUDIT_ZOOM` parser and the receipt fields; `chrome.rs::status_bar_ui` fills the rects; both new test files read the same fields. `ui.rs` diff contains only the status rect fields, the zoom override with its unit test, receipt fields and one BLOCKED_DEPENDENCY comment.

## Real-window matrix

`python tools\validate_m01_l3_002_ui_matrix.py --output build\standalone-validation\ui-artifacts\2026-10-16-m01-l3-002-closeout\matrix`: exit 0, 10 of 10 cases PASS (min-720x480, boundary-899x700, boundary-914x700, boundary-915x700, ultrawide-2560x600, tall-narrow-720x1300, normal-1600x1000, narrow-800x600, dpi150-1600x1000, dpi150-1280x800), `all_passed` true. The two dpi150 cases are SIMULATED (egui zoom factor, point space), not OS per-monitor DPI.

## The 8 items against the request

| # | Item | Result | Evidence |
| --- | --- | --- | --- |
| 1 | Status-bar overlap | Closed (machine). RED `closeout-01.red.log` exit 101, 3 of 4 tests fail; GREEN `closeout-01.green.log` 4 of 4 pass. Before image `before-narrow/workspace.png` and after images `narrow/`, `normal/` were viewed in the earlier run. | `implementation/evidence/M01/OM9-L3-01-01-002/`, `build/standalone-validation/ui-artifacts/2026-10-16-m01-l3-002-closeout/` |
| 2 | Size matrix + SIMULATED labels | Closed for the machine part; real DPI, second monitor and live drag stay human. `ui-review.json` carries `SIMULATED` labels and a shortened checklist with `machine_proven` / `human_must_verify`. | `workbench_regions_sizes.rs`, matrix summary, `OM9-L4-01-01-002-05.ui-review.json` |
| 3 | Scoped audit | Log has the exact commands: full run exit 1 with 11721 errors (0 outside private trees), scoped mirror exit 0 (557 files), control run exit 1 with exactly 2 errors, re-run exit 0. Proposal not implemented. | `closeout-03.scoped-audit.log`; re-run below |
| 4 | Skill section | Present at SKILL.md lines 22-28 in both copies. SHA-256 of repo copy and `C:\Users\nguye\.codex\skills\...` copy are equal (`453705A0A7EC1791...`). | `Get-FileHash` |
| 5 | BLOCKED_DEPENDENCY follow-up | Present in the validation doc (4), `final-report-vi.md` (4), `progress.json` (3), `verification.md` (2), and the `AttachedCmdFrame` comment in `ui.rs`. | grep counts |
| 6 | QymCAD confirmation | Report states no `qymcad` in `cargo tree`, `Cargo.toml`/`Cargo.lock`; only comments in four `.rs` files; vendor SHA `746a17f1...` clean. PENDING_HUMAN_M00_APPROVAL kept. | `final-report-vi.md` item 6 |
| 7 | planning_state | Left unchanged, explained in the validation doc, `final-report-vi.md` and `progress.json`. `OM9_Milestones` untouched by this task. | grep counts |
| 8 | Ledger and report | `docs/openmatrix9-progress.json` and `ui-review.json` parse as JSON. `behaviour_changes_pending_consent` is `[]`. The report has the done / human-only table and the three ordered next actions: (1) run the shortened M01-UI checklist and sign, (2) decide QymCAD legal/M00 approval, (3) decide the audit scoping proposal; then L3-003. | `final-report-vi.md` lines 63-68 |

## Other checks

- Scoped audit re-run in this iteration (mirror refreshed with this file and FEAT-003.json): `python tools\public_source_audit.py --project-root build\scoped-audit\2026-10-16-l3-002` exit 0, `{"source_files": 558, "errors": []}`. This is the mirror only; the full audit still exits 1.
- Staging: the index holds 5023 entries that this task did not create (user-staged `OM9_Milestones` renames, plus `tools/apply_implementation_samples.py`, `tools/record_m00_l2_01_status.py`, `tools/sync_qymcad_revision.py`, `tools/verify_m08_l4_merge.py`). This task staged nothing. `git status --short -- ref manuals QymCAD vendor-research om9-window.png` is empty.
- Unrelated modified files not touched by this task: `implementation/tools/om9_shorten_milestone_paths.py`, `implementation/tools/record_path_shortening_progress.py`.
- The temporary mirror helper `build\scoped-audit-mirror.tmp.ps1` no longer exists.
- Not verified here: whether the images were re-viewed in this iteration (I did not reopen them; the viewing was recorded by the earlier step), real DPI, second monitor, live drag.
