# RCORE-01 — preserve executable Redo after rejected publication

Status: design and native implementation remain open. This is required by the
original lifecycle contract, not a replacement for the full chapter.

## Verified problem

`tests/rhino_core_edit_result_publication_smoke.FCMacro` creates an executable
Redo witness, then changes source A from a native document observer during output
creation. Publication rejects the stale result and aborts. Native runtime2 has
three passing assertions and two failures: complete Undo/Redo baseline and actual
execution of the previous Redo. Fresh matching Runtime14 repeats3/5 with actual
host exit0: objects A/B, original geometry and Undo0 restore, but Redo1 becomes0.
Keep all five assertions unchanged.

`src/App/Document.cpp` calls `_clearRedos()` in `_openTransaction` before the new
transaction exists. `_clearRedos()` deletes the redo transaction objects.
`_abortTransaction()` reverses the new transaction but cannot restore that deleted
history. No plugin-only claim may substitute source restoration for Redo recovery.

## Required host behavior

- Add an explicit opt-in transaction mode for atomic publication. Preserve the
  existing default behavior for other FreeCAD callers.
- Retain prior native redo transactions, exact IDs, ordering and object ownership
  while the publication transaction is pending. On successful commit, invalidate
  them normally. On abort, restore the actual executable transactions.
- Use host-owned private state, preferably `Document::DocumentP` so no public
  Document data layout changes are needed. Do not replay Redo, fabricate actions,
  access private members from the plugin, or mutate user models to simulate it.
- Account for close/destruction, explicit history clearing, nested/replaced
  transactions, global transaction participants and exceptions during rollback.
  Invalidated retained history must never be resurrected.
- Native source/parent/output guards still reject stale or foreign changes;
  retaining Redo does not authorize callbacks to change the document.

## Implementation and proof gates

- [ ] Inspect the complete host transaction implementation and existing tests;
  select the smallest additive public API consistent with the host's ownership.
- [ ] Add native host tests for abort/commit, empty transactions, history clearing,
  document close and multi-document behavior using real executable Redo.
- [ ] Implement the opt-in mode and use it in the OM9 publication bridge.
- [ ] Rebuild the matching shared SDK only because actual FreeCAD core source/API
  changes require it. Explain that concrete reason before rebuilding. Reuse the
  existing SDK; do not configure another root or copy a complete host runtime.
- [ ] Require the original five native publication assertions to pass. Re-run
  output/source/group/NaN mutations, normal Undo/Redo, cold persistence and existing
  host transaction regressions on the matching host/plugin pair.
- [ ] Synchronize RCORE-01 evidence and progress. All-family results, measured
  conversion/maps, attributes, preservation/export and lifecycle remain required.

No FreeCAD core edit, SDK rebuild or Redo fix has been performed by this design
record. Further source investigation may refine the proposed API without relaxing
the required behavior or original acceptance assertions.

## Further host ownership inspection — 2026-10-11

Current `Document.cpp` still lazily materializes a booked transaction on the first
write; `_openTransaction` then deletes Redo. `setActiveTransaction` can commit or
rename a preceding booked transaction. An opt-in request must therefore be tied
to the exact transaction ID and cannot be an ambient flag applying to later work.
Global participants are materialized through the transaction initiator; they
need the same abort/commit decision, not an independently preserved partial group.

`Transaction::~Transaction` deletes detached objects recorded as `New`. Redo owns
real transaction records and, in some cases, detached native objects. Retaining
counts/names or replaying property values cannot restore that ownership. Before
choosing an implementation, characterize real creation/removal/reattachment and
linked-property transactions, including multiple transactions referring to one
object. A deferred deletion design must prove it cannot free an object still
owned by the new Undo transaction or retain a dangling pointer.

`Document::redo` commits a pending transaction before using the Redo list and then
asserts that the list is nonempty. Simply leaving old Redo publicly visible while
changing commit to clear it can violate this assumption. Pending-history visibility
and direct Undo/Redo requests need explicit tests in the opt-in mode.

`Transaction::apply` catches rollback exceptions internally and logs them. A
preserved history must not be restored blindly after a partial rollback.
`clearUndos` commits an active transaction, destroys Undo from front to back, then
clears Redo; the Document destructor calls it before destroying document objects.
Explicit history clearing and close must invalidate retained history without
resurrection and obey the same native object deletion order.

