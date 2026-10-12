# RCORE-01 common topology snapshot — 2026-10-10

The common `geometryDescriptor` now exposes owned face/loop/ordered edge-use
snapshots, pcurve restriction domains and oriented UV endpoint observations.
Rust validates copied rows and derives distinct edge identities, edge incidence
and vertex-connected free-boundary components. Native C++ only reads OCCT and
serializes the checked observations on the GUI thread; source stamps are checked
before/after by the existing descriptor adapter. No native handle escapes.

Final selected plugin passed **177 topology +55 import gate +151 descriptor =383
FreeCAD checks**, with process exit0 and the same plugin/host hashes. Full Rust
passed **585 tests**, including 13 new topology/ABI tests. Public source
audit reports zero errors. Plugin SHA256: `6b794fb818029dd1da7295ec66b0898f1f0618645c852811684d92ea84c178d1`.
[Machine evidence](2026-10-10-rhino-core-topology-snapshot.json) retains actual
runtime paths, launch/result fingerprints, source fingerprints and RED reports.

Authored host inputs include square20/holeR3, open cylinderR4/H7, sphereR3,
hollow box with twelve faces, reversed trim face, coincident distinct faces,
located shared native faces, repeated same face and opposed shared-face
occurrences. A cylinder keeps two occurrences of one seam edge, separate
pcurves and opposite periodic UV sides, plus its two circular open boundaries.
Sphere singular pole edges are retained but excluded from free boundaries.
All inspected source BRep bytes and Undo/Redo counts remain exact; returned
values stay independent after source replacement. A face subelement uses the
same contract. The previous descriptor control still covers source guards,
CAD/display independence, working exchange and close/reopen provenance.

The original RED observes null face data. A second RED proves repeated compound
references falsely made a sheet edge-closed before the review fix. Unique
TShape+location faces now supply incidence; `face_occurrences` retains all
references/orientations. Boundary components are connectivity groups, not
ordered wires. IDs are decimal strings and snapshot-local, not persistent names.
[Review and fixes](../reviews/2026-10-10-topology-snapshot-review.md).

Limits: 16384 combined face/loop/use records, checked arithmetic before native
row growth and Rust copies; finite ordered pcurve domains are checked before
native endpoint evaluation. FFI borrows only initialized/live/aligned immutable
values and writable nonoverlapping output. Reject leaves output untouched.
Unreadable arbitrary pointers and process OOM are not certified safe by this.

`edge_manifold` and `edge_closed` describe the recorded face-edge incidence,
not full vertex-link manifoldness, geometric embedding, orientation or a valid
solid. `closed`, `manifold` and `solid_orientation` remain explicitly unknown;
`solid` still declares native shape type only. Pcurve `geometry_data` remains
uninspected support basis; UV endpoints are observations, not a complete curve
or conversion map. Coordinates, attributes and broader conversion contracts
remain open. Unsupported internal/external orientations, missing pcurves,
empty trims and incomplete wire traversal reject rather than return partial data.

**Full RCORE-01 remains in progress.** Next: owned supporting geometry/pcurve
data and consistent topology/solid validation/reporting, then the remaining
numeric, raw allocation and uniform mutation/worker requirements in the
[full audit](../reviews/2026-10-10-rcore01-closure-audit.md). Existing FreeCAD
solvers and Line/Polyline constructors are reused.
