# RCORE-01 — Ellipse auxiliary focus results, 2026-10-12T01:27:24+07:00

**Status: in_progress. Task2/3 and every original RCORE-01 requirement remain required.**

The existing FreeCAD Ellipse/focus solver is reused. Rust admits only canonical
Ellipse/FromFoci/MarkFoci output roles after complete original construction replay.
Each focus retains input snapshots, units, private native vertex/master/topology
and paired construction/result metadata. Matching native root/vertex locations
are applied once. The minimal native adapter prepares all three outputs before
opening the checked booking and watches each output through recompute/commit.

## Executed evidence

- Runtime56 missing-focus-result witness RED6/30, actual host exit0, no fixture exception.
- Rust role/output tests RED6/9 → GREEN9/9; corrected full suite885/0/0. A later
  separate full Rust run including concurrent Gumball additions passed900/0/0.
- Runtime57 direct focus30/30, independent cold open12/12 of two FCStd archives,
  exact metadata bytes and independently decoded native vertex positions.
- Focus-specific callbacks130/130:26 exact-target callbacks, nine geometry/metadata/
  ownership/sibling mutations and four native close/reopen actions on each focus.
  Entire document/Undo/prior Redo restored and original native Redo executes.
- Matching native regressions662 + existing callback cohorts710 + focus172:
  **1544/1544,27 actual host exits0**. Six cold hosts/29 FCStd;41 archives overall.
- Standalone plugin build exit0; shared SDK fingerprints unchanged.

The first Rust GREEN attempt failed compilation and is excluded from behavior
acceptance. The inventory helper initially failed because sha256 was accidentally
renamed sha257. The corrected1418-input inventory was captured after build began;
it is not a prebuild compiler-input certificate. Scoped Curve hashes still match.
Full current-source matching is false: independent Gumball source/tests changed
concurrently and two new uncompiled focus fixtures were added. Neither later full
Rust900 nor an expanded file inventory certifies Runtime57 against every current
source. These limits are retained in machine evidence.

[Fresh read-only review and declined judgments](../reviews/2026-10-12-curve-focus-result-review.md).
Offscreen evidence does not certify actual mouse/OpenGL/render or deployment.

- Every original RCORE-01 requirement remains required: adapters1–7, C01–C12, T01–T11 and every enabled family/consumer, GeometryRef/Snapshot/Result, rational/periodic basis, trims/pcurves/seam occurrences, independent read/edit/retain/export.
- Task2 remains open: generic native snap/pick provenance, all original numeric/native parameter paths, full subset/index sharing across selection observations, aggregate allocation/traversal/worker/cancellation bounds. Replay currently duplicates construction sources/events in the common result; complete sharing and aggregate cost remain required.
- Task3 main direct Curve results are paired, but Ellipse foci and Circle History recompute/freeze/detach/Rebuild result lifecycle still require implementation and independent native/cold evidence.
- Named Curve output/source/layer/context/cross-family replacement and native commit/abort/reopen are checked. Every native document-close boundary, global observer close/lifetime, aggregate rollback across all enabled families and the full original publication contract remain required.
- Analytic-to-NURBS global deviation/locus and parameter mapping, complete topology maps and output attributes remain unproven. Native validity/master/physical dimensions are scoped observations; conversion exactness remains uninspected and maps null.
- Tangent/native_solution, changed CPlanes/orientation, every numeric mode and explicit non-Circle Undo, repeated/shared sources and revision exhaustion are not established by the six focused Rust tests/nine new native recipes.
- Offscreen evidence does not establish actual mouse, every viewport preview lifetime, OpenGL/render acceptance or all original cache/resource invariants. Production ABI/deployment acceptance remains pending; selected App/GUI host reuses the one SDK.
- Expanded source/resource/test inventories are not exact compiler dependency certificates. Source inspection review establishes no runtime behavior beyond the named executed fixtures.
- Both focus roles have named numeric warm/cold results and callback proof; auxiliary History, complete presentation attributes, near-circular/rotated/mixed-unit stress and all original admission branches remain required. Full current-source matching is false because Gumball changes occurred concurrently. Full Rust900 is a separate later observation, not proof that Runtime57 contains those later sources.

Next: Reproduce missing common GeometryResult on Circle associative History recompute/freeze/detach/Rebuild using existing native solvers and dependency machinery; implement only missing Rust contract/native pairing. Continue every original RCORE-01 requirement, full sharing/budgets/conversion/maps, remaining families and mouse/render acceptance.

[Machine evidence](2026-10-12-rhino-core-curve-focus-result.json).
