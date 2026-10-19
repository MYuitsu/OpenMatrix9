# OM9-L3-02-01-008 — Extrusion Vs Full B-Rep Representations

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-02-01/api.md`. Traceability: `implementation/design/M01/OM9-L2-02-01/l4-matrix.json`.

## Contract

Extrusion {profile, direction, height, capped}: closed planar polyline profile, finite non-zero direction/height, capped. to_brep_body builds a closed portable body (2n vertices, n+2 faces, one closed shell, kind Solid); convert returns source + body and keeps provenance.

## Owner and boundaries

- Scope: Lightweight closed planar polyline extrusion, on-demand portable B-Rep conversion and provenance.
- Decision: ADD authored primitive; REUSE brep_kernel::BrepBody.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Automated proof

- crates/om9-core/tests/rhino_semantics.rs:611
- crates/om9-archive/tests/rhino_semantics_persistence.rs:101
