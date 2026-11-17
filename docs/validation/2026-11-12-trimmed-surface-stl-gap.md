# Trimmed standalone NURBS STL export

## Export up to 1,024 polygon edges

The former 128-edge native face limit refused an authored reopened plane with
133 combined trim edges. The Rust and OCCT adapters now share a bounded
1,024-edge limit and the 523,776-pair topology verifier. This temporary face
conversion also serves 3DM export. OCCT transient allocations remain outside
the Rust memory callback; the larger native peak has not been measured and
the managed reservation is not a bound on process memory.

Reopened planar fixtures with 133, 712 and 1,024 edges, each with a hole,
exported STL at two tolerances. Triangle area matched the polygon-minus-hole
oracle, material was covered, and sampled hole/exterior points were not.
Simulated managed pressure preserved an existing destination, document and
Redo; 1,025 edges were refused with the same state preserved. An authored
1,024-edge cylindrical patch also exported at two tolerances with area checked
against the source geometry. A reopened 1,024-edge plane passed 3DM export
and import with material/hole/exterior ray checks, both through the admitted
host and through actual debug executable CLI commands using its sibling native
exchange adapter. These do not establish independent Rhino compatibility.

The STL integration batch passed 68 tests, the OCCT `3dm-brep` trimmed-face
batch passed nine, and the explicitly enabled native 3DM integration batch
passed ten. The first attempt at the kernel batch without its `3dm-brep`
feature ran zero tests; it is not counted as validation. No-default check,
existing debug build and public source audit passed. Arbitrary trims, all
curved sources, extreme placements, physical desktop, actual OOM and native
peak remain unverified.

### Process memory for the 1,024-edge fixtures

Two separate Windows debug test processes saved and reopened one trimmed
source, selected it and exported STL once at 0.005 mm / 0.3 rad. The plane
fixture produced 1,024 triangles in 312.143 ms; its before/after process
working set was 59,092,992 / 82,227,200 bytes, working-set lifetime peak
59,232,256 / 83,423,232, private commit 47,640,576 / 78,409,728, and peak
commit 48,164,864 / 79,695,872. The cylindrical fixture produced 9,322
triangles in 634.323 ms; corresponding values were 58,671,104 / 104,218,624,
58,671,104 / 106,586,112, 47,247,360 / 103,530,496, and 47,267,840 /
106,307,584 bytes. Document equality was checked against the reopened
document after export.

These are one process sample per fixture on this machine, not a benchmark or
worst-case bound. Lifetime peaks include setup, Rust, OCCT and OS activity;
their increments are not isolated native allocations. The pre-export fixture,
scene and document comparison clone were already live. Other trim complexity,
native transient peaks for 3DM, desktop and real memory exhaustion remain
unmeasured. Both isolated diagnostics passed; an initial probe assertion
compared the reopened document with the pre-save document and failed, so the
valid before/after comparison uses the reopened document instead.

### Estimated native headroom for large trim export

The newly supported 129–1,024-edge conversion used to enter OCCT after only
admitting Rust scratch/copy memory. It now requests an estimated allowance of
4 MiB plus 64 KiB per trim edge before entering OCCT, and includes that
allowance in callback admission while the conversion runs. At 128 edges or
fewer, the previous behavior is unchanged. The estimate is informed by the
two process samples above; it is neither an OCCT allocation tracker nor a
hard limit, and it may be insufficient for other source complexity.

A kernel test reproduced the missing request and now verifies that refusing
the allowance stops conversion. The reopened STL fixture (133 edges) and 3DM
fixture (1,024 edges) were also tested with constrained managed memory: export
failed while preserving a pre-existing destination, document and Redo, then
passed after pressure was released. These app checks do not instrument the
exact refusal stage; memory pressure was simulated, not physical OOM. Ten
OCCT `3dm-brep` trimmed-face tests, nine selected STL trim regressions, the
focused admitted 3DM test and the actual executable 3DM roundtrip passed.
No-default check, debug build and public source audit passed. Native peak,
worst-case size and all error paths remain unbounded/unverified.

## Scoped source-trimmed export

The selected-STL path now constructs a temporary OCCT face from the validated
source surface and UV polygon loops, then tessellates that face with the
requested STL profile. Rust validates the trim, loop topology and orientation,
reserves metadata and loop-copy scratch, and retains ownership of the export
transaction. The synchronous native adapter constructs and validates the face
and copies its serialized source back; it does not modify the document.

The reopened planar fixture that previously exported 100 mm² now exports the
expected 32 mm². A second reopened fixture combines a trimmed plane, a mesh
and a scale/shear placement; two STL profiles preserve the hole, expected
combined area and existing Redo. A reopened cylindrical patch with a hole
was checked against analytic area at two profiles. Kernel tests also check
both loop windings, planar material/hole/miss rays, sampled periodic-source UV
points and an admission refusal. The complete STL batch passed 64 tests; 14
native surface tests and four trimmed-face tests passed. The no-default
feature check, debug build and public-source audit passed.

