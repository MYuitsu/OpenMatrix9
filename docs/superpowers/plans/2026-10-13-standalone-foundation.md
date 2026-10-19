# RCORE-00 standalone foundation implementation plan

> **For agentic workers:** Use superpowers:executing-plans to implement this plan task by task, with test-driven-development and one fresh-context whole-change review. Existing user authorization is execution of the supplied RCORE-00 specification. Do not request duplicate design approval or commit without a user request.

**Goal:** Implement and verify the standalone OM9 foundation required by RCORE-00, retaining Matrix9 behavior and explicit capability gates.

**Architecture:** A separate `standalone/` Rust workspace owns persistent model, atomic world snapshots, recipe graph, sessions, scene and archives. Operation adapters supply only selected geometry/exchange capabilities. UI events share the document dispatcher.

**Tech stack:** Rust 2024, typed UUIDs, serde, SHA-256, selected Rust UI/render backend, optional isolated native operation adapters.

**Spec:** `OpenMatrix9_Codex_Spec_v2/specs/00-rhino-core/00-qymcad-foundation-migration.md`; implementation choices in `docs/superpowers/specs/2026-10-13-standalone-foundation-design.md`.

## Global constraints

- Rust owns document/model/session/history/Undo/transactions/UI/scene; target does not link/import/launch FreeCAD or QymCAD desktop.
- Canonical lengths mm, angles radians; placement applied exactly once; independent representation masters.
- Default workers `max(1,floor(0.60*N))`; checked aggregate admission before allocation.
- Menu/CMD/F6 use exact feature IDs and one dispatcher; unverified capabilities visible-disabled.
- Matrix9 layout/workflow/defaults are fixed by MATRIX9_UI_CONTRACT; unknowns stay explicit.
- QymCAD pin `09a7f09ebeb87fab7c43e26774f7eb0eb88907dd`; provenance required for any copied routine.
- Preserve unrelated checkout edits, v1 bytes and historical outcomes. No automatic commit/publication or host rebuild.

## Review focus

1. Corrupt archives whose declared sizes, overlapping chunks or duplicate identities bypass admission: reject before allocation/public state (Task 5).
2. Reused labels/IDs after deletion, revision overflow and canceled late jobs: reject stale publication without touching Redo (Tasks 1, 2, 4).
3. Failure after preparing some cross-document participants and caller nested booking: every participant and caller scope remain exactly unchanged (Task 2).
4. Hidden versus locked versus unselectable objects and mirrored shared instances: use separate permissions and apply world transforms once (Tasks 1, 6).
5. UI Enter while typing a numeric field, navigation mid-command and stale F6/pick state: preserve session phase and route actual intent consistently (Task 7).

## Task 1: Typed model, geometry admission and units

**Files:** `standalone/Cargo.toml`, `standalone/crates/om9-core/{Cargo.toml,src/lib.rs,src/error.rs,src/identity.rs,src/geometry.rs,src/budget.rs,src/model.rs}`, `standalone/crates/om9-core/tests/model_admission.rs`.

**Interfaces:** Produces `Document::new`, `Entity::new`, `Geometry::validate`, `Document::validate`, `SourceStamp`, `Limits`, `Usage`, `Transform`, typed IDs. Geometry and persistent state serialize through serde. Fields exposed only where validation at the boundary is guaranteed.

- [ ] Write tests: reject NaN/invalid rational weights/knots/domain/grid overflow/index bounds; distinguish CAD/mesh/cloud; mirrored nested instance world placement; unknown/empty/finite bounds; UUID label reuse does not satisfy old stamp; units preserve UV/weights/directions.
- [ ] Run `rtk cargo test --manifest-path standalone/Cargo.toml -p om9-core --test model_admission`; observe missing implementation/admission failures.
- [ ] Implement typed owned masters, checked budgets, units/transforms, validated document/graph and captured source stamps.
- [ ] Run complete `om9-core` suite; expected no failures. Record exact coverage and unsupported families.

## Task 2: Global transactions, scopes, Undo/Redo

**Files:** `om9-core/src/{transaction.rs,scope.rs}`, `om9-core/tests/{atomic_transactions.rs,transaction_scopes.rs}` under Task 1 crate.

**Interfaces:** Consumes validated `Document`, `SourceStamp`, `Limits`; produces `World`, `PreparedTransaction`, `TransactionCoordinator::{prepare,commit,undo,redo}`, typed mutation batches and caller/savepoint state. Preparation is private; public snapshots swap only after acceptance.

- [ ] Write rejection tests asserting whole world and prior Redo equality immediately for invalid/stale/cancel/overflow/reentrant/closed owner/participant failures.
- [ ] Add plain/booked/active/nested/recording-disabled/empty transaction tests, staged merge/first-write/savepoints, cross-document move and disposal callback failure cases.
- [ ] Run focused suites and observe missing coordinator behavior.
- [ ] Implement global snapshots, checked identity/context ownership, scopes, one acceptance and one Undo group. Keep fallible preparation before atomic publication; postacceptance cleanup diagnostics separate.
- [ ] Run full crate suite and real Redo execution after every rejection mode; expected original state restored.

## Task 3: Source bindings, prepared results and History

**Files:** `om9-core/src/{history.rs,request.rs,result.rs}`, `om9-core/tests/{source_binding.rs,history_replay.rs}`.

**Interfaces:** Consumes `SourceStamp` and coordinator; produces `SourceBinding`, `PreparedResult`, `RecipeGraph`, evaluation states and admission of ordered roles/subsets/parameters.

