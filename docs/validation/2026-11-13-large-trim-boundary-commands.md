# Large trimmed-surface boundary commands

ExtractWireframe and DupBorder shared an independent 500-boundary refusal even
after the trim-topology verifier and scene were extended to 707 edges. Their
shared boundary builder now uses the same topology limit, preserving existing
output-count checks, source/placement handling, memory admission and atomic
transaction behavior.

An authored 501-edge outer loop with a four-edge hole was saved/reopened and
processed by both commands through CMD and menu dispatch. Each produced 505
distinct expected world-space line edges under scale/shear, saved/reopened its
output, and passed one Undo/Redo. Simulated managed pressure before each CMD
run preserved the document and Redo; retry succeeded after pressure release.
For separate 707-edge planar fixtures both commands produced 707 curves and
passed Undo/Redo. A 708-edge fixture and a 707-edge fixture with output limit
706 were refused with document and Redo intact. The 707-edge fixture checks
output count and history, while the 505-edge fixture checks every endpoint.

The first focused test called the wrong DupBorder menu identifier; it was
corrected to the existing `OM9-CURVE-039`. The first full regression run then
failed an old test expecting a 600-edge trim to be rejected. That test was
updated to the still-unsupported 708-edge topology boundary; the final
wireframe suite passed 86 tests with eight ignored, and DupBorder passed 24.
The no-default-feature check and public source audit passed. These are authored
planar fixtures; arbitrary topology, curved source boundaries, physical RAM
exhaustion, UI recovery for these fixtures, desktop and native peak remain
unverified. The running desktop binary was not replaced.

## Curved boundary follow-up

A separate authored cylindrical patch has 501 axis-aligned UV boundary
segments, creating both circular-arc and straight source curves. After `.om9`
save/reopen, ExtractWireframe and DupBorder each produced all 501 distinct
curves under an affine placement. Every curve's endpoints and midpoint were
compared to the corresponding source surface evaluation; each midpoint also
passed the independent cylinder-radius check. The output survived another
save/reopen and one Undo/Redo. Simulated managed-memory pressure preserved the
document and Redo for both commands, and retry after releasing pressure passed.

The focused test ran twice and passed; the final run added midpoint and radius
checks. It exercises CMD invocation on one authored cylinder, not general
curved trims, diagonal parameter boundaries, physical OOM or desktop UI.
This evidence supersedes only the earlier statement that curved boundaries had
not been checked.

## Many-edge ExtractIsocurve

An authored planar trim with a 501-edge outer loop and a four-edge hole was
saved/reopened, then ExtractIsocurve was invoked through CMD and menu. At a
horizontal parameter crossing the hole, both paths returned the two expected
world-space spans, checked against independently computed polygon/line
intersections and the known hole endpoints under affine placement. A managed
memory refusal at completion preserved the document, pending input and Redo;
retry after releasing pressure succeeded, and one Undo/Redo restored each
state. The complete `isocurve` group passed 27 tests. This is one authored
planar fixture, not a proof for all many-edge loops, curved trims, physical OOM
or desktop UI.
## Boundary commands up to 1,024 edges

For a 708-edge planar source, both commands previously returned
`Unsupported("trim topology exceeds verification limit")` despite the
display path accepting the source. The two commands now use the bounded
1,024-edge / 523,776-pair topology verifier used by display. This changes
only their boundary extraction path; the separate ExtractIsocurve verifier
retains its prior limit.

After OM9 save/reopen, planar fixtures with 708 and 1,024 edges were processed
through ExtractWireframe CMD and DupBorder menu dispatch. Every output curve's
endpoints were matched uniquely against the corresponding source polygon
edge; each command had one Undo/Redo. At 1,024 edges simulated managed-memory
pressure refused before mutation and retained drawing/Redo, then retry passed.
Output limits below the required count and a 1,025-edge source were refused
with drawing and Redo unchanged. A separate reopened affine cylindrical
708-edge fixture verified endpoints, midpoints and source radius for both
commands, save/reopen outputs, Undo/Redo and simulated memory refusal.

Wireframe integration: 87 passed, eight ignored; DupBorder integration:
24 passed. The no-default check, current debug build and public source audit
passed. These authored fixtures do not prove all trim loops, imported models,
all curved boundaries, physical desktop, real OOM or native peak.

