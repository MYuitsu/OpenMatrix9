# RCORE-01 derived supporting geometry — 2026-10-10

Offset curves (3D/UV), offset surfaces, linear extrusion and revolution now have
Rust-owned supporting chains, preserving native basis type, signed distance,
unit direction, revolution axis origin and natural domains. Inner trims remain
explicit wrappers, including a surface trimmed in only one direction whose
other axis is unbounded. No solver or native geometry conversion is introduced.
Native class checks precede adaptor analytic simplification; descriptor form4
is `derived`, while legacy NURBS `basis` stays null for these supports.

The final plugin passed **91 derived +687 regression =778 FreeCAD checks**:
76 Bezier,174 analytic,54 NURBS,177 topology,151 descriptor and55 import checks.
All seven processes exited0 and loaded the exact selected plugin. Full cargo
test passed **610 tests**. Plugin SHA256: `8939c403b931bd5e5fb7d61b81288600fccccce97c1d2b625d011561db539566`.
[Machine evidence](2026-10-10-rhino-core-derived-support.json) records current
source/runtime/report/log fingerprints and the observed process outcomes.

Independent sampled oracles use Bernstein evaluation for authored single-span
rational NURBS/Bezier bases, curve tangent cross reference direction, surface
partial cross product normals, linear extrusion and Rodrigues revolution.
They match native evaluations after applying the recorded native placement once.
The fixture verifies original controls/weights, nested extrusion/offset support,
finite vs unbounded domains, UV offsets, inner one/two-axis trims, topology
reversal, exact source BRep/Undo/Redo and snapshots after replacement/close.
Finite-difference normals and sampled checks do not certify global error bounds,
offset regularity or conversion parameter maps. SDK headers establish native
parameter convention; this is OM9 metadata evidence, not Rhino semantic parity.

Rust validates a bounded16-node root-to-leaf chain before owning the typed leaf.
The native bridge uses fixed node storage and existing checked basis readers.
ABI nodes are104 bytes/alignment8; tree424 bytes/alignment8. Count/address/
alignment, category/dimension, unit vectors, finite parameters, unused slots
and domains reject before selected leaf borrowing. Infinity flags encode native
unbounded ends with canonical zero numeric slots; no native finite sentinel is
serialized as geometry. Short or invalid output remains untouched. Caller must
still guarantee initialized live immutable allocations for accepted buffers and
nonoverlapping writable output; release panic/OOM abort and native exception
boundaries remain unchanged. Existing basis and8MiB serialized limits are reused,
without claiming a native/global/expanded Qt allocation bound.

[Fresh review](../reviews/2026-10-10-derived-support-review.md) found the one-axis
trim rejection and two coverage uncertainties. The trim fix has a recorded
RED→GREEN Rust regression and real-host proof; malformed wrappers with an
unusable selected NURBS envelope reject before leaf access, and native weighted
NURBS/Bezier branches now have direct fixture coverage. The first green1 host
attempt terminated before any report with invalid-handle exit -1073741816 and
empty logs. Its cause remains unestablished and it is excluded. A later56-check
pass precedes the final91-check coverage and is also excluded from accepted totals.

**Full original RCORE-01 remains in progress.** World composition/source
attributes, full edge/pcurve correspondence, native parameter reversals,
vertex-link manifold/solid/cavity certification, complete reports and remaining
input/worker/commit/lifecycle requirements still need authoritative evidence in
the [original audit](../reviews/2026-10-10-rcore01-closure-audit.md). Existing
FreeCAD/OCCT constructors, editors and Line/Polyline commands are reused.
