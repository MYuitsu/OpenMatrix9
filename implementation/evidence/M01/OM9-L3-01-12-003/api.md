# OM9-L3-01-12-003 — B-Rep Mesh And Instance Encoding

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-01-12/api.md`. Traceability: `implementation/design/M01/OM9-L2-01-12/l4-matrix.json`.

## Contract

Geometry::Brep(Brep {source, source_sha256, display_sha256, display}) keeps the exact native source and a disposable display snapshot; Geometry::Instance(DefinitionId) is an occurrence with a placement. Reopening never downgrades a B-Rep to its display mesh.

## Owner and boundaries

- Scope: B-Rep native source plus display/topology snapshot, mesh/cloud channels and definition/instance links; no mesh substitution.
- Decision: ADD authored rust-owned B-Rep snapshot; OCCT only through the om9-kernel adapter for measurement.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Positive and negative fixtures

- `positive.json`, `negative.json` in this directory; expected values are authored, not copied from output.
- Automation: crates/om9-archive/tests/project_codec_acceptance.rs:398 brep_source_and_display_round_trip_without_mesh_substitution, crates/om9-archive/tests/project_codec_acceptance.rs:511 roundtrip_fixture_keeps_two_layers_box_instance_group_and_jobbag_reference, crates/om9-app/tests/brep_selection.rs:103 source_and_topology_survive_cold_archive_and_transform_undo_without_reviving_old_picks, crates/om9-app/tests/brep_selection.rs:157 source_binding_corrupt_cache_invalid_native_source_and_budgets_are_rejected, crates/om9-core/tests/document_graph.rs:613 l3_008_two_instances_share_one_definition_without_cloning_topology
