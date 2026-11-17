# Trimmed standalone surface display gap

## Unsupported trim fallback can select outside the source boundary

A validated planar surface with a 129-edge polygon trim previously exceeded the
128-edge display decomposition limit. The scene warned and fell back to the
complete underlying surface, selecting it at x=1.5 even though the source trim
lies inside a radius of 1. The supported edge limit is now 500, matching the
shared verifier's bounded pair-work limit, subject to the existing metadata,
topology and memory limits. In this fixture the material is selectable and a
point behind the area outside the trim is the only object returned. Full and
incremental scene outputs agree, and their estimated managed allocations cover
retained cache usage. The full scene test suite and a focused preflight test
passed. This is not a measured process peak or performance guarantee for 500
edges; no claim is made about desktop or every shape in that range.

Fallback remains possible, including for nested loops the topology verifier
does not certify. Previously a focused fixture with such loops selected the
fallback outside the outer boundary, ahead of another object at the same screen
point. Point and crossing selection now skip unverified fallback surfaces;
the fixture instead selects the object behind. A selection window enclosing
the entire displayed fallback can still select the trimmed object. The status
warns that the full surface is displayed and offers the enclosing-window path.
The acceptance test `unsupported_trim_fallback_must_not_pick_outside_the_trim`
now passes. This guard does not provide exact selection within the trim; it is
an interim refusal for unverified display. It does not establish desktop
interaction, arbitrary topology, process peak, or exact crossing selection.

An additional egui input fixture drags an enclosing window in the Looking Down
viewport and selects the fallback object without changing the document. It
exercises the application input path, not a physical desktop session. A
separate scene fixture shows that the displayed fallback still occludes a
target behind a point outside its outer trim. This may suppress a valid Surface
Snap. Simply ignoring the fallback as an occluder would also allow snapping
through actual material; resolving the geometry requires source-aware
classification. The fixture records current behavior, not a correctness claim.

## Surface Snap candidates behind fallback

Unverified fallback surfaces are now excluded from snap candidates before
source refinement. Previously they could fail validation and prevent a valid
surface behind from being considered. In a scene fixture, only the valid rear
surface is offered. An egui Line/Surface Snap fixture confirms that Wireframe
snaps to the rear surface; Shaded still refuses because the displayed fallback
occludes it. Both preserve the document and an existing Redo entry. The full
26-test `osnap_ui` batch and seven-test scene broad-phase batch passed. This
was the behavior before the scoped shaded refinement below.

## Shaded fallback outside a verified planar outer bound

For an unverified fallback whose source is a certified affine degree-(1,1)
plane, Shaded Surface Snap now checks the exact source ray intersection. It
may ignore that fallback only when the one verified hit lies well outside the
UV bounding box of its outer loop. The guard accounts for document tolerance,
coordinate scale and the conditioning of the transformed plane; uncertain,
near-boundary, degenerate, unsupported or memory-denied cases stay occluding.
Other displayed occluders still block the target. The source and document are
not mutated. This is a conservative numerical screen, not a proof of all
floating-point errors or a general trim-occlusion solution.

An egui fixture accepts a rear plane outside the outer bound in Wireframe and
Shaded, and refuses two points inside or very near the bound; document and an
existing Redo entry are preserved. A controller fixture checks identity and
scale/shear placement, near/inside/outside and simulated memory pressure with
retry. Scene tests also show another mesh remains an occluder when the
fallback is cleared. `osnap_ui` passed 26 tests, scene broad phase 7 tests,
the complete scene suite, focused proof/pressure test, no-default feature
check and public-source audit passed. Desktop, arbitrary curved or periodic
surfaces, global numeric error, native peak and UI latency are unverified.

### Grazing-ray guard

The first planar guard did not account for amplification when the viewing ray
is nearly parallel to the plane. A controller test using such a ray reproduced
an overconfident clear result before the fix. The guard now includes ray
incidence and the magnitude of ray/target/hit coordinates; sufficiently
grazing or otherwise uncertain cases remain occluded. The regression passes
for the tested identity and scale/shear placements, together with near-boundary
and simulated admission-pressure checks. This remains a conservative numeric
screen, not a proof for all floating-point inputs or a desktop test.

## Many-edge trim scratch allocation

The scanline decomposition now counts crossings before allocating its
temporary quadrilaterals. The admission planner still reserves the conservative
worst-case bound; this change reduces actual temporary vector capacity without
weakening admission or changing output geometry. A 129-edge convex fixture has
about 1,518 output quads against a former capacity bound of 97,152; a focused
test checks polygon area and that actual capacity is less than a quarter of the
old bound. The complete scene suite and the 26-test Surface Snap UI batch
passed. This is not a process-peak measurement: the count pass adds work and
the conservative reservation remains unchanged.

