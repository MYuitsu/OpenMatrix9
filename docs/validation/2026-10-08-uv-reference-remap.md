# Deferred UV owner serialization/copy/remap — 2026-10-08

Eight independent CurveOnSurface fixtures use a C2 UV reference to a separate
two-dimensional owner: line or quadratic NURBS, bare PolyEdgeSegment or
one-segment PolyEdgeCurve, both directions. Proxy domain14–20 inside owner11–23,
evaluation31–47, plane at(10,20,30). Seventeen independent physical samples
include interiors, not only endpoints. This expands the earlier direct-UV seam
and standalone/mixed3D-owner reference matrices.

The regression exposed a serialization gap: reading resolved the owner, but
the derived export model still held a deferred proxy. CurveOnSurface::Write
requires IsValid and a live2D C2. The writer now resolves the current graph after
namespace alias/copy, temporarily swaps linked owning children for serialization,
and restores the detached tree without allocation on success or failure. Graph
ownership covers the synchronous write. No source archive is modified. Known
UserStringList clone copy-count fields are restored to their original values;
payload byte comparisons remain strict. Opaque clone userdata is refused.

Native **8/8 passed895 checks**: decoded class/UUID/domain/reversal/metadata,
dependency closure, repeated namespaces and copied roots, independent17-point
evaluation at1e-12mm, unchanged source bytes and atomic publicV5 refusal.
Installed FreeCAD **8/8 passed272 checks**: repeated import/copy, correct current
owner metadata, deleted/changed-owner refusal, Undo/Redo and source-deleted FCStd
reopen. Internal SDK80 current archives are frozen separately from originalV5
source fixtures. Internal staging does not establish target compatibility.

First attempt found a fixture plane-domain mismatch; the corrected fixture then
reproduced the actual writer failure. Logs retain both separately. Native SDK
pin remains eb92af3ba1806b0a34a99aba0d3bda83e3d46083. No SDK modification,
mesh conversion, metadata removal or tolerance relaxation was introduced.

Final regression **48/48 native suites;37 installed FreeCAD reports/5,696
checks**, tools42/42 passed. All applications refreshed serially on the new
module. Both ring fixtures retain0.001mm bounds and existing area/volume limits.

Actual Rhino5 GUI Open/SaveAs/reopen is pending for this eight-file matrix.
Public V5 remains guarded until actual target data/class/reference comparisons
establish compatibility or scoped incompatibility. Display/CAD owner geometry
editing and general seam/singular/global correspondence remain open. One
prepared target batch;7 packages2–8 unclosed; total future batch count unknown.
No primary integration or push. Evidence is in uv-reference-remap-20261008,
bound by current-source-binding.json and the current regression proof.
