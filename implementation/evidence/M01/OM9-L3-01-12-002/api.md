# OM9-L3-01-12-002 — Scene And Layer State Encoding

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-01-12/api.md`. Traceability: `implementation/design/M01/OM9-L2-01-12/l4-matrix.json`.

## Contract

Entity.attributes {layer, visible, hidden, selectable, locked, color}; Layer {name, parent, visible, selectable, locked, color, preset_slot}; Document.layer_order lists every layer exactly once and current_layer is a member.

## Owner and boundaries

- Scope: Entity/layer/visibility/lock/color/membership persistence with stable typed UUIDs and layer-order validation.
- Decision: REUSE_IN_PLACE document model; container extension only.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Positive and negative fixtures

- `positive.json`, `negative.json` in this directory; expected values are authored, not copied from output.
- Automation: crates/om9-archive/tests/project_codec_acceptance.rs:358 layer_visibility_and_lock_state_round_trip_with_stable_ids, crates/om9-archive/tests/layer_schema.rs:24 schema4_roundtrip_keeps_preset_hidden_explicit_permissions_and_uuid, crates/om9-app/tests/cold_layers.rs:15 independent_owned_process_restores_layer_current_and_explicit_object_preferences, crates/om9-app/tests/file_lifecycle.rs:531 save_keeps_hidden_locked_off_layers_and_explicit_preferences_in_complete_document
