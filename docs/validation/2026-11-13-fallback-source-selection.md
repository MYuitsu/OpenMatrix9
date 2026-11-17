# Source-based selection of a verified fallback trim

When trim display cannot be constructed, the viewport shows the underlying
surface with a warning and previously discarded every point pick on it. Normal
object selection now has a source-based path for a restricted class: a planar
affine NURBS source, one strictly convex outer UV polygon and zero or more
strictly convex, separated holes, with at most 1,024 vertices in total. It
intersects the visible ray with the source, verifies
the returned UV against the placed surface, requires the hit to be inside and
well away from the boundary, and verifies every polygon edge against every
other vertex with a numerical uncertainty screen. This rejects star-shaped,
collinear or otherwise unproven loops rather than assuming convexity from
the display. Missing document tolerance and source/scene errors do not create
a pick. Source placement and nested occurrence path come from the current
scene; managed scratch is admitted before trim decoding and the source query.

The verified hit participates in normal depth-sorted overlap selection and
deduplicates the same root. Allocation or source-query failure aborts that
click and keeps the current selection instead of selecting a different object;
an off-ray fallback does not block a distant pick. This is point selection of
the object, not subobject selection, Surface Snap, or a corrected display:
the full support surface and existing warning remain visible until an actual
trim-aware display is available.

For an authored saved/reopened 708-edge convex planar trim with document
tolerance, simulated egui clicks selected the interior and rejected an
outside point. Independent controller fixtures checked identity and affine
placement, near/boundary refusal, a nested occurrence, unselectable member,
star rejection, a loop with a hole, a missing-tolerance document and simulated
managed pressure. Two coincident objects appeared in the expected depth order
in the overlap chooser, and either row could be selected without reopening the
popup. Under simulated pressure, an existing selection, document and Redo
were preserved; a distant object could still be picked. The convex certificate
also accepted a 1,024-vertex regular polygon and rejected a 1,025-vertex one
in a helper test; this does not establish full UI support for every such loop.

The focused controller tests passed, followed by four
`matrix_selection_sequence`, seven `rubber_band` and eleven `shell_docking`
tests; the no-default-feature check passed. The first UI fixture omitted
document tolerance and correctly refused the new path; the fixture was then
changed to use the document template tolerance, with the missing-tolerance
case retained as an explicit refusal test. Actual desktop interaction,
physical RAM exhaustion, native allocation peak, arbitrary nonconvex or holed
fallbacks, and global numeric proof remain open. The convex proof is
quadratic and runs synchronously on click; responsiveness on complex real
drawings remains unmeasured. The running desktop binary was not replaced.

## Synchronous source-pick timing

The full controller source-pick helper was measured on an AMD Ryzen 9 5900X
in the existing debug test build, after one warm-up for each project-authored
regular polygon. Five sequential samples, in microseconds, were:

| Vertices | Before | After |
|---|---|---|
| 708 | 132715, 132728, 132897, 132605, 133327 | 11477, 11453, 11457, 11468, 11462 |
| 1024 | 276949, 276496, 276288, 277176, 276373 | 22788, 22816, 22806, 22796, 22782 |

The change hoists a global upper bound on coordinate magnitudes out of the
pair loop and reuses each edge's differences. The numerical error screen is
at least as conservative as before for finite coordinates; it may refuse some
cases the former per-pair bound accepted. A differential test compared both
screens on regular, reversed, near-boundary and outside sample cases at two
coordinate scales, asserting the optimized screen never accepted a sample the
former screen refused. The existing fallback-source, overlap and selection
tests passed. This is an uncontrolled sequential microbenchmark of the
controller helper, not a full viewport-frame latency, physical desktop,
release-build or real-drawing performance measurement. The quadratic bound
and the 1,024-vertex cap remain.

## Convex holes follow-up