## ExtractIsocurve up to 1,024 edges

The previous 707-edge topology verifier rejected a reopened 1,024-edge planar
trim with a hole. ExtractIsocurve now uses the same bounded 1,024-edge topology
check as display. A saved/reopened affine plane with a 1,020-edge outer loop
and four-edge hole produced two source-clipped segments through CMD and menu
dispatch; each endpoint was checked against the independently computed UV
crossings, with one Undo/Redo. Simulated pressure refused while retaining the
pending session, document and Redo; retry succeeded. A 1,025-edge source was
refused with the pending session, document and Redo preserved, then canceled.

A saved/reopened affine cylinder with 1,024 trim edges including a hole also
produced two clipped curves; endpoints and interior samples matched the
surface source and Undo/Redo passed. The 29-test Isocurve integration batch,
no-default check, existing debug build and public source audit passed. These
authored fixtures do not prove arbitrary trim, all input modes, physical
desktop, actual OOM or native peak.

## Reuse of a 1,024-edge 3DM roundtrip

One authored planar trimmed source with a 1,020-edge outer loop and a
four-edge hole was exported to 3DM, imported as an editable B-rep, saved as
OM9 and reopened. ExtractWireframe and DupBorder each produced 1,024 curves.
Every output endpoint pair was matched uniquely to the original source trim
in world coordinates, and each command passed Undo/Redo. With simulated
managed pressure, each refused before mutation and preserved the drawing and
prepared Redo; retry after release succeeded.

This exercised the native exchange adapter and source-based command paths,
not an external Rhino-authored 3DM, physical desktop, arbitrary imported
topology, physical OOM or a native peak measurement. The focused integration
test passed with the required native exchange executable enabled.

## Reuse of a 1,024-edge cylindrical 3DM roundtrip

An authored cylindrical patch with a 1,020-edge outer loop and four-edge hole
was exported as selected geometry through the native 3DM host, imported as a
B-rep, saved as OM9 and reopened. Ray checks distinguished material, hole and
outside. ExtractWireframe and DupBorder each returned 1,024 uniquely matched
source boundary curves; endpoints, cylinder radius and midpoint height were
checked, with one Undo/Redo per command. Simulated managed-memory pressure
preserved drawing and Redo; both commands succeeded after pressure release.
The focused native-exchange integration test passed. This does not establish
behavior for external Rhino-authored files, arbitrary topology, physical
desktop, actual OOM, or native peak memory.

## Isocurve limitation on a reopened 3DM cylindrical face

Historical result before the single-face implementation below:

The authored 1,024-edge cylinder above imports as a B-rep. After OM9
save/reopen, ExtractIsocurve previously rejected it with a generic surface-selection
error; it now reports that B-rep faces are not yet supported, before starting
an editing session. The drawing and prepared Redo are unchanged.
The focused native exchange integration test passed. Dispatch currently
accepts Geometry::Surface only. The separate OCCT split-face isocurve adapter
constructs edges for a split and does not itself expose clipped isocurve output;
it is not evidence that extraction supports imported faces. Selecting the
correct source face, clipping at its actual wires and holes, preserving
placement and transactions require a product implementation and tests. This
is one OM9-authored 3DM fixture, not external Rhino interoperability.

The updated focused 3DM integration test and all 29 Isocurve integration tests
passed; `om9-app --no-default-features` checked successfully. This changes the
error message only, and does not implement extraction from B-rep faces. Desktop
message presentation and independently authored Rhino files were not tested.

## Single-face B-rep isocurve after 3DM

ExtractIsocurve now accepts an OCCT B-rep containing exactly one open face.
Rust retains the source stamp, session, budget and transaction; the native
OCCT bridge reads the same source, obtains face UV bounds, constructs the
isoparametric curve, intersects it with the face and returns bounded NURBS
segments. Output placement is applied by the Rust transaction. Multi-face
solids and NativeBrep remain unsupported. A 4 MiB plus four-times-source-size
reservation is an estimate, not a hard bound on OCCT allocations.

