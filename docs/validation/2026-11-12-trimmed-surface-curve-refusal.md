# Trimmed surface curve commands — 2026-11-12

## Affine trimmed-plane extraction

ExtractWireframe now supports a limited source representation: a degree-(1,1)
2×2 nonperiodic surface with unit weights, initially with canonical clamped knots and domain,
whose control net is exactly affine. Equality of its opposite-corner sums is
checked with rounding residuals, so a one-ULP warp hidden by rounded sums is
not accepted. The stored outer and inner UV polygon segments are evaluated on
the surface and emitted as source-derived lines with the source placement. The
trim record must match the surface. A 16 KiB metadata limit and a conservative
scratch reservation precede decoding; larger metadata and, at that stage,
non-affine or curved surfaces were unsupported. The later axis-aligned curved
trim extension is described below. This is not general trimmed NURBS support.

After save/open, a translated 10×10 mm plane with a rectangular outer loop and
triangular inner hole produces the seven independently expected world-space
segments through CMD and menu, alone and selected with a Box. Each case saves
and reopens the output and performs one Undo/Redo. A simulated managed-memory
refusal preserves the document and existing Redo, then succeeds after releasing
pressure. A 1e-9 warped control point and nonuniform weight are refused; a
separate test rejects oversized metadata without reclaiming history. The
rounded-sum counterexample is checked in a unit test. These cases do not prove
all floating-point, loop topology, placement, physical memory or desktop
conditions. At this stage DupBorder and ExtractIsocurve still refused trim metadata;
the later DupBorder extension is described below.

With `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`, the Curve menu, DupBorder,
Isocurve and Wireframe batches passed 6, 17, 11 and 63 tests respectively,
with five Wireframe diagnostics ignored; the separate oversized-metadata and
exact-control-net tests also passed. The no-default check, debug build and
public source audit passed. No native geometry conversion was introduced.

## DupBorder on an affine trimmed plane

DupBorder now uses the same admitted Rust trim-boundary preparation as
ExtractWireframe for the certified affine-plane representation. A translated,
saved/reopened plane with a rectangular outer loop and triangular hole produces
the seven expected source-derived lines through CMD and menu; the output
round-trips and one Undo/Redo restores the document. Simulated memory refusal
preserves the drawing and existing Redo, and retry succeeds after releasing the
reservation. At that stage, a curved trim, a 1e-9 warped control net and nonuniform weights
were refused; the later U/V-aligned extension is described below. The focused no-default-features test also creates the seven
lines and verifies Undo/Redo without OCCT.

The Wireframe batch passed 65 tests with five ignored; Curve menu passed six,
DupBorder 17 and Isocurve 11. The no-default test, debug build and public
source audit passed. Metadata above 16 KiB, other trimmed representations,
desktop interaction and physical memory pressure remain unverified or
unsupported as applicable. At this stage ExtractIsocurve still refused trim
metadata; the later affine-plane support is described below.

## ExtractIsocurve on an affine trimmed plane

ExtractIsocurve now intersects an axis-aligned UV isocurve with polygonal outer
and inner trim loops on the same certified affine 2×2 plane. It emits one line
for each interval inside the outer loop and outside all holes, preserving
placement and committing the complete set in one transaction. The metadata
must match the source and remain within 16 KiB; scratch and output slots are
reserved before decoding and allocation. At that stage non-affine/curved
surfaces remained unsupported; the later curved isocurve extension is described
below. No native geometry conversion is introduced.

On a saved/reopened translated plane with a rectangular outer loop and a
rectangular hole, U and V each produce the two expected world-space segments
through CMD and menu. The output is saved/reopened and a single Undo/Redo
restores it. A separate co-scaled and sheared placement fixture checks its
endpoints and Undo/Redo. At that stage, when the isocurve passed through a trim vertex,
coincides with an edge, has no interior interval, is out of domain, or has
numerically uncertain crossings, the command refuses and keeps its prompt.
The tested near-parallel crossing and large local-coordinate cancellation are
refused; a corrected parameter succeeds. A deleted source is refused. A
simulated memory shortage preserves the input, document and Redo; retry after
pressure release succeeds. In a separate handler fixture, accepting memory
recovery discards one oldest Undo entry that retains a mesh and successfully
produces both segments; Undo removes them together.