For a separate risk assessment, 32 consecutive fallback-occlusion proof
calls in debug took 15.6 ms for an identity fixture and 27.5 ms for a
scale/shear fixture. These are two aggregate samples, not frame latency,
release performance or a production limit. Repeated native calls and many
overlapping fallback surfaces may affect responsiveness; measure the actual UI
before deciding whether a worker or a correctly invalidated cache is needed.

### Full-scene admission for many-edge trim

The full-scene planner previously used the edge-count worst case even after
the builder stopped allocating that many quads. A 129-edge planar fixture
reproduced a 4 MiB managed-budget refusal. The planner now counts crossings
from already-decoded trim on fixed stack storage before reserving scene memory,
adds capacity headroom and retains an allowance for fallback display. A
saved/reopened application-controller fixture now builds and picks the retained
region at that limit while preserving an existing Redo entry; a smaller 64 KiB
fixture is refused. Full and incremental scene buffers agree in the existing
fixture, and the planned bound exceeds retained cache bytes there.

The incremental-addition planner still uses its previous conservative bound:
it has not been changed to parse unreserved metadata. This is managed-budget
evidence for these fixtures, not a physical process peak or a proof of every
allocator's capacity behavior. The additional scanline pass also has a CPU
cost that has not been measured at the supported upper edge limit.

The OM9 clipboard path was traced and uses a full candidate scene, so it can
use the improved planner without changing incremental admission. A controller
fixture copies the 129-edge surface and pastes it into another document with
a 4 MiB managed limit, then verifies one-step Undo/Redo. Simulated memory
pressure before paste refuses it while preserving the document and a previous
Redo; after release the paste succeeds. Both trim-budget tests passed and the
clipboard batch passed 18 tests with three ignored. This does not establish
the physical desktop clipboard path or real operating-system exhaustion.

## Scoped scene implementation

The surface scene paths now attempt source-derived, parameter-space trim
meshing, including polygon holes, using the scanline decomposition described
below. The full-capture and incremental-addition paths construct the same
buffers. The planner reserves a conservative bound for output and temporary
data, and the incremental preflight estimates without parsing the extension.
Changing trim metadata prevents reuse of an old surface buffer. The earlier
rectangular-trim scene regression now passes; a planar hole fixture exercises
scene picking and misses in the hole while selecting retained surface. An
independent prototype test samples a cylindrical patch. The complete scene
suite and the DupBorder, Isocurve and Wireframe batches passed (24, 25 and 76
tests, with four Wireframe diagnostics ignored). The debug build and public
source audit passed.

This is a bounded approximation, not CAD geometry or a global error guarantee.
It accepts at most 128 loop edges and 16 KiB trim metadata. If trim display is
unsupported or topology is invalid, the scene preserves the prior full-source
display and records a scene diagnostic; the status bar now warns about this
fallback in the headless UI fixture below. The checks
above do not establish all polygon/near-degenerate cases, seams, arbitrary
curvature, final GPU pixels, physical desktop, Undo/Redo after trim editing, or
native/process peak. Conservative bounds may refuse a scene under pressure.
Historical sections below describe earlier stages and are not current status.

## Visible fallback warning

The status bar presents a persistent warning when the current scene reports
the unsupported-trim fallback. Errors and existing warnings retain priority;
after an error is acknowledged the trim warning reappears while the scene
still contains the fallback. One headless egui fixture using nested holes
checks this behavior and renders a frame; all 11 `shell_docking` tests passed,
as did the debug build and public source audit. This does not establish
physical desktop visibility, narrow-window text legibility, accessibility,
or correctness of the fallback surface geometry. The application still shows
the full source for unsupported trim, rather than exact trimmed geometry.

## Saved trim scene and Untrim history

An application-level fixture saves and reopens a planar source with an inset
rectangular trim and a rectangular hole. The reopened source scene vertices
remain within the inset boundary. Menu dispatch of Untrim creates a second,
untrimmed surface while preserving the original trimmed source. Undo restores
one scene object and Redo restores two; the original source scene remains
within the inset throughout. Saving and reopening the result again preserves
two objects and the source's inset scene vertices. The full `surface_menu`
batch passed 63 tests. This checks bounding vertices and scene object counts,
not hole picking after history, in-place trim edits, curved surfaces, physical
desktop or complete pixel equivalence.

## GPU pixels for a reopened trim hole

