# OM9-L3-02-01-010 — Instance And Block Semantic Type

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

block_instances lists every occurrence of a definition in stable object-id order; each has a distinct ObjectId and Transform. set_instance_transform edits only the occurrence placement and bumps its revision/generation; the shared definition is untouched.

## Owner and boundaries

- Scope: Occurrences share one definition but keep distinct identity and transform.
- Decision: REUSE Definition/Instance model; ADD authored semantics.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:774
