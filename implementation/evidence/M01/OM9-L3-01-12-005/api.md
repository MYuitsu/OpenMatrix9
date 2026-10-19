# OM9-L3-01-12-005 — Atomic Save And Write Failure Recovery

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-01-12/api.md`. Traceability: `implementation/design/M01/OM9-L2-01-12/l4-matrix.json`.

## Contract

Archive::save(path, document, limits, cancel): encode, decode-equality check before file creation, readonly rejection, unique staging create_new, write/flush/sync_all, staged re-read equality, atomic rename/VerifiedFile publish. Failure or cancellation removes only the staging name.

## Owner and boundaries

- Scope: Encode/verify, unique staging file, fsync, re-read verification and atomic publication; every failure keeps the last good file.
- Decision: ADD authored atomic save; reuse om9-native-owner VerifiedFile on Windows.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Positive and negative fixtures

- `positive.json`, `negative.json` in this directory; expected values are authored, not copied from output.
- Automation: crates/om9-archive/tests/atomic_save.rs:24 successful_same_volume_replacement_reopens_current_document, crates/om9-archive/tests/atomic_save.rs:35 canceled_and_invalid_save_preserve_good_destination_and_remove_staging, crates/om9-archive/tests/atomic_save.rs:56 destination_permission_failure_preserves_existing_file_and_removes_staging, crates/om9-archive/tests/atomic_save.rs:79 validation_copy_shares_point_budget_with_live_model_before_file_mutation, crates/om9-app/tests/file_lifecycle.rs:197 save_as_cancel_failure_and_unsupported_format_preserve_identity_and_target, crates/om9-app/tests/file_lifecycle.rs:644 failing_save_to_active_path_does_not_turn_into_a_save_as_picker
