# `OM9-L2-01-12` — execution record (2026-10-19)

**Status:** `AUTOMATED_TARGET_VALIDATED_REVIEW_PENDING`
**Target:** standalone Rust OM9 · **Workspace:** `H:/OpenMatrix9` · **Head:** `b5a15dae9ece3ba4a01e7c26fe60b051c9837e2a`
**Active scope:** 9 L3 × 5 L4 = 45 tasks. `OM9-L3-01-12-008` is `DEFERRED_TO_M07_NOT_INCLUDED_IN_M01` and has no L4 records.

Machine-readable records: `crosswalk.json` (L3 audit), `l4-matrix.json` (45 L4 rows), `api.md` (contract),
`oracles.json` (positive/negative fixtures per L3).

## 1. What already existed and what this execution did

The working tree already contained the owner implementation: `om9-archive` (schema-4 container,
admission, atomic save, snapshot, restoration, migration) and `om9-exchange::stl`. This execution
did **not** rewrite that code. It:

1. audited every L3 against the L4 purpose/positive/negative oracles in the milestone records;
2. added the missing module-level acceptance tests that prove the oracles end to end
   (`crates/om9-archive/tests/project_codec_acceptance.rs`, 7 tests;
   `crates/om9-exchange/tests/stl_export_policy.rs`, 3 tests);
3. fixed two stale budget assumptions in existing tests that failed on the current model size
   (`restore_modules.rs`, `file_lifecycle.rs`) without changing product behaviour;
4. ran the focused suites and the whole workspace, and captured this receipt.

No upstream QymCAD code was copied. The pinned paths remain architecture references only.

## 2. L4 loop per L3

| L3 | -01 SOURCE_AUDIT | -02 CONTRACT_TEST | -03 IMPLEMENT | -04 INTEGRATE | -05 ACCEPTANCE |
| --- | --- | --- | --- | --- | --- |
| 001 Native Project Container Format | `container.rs:14-15,44-120,156-328`; ADD authored schema; no copy | `project_codec_acceptance.rs:317`, `container_admission.rs:66/74` | `container.rs:44 encode`, `:156 decode` | `session/files.rs:375`; `cold_reopen.rs:34` | archive 52 / workspace |
| 002 Scene And Layer State Encoding | `model.rs:13-24,289-300,330-385`; REUSE model | `layer_schema.rs:24`, `project_codec_acceptance.rs:358` | `model.rs:709-714,1132-1142,505-546` | `cold_layers.rs:15`, `file_lifecycle.rs:531` | archive 52 / app 38 |
| 003 B-Rep Mesh And Instance Encoding | `geometry.rs:490-499`, `brep.rs:14-39`, `model.rs:632-650`; ADD snapshot | `project_codec_acceptance.rs:398`, `brep_selection.rs:103` | `brep.rs:43-55,59-128`, `geometry.rs:570-604` | `brep_selection.rs:103` fresh process + 6000 mm3 | archive 52 / brep 5 |
| 004 Builder History Stub Persistence | `model.rs:199-245`, `history.rs:110-149`, `restore.rs:147-198`; EXTEND | `project_codec_acceptance.rs:435`, `document_identity_round_trip.rs:99` | `history.rs:350-364`, `restore.rs:171-191` | `file_lifecycle.rs:481`, `cold_reopen.rs:259` | archive 52 / cold 9 |
| 005 Atomic Save And Write Failure Recovery | `atomic_file.rs:26-141`, `verified_file.rs:13-52`; ADD + REUSE | `atomic_save.rs:24/35/56/79` | `atomic_file.rs:60-75,91-110,112-137` | `session/files.rs:375`, `file_lifecycle.rs:197/644` | atomic 4 / app 20 |
| 006 Project Load Validation And Compatibility | `container.rs:168-326`, `json_admission.rs`; ADD | `container_admission.rs:74-191`, `resource_chunks.rs:70` | `container.rs:180-240,241-296,310-326` | `file_lifecycle.rs:320`, `cold_reopen.rs:132` | archive 52 / app 20 |
| 007 Document Reload And Restoration | `lib.rs:174-207`, `restore.rs:13-198`, `model.rs:463-504`; REUSE + ADD | `restore_modules.rs:75-198`, `document_snapshot.rs:48-146` | `restore.rs:62-123,147-198` | `cold_reopen.rs:34`, `cold_layers.rs:15`, `file_lifecycle.rs:425` | cold 9+4 / app 20 |
| 009 Export Mesh Units And Winding | `stl.rs:84-97`, `lib.rs:17`; ADD policy | `stl.rs:258-352`, `stl_export_policy.rs:62/93/128` | `stl.rs:123-137,141-210` | public-API integration test | exchange 97 / clippy |
| 010 Project Roundtrip Regression Fixtures | codec + app file lifecycle; REUSE | `project_codec_acceptance.rs:511`, `brep_selection.rs:103` | codec (same as 001/005/007) | `cold_layers.rs:15`, `cold_reopen.rs:432`, `document_identity_round_trip.rs:99` | archive 52 / workspace |
| 008 STL Tessellation Export Codec | deferred: `L3-008/l3.json:8-10` | — | — | — | M07 |

Every row's status is `VERIFIED_WITH_EVIDENCE` in `l4-matrix.json`; the contract-test rows carry a
`red_receipt: RETROFIT` note because the original RED run predates this audit and is not retained.

