# OM9-L3-02-01-004 — Surface UV Domain And Seam Rules

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

Domains are returned verbatim; a periodic flag must agree with the knot vector shape; periodic axes report seam = domain start; geometrically closed non-periodic axes also report a seam; require_orientation refuses swapped U/V periodicity or V orientation.

## Owner and boundaries

- Scope: U/V domains, periodicity, seam parameters and V orientation.
- Decision: ADD authored seam/orientation rules.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:242