The source-pick certificate now also accepts strictly convex holes when it can
verify that every hole is fully inside the outer polygon and that each pair of
holes is strictly separated. It tests separation along the edges of both
polygons, so it can establish separation even when axis-aligned bounds overlap.
The clicked point must be securely outside every hole. Loop intersection,
nesting, near contact, an unverified concave hole or a hole outside the outer
boundary cause refusal. The total vertex cap applies to all loops combined.

An authored saved/reopened planar fixture with a 708-edge outer loop and one
four-edge hole selected material via a simulated egui click and did not select
inside the hole; document and Redo remained unchanged. Controller checks also
covered simulated managed-memory pressure and retry. Certificate tests covered
overlapping, nested, nearly touching and concave holes, an outside hole,
reversed winding and two diagonally separated convex holes with overlapping
axis-aligned bounds. The `matrix_selection_sequence` group passed four tests,
`rubber_band` eight, focused fallback tests six with one timing probe ignored,
and the no-default-feature check passed. These do not prove all possible
polygon arrangements, actual desktop clicks, physical OOM or native peak.

On the same Ryzen 9 5900X debug setup described above, the helper with one
708-edge outer loop and a four-edge hole took 11650, 11648, 11657, 11644 and
11661 microseconds in five warm sequential samples. This is not a full frame
or desktop measurement. The earlier section's no-hole-only restriction is
historical; concave loops and general curved sources remain unsupported.

## Bounded concave source selection follow-up

The point-pick path can now use the existing bounded topology validator for a
nonconvex planar source when the convex certificate cannot establish a hit.
The source-pick limit is 1,024 combined edges and 523,776 unordered pairs; the
707-edge limit for other topology operations is unchanged. Hole vertices must
be independently classified inside the outer loop, and separate loops and the
clicked point must clear the tolerance-derived margin. Ambiguity is refused.
This supersedes the previous statement that all concave loops are unsupported.

A controller fixture with a 708-edge concave outer loop and a concave hole
selected material, and refused the hole, outside, and near-boundary points.
Nested holes and a separate near-contact case were refused. Seven focused
fallback-related unit tests passed (one timing diagnostic ignored); the core
trim UV test passed. The public source audit passed. An attempted `rubber_band`
filter on the library matched zero tests; it is not UI evidence. This fixture
was not checked after save/reopen, on the desktop, under physical OOM, or for
worst-case latency. The viewport still shows the full support surface when it
uses display fallback, and this change applies only to ordinary point picking.

### Save/reopen, simulated UI, and timing follow-up

A concave 708-edge outer loop with a concave five-edge hole was saved as an
`.om9` file and reopened. Simulated egui clicks selected material and refused
the hole and an outside point; document equality and one Redo entry survived.
All nine `rubber_band` integration tests and all four
`matrix_selection_sequence` integration tests passed. This does not establish
physical desktop behavior or behavior for all concave polygons.

Five warm sequential debug helper measurements on the same machine were
16039, 15723, 15739, 15725, 15785 microseconds for a concave 708-edge outer
loop with a four-edge convex hole, and 29317, 29469, 29346, 29325, 29356
microseconds for a concave 1024-edge outer loop without holes. These are
specific diagnostic fixtures, not a worst-case bound or frame-time result;
actual cost depends on geometry, holes and machine.

### Reopened concave overlap chooser

In simulated egui interaction after `.om9` save/reopen, a point over material
of a 708-edge concave trimmed plane appeared together with the source-verified
surface in depth order. Both chooser rows were individually clicked; the
chooser closed and selected the requested object. A second point over the
hole selected directly, and document/Redo were unchanged. All five
`matrix_selection_sequence` integration tests passed. This does not verify
a physical mouse, actual desktop, every overlap arrangement or curved source.

### Surface Snap through a verified fallback hole

Fallback occlusion now also admits a rear surface when a ray intersects a
strictly convex hole of an affine planar source: a bounded topology check
rejects crossings and nested holes, hole vertices are independently checked
inside the outer loop, and the hit must be inside the hole by a
tolerance-derived margin. Any uncertainty, failure, or memory refusal remains
occluding. The existing outside-outer-bounds path is unchanged.