An authored 1,024-edge cylindrical trim with a hole was exported/imported by
the real exchange host, saved/reopened as OM9, and moved through the product
command. CMD U and menu V each produced two curves; their four endpoints were
matched to independent source-surface evaluation under translation; the
midpoint of each segment also matched its source UV midpoint. Each passed
Undo/Redo. Simulated managed-memory pressure at start and completion
preserved drawing and Redo; completion retained the pending session and retry
succeeded. One smaller reopened planar B-rep with a hole produced two
source-matched segments under translation; a solid box was refused without
changing drawing/Redo, and deleting the pending source caused stale refusal.

Focused imported-cylinder and planar tests passed; the Isocurve batch passed
30 tests before the final stale assertion was added, then that focused test
passed again. The pending-reservation unit test, no-default check and
wireframe regression (87 passed, eight ignored) passed.
This is not a proof for arbitrary topology, tangent/coincident trims, every
OCCT result, external Rhino-authored files, actual OOM, native peak or desktop.
The existing debug executable was rebuilt after confirming no OM9 process was
running; no desktop interaction was tested.

## Concave single-face B-rep and coincident boundaries

A separate authored planar B-rep was saved/reopened with an eight-edge concave
outer boundary and two four-edge holes, then translated. At U=0.5 the command
produced four distinct spans whose endpoints match the independently specified
material intervals. At U=0.4 the isocurve coincides with both hole boundaries;
its returned spans covered the expected material and boundary without gaps or
nonzero overlaps. U=0.35 crosses the two concave-notch vertices and coincides
with its bottom; U=0.1 coincides with the outer boundary. Both covered the
expected spans with no gaps or overlaps. Each successful run passed one
Undo/Redo. U=-0.01 was refused while retaining the session, drawing and Redo,
and editing it to U=0.5 succeeded. All 31 Isocurve integration tests passed.
This is one authored planar fixture; curved or arbitrary coincident edges,
external files, actual OOM and desktop remain unverified. The test extends
evidence without changing production behavior.

## Single-face NativeBrep isocurve

Previously ExtractIsocurve rejected retained NativeBrep. It now converts its
owned 3DM payload to temporary OCCT source using the existing admitted bridge,
then uses the same single-open-face clipping and output transaction as B-rep.
Rust retains the source stamp and placement, checks managed admission at
dispatch and completion, counts the converted source while outputs are alive,
and rechecks pending reservations. A bounded estimate for native work is not
a hard limit on OCCT or openNURBS allocations. No display mesh is used as the
source of the output curves.

After OM9 save/reopen, authored planar and cylindrical NativeBrep faces with a
hole each produced two source-matched spans in U and V under translation.
The cylinder endpoints and midpoints match independent surface evaluation.
The planar CMD and menu paths passed Undo/Redo. Simulated full and partial
managed pressure at start/completion preserved drawing and Redo; completion
kept the session for retry. Deleting the source while pending, a malformed
payload and a multi-face solid were refused. Pressure tests confirm a managed
budget error but do not instrument the exact native failure site.

Isocurve integration: 33 passed; Wireframe: 87 passed, eight ignored;
no-default app check passed. These are two authored fixtures, not a general
proof for arbitrary geometry, externally authored Rhino files, desktop, actual
OOM or native peak. NativeBrep conversion occurs again at completion, and
performance for large payloads has not been measured.

The shared curve-output path now maps its known Rust allocation failure to a
managed memory error, preserving callback admission errors separately from
output-count limits. No allocator failure was injected end to end for this
command. The existing debug executable rebuilt successfully with no OM9
process running; desktop interaction remains untested. Public source audit
reported zero errors.

## Picked face of a multi-face B-rep

ExtractIsocurve now accepts one preselected root B-rep face. The existing
Extract Surface display/source verifier checks all displayed triangles for the
selected face; the native isocurve operation resolves its witness again against
the same source face map instead of trusting a persisted index. Rust owns the
session, selection revision, reservation and atomic publication. A selection
change invalidates both an active command and pending memory recovery before
any old Undo can be removed. Unselected multi-face objects, nested occurrences
and face picking on NativeBrep remain unsupported.

A box fixture saved/reopened with two display face slots permuted produced
isocurves on each selected source face through CMD and menu dispatch. Their
plane and extents were checked against the 4 x 6 x 8 box, with one Undo/Redo.
Simulated managed pressure at start/completion preserved drawing, Redo and
pending input; retry passed. A mismatched triangle was refused, and changing
the selected face while the command or memory recovery was pending preserved
the document and both history stacks. The memory-recovery tests invoked the
recovery handler directly. An egui test selected the top face of a box in the
viewport with Ctrl+Shift and used CMD to create an isocurve at the correct
height; Undo/Redo passed.