This is bounded support for validated polygon loops: up to 128 edges and 16 KiB
of trim metadata. Reversed trim metadata and unverified closed-axis seams are
refused; the verified full-rectangle seam case is described below. A mixed mesh plus unsupported seam fixture keeps an existing
destination file, the document and Redo. OCCT constructs 3D boundary curves
with a 1e-7 source-unit request; source topology tolerance is reported by the
existing STL receipt and is not a measured global mesh deviation. Arbitrary
trim representations, seam reconstruction, extreme scales, actual desktop,
physical memory exhaustion and native transient peak remain unverified.
The sections below record the earlier temporary refusal, now superseded for
the supported cases above.

## Certified full-seam STL export

For one certified closed axis, a rectangular outer trim spanning its complete
parameter domain can use OCCT's natural face boundary, which represents the
paired seam; validated interior holes are then added as source UV wires. The
special case requires the full source domain, axis-aligned rectangle, a single
closed seam axis and hole vertices separated from both seam sides by a numeric
guard. Other seam-touching cases were refused at this stage; reversed trim metadata remains
unsupported. Native code checks face validity and serializes a temporary
source without changing the document.

An application fixture reopens a translated, full-seam cylinder with a hole,
exports at 0.05 and 0.005 mm profiles, compares area to an analytic formula
within 0.15 mm² and samples the hole and retained surface against the STL. The
document and a prior Redo entry remain unchanged. A mixed mesh plus
nonrectangular seam fixture is refused without changing destination, document
or Redo. Kernel fixtures check exact source area and hole rays for a cylinder,
plus sampled source agreement on a repeated-basis periodic surface. The
65-test STL batch passed before a subsequent native input-count guard; after
that guard, the focused full-seam STL test and all six trimmed-face plus 14
native surface tests passed. Debug build and public audit passed.

The analytic and point checks cover these fixtures only. Multiple closed axes,
other seam topology, trim reversal, extreme placement, native peak and desktop
remain unverified. STL tessellation and 3D edge construction are approximate;
the tests do not establish a global export deviation bound.

## Rectangular patch touching one seam side

Previously a rectangular cylindrical patch whose boundary touched either end
of the closed parameter domain was refused. The source-face path now accepts
the verified rectangle spanning only one end, using ordinary UV boundary wires;
the paired natural seam remains reserved for rectangles spanning both ends.
Holes must stay clear of both seam ends and the source domain must be complete.
Multiple closed axes, other seam topology and reversed trims still refuse.

Two cylinder fixtures, one at each end, verify source area analytically and
check hole and material rays. After save/reopen with translation, both export
at 0.05 and 0.005 mm profiles with area within 0.15 mm², sampled hole and
material positions, unchanged document and preserved Redo. The nonrectangular
seam refusal remains covered. The full STL batch passed 66 tests; the source-face
and native surface batches passed 7 and 14 tests. Debug build and public-source
audit passed. These samples do not establish global error bounds;
desktop, extreme placements, periodic variants and native peak remain unverified.

## Reproduced data error and temporary refusal

The selected-STL path previously passed a standalone surface's underlying
NURBS to the native face converter without its trim extension. A saved/reopened
10×10 mm planar source with an inset 6×6 mm outer loop and a 2×2 mm hole
therefore exported area **100 mm²**, rather than the expected **32 mm²**.
The direct regression reproduced that result before the guard; it is retained
as an ignored acceptance test until correct export is implemented.

The STL path now refuses standalone surfaces carrying trim metadata, rather
than writing the untrimmed source. In the tested saved/reopened fixture, an
existing destination file, the document and one Redo entry remain unchanged.
The existing STL integration batch passed 61 tests with the desired trimmed
export test ignored. Debug build and public-source audit passed. This is a
temporary loss of export functionality for trimmed standalone surfaces, not
a completed implementation; other input shapes and actual desktop error
presentation were not verified by the new fixture.

## Earlier implementation direction

Construct a temporary face from the validated source surface and its actual UV
trim loops using the native geometry adapter, then tessellate that face with
the independent STL export profile. Rust must own metadata validation,
topology checks, admission, cancellation and the atomic file transaction;
native code may construct and validate the OCCT face but must not alter the
document. Verify loop orientation, holes, concave loops, curved surfaces,
closed seams and placement, two export profiles, output geometry, atomic
failure, shared/mixed selections and Undo/Redo. Refuse inputs that cannot be
established rather than exporting the whole source. OCCT transient allocations
and desktop remain separate evidence requirements.
