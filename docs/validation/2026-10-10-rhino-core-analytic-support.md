# RCORE-01 owned analytic support — 2026-10-10

The common descriptor now carries owned native analytic support for Line,
Circle, Ellipse, Hyperbola, Parabola, Plane, Cylinder, Cone, Sphere and Torus.
Safe Rust validates finite values, dimensions, orthonormal frames and typed
parameters, then serializes versioned JSON. Native C++ only reads the existing
OCCT geometry. Curve 3D, face and UV pcurve readers reuse the same model;
root/common face payloads are shared. Existing constructors and solvers remain.

The selected plugin passed **174 analytic +54 NURBS +177 topology +151 descriptor
+55 import =611 FreeCAD checks**, all with process exit0 and matching current
plugin/host hashes. Full Rust passed **598 tests**. Plugin SHA256:
`574cff6f032317d839c32c6d0f54d275e9880b140ee35777445edced3ea56652`. [Machine evidence](2026-10-10-rhino-core-analytic-support.json)
records source/runtime/log fingerprints and the RED outcomes. The original
host RED failed because Line support was null; a later focused Rust RED failed
because the separate UV sense was missing.

The 174-check fixture authors all ten kinds, samples native curve/surface
parameters and compares independent analytic formulas with native evaluation.
It preserves finite edge/face restrictions separately from natural domains,
native-local origin/axes separately from location, and face orientation. Exact
source BRep and Undo/Redo stay unchanged; returned data survives replacement
and document close. Five supporting surfaces are reflected to create indirect
native frames. Eight deliberately authored UV conics cover both senses of
Circle/Ellipse/Hyperbola/Parabola with noncanonical axes and origin; their
samples match the authored native curve rather than another descriptor field.

`frame_handedness` describes the embedded XYZ determinant (+1/-1, zero for a
line with no transverse axes). UV conics embed Z=X cross Y, so their XYZ
frame is direct for either UV sense. `uv_sense` separately reports native XY
orientation (+1/-1); it is null for lines and 3D geometry. Natural unbounded
domains are explicit strings, not fabricated finite kernel sentinels. Geometry
uses `occt_native` parameter convention; no conversion map is inferred.

The 128-byte input ABI contains values only, with no retained native handles or
borrowed input pointers. Caller-owned output uses the existing query/copy ABI,
short-buffer rejection is atomic, and copied supporting payloads share the
8 MiB aggregate limit. This is not a bound on expanded Qt aliases or global
native allocations. Native conics may store zero radii/focal values; metadata
capture does not certify regularity or permit division by zero. Qt normalizes
signed zero, so the retained geometry is numeric rather than IEEE bit exact.

[Fresh review](../reviews/2026-10-10-analytic-support-review.md) found no
actionable defects; its initial schema ambiguity and validation gaps were
addressed. Native surface UReverse/VReverse reparameterization is a distinct
untested case; reflection and topology reversal do not establish that coverage.

**Full RCORE-01 remains in progress.** Bezier/offset/extrusion/revolution
support, world composition, source attributes, complete 3D edge/pcurve
correspondence, full solid/manifold/cavity certificates, conversion/error
reports and remaining native input/worker/commit requirements are required by
the [original full audit](../reviews/2026-10-10-rcore01-closure-audit.md).
Analytic metadata copying does not close analytic allocation/input-tree gates.
