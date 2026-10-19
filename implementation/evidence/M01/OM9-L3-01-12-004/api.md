# OM9-L3-01-12-004 — Builder History Stub Persistence

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-01-12/api.md`. Traceability: `implementation/design/M01/OM9-L2-01-12/l4-matrix.json`.

## Contract

Feature {id, label, kind, outputs, revision}; Recipe {id, document, command, environment, inputs, parameters, outputs, policy, state, revision}. Plain decode preserves the records; Archive::restore marks unregistered modules Unresolved(diagnostic) and never invents recompute support.

## Owner and boundaries

- Scope: Feature and recipe placeholders persist; unknown modules are reported as Unresolved at runtime restoration, never invented.
- Decision: EXTEND existing Feature/Recipe; no second history schema.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Positive and negative fixtures

- `positive.json`, `negative.json` in this directory; expected values are authored, not copied from output.
- Automation: crates/om9-archive/tests/project_codec_acceptance.rs:435 history_and_feature_placeholders_reopen_without_recompute_support, crates/om9-archive/tests/document_identity_round_trip.rs:99 entity_feature_and_group_ids_survive_save_and_reopen, crates/om9-archive/tests/restore_modules.rs:126 missing_replay_module_keeps_geometry_but_marks_recipe_and_result_unresolved_stale, crates/om9-app/tests/file_lifecycle.rs:481 missing_replay_module_is_diagnostic_and_not_a_clean_restore, crates/om9-app/tests/cold_reopen.rs:259 persistent_report_hash_keeps_recipe_parameters_named_like_session_fields
