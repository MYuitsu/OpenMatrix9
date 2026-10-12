# Curve publication — scoped review and disposition

A fresh read-only reviewer examined `CoreCurvePublication`, the existing Curve
and Ellipse callers, and the callback fixture. The reviewer made no edits,
builds, Cargo runs or native host executions. Runtime evidence belongs to the
root agent and is recorded separately. This review does not close RCORE-01.

## Actionable findings

1. **P1: cross-family replacement bypasses session checks — FIXED; matching
   Runtime55 native witness GREEN60/60.** A native
   output-created observer starting Box cancels Curve without advancing its
   generation/input revision. An old non-Circle Curve can pass those checks;
   failure cleanup may cancel the replacement Solid. Bind both publication and
   cleanup to a cancellation/session identity covering command replacement.
   Rust now retires the generation on both cancellation entrypoints. Exhaustion
   invalidates publication tokens permanently without wrapping; the existing
   generation-bound bridge and cleanup consequently reject the old Curve and
   preserve the replacement. Focused Rust RED0/1→GREEN1/1; native RED33/60→GREEN60/60.
2. **P1: publication loses transaction identity — OPEN.** The returned native
   booking ID is discarded. A callback can commit early, making the later commit
   a no-op, or open another booking that cleanup can abort. Retain and verify the
   original ID, prevent premature retirement, and abort only the owned booking.
   ID comparison alone cannot restore an already committed journal. Actual
   commit/abort/reopen witnesses are still required; no host change is justified
   merely by this finding.
3. **P2: missing required layer membership — FIXED IN SOURCE; native witness
   recorded in the accompanying validation report.** The old loop accepted an
   empty owner list after the selected layer removed the output. The bridge now
   requires exactly the expected number of owners, verifies the required plain
   OM9 owner and assignment index, and checks explicit output permissions.
4. **P2: native observation errors lose operation context — FIXED IN SOURCE;
   native witness recorded in the accompanying validation report.** Layer lock
   and unit decoder errors bypassed typed diagnostics. These observations now
   receive `StaleGeometry` with the actual command and input-construction scope.
   Source validation remains outside this catch so its existing typed errors
   are preserved.

## Positive source observations and declined judgments

Expected digest/frame/metadata precede callbacks; output identity is compared
with its original Python wrapper; full digest/orientation checks do not rely on
mutable Shape aliases; document lookup verifies pointer and UID before access;
same-Curve restart cleanup is generation-bound. These are source observations,
not independent runtime acceptance.

The reviewer declined Runtime53 acceptance, executable prior Redo and rollback
fidelity because execution was reserved for root. Root's original/source
callback fixtures are separate evidence. The later replacement witness settles
only the cross-family finding; the premature transaction close finding remains open.

The reviewer declined view-property tampering without a witness. Root subsequently
executed a new native layer fixture changing Visibility, LineColor and Selectable,
then added comparisons against the values assigned by the existing native bridge.
This preserves the existing layer contract: an off layer accepts invisible
outputs while keeping explicit Selectable permission. It introduces no new
layer business rule. Healthy off-layer and MarkFoci checks are separate witnesses.

Close/replacement safety across every setter and callback remains unproven.
The helper resolves documents safely, but callers and shared native routines
still cross multiple callback boundaries. Global commit/abort notifications,
worker/custom callbacks and aggregate rollback were inspected, not certified.

Full source provenance/sharing/budgets, foci common results, Circle History
recompute/freeze/detach, Rebuild, conversion/parameter/topology maps, complete
attributes, mouse/render/cache lifetime, adapters1–7 and C01–C12/T01–T11 remain
required. Offscreen foci geometry and one-Undo evidence cannot prove common foci
GeometryResult integration or whole-chapter acceptance.

Rust continues to own independent request/result/capability/frame validation.
C++ owns the necessary FreeCAD document, native property, membership and view
provider observation bridge. No mutable host object is sent to a Rust worker.