An offscreen GPU fixture reopens a planar surface with an inset trim and one
rectangular hole in the application, using a Shaded viewport. Comparing the
published texture with the same scene after deleting the surface, a sample in
the retained region changes while samples inside the hole and outside the
trim do not. Undo restores all three sampled pixels and Redo restores the
deleted state. The focused test used an available GPU adapter and passed; it
does not prove full image or boundary-pixel accuracy, a physical desktop
click, different cameras or curved-surface rendering.

## Surface Snap on display fallback

A nested-hole source that fails trim-topology verification falls back to a
full-source display. Before this change, a Surface Snap candidate at its
center was refined on the underlying surface and accepted despite the trim
being unverified. Display buffers now carry a typed fallback flag through
shared ownership and both full and incremental construction. Candidate
validation refuses Surface Snap on that fallback buffer. A headless viewport
Line interaction confirms no snap candidate and no new document/history
entry; a separately constructed valid planar trim still refines at its center.
The `osnap_ui` and `osnap_brep` batches passed 21 and 1 tests, the focused
scene metadata batch passed eight, the debug build and public audit passed.

This does not make Surface Snap trim-aware for all valid surfaces: exact ray
refinement still intersects the underlying surface without classifying its
UV hit against the trim, and multi-hit curved surfaces can require finding a
later valid hit. Curved cases, seam placement, physical desktop and global
accuracy remain open. The display fallback itself still shows the source.

## Bounded UV ray intersections — foundation only

The OCCT adapter now has a separate bounded query returning up to 32 forward
surface-ray intersections with world point, source UV and ray distance. It
refuses overflow instead of publishing a partial list; Rust validates inputs,
finite outputs and the configured output limit. A cylinder fixture observed
two hits for identity, translation and one scale/shear placement, and checked
each returned UV by evaluating the original surface and applying placement;
miss and output-budget refusal cases also passed. The kernel library batch
passed 33 tests, with three ignored. This native adapter is not yet called by
Surface Snap: UV trim classification, ambiguous boundaries, ordering and
later valid intersections, native allocation accounting, saved documents and
UI must be addressed before integration. One fixture does not prove UV
preservation for all affine transforms or periodic surfaces.

The existing `l2_02_05` integration target could not compile because an
unrelated pre-existing test accesses the now-private `Brep.display` field;
the new fixture was run independently as `surface_uv_ray` with the 3DM feature.
That test was subsequently updated to use `display()` and all 14 tests in
`l2_02_05` passed.

## Trim-aware Surface Snap source refinement

Surface Snap now uses bounded UV ray intersections for standalone trimmed
surfaces. It verifies the returned source UV against the original placed
surface, classifies the point against validated outer and inner loops, skips
hits outside the trim, and refuses numerically ambiguous boundaries or
inconsistent source coordinates. Simultaneous close distinct hits are
refused. Managed scratch is reserved before synchronous refinement; the
worker admission includes the additional bounded trim scratch.

For a saved/reopened cylinder, a ray crosses a front hole and meets the rear
surface: the old path selected the front, while the new path selects the rear.
Simulated managed-memory pressure refuses the query without changing the
document, and retry after release succeeds. A separate worker fixture checks
insufficient scratch refusal and the same rear result with adequate
reservation. UV tests check inside, outside, hole and ambiguous boundary.
The seven Surface Snap library tests, 22 UI tests, one B-rep snap test, 14
native surface tests and the focused UV test passed; no-default-feature check,
debug build and public-source audit passed.

The rounding screen is not an interval proof or a full geometric tolerance
analysis. Native allocator peak is not bounded by the managed scratch estimate;
multi-hit overflow is refused at 32 hits. Other imported or near-degenerate
trim loops, periodic seam boundaries, arbitrary placement, physical RAM and
desktop are unverified. The old untrimmed-display fallback remains warned and
is refused for Surface Snap.

The saved/reopened cylinder fixture was also run with a combined nonuniform
scale and shear. For both that placement and identity, the rear hit matches an
independent analytic world-space point; simulated memory pressure preserves
the full document and one existing Redo entry, and retry after releasing
pressure succeeds without consuming Redo. This extends only that fixture and
does not certify all affine transforms or actual RAM exhaustion.

## Surface Snap at a certified closed seam

On a saved/reopened full-seam trimmed cylinder, the UV classifier previously
treated the seam as an ambiguous boundary and refused an otherwise valid
Surface Snap. When the existing closed-axis and full rectangular trim checks
certify the seam, the seam hit is now accepted only at an exact domain end,
strictly away from real transverse boundaries and with hole vertices separated
from both seam sides by a floating-point guard. The saved/reopened fixture
passes for identity and translation. Focused helper checks do not certify the
special case for a true boundary, a near-but-not-exact seam parameter, or a
hole too close to the seam. Eight Surface Snap library tests, 23 UI tests and
one B-rep snap test passed; debug build and source audit passed. This is not a
proof of general periodic seams, arbitrary placement, extreme scales, physical
desktop or complete numerical error bounds.

