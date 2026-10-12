# RCORE-01 — Circle History rejection state, 2026-10-12T02:29:58+07:00

**Status: in_progress. Full original RCORE-01, Task2 and Task3 remain open.**

No production code, solver, shared host/SDK or selected plugin changed. Runtime59
is reused. No new Rust or plugin build is claimed. Python macros are native
application test instrumentation, permitted by the existing Rust-first policy.

## Exact rejection-time evidence

The diagnostic records Undo/Redo counts and names, booking ID and pending state
**before calling Redo**. Sixteen mutations are exercised in each of three caller
states: no booking; an unmaterialized preserving-Redo booking; and an already
materialized preserving-Redo journal containing an independent caller Label edit.
The fixture then explicitly aborts an existing caller transaction and executes
the original witness Redo. This is48 cases and432 assertions in one native host.

| Assertion | Passed / total |
|---|---:|
| Observer reaches nonempty replay result once |48/48|
| StaleGeometry rejection clears Shape and GeometryResult |48/48|
| Original recorded metadata restored immediately |6/48|
| Known vertex source geometry/frame preserved immediately |45/48|
| Undo/Redo counts and names preserved immediately |32/48|
| Caller booking ID preserved |48/48|
| Pending state preserved immediately |32/48|
| Independent caller edit retained immediately |48/48|
| Original executable Redo after explicit caller abort |48/48|

**Completed RED355/432**, actual host exit0, runner exit1, no fixture exception.
Forty-two recipe/points/roles/parameters/links/path/fraction/recorded/scalar/
construction/units/placement mutations remain in metadata. All three intentional
source mutations change the vertex from(6,8,10) to(30,40,50) and remain changed.

All16 unmaterialized bookings become materialized: Undo2/Redo1/pendingFalse
becomesUndo3/Redo0/pendingTrue, keeping the same booking ID. Abort recovers the
original Redo, so destroyed executable Redo is not claimed. Plain and active
caller states retain their immediate counts/names/pending/booking. Equal counts
are insufficient to certify an unchanged already-active native journal.

## Corrected oracle

An initial completed diagnostic is preserved separately: RED310/432, actual
host exit0, no exception. It measured exact pre-Redo states correctly but used
raw source BREP text as the source geometry oracle. The corrected independent
fixture retains that text comparison as an observation and compares the known
source's type, world vertex coordinates, topology counts and object/shape frame.
Every no-source-mutation case gains an explicit identity location in serialized
BREP (Locations0 becomesLocations1, identity matrix). Geometry/frame stays equal
in45/48 cases. It is not evidence of a moved or changed source vertex.

The earlier atomic60 combined assertion used raw source BREP text and did not record counts until after Redo. New exact pre-Redo measurements contradict the inference that all cases corrupt the history stack or source geometry: identity-location serialization alone changes BREP45/48 while geometry remains fixed. All16 unmaterialized booked cases do alter the immediate stack/pending state. The original report remains a historical raw assertion failure, not a geometry/history oracle.

These semantic checks cover this declared one-vertex source fixture. They do not
certify arbitrary BRep/mesh/source topology or replace full source master guards.
The historical raw fixture remains immutable; no assertions were removed from it.

## Native journal boundary

`Document::openTransactionPreservingRedo` first completes the preceding caller
booking/active journal, so unconditionally opening it during History recompute
would violate caller ownership. Direct `Transaction::applyChecked` lacks the
Document rollback/defining flags and would notify or record property restoration
into the caller journal. Property DisableNotify does not bypass onBeforeChange;
NoModify only skips starting a fresh transaction and does not isolate an active
journal. No public property replay savepoint/merge was found in the inspected
Document/Transaction APIs. This is source evidence, not a tested savepoint design.

The native API fingerprints are recorded. No FreeCAD core code was modified or
built. If a native host interface must be added, explain the specific missing
journal/lifetime contract first and rebuild only necessary matching targets
against the single shared SDK; an OM9-only change still uses the plugin builder.

- The full original RCORE-01 objective remains open: GeometryRef/Snapshot/Result for every enabled consumer; adapters1–7, C01–C12, T01–T11; rational/periodic basis, trims/pcurves/seam occurrences; independent read/edit/retain/export and complete supported attributes.
- Current GeometryResult is now directly checked across the existing all-mode History fixture: numeric/picked Center, oriented/direction, TwoPoint, ThreePoint radius and live endpoint branches, Vertical, constant Area, whole FitPoints with growing topology, deformable Center/Fit/Around/Tangent, bounded native tangent and fixed-contact point dependencies, parent placement, Lock, deletion/Undo, detach/Undo and command source pruning. Twenty recorded results cold reopen and retain exact bytes; legacy empty-recipe AroundCurve also has paired current native/cold evidence. This is named fixture coverage, not every option/backend/representation or complete historical provenance. Native-plan/source correspondence remains required.
- Exact rejection-time Circle History evidence is RED355/432 across48 cases, Runtime59. The three caller states are no booking, unmaterialized preserving-Redo booking, and already materialized preserving-Redo journal with an independent caller edit. Forty-two cases retain tampered metadata; three source callbacks retain changed point geometry. Sixteen booked cases materialize the caller journal (Undo2/Redo1/pendingFalse becomesUndo3/Redo0/pendingTrue); plain/active states retain immediate counts/names/booking/pending. Original executable Redo succeeds48/48 after explicit caller abort where applicable. Identical counts do not prove absence of pollution of an existing journal. A checked native replay savepoint/notification barrier must preserve pre-existing caller edits and booking/journal, reject observer mutations, restore protected metadata/source state and clear paired invalid output. All original close/removal/owner/lock/dependency/cache invariants remain open.
- Task2/3 remain open: generic snap/pick/parameter provenance, snapshot sharing and aggregate allocation/traversal/worker/cancellation bounds, Rebuild and every other enabled GeometryResult consumer.
- Global conversion/deviation/locus and complete parameter/topology maps remain unproven; the current native Circle output contract checks basis, full domain and shared endpoint identity. Full conversion exactness is reported uninspected; maps remain null.
- Four recorded archives were independently reopened in one new native host and replayed again; offscreen evidence does not certify actual mouse, rendering, all viewport/preview/cache lifetime or every original persistence family.
- The inventory was captured before Runtime59 build and relevant Curve/History source hashes match. Full inventory differs due parallel Gumball source and a new test fixture; it is not an exact compiler dependency certificate. Production ABI/deployment acceptance remains pending.

Next: Implement and verify the necessary native checked replay savepoint or equivalent journal boundary, preserving both an unmaterialized caller booking and an already materialized caller journal. Existing openTransactionPreservingRedo cannot be called unconditionally because it commits/retire the preceding caller transaction. Use the new semantic432-check RED unchanged for comparison, add positive replay/caller Undo/Redo and callback close/remove/dynamic-property/restore-failure coverage, then continue every original RCORE-01 requirement.

[Machine evidence](2026-10-12-rhino-core-circle-history-atomic-state.json).