The crossing and world-scale margins are conservative floating-point checks,
not interval arithmetic or a proof of all coordinate and trim configurations.
At that stage trim vertices remained unsupported; curved surfaces,
metadata above 16 KiB, arbitrary imported loops, real RAM exhaustion and
desktop interaction remain unverified or unsupported. The four Curve batches
passed 6 menu, 17 DupBorder, 17 Isocurve and 65 Wireframe tests (five Wireframe
diagnostics ignored). Two targeted UI recovery tests, no-default check, debug
build and public source audit passed. As previously requested, no new
ExtractIsocurve business rule was written to the skill.

### Transverse trim vertices

When the isocurve crosses an exact trim vertex and the adjacent edges lie on
opposite sides, the vertex is now counted once. On a planar fixture with a
diamond-shaped hole, U and V each produce the expected two intervals; reversing
the hole loop also succeeds, with Undo/Redo. A tangent vertex, a parameter
within 1e-12 of a vertex, and a coincident trim edge continue to be refused;
the tested refusal keeps the prompt, document and Redo, then accepts a
corrected crossing. These are scoped numerical fixtures, not a proof of all
polygon topologies or floating-point cases.

The Curve menu, DupBorder, Isocurve and Wireframe batches passed 6, 17, 19 and
65 tests, with five Wireframe diagnostics ignored. Ten library tests filtered
for Isocurve, the no-default check, debug build and public source audit passed.
No ExtractIsocurve business rule was added to the skill, in accordance with
the user's previous decision.

### Non-unit restricted parameter domains

The three commands now also accept finite clamped knot intervals with a source
domain contained in those intervals, while retaining the exact affine 2×2,
unit-weight, nonperiodic and metadata restrictions above. In the tested
saved/reopened input, U knots span 10–20 and V knots span 2–6, with restricted
domains 12–18 and 3–5. The translated plane has a rectangular outer trim and
a rectangular hole. ExtractWireframe and DupBorder each produce eight expected
source-boundary segments; ExtractIsocurve produces two expected segments for
each U and V cut. CMD and menu paths and one Undo/Redo passed for each case.
The new fixture does not separately round-trip the generated output.

Isocurve's conservative crossing margins now account for the source parameter
scales. A separate fixture with a parameter offset near 1e8 is refused when
the numerical margin is insufficient, retaining the prompt, document and Redo.
These floating-point checks are not a proof for arbitrary parameters or trim
topology. At that stage curved surfaces and other NURBS representations, trim metadata above
16 KiB, desktop interaction and real memory exhaustion remain unsupported or
unverified as applicable.

On the existing `L:/om9-target`, Curve menu, DupBorder, Isocurve and Wireframe
passed 6, 17, 20 and 66 tests respectively, with five Wireframe diagnostics
ignored; ten Isocurve library tests, the no-default check, debug build and
public source audit also passed. No new build directory was created. No
ExtractIsocurve business rule was added to the skill.

### Ambiguous loop topology

The core trim parser checks individual loops and tests hole vertices against
the outer loop, but it does not establish that loops are simple and mutually
disjoint. Before this change, ExtractWireframe accepted a reopened planar
fixture with intersecting holes and emitted their full boundaries, including
segments inside the overlapping hole. A product regression reproduced that
incorrect acceptance.

The three planar trim commands now share a bounded topology check. It refuses
crossing or touching loop edges, self-intersection, nested holes, uncertain
orientation or containment, and excessive verification work. At that stage,
adjacent collinear edges were refused; the later extension below accepts a
subset. In reopened planar
fixtures, intersecting holes, nested holes and a self-crossing outer loop are
refused through all three commands, with the document and an existing Redo
preserved. The Isocurve failure occurs when committing the pending operation.
Previously tested single-hole fixtures remain passing. This is a conservative
floating-point check, not a proof for all loops; desktop, physical RAM pressure,
and arbitrary imported loop data are unverified.