Existing integration tests live in `src/Mod/Test/Document.py`; C++ document tests
are in `tests/src/App/Document.cpp`. These are source findings, not runtime proof.
No new opt-in API or host behavior has yet been implemented. The original
publication fixture has fresh Runtime14 proof3/5. The new
`tests/rhino_core_atomic_transaction_smoke.FCMacro` exercises executable property
Redo ordering, creation/removal with native links and identities, successful
commit, empty abort, clearing, replacement and close. On a host without the API,
it records the absent API and deliberately uses the default transaction to
characterize the RED behavior. Syntax is checked; native execution is pending.

## Proposed additive host boundary

This is a required host integration fix within the authorized RCORE-01 work.
It is not a new Rhino default or a claim about Rhino Undo behavior.

Add `Document::openTransactionPreservingRedo(std::string)` and the equivalent
Python method. Keep every existing `openTransaction` overload and its default
behavior. The opt-in starts a fresh exact transaction ID, after normally closing
an earlier pending transaction; it must not retrofit preservation onto an already
materialized transaction whose Redo was deleted. Store the selected ID and held
native history in `DocumentP`, with no public Document layout change and no
native history/pointers crossing into Rust. Host transaction ownership cannot be
implemented in portable Rust; OM9's existing Rust admission stays unchanged.

On first materialization, move the native Redo list/map to host-owned hidden
storage without deleting or copying Transaction records. Resolve the policy for
other participants from the exact initiator/transaction ID while its application
description is live. Pending old Redo is unavailable to direct Undo/Redo. Commit
disposes it using normal ownership/deletion order; successful abort restores
the actual list/map, IDs and order after rollback. Empty transactions, explicit
clearing and document close need separate paths; a cleared history never returns.

Before implementation, resolve two ownership/rollback gates with source and
native tests, not assumptions:

1. A retained Redo record may own a detached object. A new active transaction
   must not make the old and new destructors independently own the same detached
   object. Characterize native reattachment and deletion, including linked
   properties and multiple records referencing one object. Transfer or rejection
   needs an explicit ownership rule with a test; simply retaining raw pointers
   and later deleting the old list is insufficient.
2. `TransactionObject::applyChn` catches individual property Paste exceptions,
   in addition to outer `Transaction::apply` catching failures. Missing properties,
   unsuccessful recreation/rename/removal and unmet restoration operations can
   also be silently skipped. A checked rollback must detect these conditions and
   propagate both failure levels before restoring retained history. Preserve the
   documented cross-document delegation rather than reporting that as failure. Do not
   call a void apply and infer success from its return. Preserve existing caller
   semantics while providing an additive checked path for atomic publication.

The unchanged original OM9 publication fixture, real host lifecycle tests,
multi-document exact-ID tests and matching SDK/plugin regressions are mandatory
before claiming the Redo repair. No host production code or SDK rebuild is made
by this design addition.

Read-only independent review additionally requires a group-wide rollback gate:
roll back every participant first, collect checked results, then restore held
history only if all participants succeeded. Exceptions must release the
application closing-ID guard and dispose invalidated history safely. Sequential
per-document restoration before another participant rolls back is insufficient.
The Python fixture now checks native backlinks as well as forward links. Its
snapshot checks native object IDs and history counts; C++ tests must supplement
it with ordered `getTransactionID(false, pos)` comparisons. The reviewer confirmed
that Python wrapper identity is meaningful because native objects cache wrappers.

The subsequent ownership review narrowed the cancellation concern: supported
public `addObject(existingObject)` rejects a detached Redo-owned object because
`detachFromDocument` retains its document pointer; `moveObject` imports a copy.
However, property writes through a retained native object still reach
`Document::onBeforeChangeProperty` through that pointer and can record a `Chn`
entry for the detached child. Disposing its old Redo `New` owner on commit would
then leave a dangling pointer in new Undo. The opt-in must reject these writes
before mutation or transaction recording. Existing default callers stay unchanged.

