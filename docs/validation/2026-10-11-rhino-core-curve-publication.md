# RCORE-01 — Checked Curve publication, 2026-10-11T23:49:14+07:00

**Status: in_progress. Task2/3 and the complete original chapter remain required.**

The existing six Curve solvers are reused. The native bridge freezes owned expected
CAD master digest, root/local/world frame, orientation and metadata before output
callbacks; it checks native source content, units, active layer and session tokens
at publication barriers and uses the selected host's transaction that preserves
prior Redo. Rust still owns request/result/frame validation and command generation.
Cancellation now retires the generation for explicit and command-input cancellation;
exhaustion invalidates tokens permanently without wrapping. Failure cleanup remains
bound to the original generation, preserving a replacement Box session.

The layer bridge requires the captured owner (not merely any remaining owners),
the exact assignment index and explicit output permissions. Native units/layer
observation errors now identify the actual operation as StaleGeometry. Output
Visibility/LineColor/Selectable are checked against the existing bridge's values.
Layer Off still accepts invisible outputs and preserves Selectable permission.

## Executed evidence

- Original output/session callbacks: Runtime52 RED42/210 → Runtime55 GREEN210/210.
- Full native source callbacks: Runtime52 RED12/60 → Runtime55 GREEN60/60.
- Selected layer, index, lock, units and view callbacks: Runtime53 RED78/210 → Runtime55 GREEN210/210.
- Cross-family Box replacement, with/without later Shape tampering: Runtime54 RED33/60 → Runtime55 GREEN60/60.
- Healthy hidden-layer outputs and FromFoci/MarkFoci: Runtime55 GREEN50/50; independent geometry/focus positions, Selectable, membership and one native Undo/Redo.
- Original matching Curve/Solid/Edit regressions: 662/662, including five cold hosts opening27 FCStd files;39 archives retained overall.
- Total current native1252/1252 across23 actual host exits0, no fixture exceptions. Full Rust882/0/0 across126 suites, actual exit0; focused cancellation RED0/1 → GREEN1/1; build exit0.
-1415 selected inventory inputs match after build/native. This inventory is not a compiler dependency certificate. Shared SDK fingerprints remain unchanged.

All callback rejection cases independently verify document/Undo restoration and
prior native Redo execution, or completion of the preserved replacement session.
The foci fixture proves geometry and transaction behavior; it does not establish
the still-missing common GeometryResult for auxiliary focus outputs.

Two initial exploratory fixtures are preserved but excluded: native units were
first addressed as a Python document attribute, and early Shape tampering was
overwritten by normal output assignment. Corrected RED fixtures match GREEN copies.
Runtime54's earlier green results are historical to that runtime, not substituted
for the final Runtime55 suite.

## Review and remaining work

Three actionable review findings now have source fixes and matching witnesses:
cross-family cancellation, required layer ownership and typed native context.
The P1 transaction booking/premature close finding remains OPEN. Scope and every
declined judgment are retained in [review disposition](../reviews/2026-10-11-curve-publication-review.md).
C++ is the necessary native document/property/group/view bridge; no mutable host
object crosses into a Rust worker. No FreeCAD root build or runtime clone was made.
Offscreen evidence does not certify mouse/OpenGL/render or production deployment.

- Every original RCORE-01 requirement remains required: adapters1–7, C01–C12, T01–T11 and every enabled family/consumer, GeometryRef/Snapshot/Result, rational/periodic basis, trims/pcurves/seam occurrences, independent read/edit/retain/export.
- Task2 remains open: generic native snap/pick provenance, all original numeric/native parameter paths, full subset/index sharing across selection observations, aggregate allocation/traversal/worker/cancellation bounds. Replay currently duplicates construction sources/events in the common result; complete sharing and aggregate cost remain required.
- Task3 main direct Curve results are paired, but Ellipse foci and Circle History recompute/freeze/detach/Rebuild result lifecycle still require implementation and independent native/cold evidence.
- Checked Curve publication is partial: named output/source/layer/context/same-Curve/cross-family replacement fixtures pass, but native transaction booking identity, premature callback commit/abort/reopen, every callback close boundary and full aggregate rollback remain required. No transaction-ID or document-close certification is inferred.
- Analytic-to-NURBS global deviation/locus and parameter mapping, complete topology maps and output attributes remain unproven. Native validity/master/physical dimensions are scoped observations; conversion exactness remains uninspected and maps null.
- Tangent/native_solution, changed CPlanes/orientation, every numeric mode and explicit non-Circle Undo, repeated/shared sources and revision exhaustion are not established by the six focused Rust tests/nine new native recipes.
- Offscreen evidence does not establish actual mouse, every viewport preview lifetime, OpenGL/render acceptance or all original cache/resource invariants. Production ABI/deployment acceptance remains pending; selected App/GUI host reuses the one SDK.
- Expanded source/resource/test inventories are not exact compiler dependency certificates. Source inspection review establishes no runtime behavior beyond the named executed fixtures.
- P1 transaction review finding remains OPEN: store/check the original booking, prevent premature retirement, and abort only that booking; ID detection alone cannot restore an already retired journal. Reproduce native commit/abort/reopen before implementing with existing host APIs.

Next: Reproduce native premature commit/abort/reopen in the Curve metadata callback. Bind checked publication to the returned booking ID, prevent callback retirement and abort only the original transaction using existing native capabilities; retain real prior Redo and replacement sessions. Then complete foci common GeometryResult/History/Rebuild, full sharing/budgets/conversion/maps and every original RCORE-01 requirement.

[Machine evidence](2026-10-11-rhino-core-curve-publication.json).
