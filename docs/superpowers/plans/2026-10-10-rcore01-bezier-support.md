# RCORE-01 owned Bezier support

Continue the original supporting-geometry requirement. Existing native
`NurbsBasisAdapter::checkedBasis` overloads already read Bezier curve, surface
and UV controls without converting the native geometry; reuse them. Rust owns
the validated portable control net and serializes an explicit Bezier kind.
No kernel constructor or solver is replaced.

OM9 schema: `bezier_curve` / `bezier_surface`, original Cartesian controls and
optional rational weights, degree(s), natural full domain [0,1], U-major surface
layout. Native restriction, location and orientation remain separate. The
adapter's clamped two-knot basis is a validation representation, not a claim
that the native support changed type. A Bezier wrapper must reject an arbitrary
multi-span/periodic/reparameterized NURBS basis before owned cloning.

Reuse existing NativeCurve/NativeSurface ABI layouts and basis envelope checks;
add explicit Bezier query/copy entrypoints. No pointers or native handles escape.
Same support budgets and output atomicity as NURBS. Native C++ only extracts
through the existing checked adapter and dispatches the typed Rust serializer.
Python is only the independent Bernstein/native host test oracle.

- [x] RED native descriptor lacks Bezier curve/surface/pcurve payload.
- [x] RED Rust descriptor rejects explicit Bezier form; owned/ABI API absent.
- [x] Owned Bezier wrappers reject non-Bezier basis and preserve control nets.
- [x] Shared bounded ABI, native curve/face/pcurve integration and form facts.
- [x] Real-host rational/polynomial Bernstein samples, trim/location/orientation,
  source/history/lifetime controls and existing support regressions.
- [x] Fresh review, full Rust, matching plugin evidence, public audit, progress
  synchronization and original chapter audit updated without closing its gaps.

Offset/revolution/extrusion supports, full edge/pcurve correspondence, world
composition, attributes, solid certification, reports and remaining original
input/commit/lifecycle requirements continue to be required and open.

Accepted evidence: 76 Bezier +611 regression =687 FreeCAD checks on the same
selected plugin, process exit0; full cargo test passed603 tests. Source/runtime
fingerprints, REDs and reviewed scope are in the
[evidence record](../../validation/2026-10-10-rhino-core-bezier-support.md).
This plan's owned-support increment is verified; the original chapter is open.
