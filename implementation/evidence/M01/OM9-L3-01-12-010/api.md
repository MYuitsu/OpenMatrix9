# OM9-L3-01-12-010 — Project Roundtrip Regression Fixtures

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-01-12/api.md`. Traceability: `implementation/design/M01/OM9-L2-01-12/l4-matrix.json`.

## Contract

A document with two layers, a 10x20x30 mm box, an instance, a group, a semantic feature, a history placeholder and an opaque jobbag reference must reopen byte-for-byte equal with stable IDs; a damaged bundle and a duplicate-identity bundle must fail without producing a document.

## Owner and boundaries

- Scope: Roundtrip fixture with two layers, 10x20x30 mm box (6000 mm3), instance, group, feature, history placeholder and opaque jobbag reference; damaged/duplicate bundles fail closed.
- Decision: REUSE codec and app file lifecycle; fixture authored for this acceptance.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Positive and negative fixtures

- `positive.json`, `negative.json` in this directory; expected values are authored, not copied from output.
- Automation: crates/om9-archive/tests/project_codec_acceptance.rs:511 roundtrip_fixture_keeps_two_layers_box_instance_group_and_jobbag_reference, crates/om9-app/tests/brep_selection.rs:103 real OCCT box cold archive volume 6000 mm3, crates/om9-archive/tests/document_identity_round_trip.rs:99 entity_feature_and_group_ids_survive_save_and_reopen, crates/om9-app/tests/cold_reopen.rs:67 corrupt_archive_causes_real_process_failure_without_success_json, crates/om9-app/tests/cold_layers.rs:15 independent_owned_process_restores_layer_current_and_explicit_object_preferences