The Curve menu, DupBorder, Isocurve and Wireframe batches passed 6, 17, 20 and
67 tests, respectively; five Wireframe tests were ignored. The no-default check,
existing debug build and public source audit passed. No new build directory was
created. The user's previous decision not to update Isocurve guidance stands.

### Subdivided straight trim boundaries

When a shared endpoint lies strictly between the other endpoints along either
parameter axis, adjacent segments can meet only at that endpoint. The topology
check now accepts this case. In one reopened translated planar fixture, both
the outer boundary and hole contain a subdivided straight side. ExtractWireframe
and DupBorder each produce ten independently checked source segments;
ExtractIsocurve creates two expected intervals in both U and V. CMD and menu
routes and one Undo/Redo passed. The V cut also crosses a subdivision vertex.
Reopened output and desktop interaction were not separately tested.

A separate reopened fixture with adjacent edges that double back is refused by
all three commands, preserving the document and Redo, as are the previously
tested intersecting, nested and self-crossing loops. The numeric orientation
checks and 250,000-comparison limit remain conservative; this is not general
proof for arbitrary loops. Curve menu, DupBorder, Isocurve and Wireframe passed
6, 17, 20 and 68 tests, respectively, with five Wireframe diagnostics ignored.
No-default check, existing debug build and public source audit passed. No
ExtractIsocurve business rule was added to the skill.

### Large trim parameter offsets

The core trim validator previously computed polygon area from products of
absolute UV coordinates. In a 1e8-offset fixture, cancellation made a valid
small rectangular hole appear to have zero area. The area sum now uses a loop
vertex as its local origin. Core tests cover positive and negative 1e8 offsets,
reversed loop order and rejection of a truly collinear zero-area hole.

On a saved/reopened translated plane whose U/V knots span 1e8 to 1e8+4,
ExtractWireframe and DupBorder each produce the eight independently checked
outer and hole edges through CMD and menu; one Undo/Redo restores the result.
This fixture does not separately reopen its output. The existing Isocurve
large-offset uncertainty refusal remains covered by its regression. The area
calculation remains floating-point and the absolute 1e-12 area threshold is
unchanged; arbitrary parameter scales, malformed loops, desktop and physical
memory pressure are not certified.

The core Rhino semantics batch passed 12 tests. Curve menu, DupBorder,
Isocurve and Wireframe passed 6, 19, 20 and 71 tests respectively, with four
Wireframe diagnostics ignored. No-default check, existing debug build and
public source audit passed. No Isocurve business rule was added to the skill.

### Curved trimmed surface isocurves

ExtractIsocurve now clips a source NURBS isocurve to intervals inside a
polygonal outer UV loop and outside its holes, retaining source placement and
committing all intervals together. A control-hull, weight and knot-spacing
bound scales the conservative crossing checks for non-affine surfaces; large
or non-finite numerical uncertainty is refused. This floating-point screening
is not interval arithmetic or a proof for all surfaces and loop configurations.
Output and workspace are reserved before constructing curves.

On a saved/reopened cylindrical surface with rectangular outer loop and hole,
U and V each produce two intervals in the tested CMD/menu paths, with translated
and scale/shear placements. Four samples per output match the source surface;
independent radius and height checks also pass. Translated U arcs are measured
after output save/reopen against analytic arc lengths. Output entities survive
roundtrip and one Undo/Redo. Simulated memory pressure preserves the pending
parameter, document and Redo; retry succeeds. A separate cylindrical fixture
clips around a triangular hole with diagonal edges. At that stage a full closed-seam trim
and a placement of 1e12 units were refused in separate fixtures, retaining the
pending session and Redo. At that stage DupBorder/ExtractWireframe refused diagonal
boundaries on a curved surface; the later bilinear extension is described below.
Isocurve's UV clipping is a separate operation.

