# Rhino core: guarded delayed Rebuild result publication

Date: 2026-10-10. Baseline `3f5fa93` on `codex/rhino-core-object-foundations`.
**Authored RCORE-01.T08 Rebuild publication fixture is verified; all P0 remains
in progress.** Reuses the existing Rebuild fitter, native Part shape factory,
source guards, Coin preview and native transaction. No replacement command or
new solver was added.

The previous controller validated source inputs before solving and before
commit, while preview publication had no independent post-solve guard. A real
native result is now an owned envelope with serialized document name, copied
ObjectSnapshot values and native output Shape references. It retains no source
Python wrapper or Document/DocumentObject/view-provider pointer. Rebuild's
existing Rust solve remains synchronous; normal preview delivery uses the GUI
event queue. Commit is immediate through the same guarded publication method.

At delivery, C++ resolves the document by name and checks active/edit/closing
permissions plus the existing session, UID, source incarnation/revision, Undo
epoch, unit context, ancestor and world-placement guards. Rust then grants
one-shot publication only for the latest pending request and a valid context.
Validation occurs before Coin attachment or transaction creation. Options,
cancel, a new solve and document switch/close invalidate acceptance. An old
delivery cannot clear a newer preview or overwrite its deviation display.

Safe Rust stores sixteen fixed request slots without allocation or host
addresses. IDs advance with checked arithmetic and never wrap/reuse during the
gate lifetime; capacity/exhaustion rejects before another native result is
constructed. Ownership remains pending after acceptance/cancellation until the
native envelope releases its owned Shape references and retires the slot once.
This bounds envelope count; it does not measure aggregate/native allocation.

An independent review found a material teardown defect in the first integration:
an envelope's State reference could outlive the controller's State when Qt
discarded queued callback functors. The repaired envelope holds a shared,
value-only lease containing the Rust gate and counters. It owns neither State
nor a document and forms no cycle with the held queue. A dedicated child QObject
receives queued deliveries. The production shutdown helper cancels interaction,
deletes this receiver and drains held results before interpreter shutdown;
`aboutToQuit` and the controller destructor invoke it idempotently. Further
solve/requeue after shutdown is rejected. The fixture queues an actual native
result and calls this same cleanup helper, then settles the event loop and
proves no delivery callback ran and the owners retired.

## Evidence

The [evidence file](2026-10-10-rhino-core-delayed-publication-evidence.json) pins
sources, review, build/Rust logs, macro/module, result/process proofs and raw
held/delivered BReps. Final native module is `e97cad9e…`.

Three fresh native processes pass **245 checks** and exit0:

- Delayed Rebuild:173 checks, twenty named case groups,
  `build/merge_replay-1/e0e0c06454aa41e5a9bf5594268afc24`.
- Unchanged curve_spline:41 checks,
  `build/merge_replay-1/1bc3ce3c87aa4ed6af70ff4a6119b4d5`.
- Unchanged object lifecycle:31 checks,
  `build/merge_replay-1/054d5589d4c944dfb4e6401c5c652195`.

The authored source is a native clamped degree2 quadratic curve through poles
(0,0,0),(2,4,0),(6,0,0). The actual existing Rebuild solve produces a valid
degree3/eight-pole output. The fixture holds that output before publication,
then changes the source/context. Delivery reports the same ticket and exact
raw BRep as held, with no additional solve/native output construction.
All document objects/property names, persisted Shape bytes, placements/groups,
Undo and Redo are unchanged by delivery, relative to the intentional mutation.
Coin searches visit every native CAD view, including all four default OM9 views.

Cases include unchanged positive delivery; source edit; byte-identical retained
owner restoration; replacement; Undo epoch; units; parent placement; PointCount,
Degree and DeleteInput options; cancel; newer result delivered before older;
document switch; close with missing document; same-name document/source
recreation; fresh-session positive delivery; default queued delivery;
sixteen-slot capacity plus seventeenth rejection/recovery; and shutdown discard.
Final counters balance **36 envelopes and36 owned native Shape references**,
with no live envelope, held result, Coin node or retirement failure.

Behavioral RED is retained:

- Rust stub:seven executed gate tests failed before policy implementation.
  Final full Rust suite548 passes; Clippy exits0 with existing warnings outside
  this gate. The [gate review](../reviews/2026-10-10-delayed-gate-review.md)
  independently checks the exact gate and raw logs.
- Native publication extraction before the post-solve repair:
  `b45e8568555e4c2f8044489430fbefab` passes four checks then fails because the
  real held output attaches preview after source edit. This is an intermediate
  extraction of the previous publication behavior, not a claim that the old
  synchronous command already exposed asynchronous completion. The exact
  pre-repair adapter and build log are retained separately.

Earlier7/144-check passes and the old41-check control are superseded by the
final repaired module. Excluded fixture diagnostics are also retained:5791cd
used Python `len` on native SoPathList;218389 counted only the active viewport;
947cec demanded byte identity from a new deep-copy owner rather than restoring
the exact owner;1bac81/7a4b1f read computed App::Part.Shape in inventory, creating
`_Part_ShapeCache` and changing cached native owner flags. The latter raw
inventory differential is preserved. Final inventory reads persisted Shape
properties and compares all property names/placements/groups without filtering
new properties or normalizing BRep fields.

The [native review](../reviews/2026-10-10-delayed-publication-review.md) audits
the final guards/ownership/shutdown path, actual artifacts and resolved lifetime
finding. The plugin builds against the shared FreeCAD SDK. Prior native basis,
Hatch and kernel validator proofs are unchanged and explicitly reused, not new
test runs in this checkpoint.

## Remaining scope

This closes the authored delayed native Rebuild publication fixture. The solver
is synchronous and the completion/publication stage is queued on the GUI thread;
this does not establish concurrent OCCT solving, worker-thread cancellation,
cross-thread lifetime, all controllers, sanitizers or global absence of UAF.
Counters witness release of the envelope's owned native/Python Shape references
and Coin attachments, not every OCCT internal/cache allocation. Serialized
diagnostic strings retain no native Shape owner. Broader RCORE-01.C11 native
worker lifetime coverage remains open.

T07 full allocation coverage, T10 guarded host parameter dependency and the
remaining explicit P0 contracts/chapters continue. Existing Line/Polyline and
other implemented native commands remain reused.
