# Circle History native replay savepoint

Status: approved for implementation by the user on 2026-10-12, with the reply
“Đồng ý thiết kế native savepoint”. Approval permits adding the native interface
described below; it does not establish implementation or verification. The complete RCORE-01 objective, its
adapters1–7, C01–C12/T01–T11 and existing Task2/3 remain open.

## Intended result and evidence

Keep the existing Circle solvers and Rust-owned validation/snapshots/results.
Make History replay a checked native property operation. Rejected replay must
restore protected metadata, preserve source geometry and the caller's original
booking/journal, clear paired invalid output, and keep its document/objects alive
until cleanup finishes. Successful replay must still participate in the caller's
actual Undo/Redo and retain supported cold reload behavior.

Runtime59 currently fails this contract. The completed semantic fixture is
RED355/432 across48 observer actions and three caller states: plain, booked, and
already materialized. Metadata restores6/48, source geometry45/48, immediate
stack/pending state32/48. Booking IDs, independent caller edits and original Redo
after explicit caller abort pass48/48. Equal counts do not prove an unchanged
active journal. Raw BREP identity-location changes are not moved source geometry.

The additional isolated close callback reaches a recorded replay result, closes
the owning document, and persists `document_alive_after_callback=false`. Native
recompute does not return within300 seconds. The runner kills its exact owned
process and confirms it is terminal. This is a lifetime failure witness, not a
completed green suite or proof of a particular native crash mechanism.

Sources of evidence:

- `docs/validation/2026-10-12-rhino-core-circle-history-atomic-state.md/.json`.
- `tests/rhino_core_circle_history_atomic_semantic_smoke.FCMacro`.
- `tests/rhino_core_circle_history_close_callback_smoke.FCMacro` and preserved
  `build/solid-regression-circle-history-close-red63-offscreen-20261011` artifacts
  in the FreeCAD workspace.

## Approaches and selected design

1. Opening `openTransactionPreservingRedo` unconditionally is rejected. It
   completes the preceding caller transaction and does not preserve ownership of
   an already booked/materialized caller operation.
2. Manual property compensation through the plugin is rejected. DisableNotify
   still invokes onBeforeChange; NoModify does not isolate an active journal.
   Direct Transaction::applyChecked lacks Document's private restore flags and
   lifecycle/observer protection. Compensation can pollute the caller's journal.
3. Add an explicit native **property replay savepoint** to App::Document. Reuse
   native Property copies, TransactionFactory, checked apply and private history
   state. Stage successful merging; keep rollback and lifetime protection inside
   the native owner. This is the selected approach.

## Native API and ownership

Proposed C++ API: `App::Document::PropertyReplayTransaction`, a noncopyable RAII
scope created on the GUI/native document thread. Its state owns a private native
property journal and captures the original booking ID, active caller journal,
transaction lock and native critical state. Methods are `verify`, `commit`,
`rollback` with a completion verdict, and explicit completion of a checked error
state. Method names may be adjusted while preserving these semantics.

The private journal never replaces the caller's active pointer and never enters
visible Undo/Redo history while replay is being prepared or rejected. Document's
property bookkeeping routes native replay writes into the private journal.
Recompute must be supported with UndoMode disabled as well as enabled.

This API handles native property writes and permitted native dynamic-property
addition/removal needed for result/unit metadata. It must not pretend to support
object creation/deletion, document restoration, arbitrary property moves or
renames as part of replay. Attempts from observers are rejected before mutation;
existing caller operations of those kinds remain intact in the caller journal.

C++ is a necessary FreeCAD Document/Property/Transaction boundary exception under
the user's Rust-first policy. Document, property and journal pointers stay on the
GUI/native thread and are never passed to Rust workers. Rust continues to own
portable policy, numeric solving/validation, independent geometry, and results.
Python remains test/bootstrap tooling only.

## Lifetime and notification boundary

Protection starts before Circle reads its replay metadata/sources and lasts
through solve, publication, rollback, invalid-output cleanup and journal release.

- Prevent closing/restoring the owner, removing replay objects, or altering
  caller Undo/Redo/booking/locks while the scope is live. Check removal before
  PendingRecompute can enqueue it. Check insertion before allocating/attaching
  an object. Cover both Document and Application entry points.
- Reuse the native observer-depth discipline. Observer writes to protected
  document properties/metadata or dynamic properties fail before changing data.
  Latch the attempted mutation so a Python observer swallowing its exception
  cannot cause replay to be reported as accepted.
- Distinguish native setter/restore permission from observer permission. Native
  onChanged placement/shape propagation remains possible; emitted callbacks do
  not inherit that permission.
- Cover Property::Copy/Paste callbacks during journal capture, staged merging
  and restoration. Verification must run after preparation and before any
  irreversible journal transfer. Cleanup must not release lifetime protection
  while journal-owned property values can still invoke native/Python callbacks.
- Reads are not silently blocked. Transient setter observations do not establish
  atomic consumer/cache correctness; publication generations and every remaining
  RCORE-01 consumer contract still require their own verification.

## State transitions and caller history

`Captured → Solved → Published → Verified → Committed` is the success path.
Any rejection transitions through checked rollback and protected invalid-output
cleanup. The original error remains a typed StaleGeometry or applicable kernel
error. A failed rollback must never be certified successful.

