# OM9-L3-01-12-001 — Native Project Container Format

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-01-12/api.md`. Traceability: `implementation/design/M01/OM9-L2-01-12/l4-matrix.json`.

## Contract

Archive::encode(&Document, &Limits) -> Result<Vec<u8>>; Archive::decode(&[u8], &Limits) -> Result<Document>. Schema-4 header: magic, version, chunk count; entry 0 is the metadata chunk with its SHA-256; resource chunks carry typed ResourceId bytes and payload hashes.

## Owner and boundaries

- Scope: Versioned schema-4 container: magic, exact schema, chunk table, document metadata manifest and deterministic encoding.
- Decision: ADD authored OM9 container; pinned qcad project_file.rs read as architecture reference only, no code copied.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Positive and negative fixtures

- `positive.json`, `negative.json` in this directory; expected values are authored, not copied from output.
- Automation: crates/om9-archive/tests/project_codec_acceptance.rs:317 container_states_schema_document_id_layers_and_brep_payload, crates/om9-archive/tests/container_admission.rs:66 typed_document_roundtrip_preserves_master_identity_basis_attributes_and_channels, crates/om9-archive/tests/container_admission.rs:74 checksum_truncation_version_and_trailing_payload_corruption_reject
