# OM9-L3-02-01-009 — Mesh Vs NURBS Object Type Rules

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

is_exact_geometry is true for Point/Curve/Surface/Brep/Instance and false for Mesh/Cloud; require_exact_nurbs refuses a mesh with Unsupported(mesh is not exact NURBS).

## Owner and boundaries

- Scope: Mesh and exact box keep different kinds; no implicit mesh-to-NURBS conversion.
- Decision: ADD authored refusal; no conversion exists by design.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:756
- crates/om9-archive/tests/rhino_semantics_persistence.rs:165