The broader Curve batches passed 6 menu, 24 DupBorder, 23 Isocurve and 73
Wireframe tests with four Wireframe diagnostics ignored. A subsequently added
large-coordinate refusal test passed separately, as did the updated cylinder
roundtrip test. Ten Isocurve library tests and one no-OCCT cylinder test passed;
the debug build and public source audit passed. These are scoped fixtures, not
certification of arbitrary surfaces, imported trims, all numerical extremes,
desktop or physical RAM exhaustion. No ExtractIsocurve business rule was added
to the skill, in accordance with the user's previous decision.

### Curved surface boundaries along U/V

ExtractWireframe and DupBorder now derive an axis-aligned polygon trim segment
on a non-affine NURBS surface from the corresponding source isocurve, restrict
its parameter domain to the segment, and preserve its direction and placement.
This uses the source NURBS representation rather than display polylines. Output
and basis storage are reserved before building the curves. Diagonal trim edges
were refused for non-affine surfaces at that stage; the later bilinear case is
described below. At that stage a trim spanning both ends
of a certified closed surface was refused. The later full-seam rectangular
case is described below. A possible degenerate segment is also
refused. These checks are not a universal numerical or topology proof.

In the reopened cylindrical fixture, the outer rectangle and rectangular hole
each contribute four U/V segments. ExtractWireframe and DupBorder each produce
the eight source curves alone and when selected with a separate plane, through
CMD and menu, with translated and scale/shear placements. Four samples per
curve are compared to the source surface in trim-loop order; a separate
analytic radius and height check is applied. One translated circular boundary
is measured after output save/reopen and agrees with its analytic arc length.
One Undo/Redo and output roundtrip pass. Simulated memory refusal preserves the
document and Redo before a successful retry. A separate saved/reopened
full-seam trim fixture was refused at that stage with document and Redo unchanged; the
existing diagonal-trim rejection remains tested. DupBorder also passes one
standalone-cylinder controller test without OCCT.

The 16 KiB metadata and bounded topology checks still apply. Curved trim paths
not parallel to U or V, arbitrary imported surfaces,
actual RAM exhaustion and desktop are not
covered or remain unsupported as applicable. Curve menu, DupBorder, Isocurve
and Wireframe passed 6, 24, 21 and 73 tests, with four Wireframe diagnostics
ignored; the focused no-OCCT test, debug build and public source audit passed.
No new build directory was created. No Isocurve business rule was added.

### Diagonal trim on a bilinear surface

For a nonperiodic, unit-weight, clamped degree-(1,1) 2×2 surface, the surface
restricted to a straight UV segment is a quadratic curve. For diagonal trim
segments, ExtractWireframe and DupBorder now construct that quadratic NURBS
from the source surface at the endpoints and midpoint, retaining direction and
placement. This identity is algebraically exact; its floating-point evaluation
uses a conservative scale screen and is not an interval proof. Numerically
uncertain cases are refused before publishing output. Other rational or higher-degree curved diagonal trims were unsupported at that stage; see the rational bilinear extension below.

The saved/reopened fixture uses a warped bilinear surface `z = 4uv`, a
four-edge outer loop and a triangular hole, all with diagonal UV edges. Both
commands, alone and selected with a plane, produce the expected seven surface
boundaries through CMD/menu, with co-scaled/sheared placement. Five points per
curve and their order are compared to the independent analytic surface
formula; output entities roundtrip and one Undo/Redo restores the drawing.
Simulated memory pressure preserves the document and Redo before successful
retry. A separate 1e12-placement fixture refuses the numerically uncertain
case and keeps document/Redo. Rational-weight diagonal fixtures were refused at that stage; one no-OCCT DupBorder test passed.

The Curve menu, DupBorder, Isocurve and Wireframe batches passed 6, 24, 25 and
76 tests, with four Wireframe diagnostics ignored. The no-OCCT test, debug
build and public source audit passed. Arbitrary bilinear scales, numerical
proof, other curved representations, desktop and actual RAM exhaustion remain
unverified. No new build directory was created.

### Rational bilinear diagonal trim