| Original caller state | Rejected replay | Successful replay |
|---|---|---|
| No booking/journal | Restore metadata/source, clear Shape/result; preserve visible history | Commit derived native property update without inventing a user Undo operation |
| Unmaterialized booking | Keep exact ID, description, Redo and pendingFalse; do not materialize | Materialize that same booking only after acceptance, then merge replay values |
| Active caller journal | Preserve pointer, original entries, ID, prior edits and held Redo | Merge accepted changes into that same journal, retaining first-write originals |

Scope construction and rejected replay must not finish a previous caller booking
or acquire an unrelated sticky global transaction. Source edits made before the
scope remain caller-owned and must not be rolled back as replay changes.

## Strong journal merge

Reuse native TransactionObject/Property semantics; do not reconstruct geometry or
metadata from JSON to compensate native state. Prepare a property-only merge
without altering the caller journal. Existing caller entries win for repeated
property IDs, matching native first-write Undo semantics.

Stage all allocation, property copies and container entries before transfer.
Preserve object status, names and native ownership; copying a caller record must
not duplicate ownership of a detached object. One viable implementation prepares
a replacement container referencing unchanged native records, with owned copies
only for affected records, then swaps after a final Document verification. Its
failure path destroys only staged property records, never caller-owned objects.

If accepted replay must materialize a previously empty booking, open callbacks
must be protected and checked. A failed open/merge restores the original empty
booking, descriptor and executable Redo. Regular bookings and preserving-Redo
bookings both need evidence. No caller state may be retired to simplify this.

Journal allocation/traversal must have an explicit checked resource bound and
fail before altering caller state. Reuse the actual request/publication budget
contract where applicable and report native retained resources honestly; do not
claim the entire RCORE-01 budgeting requirement from one journal fixture.

## Rejection and error output

Native checked rollback restores private-journal changes with Document's private
rollback/defining state, while observers remain unable to mutate participants.
Keep the live scope during Circle's error cleanup. Clear Shape and existing
GeometryResult without creating entries in the caller journal, revalidate the
invalid pair, then release the private journal and protection. Do not restore
cleared error output again from a destructor or recapture tampered metadata as
the new baseline.

If capture/restore/cleanup cannot complete, return a typed failure and retain
explicit evidence of the unverified state. This does not satisfy completion;
the required recovery and observable-state contract must still be established.

## Native build and ABI constraints

A concrete core change is necessary because the missing journal isolation,
checked restore flags and lifetime barrier are private FreeCAD native state.
An OM9-only plugin fix cannot safely emulate them through the inspected public
APIs. This is the specific permitted reason for a native App target build.

Reuse the single SDK and object/dependency cache at
`H:/FreeCAD-src/build/relWithDebInfo`. Build only necessary FreeCADApp objects and
redirect the App DLL/import library/PDB into a new native test output, using
`build/prepare-isolated-native-app.py`. Never reconfigure the root, build all
FreeCAD targets, copy a complete runtime, or replace DLLs used by a live user
host. Preserve all prior runtime/fixture/RED evidence.

Build OpenMatrix9Gui through `build/build-om9-plugin.ps1`, using the existing
plugin cache and the matching isolated App import library via an explicit,
validated SDK import override. Extend the standalone adapter/builder only as
needed to select that matching native import; retain the same shared SDK for all
other libraries. Native test launch must select and fingerprint the new App plus
the accepted GUI/host. The old App19 runtime is not matching new-API acceptance.

## Acceptance and implementation sequence

1. Freeze RED432, close-callback witness, source/host and native API fingerprints.
2. Implement the native scope/checked property routing, lifetime/history barriers
   and strong merge. Validate native isolated build/ABI selection before testing.
3. Integrate the scope into both recipe-v2 and legacy Circle replay. Keep every
   existing solver and source/GeometryResult guard. Keep cleanup in the scope.
4. Run the unchanged semantic432 fixture and close-callback fixture on the new
   matched native host/plugin. A prefix report or killed process is not a pass.
5. Add positive replay tests for all caller states, both booking modes,
   UndoMode0/1, explicit commit/abort, and actual Undo/Redo of source/child changes.
   Include earlier caller edits to the same property and object-ownership cases.
6. Add before-change, result, unit, merge/copy/restore callbacks attempting
   close/remove/create, dynamic-property mutations, commit/abort/clearUndo/Redo,
   caller lock/booking changes and unit-context edits. Verify typed rejection,
   no silent overwrite, paired error output, and exact native state.
7. Run matching Circle result/callback/source/roundtrip/mode/legacy/cold and
   existing Curve/publication regressions. Reuse named tests; broaden only for
   changed native contracts. Retain failed witnesses and source drift limits.
8. Update Spec v1, feature catalog JSON/YAML, manifest, progress ledger and README
   from actual evidence. Full RCORE-01 remains open until its original complete
   requirement-by-requirement audit passes, including all other consumers,
   conversions/maps, sharing/budgets, generic input, lifecycle and persistence.

Self-review: the proposal preserves all three caller states and native object
ownership, separates rollback from error-state cleanup, covers swallowed observer
errors and delayed removal, and states all build/verification limitations.
The strong merge, callback discipline, resource limits and native deployment are
proposed requirements, not implemented or verified claims.
