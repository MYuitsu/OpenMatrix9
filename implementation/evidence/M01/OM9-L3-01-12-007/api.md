# OM9-L3-01-12-007 — Document Reload And Restoration

Status: `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING` · M01 · 2026-10-19 · target standalone Rust OM9.
Module record: `implementation/design/M01/OM9-L2-01-12/api.md`. Traceability: `implementation/design/M01/OM9-L2-01-12/l4-matrix.json`.

## Contract

Archive::restore/open_restored decode and establish a fresh owner session; recipes with no registered ReplayKey become Unresolved(diagnostic) and their results Stale; two restores of the same bytes have distinct sessions. A failed read returns no document and never mutates the caller's world.

## Owner and boundaries

- Scope: Reopen restores durable IDs/layers/geometry/instances and fresh runtime ownership; failed reads leave the open document untouched.
- Decision: REUSE Archive::open/decode; ADD authored restoration diagnostics.
- Owner crate(s): see the symbols in `source_crosswalk.json` and the module record.
- No upstream code, no second engine, no UI logic in this owner.

## Positive and negative fixtures

- `positive.json`, `negative.json` in this directory; expected values are authored, not copied from output.
- Automation: crates/om9-archive/tests/restore_modules.rs:75/87/126/150/175/198, crates/om9-archive/tests/document_snapshot.rs:48/84/103/146, crates/om9-archive/tests/project_codec_acceptance.rs:589 replay_key_matches_are_not_required_for_plain_document_reopen, crates/om9-app/tests/cold_reopen.rs:34 actual_standalone_process_creates_and_cold_reopens_owned_om9_document, crates/om9-app/tests/file_lifecycle.rs:425 open_candidate_must_fit_alongside_old_world_scene_and_undo