For a nonperiodic clamped degree-(1,1) surface with four finite positive
weights, the restriction to a straight UV segment is a rational quadratic.
The commands construct its homogeneous Bézier controls from endpoint and
midpoint evaluations. Positive weights and scale-dependent floating-point
screens are required; these screens are conservative safeguards, not an
interval proof of error. Numerically uncertain cases are refused.

The warped saved/reopened fixture uses weights [1, 2, 3, 4], a diagonal
four-edge outer loop and a triangular hole. Five ordered points on each edge
were checked against an independently evaluated rational bilinear formula.
Both commands passed CMD/menu, single and mixed selection, affine placement,
output save/reopen, simulated memory pressure and Undo/Redo. Extreme weight
1e-20 is refused with document and Redo retained. The existing large-placement
refusal also passed. DupBorder passed with and without rational weights in a
no-OCCT test. The four Curve batches passed 6, 24, 25 and 76 tests; four
Wireframe diagnostics were ignored. Debug build and public-source audit passed.
Other rational surface representations, higher degrees, all numerical scales,
actual memory exhaustion and desktop behavior are not certified.

### Certified full-seam rectangular trim

For a verified closed surface axis, the full-seam case is admitted when the
outer trim is exactly a four-corner rectangle spanning that axis and all hole
vertices lie strictly inside its seam limits. Other full-seam shapes continue
to refuse. On the tested reopened cylindrical surface with a rectangular hole,
DupBorder returns the two physical circular boundaries plus four hole edges,
omitting the two internal seam edges. ExtractWireframe returns those six curves
and one displayed seam. Tests check analytic cylinder radius and heights, seam
count, CMD/menu, Undo/Redo and saved/reopened output. Six and seven output
limits respectively admit the expected sets; one less refuses while preserving
the document and Redo. Simulated memory refusal also preserves both.

On the same type of reopened input, ExtractIsocurve produces a full circle away
from the hole or two clipped intervals through it, for the tested U and V cuts.
Samples match the source and analytic cylinder; the full circle measures 4π mm.
CMD/menu, Undo/Redo and simulated memory refusal pass. A nonrectangular outer
loop that spans the seam still refuses in both boundary and isocurve fixtures.
The Isocurve output was not separately reopened in this fixture. A no-OCCT
test covers DupBorder and ExtractIsocurve on the full-seam cylinder without a
hole. Curve menu, DupBorder, Isocurve and Wireframe passed 6, 24, 25 and 74
tests, with four Wireframe diagnostics ignored; targeted output-limit tests,
the no-OCCT test, debug build and public source audit also passed. Other seam
topologies, imported surfaces, desktop and physical memory pressure remain
unverified or unsupported. No Isocurve business rule was added to the skill.

## Earlier blanket refusal

The following observations are historical for supported affine planes.

A standalone planar NURBS surface can have an `om9.trim.surface` extension
containing an outer UV loop and inner holes. Previously ExtractWireframe and
DupBorder generated the four untrimmed domain boundaries instead of those
loops; a reopened fixture reproduced both commands accepting this source.
ExtractIsocurve also started with the untrimmed surface and its implementation
did not clip the result to the trim. The red regression observed the command
starting; it did not separately commit an incorrect isocurve.

At this stage the commands refused a surface carrying the trim extension, with an
explicit unsupported reason, until source trim boundaries or clipping are
implemented. ExtractWireframe checks every selected root before preparing any
output. The reopened plane fixture includes an outer rectangle and triangular
hole. For ExtractWireframe, CMD and menu dispatch each reject single-surface
and Box-plus-surface selections, preserving the document and an existing Redo.
DupBorder and ExtractIsocurve also reject it without changing the document or
Redo. This is a safe refusal, not support for extracting trimmed boundaries or
isocurves; arbitrary other surface types, source corruption, and desktop
interaction have not been exhaustively tested.

On Windows x64 with `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`, the Wireframe
suite passed 61 with five ignored, Curve menu six, DupBorder 17 and Isocurve
11. The no-default-features check, debug build and public source audit passed.
The user previously declined to update ExtractIsocurve guidance; that choice
is retained. The user separately approved recording the tested Sphere
AroundCurve mixed-display scope, and its source and installed skill copies
were updated identically.
