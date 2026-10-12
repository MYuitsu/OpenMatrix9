# RCORE-01 — Curve native booking, 2026-10-12T01:07:44+07:00

**Status: in_progress. Task2/3 and the full original chapter remain required.**

The bridge retains the returned transaction ID and owns a native lock while
creating/writing/recomputing outputs. Existing host APIs allow writes into that
booking but prevent premature commit/abort and replacement open. Each barrier
requires the same live booking/lock. Final commit/abort releases the owned lock,
passes the original ID and checks the commit verdict; cleanup never aborts a
different booking. Existing solvers, Rust session and result validation are reused.
This is a necessary native C++ transaction adapter; no FreeCAD root build was made.

## Actual executed evidence

- Runtime55 real commit callbacks: RED12/30, six commands, exit0/no fixture exception.
  Typed geometry rejection occurs too late: committed output remains and prior Redo is lost.
- Separate Runtime55 Line/abort: RED3/5, exit0/no fixture exception. Mutation outside the retired journal survives and the diagnostic is untyped.
- Runtime56 four close/reopen actions × six commands: GREEN120/120, all24 callbacks fire once. Complete document/Undo restoration, typed operation diagnostics and prior native Redo execution are independently checked.
- Matching Runtime56 regressions662/662 plus original/source/layer/replacement/hidden-layer cohorts590/590: total1372/1372 in24 actual host exits0.
- Five cold hosts reopen27 FCStd files;39 archives retained overall. Full Rust882/0/0 across126 suites; build exit0.
-1416 selected inventory inputs match postbuild/postnative. Shared SDK fingerprints match. Inventories are not exact compiler dependency certificates.

The first full exploratory witness timed out without a result and is excluded.
Its exact owned process was terminated by the runner with terminal=true; no RED
count or success is inferred from that run. Subsequent isolated commit and abort
runs completed and establish the stated failures. Current full close/reopen120
uses the same native operation fixture with progress instrumentation.

The original review finding is resolved for these named operations. Original
review observations/declined judgments remain historical, and the new
[disposition](../reviews/2026-10-12-curve-booking-disposition.md) records the native
reason and limits. This does not certify arbitrary document-close callbacks,
every global notification or publication/lifetime across all families. Offscreen
tests do not certify mouse/OpenGL/render or production deployment.

- Every original RCORE-01 requirement remains required: adapters1–7, C01–C12, T01–T11 and every enabled family/consumer, GeometryRef/Snapshot/Result, rational/periodic basis, trims/pcurves/seam occurrences, independent read/edit/retain/export.
- Task2 remains open: generic native snap/pick provenance, all original numeric/native parameter paths, full subset/index sharing across selection observations, aggregate allocation/traversal/worker/cancellation bounds. Replay currently duplicates construction sources/events in the common result; complete sharing and aggregate cost remain required.
- Task3 main direct Curve results are paired, but Ellipse foci and Circle History recompute/freeze/detach/Rebuild result lifecycle still require implementation and independent native/cold evidence.
- Named Curve output/source/layer/context/cross-family replacement and native commit/abort/reopen are checked. Every native document-close boundary, global observer close/lifetime, aggregate rollback across all enabled families and the full original publication contract remain required.
- Analytic-to-NURBS global deviation/locus and parameter mapping, complete topology maps and output attributes remain unproven. Native validity/master/physical dimensions are scoped observations; conversion exactness remains uninspected and maps null.
- Tangent/native_solution, changed CPlanes/orientation, every numeric mode and explicit non-Circle Undo, repeated/shared sources and revision exhaustion are not established by the six focused Rust tests/nine new native recipes.
- Offscreen evidence does not establish actual mouse, every viewport preview lifetime, OpenGL/render acceptance or all original cache/resource invariants. Production ABI/deployment acceptance remains pending; selected App/GUI host reuses the one SDK.
- Expanded source/resource/test inventories are not exact compiler dependency certificates. Source inspection review establishes no runtime behavior beyond the named executed fixtures.

Next: Pair Ellipse auxiliary focus outputs with the common owned GeometryResult and original construction/source evidence, native vertex/master/attributes/units and checked publication. First reproduce missing focus results, then implement Rust role/output admission using the existing solver. Continue Circle History/Rebuild, full sharing/budgets/conversion/maps and every original RCORE-01 requirement.

[Machine evidence](2026-10-12-rhino-core-curve-booking.json).
