# OM9-L3-02-01-007 — Polysurface And Closed Solid Semantics

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

classify_shell validates the body, refuses an open body that claims Solid and duplicate coincident faces, returns Solid for a closed shell, Surface for one open face and Polysurface for multi-face open shells. Geometric face-face self-intersection remains an OCCT boundary.

## Owner and boundaries

- Scope: Closed shell = Solid; open one-face = Surface; open multi-face = Polysurface; open/self-overlapping cannot claim solid.
- Decision: REUSE BrepBody topology; ADD authored classification.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:523
