# OM9-SURFACE-009 — Loft

Menu ID: `OM9_SurfaceLoft`. CMD alias: `Loft`.
Verified supported slice: 2026-10-09; full feature remains partially implemented.

## Reference and implementation contract

Matrix 8 Book 1 PDF pp.193–195 (printed pp.183–185) were read through the Loft
continuation. Shares [Sweep 1](OM9-SURFACE-001.md)'s profile selection,
Reverse/seams, Automatic/Natural, section Rebuild, Preview and lifecycle contract.
No rails; at least two ordered sections, all open or all closed.
Closed Loft connects last to first and requires three or more sections.
Host algorithms below do not claim exact proprietary solver equivalence.

- Normal (initial host style): native smooth `Part.makeLoft`.
- Straight Sections: native ruled loft.
- Loose: OCCT's profiler harmonizes native single-edge NURBS profiles; their
  exact control rows/weights form a uniformly knotted V control net. Internal
  sections pull away rather than being interpolated. Closed V is periodic.
  Multi-edge profiles require explicit section Rebuild.
- Tight: native section interpolation with centripetal parameterization,
  maximum degree three and unchanged explicit profile correspondence.
- Uniform: harmonize single-edge U profiles; Rust globally interpolates the
  homogeneous coordinates and weights on genuinely uniform distinct V knots.
  Reject nonpositive resulting weights. U distinct knots must be uniformly
  spaced after harmonization, otherwise enable section Rebuild explicitly.
  Multi-edge profiles also need Rebuild. Closed V is periodic. Host bounds:
  256 sections, 4096 U poles and `U_poles * sections^3 <= 50,000,000` to bound
  dense solves; reduce sections/Rebuild count when exceeded.
- Developable: separate native ruled surface for each consecutive profile pair.
  Reject two nonparallel straight profiles. Each face must have defined
  curvature and `abs(GaussianCurvature)*max(1 mm², faceArea) <= 1e-7` at 81
  interior samples. Any failing pair rejects the whole operation.
  Closed Developable is unsupported. This bounded check is not an unroll
  certificate or a general developable-surface solver.
- Output must be valid, contain faces and no solids. Independent source copies
  are used for construction. History OFF creates a snapshot; History ON creates
  a native associative feature.

## Advanced options

Match Start/End Tangents use eligible supporting face edges with open nonrational
single-edge profiles, Normal/Tight styles and original section mode. Refit adapts
the section spline to a finite millimetre tolerance and applies continuous native
chord/hull deviation bounds. The History checkbox persists inputs/settings and
recomputes from original shape/placement changes. See the precise
[algorithms, bounds and lifecycle](surface-advanced-options.md).

## Remaining options

SplitAtTangents, Point endpoints, dragged seams and exhaustive profile correspondence remain
unsupported. Complete Matrix/Rhino geometry equivalence remains unverified.
Tangent matching with closed/rational/multi-edge sections, other styles or
section fitting remains unsupported; numeric Refit bounds are not formal
floating-point certificates.

## Validation

Tests measure Loose interior deviation, Tight/Uniform interpolation, six-row
uniform knot spacing, periodic Uniform, rational weighted profiles, nonuniform-U
rejection/Rebuild recovery, separate Developable pairs and analytic area,
nonparallel-pair recovery, source preservation and lifecycle/persistence.
See [the validation record](../validation/2026-10-09-surface-options.md).
Advanced continuation: [constraints, Refit, Slash and History validation](../validation/2026-10-09-surface-constraints-history.md).