Isocurve integration: 35 passed; filtered Isocurve unit/UI tests: 16 passed;
reopened 1,024-edge cylinder native exchange test passed; the existing mixed
face Extract Surface regression and no-default check passed. These fixtures
do not prove all topology, curved picks, external Rhino files, physical
desktop, real OOM, native peak or responsive performance on large faces.
The OCCT reservation remains an estimate, not a hard native allocation cap.

The current debug binary rebuilt in the existing target after checking that no
OM9 process was running. Public source audit reported zero errors. The egui
viewport test is simulated and does not replace physical desktop acceptance.

## Picked curved faces and synchronous timing

A cylinder B-rep with its side and cap display face slots swapped was saved
and reopened under nonuniform scale, shear and translation. For its selected
side, CMD U=2.5 produced the full circle and menu V=0.5 the axial line. Five
samples of each were transformed back and checked against radius 2, the
expected height or angle and the original 0..5 height range; one Undo/Redo
passed for each. A separately authored reopened sphere with a 30,016-triangle
display produced the selected equator: five samples matched radius 3 and z=0,
with Undo/Redo. These checks do not certify all trimmed or imported curved faces.

Three standalone debug test runs on this machine measured synchronous
controller dispatch at 498.70, 495.59, 497.38 ms and completion at 363.18,
365.92, 361.43 ms for that sphere. They are individual wall-clock samples,
not a release benchmark, full-frame measurement or desktop latency percentile.
The source verifier and OCCT clipping currently run synchronously; investigate
reusing the existing cancelable face-verification worker with immutable
source/scene captures, selection and stale checks, bounded admission and atomic
publication. Do not weaken the all-triangle verification merely to improve time.
The Isocurve integration suite passed 37 tests. No product code changed in this
follow-up; no desktop interaction, native peak or real OOM was measured.

## Selected-face verification worker for Isocurve

For a preselected B-rep face whose display exceeds 512 triangles, Isocurve
now reuses the existing cancellable face-verification worker. It captures the
source and display, checks all selected-face triangles off the UI thread and
revalidates session, scene publication, root/subobject selection and command
revision before starting the pending Isocurve. Small displays retain the
existing synchronous route. Cancel, stale selection and budget failure keep
the drawing and Redo. The recovery UI waits for an asynchronous retry; after
one consent it retries once after worker memory is released and prunes the
oldest Undo only if still necessary. If the selection changes, recovery stops
before pruning. An OCCT call cannot be interrupted in its interior.

For the same authored 30,016-triangle reopened sphere used above, three debug
controller runs measured dispatch 2.67/2.24/2.27 ms, maximum individual poll
0.98/1.07/1.00 ms, and synchronous final curve creation 359.01/364.34/359.20 ms.
The earlier synchronous dispatch was 498.70/495.59/497.38 ms. These are three
wall-clock samples on one machine, not a release benchmark, full UI frame,
percentile or proof that all models are responsive. The worker still takes
time to complete; the final clipping and transaction remain synchronous.

A simulated egui test started the worker on a 30,016-triangle sphere, polled
to the prompt, created the curve and checked Undo/Redo. Controller tests
covered cancel, stale selection including concurrent pressure, no-root
subselection, budget refusal and retry. One recovery-handler test simulated
two worker budget failures under one consent and verified retry-before-prune
and retained Redo; another egui test confirmed that the warning stays open
until verification completes. These are simulations, not physical OOM or
desktop testing. Isocurve integration: 38 passed; filtered Isocurve unit/UI:
18 passed; filtered egui: 2 passed; face-worker regressions: 2 passed;
no-default app check passed. Native peak and general imported topology remain
unverified.

The existing debug executable rebuilt successfully after confirming that no OM9
process was running. The public-source audit reported zero errors. No separate
build directory was created.

## Large retained-model insertion validation reuse

For the 30,016-triangle sphere, temporary debug instrumentation measured
about 356 ms inside transaction preparation of a single isocurve insertion,
while three direct source-clipping calls took 0.76/0.84/0.80 ms with a no-op
admission callback. These separate calls do not measure every product cost.
One instrumented baseline attributed approximately 148 ms to pre-clone work,
74 ms to staged validation and 145 ms to subsequent admission. The temporary
instrumentation was removed.

