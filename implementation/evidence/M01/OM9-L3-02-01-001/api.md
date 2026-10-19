# OM9-L3-02-01-001 — Point And Polyline Semantic Types

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

geometry_semantic_kind/curve_semantic_kind distinguish Point, Line, Circle, Polyline and Polycurve; connected line segments form a Polyline, any other Polycurve stays a Polycurve; endpoints are sampled first/last points; an exact zero-length line and a curve at or below tolerance are refused.

## Owner and boundaries

- Scope: Distinct Point/Line/Circle/Polyline/Polycurve kinds, endpoints and degenerate-line refusal.
- Decision: EXTEND existing geometry; ADD authored classification.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:83
- crates/om9-core/tests/rhino_semantics.rs:134
