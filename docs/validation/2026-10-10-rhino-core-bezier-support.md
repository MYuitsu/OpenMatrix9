# RCORE-01 owned Bezier support — 2026-10-10

Native Bezier curves, surfaces and UV pcurves now have independent Rust-owned
supporting data. The existing FreeCAD/OCCT checked adapters already read these
controls and weights; they are reused without converting the native geometry.
Rust privately wraps validated single-span clamped basis data and emits explicit
`bezier_curve` / `bezier_surface` JSON. Arbitrary multi-span, periodic or
reparameterized NURBS cannot acquire those tags. Root descriptor form3 is
`bezier`; analytic/NURBS tags retain their values. Legacy NURBS `basis` remains
null for Bezier; full natural domains are [0,1] independently of restrictions.

The selected plugin passed **76 Bezier +174 analytic +54 NURBS +177 topology
+151 descriptor +55 import =687 FreeCAD checks**, all with matching current
host/plugin fingerprints and process exit0. Full Rust passed **603
tests**, including four owned Bezier/ABI tests and a distinct-form test.
Plugin SHA256: `849cb54fd31d648cb43dcf2aa84953cebd076934550542c177c4e5f2829fe831`.
[Machine evidence](2026-10-10-rhino-core-bezier-support.json) retains actual
reports, source/runtime/log hashes and REDs. The native RED fails at missing
rational curve support; the Rust form RED rejects the newly required Bezier tag.

The 76-check fixture samples rational and polynomial curves/surfaces through
independent Bernstein formulas and native evaluation, retaining original
controls, rational weights, U-major rectangular surfaces and weight omission
for polynomial data. Separate edge/face restrictions, native-local controls,
nonidentity translations/rotations and face orientation are checked. An authored
rational Bezier UV pcurve is sampled against its original native curve. Exact
source BRep and Undo/Redo stay unchanged; returned JSON survives source
replacement and document close. Root/common face payloads share the same owned
data, with no duplicate native extraction.

The ABI reuses existing 96-byte curve/176-byte surface envelopes and bounded JSON
query/copy. Shared basis validation bounds counts/addresses before borrowing;
the caller still guarantees initialized aligned immutable live allocations.
Bezier cardinality/domain checks precede owned cloning. No pointers or allocator
ownership escape, and rejected/short output stays untouched. Same budgets apply:
degree 25, 8192 poles, 2 MiB logical basis and 8 MiB aggregate serialized support.
These do not bound expanded Qt aliases or native/global allocation. Qt normalizes
signed zero; numeric geometry preservation does not imply bit-exact IEEE JSON.

[Fresh review](../reviews/2026-10-10-bezier-support-review.md) found no actionable
defect. Its initial polynomial-surface coverage gap is addressed and verified
in a final read-only follow-up. Review and host tests are evidence for this
read-only support increment, not a global conversion or allocator certificate.

**The full original RCORE-01 chapter remains in progress.** Offset/extrusion/
revolution support, complete 3D edge/pcurve correspondence, world composition,
source attributes, full manifold/solid/cavity validation, conversion/error
reports and remaining input/worker/commit requirements still need evidence in
the [original audit](../reviews/2026-10-10-rcore01-closure-audit.md). Existing
kernel constructors, solvers and Line/Polyline commands are reused.