The existing immutable candidate validation proof was previously seeded only
for insertion batches of at least 20. It is now also seeded for insert-only
transactions when retained usage has at least 10,000 points. The candidate
still receives a full validation at a tighter budget, falling back to the
normal budget when necessary; reuse requires the same immutable document and
compatible limits. This threshold is a workload heuristic, not a guarantee.
The branch also applies to other insert-only commands with large retained data.

Three debug runs on the same sphere measured final completion at
292.03/285.90/288.84 ms, compared with the earlier 359.01/364.34/359.20 ms.
The measurements are sequential, on one machine and not controlled for load;
there is no release, full-frame, desktop or worst-case claim. Verification
still happens, and roughly 0.29 s of synchronous completion remains.

A 10,001-point retained-model unit test checks one-insert proof creation,
invalid input, tightened byte and name limits, copy-on-write invalidation,
and drawing/Redo preservation plus Undo/Redo. Retained-validation group:
147 passed; atomic transaction integration: 29 passed; Isocurve integration:
38 passed; filtered Isocurve library/UI: 18 passed; filtered egui: 2 passed;
no-default check passed. No physical OOM, actual process/native peak, desktop
or concurrent timing benchmark was measured.

The existing debug executable rebuilt successfully with no OM9 process running;
public source audit reported zero errors. No new build directory was created.

## Reuse validated owner for clone admission

The pre-clone allocation estimate previously revalidated the entire source
document even when the coordinator already held a proof for that immutable
owner. Transaction preparation now asks the existing retained-document
validator for a proof under its current limits, then derives the clone estimate
from the validated usage while retaining the same shared-allocation deductions.
The public estimate API still performs its own full validation. A different
owner or tighter limits without a covering proof are validated again. This
change applies to transaction participants generally, not just Isocurve.

The same authored 30,016-triangle sphere yielded debug completion samples
230.67/218.30/219.80 ms, following the earlier 292.03/285.90/288.84 ms.
These sequential measurements are from one machine and are not controlled for
load; they do not establish desktop, release, worst-case, or concurrent latency.
A 10,001-point model test compared the cached and fully validated clone
estimates and exercised invalid input, tightened byte/name limits, owner
copy-on-write and Undo/Redo. The retained-validation group passed 147 tests;
atomic transactions 29; Isocurve 38; filtered Isocurve unit/UI 18; simulated
egui 2; no-default app check passed. Real OOM and process/native peak remain
unmeasured.

User approved recording the previously tested selected-face B-rep Isocurve
behavior; repository and installed guidance were updated to that exact scope.

The existing debug executable rebuilt successfully after confirming no OM9
process was running. The public source audit reported zero errors; the source
and installed guidance copies matched after the approved update.

## Discarded candidate-validation experiment

An experimental reuse of a validated B-rep baseline for candidate validation
produced 246.49/236.18/242.27 ms in three sequential debug completion samples
on the same sphere, compared with the earlier 230.67/218.30/219.80 ms. These
uncontrolled measurements do not establish a general performance difference;
they provide no evidence that the extra complexity helps this case. The
experiment was removed. Candidate validation and the previously retained
immutable-owner clone-admission optimization remain in place.
After removal, the focused immutable-owner transaction test passed, the
existing debug executable rebuilt, and the public source audit reported zero
errors. No additional build directory was created.

## NativeBrep multi-face selection gap

The product-controller regression `native_multiface_cannot_select_original_brep_face_after_reopen`
converts an authored 4×6×8 box to NativeBrep, saves and reopens it, and
confirms that selecting the original B-rep face key fails. Selecting the whole
body and invoking ExtractIsocurve also fails; the document and an existing
Redo entry remain unchanged. One focused test passed. This does not exercise a
physical viewport or an independently authored Rhino file.

Scene capture currently renders NativeBrep using its mesh, while
`pick_brep_display` only accepts OCCT B-rep geometry. The existing selected-face
verification and worker therefore cannot simply be reused for this mesh: a
temporary source-derived topology display must first be bound to the current
NativeBrep payload and scene, with admission and stale/cancel checks. A mesh
triangle index or a pre-conversion face UUID is not proof of a source face.
This identifies a missing path; it does not implement multi-face selection.