A saved/reopened source with a 708-edge outer loop and one square hole allowed
a rear surface snap in simulated wireframe and shaded interaction; material
in front blocked it. The document and Redo were unchanged. Controller tests
covered identity and affine placement, near-boundary refusal, invalid nested
holes and simulated memory pressure. Fourteen Surface Snap UI tests and eight
fallback-related unit tests passed; one unrelated timing diagnostic was
ignored. Physical desktop, physical OOM, arbitrary holes and worst-case
synchronous latency were not measured. The fallback still renders the full
support surface, so this change does not make the hole visually accurate.

### Concave hole Surface Snap

The source-occlusion exception now classifies the ray intersection inside a
bounded hole and checks its distance from every hole segment, then validates
all loop topology and verifies hole vertices inside the outer loop. It no
longer requires strict convexity. The UV guard and numerical margin are kept;
ambiguous geometry, nesting, or admission failure remain occluding.

An authored 708-edge affine planar source with a five-edge concave hole was
saved/reopened. Simulated wireframe and shaded Surface Snap found the rear
surface through the hole and did not pass through material; document and Redo
were retained. A focused classifier test rejected near-edge, outside and
non-finite points. Seven `hole_` unit tests and 14 Surface Snap integration
tests passed. The current debug application built in the existing target.
Arbitrary concave topology, visible desktop, physical OOM and worst-case
latency remain unverified. The display fallback still draws the full support.
### Curved fallback: valid rear hit and seam duplicates

One authored cylindrical surface with a 708-edge trim was saved and reopened.
Its scene uses the untrimmed display fallback. For an oblique center ray, the
source query returned two distinct forward intersections: the nearer one was
outside the trim and the farther one was inside, with the latter's world
position checked against the analytic cylinder. The current point-pick path
rejects this surface before classification because it requires an affine
plane, so this valid material remains unselectable through that path.

A separate ray through the periodic seam returned three records: two at the
same front distance and a distinct rear hit. An initial assumption of exactly
two records failed for this seam ray; the retained test checks the duplication
explicitly. The source API documents the returned intersections as unordered.
Any future implementation must handle their order and coincident seam UVs,
classify every relevant hit conservatively, and establish a suitable numerical
margin for curved/placed surfaces before accepting a hit. This investigation
does not implement or certify such a solution, nor does it prove physical
desktop selection or every curved surface. One focused test passed; the
fallback-filtered library batch passed nine with one timing diagnostic ignored.

#### Simulated viewport confirmation

The same 708-edge cylinder was saved/reopened and exercised through a
simulated egui viewport click, with an oblique camera aimed through the
excluded front and retained rear region. The surface was not selected, while
the document and a prepared Redo entry remained unchanged. As a control, the
same camera and click selected the untrimmed support cylinder. This confirms
the reproduced limitation reaches the interaction path for this fixture;
the egui test is not a physical mouse or GPU-present test. All seven tests in
`matrix_selection_sequence` passed. No source-picking behavior was changed.

#### Scoped cylinder source-pick implementation

The reopened 708-edge cylinder and simulated viewport click above now select
the verified rear material. The source surface must exactly reproduce the
Rust cylinder constructor, and the occurrence currently permits translation
only; other curved sources and affine linear transforms remain refused. Hits
are sorted by forward distance, checked against the source, and classified
with a numerical UV margin; ambiguous, near-tangent, near-boundary and
unverifiable cases are refused. The margin is a conservative numerical
heuristic, not a general proof for all geometry or floating-point errors.

The focused controller fixture also checks the duplicate front seam records,
a hole, exterior and near-boundary points, a tangent ray, a modified source,
and simulated managed-memory refusal followed by retry. The reopened egui
fixture checks the selected object, unchanged drawing and Redo, with an
untrimmed support control. Nine fallback-filtered library tests passed, one
timing test ignored; seven selection integration tests, the no-default check
and the existing debug build passed. Physical desktop, physical OOM, native
peak, broad geometry and all placements remain unverified. The fallback still
displays the full support surface. The earlier rejection observations are
historical for this tested cylinder.

