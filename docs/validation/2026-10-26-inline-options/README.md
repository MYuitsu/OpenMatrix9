# Inline command options + post-close re-verification (2026-10-26)

Scope: the Command Line shows every command's clickable options inside the prompt line, like
Polyline, instead of a floating chip row over the transcript. The Curve menu module
(`OM9-L2-03-16`, 58 rows) was re-verified in the same pass after the parallel sessions closed.

## Change

| File | Change |
| --- | --- |
| `crates/om9-app/src/session.rs` | `Controller::prompt_with_options()` composes `(<label> <label> …)` before the trailing colon when the prompt does not already name the options. `Controller::prompt()` is untouched so history echoes and existing assertions stay stable. |
| `crates/om9-app/src/ui.rs` | The three `transcript.set_prompt` call sites use `prompt_with_options()`. |
| `crates/om9-app/src/ui/files.rs` | The two file-dialog prompt refreshes use `prompt_with_options()`. |
| `crates/om9-app/tests/command_inline_options.rs` | 4 tests: 8 started providers compose the signature in order; Polyline is unchanged; the idle prompt stays `Command: `; the egui bar registers clickable `Degree`/`Cancel` rects inside the prompt. |

## Verification (all commands run with `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`)

- Evidence logs, current sources: **320 passed / 0 failed** in
  `implementation/evidence/AFTER/m03-l2-03-16-curve-menu-2026-10-26/`.
- Targeted re-run after the parallel sessions closed: **15 targets passed** (`command_inline_options`,
  `command_options_interface`, `command_sessions`, `command_state_machine`, `command_shortcuts`,
  `command_tokens`, `polyline_command`, `polyline_mode_regression`, `polyline_options_interface`,
  `polyline_shortcuts`, `curve_menu`, `crv2view`, `crv_ends`, `chrome_actions`, `surface_menu`).
- `rustfmt --check` (9 files): pass. `cargo clippy -- -D warnings`: pass, no allow needed.
- `tools/audit_m03_l2_03_16.py`: integrity 0 problems, 58 menu rows / 58 documented rows.
  `tools/audit_m03_l2_03_17.py`: 0. `tools/audit_m01_l2_03_14.py`: 0.
  `tools/public_source_audit.py`: 12674 files, 511 bindings, 510 SVG, 0 errors.

## Full-suite state: 12 failures in 6 targets, none from this change

`cargo test -p om9-app --no-fail-fast` → log kept as
`full-suite-2026-10-26.log` next to this record's evidence.

| Target | Failing tests | Owner of the red |
| --- | --- | --- |
| `file_widgets` | 7 (`close_save_failure…`, `shortcuts_preserve…`, `owned_overwrite_cancel…`, `held_opening_enter…`, `overwrite_replace_button…`, `failed_picker_path…`, `real_file_menu_cancel…`) | Uncommitted file-lifecycle staging: `stage_project_state()` writes four resources and bumps `document.revision` **before** `Controller::accept_file_path` validates the pending `FileToken`, so `validate_file_operation` returns `Stale("file operation document/context")`; the pre-staging in `MatrixApp::invoke`/`submit_draft` also runs before a Save As that is later cancelled, so the snapshot no longer matches. |
| `global_status_surface` | `x1_escape_still_cancels_a_running_command_under_a_failure` | New `feedback` branch `status.info("<prompt> — type a value…")` bypasses `StatusKind::is_failure`, so a failure is replaced by `Info`. |
| `shell_docking` | `invalid_command_stays_visible_in_the_status_surface_and_the_transcript` | Same branch: after the failure, starting Line turns the line into `Info`, and the final `complete_success()` then yields `Ready`. |
| `mesh_fill` | `a_mesh_object_fills_its_faces_after_the_grid` | Known pre-existing (ledger: "2 known unrelated failures (mesh_fill, ui_events)"; still expects the removed CPU painter). |
| `ui_events` | `construction_grid_colors_only_positive_axes_like_the_reference` | Known pre-existing, same ledger entry. |
| `solid_workflows` | `remaining_primitive_menu_prompts_produce_analytic_volumes` | Recorded baseline failure from 2026-10-25 (`docs/validation/2026-10-25-command-parameter-interface/baseline-solid-workflows.json`): menu Box followed by `2 3 4` keeps waiting for another point. |

Isolation proof: setting `prompt_with_options()` to return `self.prompt()` unchanged reproduces
all five non-file failures with identical assertions, so none of them is caused by the inline-option
change.

## Next actions

1. `feedback`: only show the running-command hint when `StatusKind::is_failure()` is false, so the
   documented "failure stays until acknowledged" rule holds (2 tests).
2. File lifecycle: stage project/display/F6/picture-frame state at the moment the archive is
   written (or re-key the pending token after staging), and stage only when a destination is
   confirmed (7 tests).
3. `mesh_fill`, `ui_events`, `solid_workflows`: keep as recorded known failures until their owners
   update the expectations.
