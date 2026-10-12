# Independent review — Mesh and PointCloud input gate

## Verdict

Approved for this bounded post-decode increment. The reviewer found no remaining
code-level blocker after inspecting the working tree and the numeric regression
cases. This review does not establish safety for `ONX_Model::Read`, decoder-time
native allocations, process-wide allocation, or aggregate worker memory.

## Verified behavior

- `applyPointCloudFields` preflights the live source arrays immediately before
  copying and permits an empty source for a field overlay. Malformed source
  storage rejects before the copy trace.
- A zero-capacity unmanaged mesh N-gon rejects before reading either index
  buffer. Curvature accepts a paired-NaN sentinel and finite `IsSet()` values;
  partial-NaN pairs reject.
- Nonfinite PointCloud points, normals and intensities reject before value
  traversal. Inactive planes are serialized without reading their native plane
  coordinates.
- Existing archive Mesh/PointCloud adapters remain authoritative. Existing
  Line/Polyline command/controller and `Part.makePolygon` construction are
  reused; no duplicate command or wire solver was added.

## Evidence follow-up

The reviewer observed that earlier validation records lagged the 92-check native
fixture and current plugin hash. After the review, the focused native fixture
passed 92/92, all six archive/block CTests passed, and a fresh shared-host run
passed 21/21 with exit code 0. That run loaded
`H:/FreeCAD-src/build/reusable-history-runtime/bin/OpenMatrix9Gui.pyd`, SHA-256
`85f1ed567d3a68097cb264972b4bee109719560365182fbc0a680b25279fbff2`; the host
SHA-256 is
`98eef899354a8aa84cfa2934e6e1a3c8558c7e6fff3b89c347c06731f06018a8`. The
result and launch record are retained in
`H:/FreeCAD-src/build/native-input-tree-mesh-cloud-current-plugin-20261010`.

The bounded source review is the reviewer’s judgment. The fresh host smoke and
its hashes are a subsequent validation run, recorded separately above.