### Mesh-triangle witness experiment

An experimental path passed a NativeBrep mesh triangle directly to the source
face resolver. It selected an authored box face and rejected a displaced
triangle in controller tests, but that was insufficient: a planar trimmed-face
fixture contains a small hole within the triangle while its three vertices and
centroid remain on material. `source_witness_samples_do_not_prove_triangle_avoids_trim_hole`
confirms the existing resolver accepts this witness. The experimental product
path was removed. The test establishes a sampling limitation of that resolver;
it does not demonstrate a separate incorrect product selection. A source-built
display or a stronger verification of the clicked location is still required.

### Reproduced product path and discarded planar Boolean check

For an authored B-rep with a deliberately malformed display triangle spanning
a small source-face hole, a screen-space face pick at the hole returns that
face. After save/open, the selected-face ExtractIsocurve path accepts it. The
regression `malformed_brep_display_through_trim_hole_is_refused` is intentionally
ignored and fails when run explicitly against the current implementation.
This reproduces the product controller and scene-picking path, not a physical
desktop event or a naturally generated invalid display.

An experimental planar check subtracted the source face from each projected
triangle. It rejected that fixture and accepted a valid trimmed-face control;
41 Isocurve, 200 Solid and 118 Solid workflow cases passed, with one ignored
in each Solid suite. However, a 200-triangle planar debug probe took 1.552 s
with the check versus 0.016 s after removing it (one sequential sample each).
The existing synchronous path can handle up to 512 triangles, so this cost is
not acceptable without further design. The experiment was removed; the defect
remains. The timings are not a release or desktop benchmark, and the discarded
Boolean check did not establish correctness for curved faces or native peaks.

### Bounded planar-hole display check

The shared source-face display verifier now computes bounds for internal wires
of a face that OCCT recognizes as planar. Only triangles whose bounds overlap
an internal-wire bound need the more expensive projected-triangle minus face
check; the check refuses outside area above a tolerance proportional to vertex
tolerance and triangle perimeter. The native call checks cancellation between
wire and triangle operations, but cannot interrupt an individual OCCT call.

The malformed reopened B-rep fixture described above now refuses the command
and preserves the document and Redo. A genuine planar face with a hole accepts
selection, extracts two curves and passes Undo/Redo. Isocurve: 42 passed and
one timing probe ignored. The preceding candidate without the extra cancel
checks passed 118 Solid workflows, one ignored. The debug executable rebuilt;
public-source audit found no errors.

One sequential debug timing probe measured 17 ms for 200 triangles without a
hole and 201 ms for 192 triangles around a hole. These are different fixtures,
not a controlled benchmark; the second may still block a UI path under the
512-triangle worker threshold. The area tolerance can miss sufficiently small
outside regions; this does not certify arbitrary trim, outer concavities,
curved faces, file imports, physical desktop, release performance, or native
peak memory. Internal-wire bounds and Boolean work are native allocations, not
a hard managed-memory cap. Multi-face NativeBrep selection remains unsupported.
The user approved recording the tested refusal scope; repository and installed
feature guidance were updated to that scope.

### Discarded narrow-overlap optimization

An experimental axis-aligned planar shortcut skipped the Boolean when a
triangle and hole bound overlapped only in a thin strip. One debug sample of
the 192-triangle hole fixture decreased from 201 ms to 17 ms; the malformed
triangle fixture was still refused and a rotated-plane fixture also passed.
However, the Boolean currently checks the entire face, so its detected outside
area may come from a concave outer boundary rather than the hole. A thin
overlap with the hole bound alone cannot justify skipping that full check.
This is a reasoning-based risk, not a separately reproduced product failure.
The shortcut was removed; the earlier measured latency remains relevant. A
future optimization must establish an outer-boundary certificate or otherwise
preserve the check before comparing timings. The rotated-plane regression is
retained. Neither timing is a controlled release or desktop benchmark.

### Rectangle-certified narrow-overlap optimization