The normal `Scene::capture` route currently passes only `entity.geometry` to
`cache`, and `Geometry::Surface` produces a 12×12 grid over the underlying
surface's entire parameter domain. The entity's trim extension is not passed
to that function. The incremental added-entity path also calls `cache` on the
geometry alone. Thus this is a scene-data mismatch, independently of the recent
source-based DupBorder and ExtractWireframe boundary calculations.

An explicitly ignored regression in `crates/om9-scene/tests/metadata_cache.rs`
creates a 10×10 planar source with a rectangular trim from (0.2, 0.2) to
(0.8, 0.8), captures a normal scene, and checks its display vertices against
the 2–8 mm retained region. Running that regression with `--ignored` fails as
expected: the scene contains vertices outside the retained region. The regular
metadata-cache batch passed four tests with this one ignored. This does not
establish the appearance of an actual desktop window or exercise click picking.

This gap makes it misleading to infer that drawing and selecting the source
match the exact curves from the trim-command fixtures. A fix needs a bounded,
trim-aware scene representation for the supported polygon loops, including
holes and curved underlying surfaces, consistent wire and triangle picking,
source/extension invalidation, full capture and incremental update paths, and
correct memory accounting. Invalid or numerically ambiguous trims need an
explicit safe policy; simply clipping triangle centroids may leak geometry
across a trim or hole. Then test placement, save/open, edits, undo/redo and GPU
rendering. No viewport implementation or user document was modified in this
diagnostic; actual behavior remains unresolved.

## Incremental buffer reuse prerequisite

`Scene::update_owned` formerly considered two entities identical when their
underlying geometry and placement matched, even if their trim extension had
changed. It now refuses incremental buffer reuse if the surface trim extension
differs, leaving the caller to use a full scene construction path. A focused
test covers adding and removing trim metadata while leaving the NURBS geometry
unchanged. The complete `om9-scene` test suite passed, with the known display
regression still ignored. This guards against retaining stale buffers; it does
not implement trim-aware display, prove automatic fallback in every caller, or
fix the full-domain rendering issue described above.

## Surface display admission prerequisite

The scene allocation planner previously treated `Geometry::Surface` as an
unsupported display type, although actual scene construction tessellated it.
A test with twenty separate planes reproduced an estimated peak smaller than
the retained scene cache. The planner now accounts for each surface's fixed
grid buffers and the largest simultaneously needed pair of basis rows, as well
as including it in the scene-object count. The regression and all nine tests
in `instance_admission` passed; the complete `om9-scene` suite also passed.
The estimate is managed allocation accounting, not a measurement of allocator
overhead or process peak. The future trim-aware representation must replace
the fixed-grid bound with its own bounded planning and scratch estimate.

## Experimental parameter-space decomposition

A test-only scanline prototype divides noncrossing polygon loops into
horizontal trapezoids using even-odd crossings, with eleven subdivisions in
each parameter direction. Its two tests check area and sample centers for a
concave boundary with a hole, and area for a triangular diagonal boundary;
an over-128-edge input is refused. The full `om9-scene` suite passed, with
the existing display regression still ignored. This is not wired into scene
construction: loop topology validation, floating-point ambiguity, curved
surface evaluation and seam behavior, wire boundaries, memory admission,
incremental updates, GPU and picking still need verification. The prototype
requires noncrossing validated loops and is not a proof for arbitrary inputs.

The existing command-side topology check has subsequently been moved into
`om9-core` so scene work and the existing commands can share it. The test-only
prototype now calls both trim metadata validation and that topology check
before decomposition. Focused fixtures reject a self-crossing outer loop and
nested holes and accept a valid rectangular hole. The screen still has not
been connected to these results, and this check does not establish numerical
robustness for every near-degenerate input.

The test-only prototype now evaluates the decomposed UV quads on their source
surface and creates triangle and boundary-edge buffers with fallible vector
allocation and retained-buffer usage accounting. A planar rectangular patch
with a rectangular hole checks triangle centroids and boundary count; a
cylindrical patch checks sampled radius and height. These tests pass alongside
the earlier three prototype tests. This is not a proof of coverage or exact
boundary pixels: triangle interiors and curved chord error are approximations.
The temporary trapezoids and parsed metadata are not yet integrated into the
scene's peak admission; placement, source invalidation, seams, hole-edge
picking and actual scene publication remain unverified. Nothing is wired into
the product scene.
