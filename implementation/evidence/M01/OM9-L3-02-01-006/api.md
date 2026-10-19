# OM9-L3-02-01-006 — Surface Edge And Isocurve Semantics

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

DisplayIsocurve is display-only, not a model edge, not pickable, and to_geometry() always returns Unsupported(isocurve is display-only). BoundaryEdge requires non-nil object/face/edge UUIDs and is model edge + pickable.

## Owner and boundaries

- Scope: Boundary edges are pickable topology; exact isocurves are display-only and never persistent geometry.
- Decision: REUSE surface_kernel::isocurve; ADD authored display/pickable distinction.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:391
