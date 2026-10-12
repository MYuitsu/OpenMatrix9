# RCORE-01 analytic support snapshots

Continue the full chapter requirement for owned supporting geometry. Reuse
OCCT Line/Circle/Ellipse/Hyperbola/Parabola and Plane/Cylinder/Cone/Sphere/Torus
metadata; do not replace constructors or silently convert them to NURBS.
Rust owns a validated value-only analytic model and versioned JSON. Native C++
only reads native frames/parameters and uses the existing caller-output ABI
and aggregate support budget. Python is only the authored real-host fixture.

OM9 schema choice: native-local origin and explicit orthonormal axes (including
indirect surface frames), typed parameters and `occt_native` parameter convention.
Line keeps its native unit direction without inventing transverse axes. Natural
domains distinguish unbounded support from finite edge/face restrictions. UV
curves embed their native 2D frames in XYZ with zero Z controls/origin; they stay
`face_uv`, not world coordinates. No parameter-map or world composition claim.
The embedded frame determinant is distinct from native XY sense: UV conics
carry `uv_sense` (+1/-1), while lines and 3D supports have null UV sense.
`frame_handedness` describes XYZ; lines have zero because no transverse axes
are invented. Reflected supporting surfaces test real indirect native frames.

ABI:128-byte repr(C) value {kind,dimension,origin,three_axes,two_radii,extra};
no input pointers or retained native lifetime. Outputs use the existing synchronous
JSON query/copy protocol, with rejection atomicity. Rust validates finite bounded
values, native dimensions, frame orthogonality, per-kind parameter constraints
and unused fields before serialization; no silent normalization.

- [x] RED host: existing analytic face/curve/pcurve geometry_data is null.
- [x] RED Rust: typed values, direct/indirect/UV frames, invalid kinds/NaN/
  degenerate axes/parameters and output atomicity.
- [x] Rust owned analytic model/JSON and pointer-free input ABI.
- [x] Native analytic metadata extraction for 2D/3D curves and surfaces;
  preserve legacy NURBS companion fields and other uninspected support types.
- [x] Real host sampled evaluation against native analytic geometry, reversed/
  located/trimmed data, source BRep/Undo/Redo invariants and NURBS regressions.
- [x] Fresh review, full Rust, matching plugin/runtime, audit and progress sync.

Unsupported Bezier/offset/extrusion/revolution families, complete 3D edge/pcurve
correspondence, attributes, world composition, solid validation and all other
original RCORE-01 requirements remain required and open.

Native UReverse/VReverse reparameterization remains distinct from the tested
reflection and TopoDS reversal cases. See the
[read-only review](../../reviews/2026-10-10-analytic-support-review.md).

Accepted evidence: 174 analytic +54 NURBS +177 topology +151 descriptor +55
import =611 host checks, matching plugin/host fingerprints and process exit0;
full cargo test passed 598 tests. This closes this plan's metadata increment,
not the original RCORE-01 chapter. See the
[evidence record](../../validation/2026-10-10-rhino-core-analytic-support.md).
