# OM9-L3-02-01-005 — Trimmed Vs Underlying Surface Model

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

TrimmedSurface {underlying, outer, inners, reversed}: loops need >=3 finite points inside the domain, non-zero area, no repeated consecutive point, inner points inside outer. to_untrimmed() always fails. The slot om9.trim.surface is validated by Document::validate.

## Owner and boundaries

- Scope: Underlying NURBS surface plus outer/inner UV trim loops, persisted and never promoted to an untrimmed surface.
- Decision: ADD authored trim record; REUSE entity extension persistence.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:328
- crates/om9-archive/tests/rhino_semantics_persistence.rs:62
- crates/om9-archive/tests/rhino_semantics_persistence.rs:139
