# RCORE-01 — Curve publication callbacks RED, 2026-10-11T23:09:13+07:00

**Status: in_progress. All original requirements remain required.**

Six existing commands were exercised with real FreeCAD document observers on
Runtime52. For each command, six callbacks change Shape, Placement, construction,
result, units or placed parent after result metadata assignment; a seventh starts
the same command when the output is created. All42 callbacks fired exactly once.

Authoritative fixture: **RED42 passed/168 failed in210 checks**, actual host exit0,
no fixture exception. Every case increases UndoCount by one and loses the prior
executable native Redo. Changed output is admitted instead of rejected, no typed
StaleGeometry diagnostic is emitted and document restoration fails. Replacement
completion requires the old output to have been rejected first; a surviving old
output cannot satisfy this check. Literal independent lengths/ellipse axes guard
the replacement result. One initial48/210 exploratory witness had a weak restart
assertion; it is excluded and preserved with the strengthened retry evidence.

The main direct result checkpoint52 **662/662 native,18 exits0,880 Rust** remains
valid for its stated non-adversarial fixture scope. This RED demonstrates the
explicitly open callback publication requirement; it does not invalidate those
separate executed cases or establish whole RCORE-01 completion.

## Root cause and existing reusable mechanisms

CurveController/CurveEllipse open ordinary transactions and commit without a
complete post-observer geometry/frame/metadata/session check. Their result cleanup
can erase a replacement generation. SolidController already uses the selected
host's `openTransactionPreservingRedo`, generation-bound cleanup, native normalized
solver fingerprint and local/world/root Placement validation. Reuse the host
transaction capability and native capture primitives; do not rebuild FreeCAD.
Rust retains independent policy/validation ownership; C++ performs native checks.

## Next action

Use the existing native openTransactionPreservingRedo API for Curve. Freeze execution command/document/units/generation/admission revision/source guards before callbacks. Recheck session/source/layer/ownership and full output geometry/frame/metadata at creation, metadata, recompute and commit barriers. Keep callback replacement sessions intact on both failure and cleanup; run unchanged RED210 and full matching regressions. Original RCORE-01 scope, auxiliary foci/History/Rebuild, sharing/budgets/conversion/maps remain required.

## Remaining required work

- Every original RCORE-01 requirement remains required: adapters1–7, C01–C12, T01–T11 and every enabled family/consumer, GeometryRef/Snapshot/Result, rational/periodic basis, trims/pcurves/seam occurrences, independent read/edit/retain/export.
- Task2 remains open: generic native snap/pick provenance, all original numeric/native parameter paths, full subset/index sharing across selection observations, aggregate allocation/traversal/worker/cancellation bounds. Replay currently duplicates construction sources/events in the common result; complete sharing and aggregate cost remain required.
- Task3 main direct Curve results are paired, but Ellipse foci and Circle History recompute/freeze/detach/Rebuild result lifecycle still require implementation and independent native/cold evidence.
- Checked Curve publication remains required: generation/revision/context, full source content/units/frame/locks/owners, layer/output/metadata at callback barriers, with prior executable native Redo preserved on rejection. Existing ordinary transaction abort is not certification.
- Analytic-to-NURBS global deviation/locus and parameter mapping, complete topology maps and output attributes remain unproven. Native validity/master/physical dimensions are scoped observations; conversion exactness remains uninspected and maps null.
- Tangent/native_solution, changed CPlanes/orientation, every numeric mode and explicit non-Circle Undo, repeated/shared sources and revision exhaustion are not established by the six focused Rust tests/nine new native recipes.
- Offscreen evidence does not establish actual mouse, every viewport preview lifetime, OpenGL/render acceptance or all original cache/resource invariants. Production ABI/deployment acceptance remains pending; selected App/GUI host reuses the one SDK.
- Expanded source/resource/test inventories are not exact compiler dependency certificates. Source inspection review establishes no runtime behavior beyond the named executed fixtures.

[Machine checkpoint](2026-10-11-rhino-core-curve-callback-red.json).
