# OM9-L3-01-12-006 — Project Load Validation And Compatibility

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-01-12/api.md`. Traceability: `implementation/design/M01/OM9-L2-01-12/l4-matrix.json`.

## Contract

decode validates magic, exact schema, input budget, chunk table bounds/gaps/overlaps/trailing bytes, duplicate IDs, flags, per-chunk SHA-256, JSON admission, typed document invariants and legacy ownership before returning. Failure returns a typed error and publishes no document.

## Owner and boundaries

- Scope: Validate magic/schema/chunk table/checksums/JSON admission/typed invariants/legacy ownership before publishing a document.
- Decision: ADD authored admission; no ZIP reader, so there is no decompression or external-path surface.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Positive and negative fixtures

- `positive.json`, `negative.json` in this directory; expected values are authored, not copied from output.
- Automation: crates/om9-archive/tests/project_codec_acceptance.rs:472 damaged_container_is_refused_before_any_document_is_published, crates/om9-archive/tests/container_admission.rs:74 checksum_truncation_version_and_trailing_payload_corruption_reject, crates/om9-archive/tests/container_admission.rs:91 offset_overlap_duplicate_and_uuid_alias_refusals, crates/om9-archive/tests/resource_chunks.rs:70 missing_or_unreferenced_resource_chunks_reject_even_with_valid_container_checksums, crates/om9-app/tests/file_lifecycle.rs:320 open_invalid_or_missing_candidate_keeps_document_scene_command_and_redo, crates/om9-app/tests/cold_reopen.rs:132 actual_process_requires_explicit_v1_upgrade_and_preserves_durable_identity
