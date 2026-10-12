# RCORE-01 — Common main Curve GeometryResult, 2026-10-11T23:02:30+07:00

**Status: in_progress. Task2/3 and the full original chapter remain required.**

The existing Line, Polyline, Rectangle, Circle, Ellipse and InterpCrv solvers
are reused. Rust replays accepted original entrypoint events with captured
units/frame/revisions and compares the entire canonical construction, including
options/plans/basis, normalized points and native source associations. Common
input0 retains pre-normalization points; native source refs1..N resolve full
owned parent CAD/cloud values, source attributes, ordered ranges and frames.

The minimum C++ native adapter captures actual output CAD master, topology,
validity and identity before opening the ordinary document transaction.
Main Shape, construction, result, units and layer are published together.
This is a native API exception to Rust-first: Rust owns replay/schema/validation;
C++ accesses FreeCAD/OCCT/Qt values. There is no new solver or retained host pointer.

Conversion exactness remains uninspected, maps null and constructed output
attributes explicitly unavailable. Full conversion/mapping requirements remain
required; independent physical dimensions are not substitutes for them.

## Verification

- Unchanged direct fixture: historical Runtime50 **RED12/54 → Runtime52 GREEN54/54**.
  Literal independent dimensions cover BothSides Line20, closed3–4–5 Polyline12,
  Rectangle4×3 perimeter14, Circle radius1, Ellipse axes5/3 and degree1 InterpCrv7.
  Original inputs, full independent CAD masters, one Undo/Redo and FCStd bytes pass.
- New native sources33/33 cover CirclePath/EllipsePath full two-edge compound120
  and FitPoints branch Undo with RemovedCloud absent. Common result indexing
  resolves full source owners. Fresh host58/58 opens all nine direct/native archives
  and reconstructs independent CAD output and source masters/cloud transformations.
- Matching **662/662 native checks,18 actual host exits0**; five cold hosts reopen
  **27 FCStd archives**. There are39 total saved archives, including Solid/Edit
  warm persistence. All copied fixtures match current source; selected App/GUI,
  probe and shared SDK fingerprints are verified.
- Full Rust **880/0/0** across125 suites, actual exit0; focused behavioral
  **RED0/4 → GREEN6/6**. Initial E0603 harness compile failure is excluded; a wrong
  test mutation literal in intermediate3/4 green is documented separately.
- Standalone OpenMatrix9Gui build exit0. One shared SDK/cache is reused; plugin
  outputs remain separate. No FreeCAD root build/runtime clone was made.
- Frozen1407 selected source/resource/test inputs matched both after build and
  after native runs. Current recording drift: `[]`. Inventory matching is
  scoped to those selected files, not an exact compiler dependency certificate.
- Fresh read-only source review found no introduced concrete bug. Its explicit
  declined judgments and root rulings are retained in the linked review.

## Remaining required work

- Every original RCORE-01 requirement remains required: adapters1–7, C01–C12, T01–T11 and every enabled family/consumer, GeometryRef/Snapshot/Result, rational/periodic basis, trims/pcurves/seam occurrences, independent read/edit/retain/export.
- Task2 remains open: generic native snap/pick provenance, all original numeric/native parameter paths, full subset/index sharing across selection observations, aggregate allocation/traversal/worker/cancellation bounds. Replay currently duplicates construction sources/events in the common result; complete sharing and aggregate cost remain required.
- Task3 main direct Curve results are paired, but Ellipse foci and Circle History recompute/freeze/detach/Rebuild result lifecycle still require implementation and independent native/cold evidence.
- Checked Curve publication remains required: generation/revision/context, full source content/units/frame/locks/owners, layer/output/metadata at callback barriers, with prior executable native Redo preserved on rejection. Existing ordinary transaction abort is not certification.
- Analytic-to-NURBS global deviation/locus and parameter mapping, complete topology maps and output attributes remain unproven. Native validity/master/physical dimensions are scoped observations; conversion exactness remains uninspected and maps null.
- Tangent/native_solution, changed CPlanes/orientation, every numeric mode and explicit non-Circle Undo, repeated/shared sources and revision exhaustion are not established by the six focused Rust tests/nine new native recipes.
- Offscreen evidence does not establish actual mouse, every viewport preview lifetime, OpenGL/render acceptance or all original cache/resource invariants. Production ABI/deployment acceptance remains pending; selected App/GUI host reuses the one SDK.
- Expanded source/resource/test inventories are not exact compiler dependency certificates. Source inspection review establishes no runtime behavior beyond the named executed fixtures.

## Next action

Implement checked Curve publication using existing host transaction/Redo preservation utilities; first reproduce callback source/output/metadata/context mutations in real native witnesses. Pair Ellipse foci and Circle History/Rebuild results, finish generic input/sharing/budgets and every original mapping/lifecycle requirement.

[Machine checkpoint](2026-10-11-rhino-core-curve-result.json).
[Scoped review](../reviews/2026-10-11-curve-result-review.md).
