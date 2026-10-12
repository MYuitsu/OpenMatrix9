# RCORE-01 analytic support review — 2026-10-10

A fresh read-only reviewer inspected the Rust analytic value model and JSON
ABI, native OCCT extraction, root/common face payload reuse and real-host
fixture. No actionable extraction, ownership or validation defect was found.
The reviewer did not build or launch native tests. A final follow-up inspected
the changes below and the saved 174-check host report; no actionable defect
or regression was found.

The initial fixture covered ten analytic kinds, finite restrictions, native
locations and reversed TopoDS orientation, but did not deliberately create
indirect native frames or all four UV conics. The initial Rust rejection tests
also omitted malformed UV Z embedding, nonunit axes, unused positive radii,
ellipse major<minor and the cone angle endpoints. Those controls were added.

UV axes preserve native X/Y and embed Z=X cross Y. Therefore the determinant
of the embedded XYZ frame does not distinguish the two native XY senses.
The final schema has separate `uv_sense` (+1/-1 for 2D conics, null for lines
and 3D geometry) and `frame_handedness` (XYZ determinant sign; zero for lines
without transverse axes). A focused RED failed because `uv_sense` was absent;
the Rust implementation and final host fixture pass.

Eight authored Circle/Ellipse/Hyperbola/Parabola UV fixtures cover both senses
with noncanonical axes and origin. Sampled results are compared against the
authored native conic, independently of the descriptor. Five more fixtures
reflect the supporting analytic surface itself to obtain indirect native
frames, independently of topological face reversal. All preserve source BRep
and Undo/Redo. Native surface UReverse/VReverse reparameterization remains a
distinct untested case; reflection and TopoDS reversal do not establish it.

The reviewed scope is read-only metadata capture, not native shape regularity,
analytic-to-NURBS parameter mapping, conversion fidelity, world composition or
solid certification. Native conics may store zero radii/focal parameters;
accepting that metadata does not permit division by zero. No kernel constructor
or solver is replaced. Other support types and the full RCORE-01 audit remain
open.