Nine new C++ tests in `tests/src/App/Document.cpp` cover explicit API availability,
exact Redo IDs/order, creation ownership, commit, empty abort, clearing, shared-ID
participants, default behavior and detached-owner property rejection. The new
test translation unit is compiled and linked in an isolated test executable with
the shared SDK's cached dependencies and existing compiler/link flags (exit0).
This is an explicit baseline against the existing host DLL, not a build of
current FreeCAD core source. Native baseline execution now returns exit1 with
4/9 passing: exact IDs/order, creation ownership and shared-ID participants lose
prior Redo; API availability and the guarded detached-owner test also fail because
the API is absent. The separate native Python fixture completes15/23 with no
fixture exception and actual host exit0, wrapper exit1. It checks creation/removal
forward and backlinks, property Redo order and lifecycle controls. No host DLL
or shared test executable was replaced, and no root reconfigure was performed.
The unsafe detached-owner witness deliberately
requires the API before writing; it never dereferences an owner the default first
write could destroy. Host production implementation and matching SDK build remain
pending. Existing core changes from other work are preserved.

## Checked rollback prerequisite — 2026-10-11T11:47:17+07:00

Native applyChecked and checked-only native deletion/move/restoration barriers are now implemented. Nine targeted native tests pass; full App executable569 passed/2 skipped/5 known opt-in API failures, actual exit1. This updates the preceding historical statement that no host production code/SDK rebuild exists: FreeCADApp was rebuilt for the additive checked API, while preserving-Redo itself remains pending. Application closeDocument currently permits deletion even during transaction callbacks; the group design must protect participant lifetime during checked rollback rather than reusing raw document pointers blindly. Historical Runtime15 tests used the old App DLL; require matching new host/plugin reruns. [Checkpoint](../../validation/2026-10-11-rhino-core-checked-rollback.md).

## Verified native retention and Edit integration — 2026-10-11T12:55:37+07:00

The additive opt-in API and same-thread checked group rollback now have native43/43 and matching Runtime18/API9 callback5/5/Python23/23 plus401 host checks. App599pass/2skip/0fail of601; Rust808pass. The original API/callback gap is superseded within this scope. Worker/custom callback paths, aggregate postconditions/budgets and every original RCORE-01 family/consumer remain required. [Checkpoint](../../validation/2026-10-11-rhino-core-atomic-redo.md).

## Publication/booking checkpoint — 2026-10-11T13:34:12+07:00

Builder/Surface callbacks48 checks and matching462/16 serial hosts pass on Runtime21/API11; native46/46 and App602pass/2skip/0fail of604. Independent native Delete journal separation and observer-closed group join are verified with RED/GREEN. Separate global lifecycle witnesses1pass/2fail of3 keep finalization open. All original RCORE-01 requirements remain active. [Evidence](../../validation/2026-10-11-rhino-core-publication-booking.md).

## Isolated native publication checkpoint — API19, 2026-10-11

Global28/28, root atomic46/46 and App602pass/2skip/0fail of604 (7 disabled)
pass on an actual isolated App DLL selected and reported by each native test
process. Structural/activation observers, nested receiver-first property moves,
independent checked rollback verdicts, interleaved same-ID failures and default
nested apply now have executed RED → GREEN evidence. Shared SDK remains API12;
its DLL/import-library/PDB are unchanged while the user's host stays live.
Matching OM9 GUI/plugin callers, caller diagnostics/result ownership,
worker/custom callback and aggregate contracts remain required, together with
every original RCORE-01 family/consumer/adapters/C/T item. This checkpoint does
not complete the chapter. [Evidence](../../validation/2026-10-11-rhino-core-isolated-publication.md).

## Actual matching caller commit checkpoint — 2026-10-11T15:12:00+07:00

Builder69/69 and Surface72/72 fix failed-close return-reference ownership and operation/source diagnostics. Matching Runtime24/API19 649 checks/23 actual exits0,5 cold hosts, full Rust808pass. Isolated GUI launcher reuses shared SDK resources and leaves shared binaries unchanged. Full chapter remains in_progress; all original requirements stay active. [Evidence](../../validation/2026-10-11-rhino-core-commit-publication.md).

## Preview and camera teardown — 2026-10-11T15:56:06+07:00

Surface RED420/468 → GREEN468/468 with matched close reference calibration; native camera RED15/18 → GREEN18/18. New isolated GUI486/2; previous plugin caller/cold649/23 retained separately, no new-GUI rerun claim. Original full chapter remains required. [Evidence](../../validation/2026-10-11-rhino-core-surface-preview.md).
