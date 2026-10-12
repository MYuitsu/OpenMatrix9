# RCORE-01 owned native3D edge/vertex geometry — 2026-10-10

**Scope:** Rust-owned native3D edge/vertex/tolerance tables joined with UV topology, loose/standalone Part geometry and exact-once direct/group/link frames. Full original RCORE-01 remains in progress.

Rust now owns the native3D edge and vertex rows associated with the common
face/loop/pcurve graph: local IDs, canonical endpoints, restriction domains,
tolerances, locations and observed Degenerated/SameParameter/SameRange flags.
Vertices retain raw native-local XYZ, separately from their occurrence location.
Native enumeration indices are relative to the inspected shape; they are not
persistent topological names. Face tolerances use the same finite bounded policy.

The minimal OCCT reader reuses native shape storage and existing owned curve
serializers. It unwraps trimmed3D supports and reads NURBS/Bezier/analytic/derived
data without rebuilding the native curve. All Part representations1–7 now use
the shared topology capture, including standalone points, curves, wires and loose
mixed-compound geometry. Standalone roots reuse the owned row's geometry/status/
location. Missing nondegenerate Curve3D rejects; degenerate absent support retains
null geometry/domain with an explicit status. No native healing or source edit
occurs. Existing Line/Polyline/kernel/solvers remain reused.

Counts gate before staging/slices; IDs/endpoints, coordinates, tolerance, domains,
flags and matrices validate before Rust cloning. Edge192/Vertex168/Input32 byte
ABI layout/alignment checks and live output canaries cover invalid input, short
output and count-valid serialized overflow. Combined edge+vertex limit16384 and
8MiB retained/serialized numeric budgets apply; support payloads share the existing
aggregate bound. These are not process/global/native/Qt peak-allocation certificates.
Serialized overflow follows bounded staging/cloning. FFI initialized immutable
storage and nonoverlapping writable output remain caller obligations.

**632 full Rust tests and 1937 FreeCAD checks passed** on one
matching plugin, all12 actual host process exits0:921 new edge/vertex checks and
1016 discrete/cold/instance/world/support/topology/descriptor/import regressions.
Only OpenMatrix9Gui was built against the existing shared SDK,14 workers; the
FreeCAD host was unchanged. Plugin SHA256:`029b0b26da2babdafad4fc860244fc38a0993ffd4adf3875958f264670b30fda`.
[Machine evidence](2026-10-10-rhino-core-edge-geometry.json) records source/native
dependency/test/helper/report/log/runtime fingerprints.

Native fixtures compare numeric rows and exact-once world points with independent
OCCT/FreeCAD curve, surface, vertex and matrix queries. They cover holes, seams,
sphere poles, cavities, rational trimmed NURBS, Bezier, standalone point/curve/wire,
loose mixed topology and mirrored/nonuniform App::Link with both LinkTransform
settings. An SDK-matched test-only authoring helper supplies differing UV/3D
domains with false native flags, missing nondegenerate3D support, an out-of-policy
vertex and both pcurve-retaining/bare located degenerate edges. False flags and
separate domains remain separate observations; explicit native rescaling only
supports the authored sampled locus checks. No global correspondence certificate
is invented. Inspection/rejection preserves BRep bytes and Undo/Redo; returned
owned values survive replacement/close.

The reviewer raised possible absent-support frame loss from another OCCT version.
Current-SDK RED checks reproduced wrong root absence status, while both frame
checks passed. Root metadata now comes from the owned edge before null return;
the final direct/parent/link endpoint/status/domain checks pass. The follow-up
[read-only review](../reviews/2026-10-10-edge-geometry-review.md) found no remaining
actionable source defect in this increment. Existing SDK OCCT deprecation warnings
remain in plugin build logs; the native test-helper build required no host rebuild.

**Full original RCORE-01 remains in progress.** Global edge/UV correspondence,
native parameter reversals, manifold/solid/cavity certification, complete reports,
uniform typed capabilities/errors, arrays/reference contexts, attributes/discrete
channels and all original allocation/worker/mutation/lifecycle gates remain in the
[completion audit](../reviews/2026-10-10-rcore01-closure-audit.md). Read-only geometry
copying and finite sampled comparisons do not establish full Rhino parity.
