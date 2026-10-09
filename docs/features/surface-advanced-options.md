# Native Surface advanced options

These host algorithms extend OM9-SURFACE-001/003/009. Source behavior and
defaults were read in Matrix 8 Book 1 PDF180–186 and193–195; the particular
FreeCAD algorithms and bounds below are OpenMatrix9 decisions.

## Section Refit

Sweep1, Sweep2 and Loft expose Refit and a millimetre tolerance, initially
0.01 mm, range 1e-7–1e6 mm. Rust proposes a degree-at-most-three spline from
1025 arc-length samples, increasing control points up to 256. Closed sections
remain periodic; sources are copied and preserved.

Sample error alone cannot accept the curve. Native acceptance subdivides each
original edge, bounds positive-weight trimmed NURBS against their endpoint
chords, and adds both hull flatness bounds and corresponding endpoint distances.
This bounds both directions of continuous curve deviation in exact arithmetic.
Native conversion/arc-length arithmetic uses coordinate-scaled numerical
allowances; this is an OCCT numerical bound, not formal interval arithmetic.
Reject nonpositive weights, unresolved conversion, excessively tight tolerance,
or subdivision beyond 16384 nodes/depth24. No sampled-only fallback is committed.
This fits cross-sections; it does not implement Refit Rail.

## Sweep2 G1/G2

Each rail has independently selectable G0/G1/G2. G1/G2 need an explicit
surface `EdgeN` belonging to exactly one supporting face with a parameter curve.
Bare curves, chains and edges shared by two solid faces cannot supply a unique
support. Inputs must be open single-edge rails and 2–32 open nonrational
Line/Bezier/BSpline profiles with matching degree, poles and normalized knots.
Original cross-section mode is required; Closed, fitting, Maintain Height and
Add Slash cannot be combined with this bounded constraint solver.

Original profiles must meet both rails. Their contact tangents must lie in the
requested support tangent plane; G2 also requires compatible profile normal
curvature. OCCT filling constrains the boundary/support faces and original
sections. The installed SDK plate API takes order integers 1/2 for tangency/
curvature; its shape-enum ordinal differs, so the adapter documents that bridge
and independently verifies the resulting curvature.

Acceptance checks contact at 65 points per original curve, trimmed-face
membership, and support/output normals at 63 interior points per constrained
rail. G1 angle tolerance is 2e-3 rad. G2 compares the second fundamental forms
in three independent tangent directions, tolerance
`1e-3 * max(1, abs(sourceCurvature), abs(outputCurvature))` per mm. Contact
tolerance is 1e-4 mm. These are bounded numerical checks, not global continuity
certificates. Incompatible constraints leave no output and can be corrected.

## Loft tangent matching

Match Start/End Tangents independently use the first/last selected supporting
surface edge. Supported styles are Normal/Tight, with 2–32 open nonrational
single-edge profiles, original section mode and no Closed Loft. Cubic Hermite
side connectors use endpoint tangents in the support tangent planes; native
filling preserves the original profiles. Contact and normal tests are independent
of the solver's success flag. Other style/fitting combinations fail explicitly.

## Sweep2 Add Slash

For one profile and two open rails, click Add Slash and pick rail A then rail B;
Enter finishes picking without committing. The table supports removal. CMD
`AddSlash=a,b` uses interior normalized arc fractions; `ClearSlashes` removes all.
At most 64 pairs must increase strictly on both rails. Crossing, duplicate,
endpoint and nonfinite pairs are rejected while retaining the last valid state.

Rust supplies a piecewise-linear correspondence with fixed endpoints (0,0)/(1,1).
Native exact arc-length locations drive transported section frames, including a
section at every slash station. The ordinary 65 stations also remain. Native
loft joins them and verifies rail/profile contact. Slash therefore changes the
constructed surface rather than merely drawing a guide. Multiple profiles,
closed rails and G1/G2 combinations remain unsupported.

## Associative Surface History

The options checkbox creates `OpenMatrix9Gui::SurfaceHistory`, a native
`Part::Feature` subtype. CMD aliases `gvSweepHistory` and `gvSweep2History`
force History and retain exact IDs OM9-SURFACE-002/004. Plain commands default
to snapshots unless History is checked. There are no dedicated History menu
entries in the current menu catalog.

Ordered `SourceCurves`, versioned JSON `SurfaceOptions`, and hidden parent frame
links persist. Source shapes, placements, parent placements and regrouping touch
the dependent surface. All supported geometry options rebuild through the same
adapter. `UpdateHistory=false` suspends updates while retaining links; enabling
it resumes. Directly editing the result Shape detaches its parents and disables
History, while its children remain dependent; native Undo restores the links.
Invalid/deleted inputs or malformed settings clear stale geometry and report an
error. Self/descendant cycles and foreign-document inputs are refused.

History aliases default Closed to Yes only for closed rails with at least two
profiles. Sweep2 History defaults Maintain Height to Yes for one profile; the
multiple-profile host solver cannot provide this option. Reverse/Flip and seam
alignment reuse the existing input table. These bounded choices do not establish
complete Matrix History, global Record/Update controls or persistent topological
renaming compatibility. Fresh-process FCStd restore loads the native type and
recomputes after source changes without activating the workbench.
