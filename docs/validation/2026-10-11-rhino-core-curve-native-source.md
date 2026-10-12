# RCORE-01 — Missing Curve native source witness, 2026-10-11T21:08:00+07:00

**Status: in_progress. All original RCORE-01 requirements remain required.**

Runtime45 retains numeric values at Rust entrypoints, but complete accepted GUI
input and native source ownership remain missing. The unchanged existing solvers
were exercised by a new independent native fixture:18 checks,9 passed/9 failed,
actual host exit0 and no fixture exception. This RED is separate from the previous
466/466 scoped native regression cohort; it does not lower or enlarge that cohort.

| Witness | Already passes | Missing evidence |
|---|---|---|
| Circle AroundCurve | Radius2, center(10,20,35), one Undo/Redo, warm FCStd bytes | Exact `OnCurve=0.2500`, source identity/revision/frame, full compound parent master and attributes |
| Ellipse AroundCurve | Major5/minor3, same world center, one Undo/Redo, warm FCStd bytes | Same original parameter/source contracts |
| Circle FitPoints branch | Radius1, center(10,20,30), Undo removes radius2 batch, one Undo/Redo, warm FCStd bytes | Exact native `Selection`, surviving source/range identity, full accepted cloud/attributes/world points |

The source path is a two-edge compound. Reconstructing only selected Edge1 cannot
pass the full-parent source check: its total parent length must remain120 and both
edges must survive. The cloud has local XYZ and translation(10,20,30); expected
selected world points are(11,20,30),(10,21,30),(9,20,30). RemovedCloud must not appear
in the surviving source references after Undo. These are geometry and branch
expectations, not metadata field-presence checks alone.

Task2 will reuse existing snapshot/topology/source-attribute/stamp/discrete/fit
capture APIs. Rust must validate owned records before accepting mutable state.
The proposed persisted envelope uses `source_refs` and `source_snapshots` alongside
the constructor value, compatible with the existing common result conventions;
`accepted_input_events` must retain the original native text. Parent geometry
and selected subelement/ranges remain distinct. Source copies are immutable,
indexed sharing is bounded, and original GUI text attaches to the same accepted
context. C++ remains the required FreeCAD/OCCT capture and GUI-thread adapter.
Full native History point/direction/radius/tangent sources remain required too.

Parallel Gumball/Rust module-root changes invalidate complete current-checkout
matching; this witness used the preserved, hash-verified Runtime45 plugin. Native
sources remain RED regardless of those unrelated changes. Offscreen evidence
does not certify actual mouse/OpenGL/render or complete chapter acceptance.

- Every original RCORE-01 requirement remains required: GeometryRef/Snapshot/Result, rational/periodic basis, UV trims/pcurves/seam occurrences, complete attributes, every enabled consumer, independent family read/edit/retain/export, adapters1–7, C01–C12 and T01–T11.
- Common produced GeometryResult now enforces bytes16MiB/nodes1million/depth64 before serialization/caller-buffer copy. Source-envelope admission is still necessary only: complete request/output overhead reservation and aggregate allocation/traversal/worker/cancellation bounds remain required.
- Global conversion/deviation bounds and complete parameter/topology maps remain unproven. Valid closed native Solid is not a certificate of these contracts.
- Curve retains owned pre-normalization values at Rust entrypoints: exact text/units/frame/previous/pending options, original mouse points, native reference values and whole FitPoints batches with Circle branch Undo. Full native source identity/revision/attributes/snapshots, GUI-native parameter text such as OnCurve fractions, common ConstructCurve GeometryResult, checked publication, Ellipse foci, Circle History and Rebuild result pairing remain required.
- Offscreen evidence covers native kernel, command, Qt keys, callbacks and scoped FCStd persistence only. Actual mouse, OpenGL rendering and viewport acceptance remain required; historical graphics cohorts are not current reruns.
- The two cold hosts verify five named recipes and one indexed54-reference result, with OM9 imported by bootstrap. Unsignalled native root-frame mutations, actual Solid preview leases and complete original lifetime/cache invariants still require evidence.
- Production deployment/ABI acceptance remains pending. An isolated App19/camera-guard GUI host reuses the single shared SDK; no root FreeCAD build or full runtime copy was made.
- Runtime45 native source witness is RED9/18: raw native token, immutable identity/revision/frame/ranges, and full parent native source snapshots/attributes are absent for Circle path, Ellipse path and Circle FitPoints after whole-batch Undo. Implement these without replacing existing solvers.
- All non-Gumball/non-lib.rs frozen inputs match. Gumball and Rust module-root files changed in the parallel gumball chat; complete current checkout/build correspondence is unproven. The 448 rows are a pre-build inventory, not an exact compiled-input certificate.

The [machine checkpoint](2026-10-11-rhino-core-curve-native-source.json) preserves
fixture, reports, launch, selected binaries, warm archives and prior scoped proof.
No implementation or completion status was promoted by writing this witness.
