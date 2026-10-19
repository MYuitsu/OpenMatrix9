# OM9-L3-01-12-009 — Export Mesh Units And Winding

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-01-12/api.md`. Traceability: `implementation/design/M01/OM9-L2-01-12/l4-matrix.json`.

## Contract

write_binary_stl(soup, options) writes an 80-byte header stating the millimetre scale, a u32 triangle count and 50-byte triangles with unit outward normals. scale is millimetres per model unit; flip_winding swaps triangle order; strict refuses open surfaces, degenerate triangles, bad indices, bad scale and non-finite coordinates.

## Owner and boundaries

- Scope: STL policy: explicit millimetre scale in the header, winding/normal agreement, watertight strict refusal.
- Decision: ADD authored STL policy; the document export command remains M07.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Positive and negative fixtures

- `positive.json`, `negative.json` in this directory; expected values are authored, not copied from output.
- Automation: crates/om9-exchange/src/stl.rs:258-352 seven unit tests, crates/om9-exchange/tests/stl_export_policy.rs:62 the_file_header_states_the_scale_so_units_are_never_implicit, crates/om9-exchange/tests/stl_export_policy.rs:93 a_closed_surface_is_watertight_and_its_normals_point_outwards, crates/om9-exchange/tests/stl_export_policy.rs:128 geometry_a_slicer_would_misread_is_refused_before_a_file_exists
