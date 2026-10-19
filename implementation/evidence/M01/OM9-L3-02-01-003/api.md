# OM9-L3-02-01-003 — Open Closed And Planar Curve Rules

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

A curve is closed iff distance(sampled start, sampled end) <= the stated tolerance; planar iff every sample lies within tolerance of the Newell plane. Tolerance must be finite and in (0, 1e3] mm. Raw equality is never used.

## Owner and boundaries

- Scope: Closed/planar classification under an explicit tolerance policy.
- Decision: ADD authored policy over existing sampling.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:191