- [ ] Write real-source tests for wrong document/generation/role, duplicate/overlapping subsets, units/context change, current output/report stamps and stale auxiliary results.
- [ ] Write independent D0–D3 analytic/rational/domain oracles for admitted families, chosen-contact versus probe parameters, graph cycle/missing source/replay-module cases.
- [ ] Observe focused suite failures; implement only validated families, source-resolved preparation, graph/replay and atomic result updates. Unsupported replay remains unresolved.
- [ ] Verify successful replay and rejected replay preserve source/result consistency and Undo/Redo.

## Task 4: Capability registry, aggregate budgets and jobs

**Files:** `om9-core/src/{capability.rs,jobs.rs}`, `om9-core/tests/{capability_gates.rs,job_lifecycle.rs}`; `standalone/provenance.json`.

**Interfaces:** Consumes model/usage/stamps; produces `CapabilityRegistry`, `OperationReceipt`, `JobToken`, owned job snapshots and cancellation/epoch validation. Backend receipts name representation/options/toolkits/evidence.

- [ ] Write tests that source-only/partial evidence cannot enable an operation; CPU floor60%, tasks/RAM reductions, overflow/peak-copy limits and late/canceled/closed jobs reject.
- [ ] Observe failures, implement admission/registry/scheduler and explicit native owner/concurrency policy.
- [ ] Verify no partial document publication and bounded cancellation with actual owned worker inputs.

## Task 5: `.om9` archives, retained 3DM and clipboard

**Files:** `standalone/crates/om9-archive/{Cargo.toml,src/lib.rs,src/container.rs,src/atomic_file.rs,src/clipboard.rs}`, `tests/{container_admission.rs,cold_reopen.rs,atomic_save.rs}`; operation-scoped native exchange sources/receipts selected during implementation.

**Interfaces:** Consumes validated document/world/History and capability receipts; produces `Archive::{encode,decode,save,open}`, explicit schema migration and owned clipboard payload. Cold-reopen child uses actual standalone executable.

- [ ] Read the exact RCORE-10 archive/3DM contracts before adapter selection. Write corrupt sizes/overlaps/checksums/duplicate IDs/missing resource/backend/schema tests and current-state tombstone/shared-definition fixtures.
- [ ] Observe failures; implement bounded checksummed versioned container, canonical metadata, explicit migration and Windows same-volume atomic replacement.
- [ ] Build selected openNURBS adapter with no host dependencies, native ownership/exception/budget receipt; exercise structural type/field preservation and reject unsupported edits.
- [ ] Verify disk/permission/format failure preserves destination, fresh process reload without external source, clipboard single transaction/Undo and separately gated Rhino formats.

## Task 6: Four-view scene, picking, permissions and Gumball

**Files:** `standalone/crates/om9-scene/{Cargo.toml,src/lib.rs,src/camera.rs,src/picking.rs,src/affine.rs}`, `tests/{world_picking.rs,affine_gesture.rs}`.

**Interfaces:** Consumes immutable model stamps/permissions; produces per-view camera/CPlane/grid state, cache keys, exact pick IDs, synchronized affine overlays and one prepared transform commit.

- [ ] Read relevant v2 UI/view/Gumball/layer contracts; write stale camera/source, hidden/locked/unselectable, occurrence and CAD/display-mesh identity tests.
- [ ] Observe failures; implement camera/world transforms, independent permissions, finite grid, revision caches and immutable affine gestures.
- [ ] Verify Cancel leaves geometry/Undo unchanged; commit gives one Undo; measure warm pointer/frame/commit on authored fixture with hardware/binary/hash receipt.

## Task 7: Actual standalone UI and shared command sessions

**Files:** `standalone/crates/om9-app/{Cargo.toml,src/main.rs,src/app.rs,src/dispatcher.rs,src/session.rs,src/ui.rs}`, integration UI event fixtures; standalone build/run scripts.

**Interfaces:** Consumes world/coordinator/capabilities/scene/archive; produces actual native executable and shared `dispatch(FeatureId, Invocation)` paths from menu/CMD/F6.

- [ ] Read OM9-IFACE-001/UI_DETAILS and existing authored menu/assets. Write shared route/cancel/Enter/focus/nine-recent-command tests before dispatcher/reducer implementation.
- [ ] Implement Matrix9 panel order/width, four viewports, CMD options/history, F6 Pin/restore, Layers, Builder and Gumball against the same owned session. Keep unsupported commands visible with reasons.
- [ ] Build and launch actual app in isolated profile; verify linked libraries/child processes and true interactions/screenshots/resize/DPI/cold restart within accessible test surfaces. Report unmeasured reference parity explicitly.

## Task 8: Acceptance replay, review and synchronized evidence

**Files:** `tools/validate_standalone_foundation.py`, `docs/validation/2026-10-13-standalone-foundation.*`, `docs/openmatrix9-progress.json`, v2 spec/status/registry/manifest and README views.

- [ ] Run standalone workspace tests, formatting/Clippy, release executable receipts, relevant portable/numerical and selected native suites, public-source audit and v1 hash comparison.
- [ ] Replay M01–M12 and related RCORE/DUI IDs with source/binary/fixture/environment fingerprints; incomplete global gates remain unchecked even when slices pass.
- [ ] Use one fresh-context reviewer for all authored changes, address material findings with regression RED→GREEN and final suite.
- [ ] Synchronize tested scopes in v2/checklists/catalog/lookup/status/migration/manifest, READMEs and live ledger; retain source semantics and historical evidence.
- [ ] Mark goal complete only when the requested RCORE-00 work and required validation are actually complete.