The verifier now uses the thin-overlap shortcut only after certifying that the
outer wire consists of exactly four distinct, straight, axis-aligned rectangle
edges and the triangle vertices lie inside that rectangle. Overlap bounds are
summed over all holes before comparing them with the area threshold. Any uncertain edge,
non-rectangular outer boundary or tilted plane keeps the previous full check.
For the certified rectangle, the overlap bounds the possible area outside the
face against the already-existing tolerance times triangle perimeter. No new
native ownership crosses the bridge.

For the same 192-triangle planar-hole debug probe, three sequential runs took
19.39/17.27/18.35 ms; the previous implementation had one 201.26 ms sample.
The probe for a 200-triangle face without a hole took 21.37/16.74/17.11 ms in
the same runs. After adding the sum over holes, one additional run took 17.57 ms
with a hole and 17.48 ms without. After the final cancellation check, one run
took 18.91 ms with a hole and 16.88 ms without. These are uncontrolled single-machine measurements, not release,
desktop, worst-case or process-memory benchmarks. A concave-outer fixture with
a barely overlapping hole bound still rejects a triangle spanning the outer
notch; the malformed hole pick, two-hole and tilted-plane tests also pass. The Isocurve
suite passed 45 tests with one timing probe ignored; separate simulated viewport
Shell and Extract Surface tests passed before the final cancellation check. Debug executable rebuilt in the existing
target, public-source audit passed.

The certificate is intentionally strict and provides no speedup when it cannot
be established. Existing area tolerance can still miss sufficiently small
outside regions. General trim, curved faces, native peak memory, real OOM and
physical desktop remain unverified; NativeBrep multi-face selection is still
unsupported.

### NativeBrep source-ray face bounds foundation

The kernel exposes a local-space source-ray query for an isocurve face's UV
bounds. OCCT reads the canonical source once, intersects the ray with its
faces, chooses the hit nearest an admitted reference within a caller-provided
distance, and refuses equally near distinct faces. It returns no persistent
face index or authorization for a later operation. Rust validates finite
inputs, source bytes and caller limits; the native adapter checks face, edge
and hit counts. The caller is responsible for input admission and for binding
the ray, reference, transform, source and scene to the actual interaction.

For an authored NativeBrep box saved/reopened as OM9, the converted source
query returned the expected 4×6 and 6×8 face spans for two rays. A distant
reference, zero direction, nonfinite coordinate, topology limit and a ray
through a shared edge were refused. An independently constructed trimmed plane
roundtripped through the NativeBrep converter accepted a ray through material
and refused rays through its hole or outside the face. Isocurve suite: 47
passed, one timing probe ignored. Kernel library: 33 passed, three ignored.

This is a kernel foundation, not a completed product path: no UI hit is yet
retained, no later isocurve extraction is bound to the same source/face, and
there is no command-level history, stale-state or worker test for this query.
Native allocations and the intersection itself are not hard-bounded by Rust's
admission callback or interruptible mid-call. General imports, coincident
faces, perspective/affine view binding, native peak and desktop are unverified.
The app's no-default-features check and existing debug executable build passed;
public-source audit reported zero errors. No new build tree was created.

### Source-ray isocurve extraction foundation

The source-ray resolver now also feeds the existing clipped isocurve emitter.
Bounds and extraction each resolve the face from the supplied local-space ray
in the native read; no face-map index is transported between reads. The caller
must hold the same immutable source and ray, revalidate the selection and
scene, and admit Rust and native work. The existing typed output admission
callback is reused; OCCT internal allocations remain outside its hard bound.

From an authored NativeBrep box saved/reopened as OM9, two rays produced curves
on the respective 4×6 and 6×8 faces; endpoints and midpoint were checked in
source coordinates. An ambiguous shared-edge ray and simulated output
admission refusal were rejected. For a NativeBrep-converted planar face with
a hole, the ray-selected isocurve produced two curves with the four expected
endpoints; a ray through the hole was refused. Five NativeBrep-filtered tests
passed; the broader Isocurve suite passed 47 with one timing probe ignored
after the final added assertions, and the kernel library passed 33 with
three ignored. The no-default-features check, debug build and public-source
audit passed. Real OOM and native allocation peak were not tested.

The product still cannot select a face of a multi-face NativeBrep: the viewport
ray must be retained and bound to the correct source/scene and safely converted
to local coordinates, and command publication, Undo/Redo, stale, cancel,
worker and desktop paths remain unimplemented or unverified.
