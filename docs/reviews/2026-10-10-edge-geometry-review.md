# RCORE-01 edge/vertex geometry review — 2026-10-10

Reviewer `/root/edge_geometry_review` inspected the Rust-owned rows/ABI, native
reader, common topology integration, support reuse, frames and runtime fixtures
read-only. No builds, hosts, edits or child reviewers were performed by the
reviewer. The review does not close the full original RCORE-01 contract.

The first review identified a possible root-frame inconsistency for a standalone
located degenerate edge with no Curve3D. The concern came from another OCCT source
version resetting the Curve location output when no support exists. Controller
runtime tests on the actual shared SDK found both a sphere pole retaining pcurve
data and a bare edge without any CurveRepresentation kept the correct root frame.
**Frame loss was not reproduced on this SDK.** Both roots did reproduce a narrower
defect: `geometry_data_status` remained `uninspected_support_type` although the
owned edge row established `absent_degenerate_curve3d`. The retained native RED
reports fail specifically on that absence-status assertion, after frame checks
pass. The reports also establish null restriction domains and native process exit0.

`curveFacts` now copies the already validated edge row's native location, payload
and status before its null return, and reads restriction domains only for present
Curve3D. This corrects the reproduced absence status and removes reliance on an
absent-support native out-parameter for location. No implicit curve reconstruction,
healing, source mutation or invented zero-length support/domain was introduced.
Tests cover direct source/parent rotations and translations, both LinkTransform
settings with mirror/nonuniform scale, root/edge frames and native endpoints,
explicit null support/domain, BRep/history preservation and owned returned values.

The follow-up review found no remaining actionable source defect in this increment.
The earlier apparent collection-budget gap is protected by NativeInputTree's
occurrence preflight before any topology staging; unique edge/vertex rows then pass
the independent Rust combined-row/retained-byte gate before allocation/slices.
IDs/endpoints, finite tolerances/coordinates, transforms and flags validate before
Rust cloning. Bounded JSON finishes before output copying, including unchanged
sentinel output on invalid values or count-valid serialized overflow. Vertex Pnt
extraction removes location from a detached view, leaving native source untouched.

Runtime GREEN, exact source/runtime fingerprints and regression results belong to
the [validation record](../validation/2026-10-10-rhino-core-edge-geometry.md).
The review itself is source evidence, not a runtime success claim.

Original obligations remain: global edge/pcurve correspondence and native parameter
reversal provenance, manifold/solid/cavity certificates, typed reports/capabilities,
arrays/reference contexts, full attributes and discrete channels, raw decoding,
global/native/Qt allocation and peak staging, all workers/commit/lifecycle boundaries.
Live initialized immutable FFI storage and nonoverlapping output remain caller
obligations. No whole-program memory-safety or full-chapter completion claim follows.
