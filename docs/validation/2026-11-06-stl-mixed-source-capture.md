# STL capture: OCCT and NativeBrep sources

Selection containing only root OCCT B-reps and NativeBreps now captures
tracked source leases in one admitted batch. NativeBrep retains its original
payload; conversion uses the document tolerance captured at invocation.
Placement, label and visibility are also captured. A selection containing an
instance or another geometry type continues to hold a document snapshot.

The new targeted fixture used an OCCT sphere and its NativeBrep conversion,
with the native occurrence reflected. It compared source-only capture with
document capture for the same two objects plus a skipped point, after deleting
the objects from the live document. Positions, triangles, warnings, reported
OCCT topology tolerance and binary STL bytes matched. After dropping the
controller and document capture, neither display allocation remained alive.

Validation: 130 transaction tests, 66 STL library tests, and 60 STL integration
tests passed with `-j 2` and `CARGO_TARGET_DIR=L:\om9-target`; the mixed fixture
was rerun after adding the edit-after-capture assertion. The no-OCCT developer
configuration compiled with warnings. These tests do not certify global STL
deviation, arbitrary native topology or desktop behavior. The existing desktop
Mốc A evidence predates this source-only path; a new release/desktop run is
still required for the changed path.

## Authored mesh capture experiment (not retained)

An experimental mixed B-rep and authored-mesh capture copied vertices and
polygon boundaries once per definition member. A sphere and two instances of
one authored quad, including a reflection, produced the same positions,
triangles, warnings, tolerance and binary STL as document capture after live
deletion. Simulated physical-memory pressure refused during capture and
released its reservation. The STL unit group passed 68 tests and the pressure
fixture passed separately; 60 integration tests passed during the experiment.

A synthetic 200,000-face/200,002-vertex fixture in a release test build measured
70.700–79.496 ms for mixed capture, versus 8.809–18.575 ms for a tracked document
lease alone (three paired runs); these are not equivalent end-to-end operations.
The capture reservation was 8,800,554 bytes. Vertices were degenerate and this
is not a desktop responsiveness or native-memory measurement. A debug build
measured 2.156–2.896 s versus 0.369–0.661 s. Because the copy and repeated
admission ran synchronously before worker launch, the experiment was removed;
the product continues to retain a document snapshot for mixed authored-mesh
selections. A shared immutable mesh source or an admitted worker-side transition
needs further design and measurement. Sphere document/Undo display eviction and
the production source/display gate remain open.