#### Affine cylinder source-pick follow-up

Two saved/reopened 708-edge cylinder fixtures now accept an oblique rear
material pick with (1) nonuniform scale and shear and (2) reflection and
shear, including simulated egui clicks. The document and a prepared Redo
entry were unchanged. Controller checks also refused exterior and near-edge
points for those placements. A computed inverse Frobenius norm bounds linear
world-to-local error amplification; a diagonal inverse-norm check passed and
an ill-conditioned diagonal matrix was rejected by the bound helper. The
world normal used for the ray incidence includes the placement. This does not
prove all affine matrices or numerical errors, and the finite-precision UV
margin remains a conservative heuristic. Physical desktop, physical OOM,
general curved sources, native peak and worst-case latency are unverified.
The fallback still displays the full support surface.

Ten fallback-filtered library tests passed, with one timing test ignored;
eight selection-sequence integration tests, no-default check, existing debug
build and public source audit passed. The earlier translation-only restriction
is superseded for these tested cases.

#### Overlap chooser and unsupported-source admission

A saved/reopened 708-edge cylinder with scale/shear and a point in front of
its retained rear material appeared in depth order in the simulated egui
overlap chooser. Each row was chosen in a separate run; the selected identity,
unchanged drawing and prepared Redo were checked. This covers one overlap
arrangement, not a physical desktop or all visibility cases.

The new cylinder path initially attempted memory admission and an OCCT ray
query even for fallback curved sources that could not pass its cylinder
certificate. It now checks the certificate first and skips unsupported
sources without requesting the larger reservation or starting that query.
In the controller fixture, a slightly altered non-certified cylinder was
skipped even under simulated managed-memory pressure; the supported cylinder
still reports budget refusal under the same kind of pressure and succeeds
after release. No physical OOM or all allocator failures are established.

Nine selection-sequence tests and ten fallback-filtered library tests passed;
one unrelated timing test was ignored. No-default check, the existing debug
build and public source audit passed.

#### Curved fallback occlusion for Surface Snap

The earlier affine-cylinder fixture was extended with a plane between the
front and rear cylinder intersections. After save/reopen, simulated egui
Surface Snap reached the plane through the excluded front region in Wireframe
and Shaded. A separate source with material at the front did not snap to the
plane behind it. Drawing and Redo were unchanged. The controller fixture also
checks a verified hole, a target behind retained rear material, a target off
the ray, and a simulated memory refusal followed by retry. Hole behavior in
this batch was checked at controller level, not by a viewport click.

The source query verifies every returned hit before the target, permits only
hits sufficiently outside the outer bounds or inside a topology-checked hole,
and rejects uncertainty. It accepts the same exactly reconstructible cylinder
scope as source picking and uses the same inverse-placement and numerical
margin limitations; these tests are not a proof for arbitrary geometry,
placement or floating-point error. The fallback continues to display the full
support, and physical desktop, native peak and physical OOM are unverified.
All 28 `osnap_ui` tests and 11 fallback-filtered library tests passed; one
unrelated timing diagnostic was ignored. No-default check, existing debug
build and public source audit passed.

#### Viewport Line through the curved hole

The reopened scale/shear cylinder fixture now also exercises a rectangular
hole through simulated egui input in both Wireframe and Shaded. Surface Snap
finds the rear plane, then Line commits from the snapped point to a typed
endpoint; the line start matches both the snap and the expected target within
the fixture tolerances, and Undo/Redo succeeds. The existing excluded-outer
cases also exercise that Line continuation. A Shaded click near the hole edge
does not snap to the rear plane, and a retained front-material case remains
blocked. Drawing and Redo are checked before the successful Line commit.

The focused integration test passed all six fixture variants. This is
simulated UI input, not physical desktop or GPU-present evidence; other hole
topologies, placements, numerical edge cases and physical OOM remain open.
The fallback still displays the full support surface.