## 3. Commands and observed results

All commands used `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`. Logs are in
`build/standalone-validation/ui-artifacts/m01-l2-01-12-20261019/`.

| # | Command | Result |
| --- | --- | --- |
| 1 | `cargo test -p om9-archive -j 2` | exit 0, **52 passed**, 0 failed |
| 2 | `cargo test -p om9-exchange -j 2` | exit 0, **97 passed**, 0 failed |
| 3 | `cargo test -p om9-core -j 2` | exit 0, **374 passed**, 0 failed |
| 4 | `cargo test -p om9-app --features gpu-sphere --test file_lifecycle --test cold_reopen --test cold_layers --test brep_selection -j 2` | exit 0, **38 passed** (file 20, cold_reopen 9, cold_layers 4, brep_selection 5) |
| 5 | `cargo clippy -p om9-archive -p om9-exchange --all-targets -j 2 -- -D warnings` | exit 0 |
| 6 | `rustfmt --check --edition 2021` on the four touched test files | exit 0 |
| 7 | `cargo test --workspace --no-fail-fast -j 2` | exit 101: **1,184 passed / 168 suites**, 2 known unrelated failures (`mesh_fill`, `ui_events` still expect the removed CPU painter; recorded in `docs/openmatrix9-progress.json:12939`). The third failure recorded there (file_lifecycle budget) was fixed by this execution. |
| 8 | `python tools/public_source_audit.py` | exit 1 (pre-existing repo-wide finding: 6,136 unreviewed rasters and 292 private/binary artifacts; this change adds no external source or asset) |

New acceptance tests:

- `crates/om9-archive/tests/project_codec_acceptance.rs` (7):
  `container_states_schema_document_id_layers_and_brep_payload` (L3-001),
  `layer_visibility_and_lock_state_round_trip_with_stable_ids` (L3-002),
  `brep_source_and_display_round_trip_without_mesh_substitution` (L3-003),
  `history_and_feature_placeholders_reopen_without_recompute_support` (L3-004),
  `damaged_container_is_refused_before_any_document_is_published` (L3-006),
  `roundtrip_fixture_keeps_two_layers_box_instance_group_and_jobbag_reference` (L3-010),
  `replay_key_matches_are_not_required_for_plain_document_reopen` (L3-007 boundary).
- `crates/om9-exchange/tests/stl_export_policy.rs` (3): scale header, watertight/outward winding,
  strict refusal.

## 4. Adjustments made during acceptance

1. `crates/om9-archive/tests/restore_modules.rs`
   `restoration_keeps_long_diagnostics_owned_at_the_source_parser_admission_boundary` calibrated its
   budget from the *decode* minimum and then assumed restoration would fit. Restoration keeps the
   input plus the long recipe-state and the returned diagnostic copy, so it legitimately needs more
   headroom. The test now calibrates the *restore* minimum and still asserts that one byte less is
   refused. Product code unchanged.
2. `crates/om9-app/tests/file_lifecycle.rs`
   `open_candidate_must_fit_alongside_old_world_scene_and_undo` hard-coded 10,500,000 bytes. The
   1,000-entity old document now needs 10,620,014 bytes inside `Controller` (measured by bisection
   during the audit). The budget is now 11,500,000 bytes with a comment; the old world and the
   candidate alone fit, both together still do not. Product code unchanged.

Both were test-calibration defects, not runtime defects: the product already returned the correct
`Budget` error and preserved state.

## 5. Explicit deviations / residuals

- `OM9-L3-01-12-008` (STL Tessellation Export Codec) is deferred to M07 by its own record and has no
  M01 L4 tasks. The STL writer policy that exists is recorded under L3-009; no tessellation or export
  command is claimed here.
- The L3-010 fixture's "jobbag reference" is an embedded opaque resource
  (`ResourceKind::OpaqueBinary`, payload `jobbag/A-102.om9`) because the jobbag object model is owned
  by M01 L2-03-15, not by this codec. Its bytes are proven equal after reopen; no jobbag semantics are
  invented.
- The container is not ZIP. ZIP-bomb and path-traversal oracles are therefore replaced by the real
  bounded-allocation/no-external-path oracles (`json_admission`, `resource_chunks`, container bounds).
- Real B-Rep volume is proven by `om9-app/tests/brep_selection.rs` through the existing OCCT adapter;
  the archive-level fixture proves display volume and byte equality so it can run without OCCT.
- No external reviewer has signed this record. `human_approval` remains `null`.
- The global M00 execution gate is **NOT READY** (`python tools/validate_m00_execution.py
  --require-ready` exit 1; the missing items are pre-existing M00 L4 records). This execution imported
  no QymCAD code and no new upstream source, so the module's automated verification is unaffected;
  the M00 gate remains a release/legal residual, not a claim made here.

## 6. Evidence pointers

- Design/audit: `implementation/design/M01/OM9-L2-01-12/`
- Checks receipt: `implementation/evidence/M01/2026-10-19-l2-01-12-checks.json`
- Logs and raw checks: `build/standalone-validation/ui-artifacts/m01-l2-01-12-20261019/`
- Milestone runtime records: `OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project
  Codec Stl Export/L2.json` and each `L3-*/l3.json` `runtime_implementation`
- Live ledger: `docs/openmatrix9-progress.json` (`m01-l2-01-12-native-project-codec-2026-10-19`)
