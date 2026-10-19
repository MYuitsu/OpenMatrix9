# OM9-L3-02-01-002 — NURBS Curve Control Points Knots Weights

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

inspect_nurbs_curve reports degree, pole/weight/knot counts, expected knot count (poles+degree+1), rationality, closedness and planarity; a short knot vector or a zero weight is refused by validate before OCCT.

## Owner and boundaries

- Scope: Degree/pole/weight/knot cardinality and rationality before any kernel conversion.
- Decision: REUSE existing NurbsCurve validation; ADD authored inspection.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:156
