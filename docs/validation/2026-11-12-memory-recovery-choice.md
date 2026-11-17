# Memory warning and explicit Undo removal

## ExtractWireframe B-rep admission recovery

ExtractWireframe now uses the same admission-error preservation as DupBorder
for single and multiple B-rep source-edge extraction. A focused Box-source
test injects a managed-memory refusal in the adapter callback and confirms the
typed error survives; a separate output-count limit remains an adapter error
rather than a recognized RAM shortage. The existing DupBorder callback test
still passes through the shared helper.

In a separate 64 MiB managed UI fixture, an ExtractWireframe Box invocation
under held simulated pressure offers recovery, reclaims one oldest Undo step
that owns a mesh, and creates 12 curves; one Undo restores the pre-command
document. The end-to-end fixture does not localize the failing allocation;
callback propagation is verified separately. Other adapter allocation errors,
arbitrary B-reps, physical memory exhaustion and desktop interaction remain
unverified.

The UI recovery batch passed 54 tests. The two callback tests passed; Curve
menu, DupBorder, Isocurve and Wireframe passed 6, 24, 21 and 71 tests, with
four Wireframe diagnostics ignored. The no-default check, existing debug build
and public source audit passed. No new build directory was created.

## DupBorder with two standalone surfaces

For the typed `DupBorder` command and its menu handler, recognized memory
refusals now offer the same explicit-consent retry used for ExtractWireframe.
The recovery target retains the selected IDs and controller authority/epoch;
each step rechecks them before retrying or discarding the oldest Undo. In the
two-plane fixtures, a simulated reservation causes refusal with document and
Redo intact. Releasing pressure before consent makes retry succeed without
removing history; an egui dialog test verifies no work happens before the
button is clicked. A separate 64 MiB managed fixture keeps pressure active,
reclaims one history-only mesh through the oldest Undo step, and completes
with eight boundary curves. One Undo restores the pre-command document.
Changing the selection before recovery is rejected without discarding Undo.

This covers the tested direct `DupBorder` spelling and menu invocation, not
all aliases, physical desktop interaction or actual RAM exhaustion. Errors
wrapped by a native B-rep adapter did not retain a recognized memory error at
that stage; the follow-up below preserves callback refusals. B-rep recovery was
not tested in the earlier batch. The UI Undo settings batch passed 51 tests; Curve menu, DupBorder,
Isocurve and Wireframe passed 6, 23, 21 and 71 tests, with four Wireframe
diagnostics ignored. The no-default check, existing debug build and public
source audit passed. No new build directory was created.

## DupBorder B-rep admission recovery

The source-boundary adapter formats admission-callback errors into text. The
DupBorder caller now retains the original callback error and returns it when
the adapter fails; it does not reinterpret unrelated adapter errors as memory
pressure. A focused test injects a managed-memory callback refusal into an open
cylindrical B-rep and verifies that its type survives. A separate limit of one
output on the two-boundary shell remains an adapter error, not a recognized
memory refusal.

For a single open cylindrical B-rep, typed and menu UI fixtures under simulated
managed pressure open the retry offer while preserving the document and Redo;
after pressure is released, retry creates both boundaries without removing an
Undo step. Another 64 MiB managed fixture keeps pressure active, removes one
oldest history step that owns a mesh and then creates both boundaries; one
Undo restores the pre-command document. The end-to-end pressure fixtures do
not identify the exact allocation that failed; callback propagation is checked
separately. Multi-B-rep recovery, other adapter allocation failures, desktop
interaction and actual RAM exhaustion remain unverified.

The UI recovery batch passed 52 tests before the additional focused reclaim
test, which also passed. The callback test passed; Curve menu, DupBorder,
Isocurve and Wireframe passed 6, 24, 21 and 71 tests, with four Wireframe
diagnostics ignored. The no-default check, existing debug build and source
audit passed. No new build directory was created.

Surface evaluation and isocurve construction now reserve basis rows, output poles, weights and a copied knot vector with fallible allocation. They reject an impossible allocation layout as overflow and return the existing `bytes` memory refusal for an allocator failure; the previous knot-vector clone could abort on allocation failure. Two focused tests cover an impossible layout and, on 64-bit Windows, a representable one-EiB allocator refusal. Core library 377 passed with one ignored; curve/surface operations 31 passed; app Wireframe 55 passed with five ignored, ExtractIsocurve 11, DupBorder 17, Curve menu 6 and UI recovery 46 passed. This does not force the allocator failure inside an actual interactive command, certify all allocations in those commands, or exercise physical RAM exhaustion or native allocations. The user approved recording the tested ExtractWireframe-specific scope. The user previously declined an ExtractIsocurve guidance update; no such update is made here.

ExtractWireframe's own edge/output vector allocations formerly returned operation-specific `Error::Budget` labels that the recovery UI did not recognize. They now first check addition and allocation-layout overflow and report those as non-memory overflow; a fallible allocator refusal for an otherwise representable layout reports the existing `bytes` budget error recognized by the recovery path. Two focused tests passed: both count/layout overflow preserve the vector, and on 64-bit Windows an impossible but representable one-EiB allocation returns the expected memory error. The existing 46 UI recovery tests passed, as did Wireframe 55 (five ignored), DupBorder 17 and Curve menu 6. These tests do not force an allocator failure within an end-to-end desktop command; other errors, including errors wrapped by native adapters or allocations in other modules, may still bypass this offer. The user approved recording this tested scope in guidance.

The user selected a warning followed by automatic removal of the oldest Undo steps only as needed. The initial one-step manual design was superseded. After consent, the implementation first retries without deleting history, then removes one oldest step and retries each time only while memory refusal persists. It stops on success, non-memory error, changed context, or exhausted history. The window warns that all steps might be lost without enough memory; cancel removes none. A successful new transaction follows normal Redo branching; failed retries keep document and Redo. This overrides the usual minimum Undo count only after user consent.

The earlier one-step manual result is historical. A simulated managed-memory reservation causes ExtractWireframe to fail via the typed command; an egui click after releasing pressure retries successfully without removing any Undo, and one Undo returns the drawing to its pre-command state. Another test invokes the menu handler and verifies the same no-removal behavior. A further case keeps pressure in place and verifies that exhausting history leaves the drawing and Redo unchanged. Another changes the selection and verifies refusal before any removal; a structural budget error does not open the offer. Ten undo-settings UI tests passed. A core test with a source-evidence owner verifies rehoming while another history step needs it and releasing it after the last owner disappears; seven existing history-evidence tests passed.

A subsequent targeted test used a 32 MiB managed limit, a 50,000-vertex mesh retained only by the oldest Undo entry, and a simulated reservation that remained active throughout retry. The initial extraction was refused; discarding that one history entry released enough managed capacity for the command to complete, and Undo restored the pre-extraction drawing. This demonstrates reclamation for that fixture, not every model or physical RAM pressure. A separate attempt was constrained by OS physical-RAM headroom and still failed after Undo was exhausted, illustrating that deletion is not guaranteed to make retry possible.

The same consent-based recovery is now connected to ExtractIsocurve when its final Enter is submitted through CMD and its pending source and parameter remain unchanged. A simulated reservation test verifies the pending input survives refusal and that retry after pressure is released does not discard Undo; changing the parameter before confirmation causes refusal before history removal. In another 32 MiB fixture, one history-only 50,000-vertex mesh is released and the isocurve succeeds while the simulated reservation remains active; Undo restores the pre-extraction drawing. When pressure remains after all Undo steps are removed, a separate fixture confirms the pending input, document and Redo survive; after releasing pressure, the same input completes. These are one planar surface and controlled memory fixtures, not proof for arbitrary sources, interaction routes or physical RAM exhaustion. The user previously requested no guidance update for the Isocurve retry work; no such skill update was made.

Sphere recovery is also connected for the default center mode when the final positive numeric radius is submitted through CMD. A retained history-only 400,000-vertex mesh under a simulated 64 MiB limit releases enough capacity after one Undo removal while the reservation stays active; the result's independent OCCT volume and placement match radius 5 and center `[10,20,30]`, and Undo restores the prior document. If pressure persists after history is exhausted, the center, document and Redo remain and a later retry succeeds. Canceling and starting a new identical Sphere session, or replacing the controller with matching input, refuses a stale recovery before deleting history. The equivalent same-input session replacement check is also enforced for Isocurve. These fixtures do not cover Sphere's other completion modes, physical clicks or actual RAM exhaustion.

For default center-mode Sphere, the viewport point handler now retains the selected world point and its snap context on a recognized memory refusal; after consent, retry uses that point, then applies snap/SmartTrack completion bookkeeping on success. If the one-shot snap selection changed while the warning was open, the new selection is not consumed. Two handler-level tests checked a point `[15,20,30]` relative to center `[10,20,30]`, including one fixture with a 64 MiB limit where removing a history-only mesh allowed retry while simulated pressure stayed active. Both checked OCCT volume, placement and Undo; one also checks a non-empty End snap candidate and a later Mid selection remains intact. These call the handler directly; actual viewport mouse, broader snap cases, other Sphere modes and real physical pressure remain unverified.

For Sphere TwoPoint, recovery now retains the first endpoint, mode and working plane, plus either a numeric diameter entered via CMD or a second viewport world point. Both fixture paths produced center `[15,20,30]`, radius 5 and one Undo after simulated pressure was released. A second fixture kept the managed reservation active and succeeded after removing a history-only mesh. The two focused tests passed. Physical mouse, other planes, other Sphere modes and physical RAM pressure remain unverified. User approved recording this scoped behavior in both feature-guidance copies.

For ThreePoint and FourPoint, recovery retains the previously accepted points and derived sphere state, as well as the final viewport world point. Fixtures checked independent OCCT volume, placement and Undo after releasing simulated pressure. In an additional FourPoint fixture, removal of a history-only mesh released sufficient managed capacity while the simulated reservation remained active. The full memory-recovery UI test group passed 26 tests. These tests call the point handler directly and do not prove physical mouse behavior, all input geometries, or actual RAM exhaustion. The user approved this scoped guidance update.

FitPoints now offers the same recovery when Enter is submitted after at least four collected points. A generation counter detects point additions, removals and selection imports without copying the complete point list into the recovery state. In a five-point fixture, one history-only mesh was removed while a 64 MiB managed reservation remained active, and the resulting sphere's volume, center and Undo were checked. A stale-input fixture replaces a later point while keeping the first three and total count; recovery refuses before removing Undo. The full UI group passed 28 tests. Actual RAM exhaustion, arbitrary point sets, desktop and the selection-import route remain unverified. The user approved this scoped guidance update.

Vertical and single-source AroundCurve/Tangent now participate when a center or tangent point has been established; the recovery key includes the source stamp, normal and solution choice. Six cases (three modes, numeric or point completion) checked center, OCCT volume and Undo after simulated pressure was released. An AroundCurve line fixture separately reclaimed a history-only mesh under held 64 MiB managed pressure and completed with correct center and volume. The full UI group passed 30 tests. Curved or multiple source cases, other tangent modes, physical mouse and real physical RAM pressure remain unverified. The user approved this scoped guidance update.

Radius and Diameter option values now also enter the recovery path when they produce a recognized memory refusal. Four fixtures cover Center `Radius=5`, Center `Diameter=10`, TwoPoint `Diameter=10`, and a single-line Tangent `Radius=5`; each retained the original input and produced the expected center, OCCT volume and Undo after simulated pressure was released. A separate test changes the command draft while the warning is open and verifies the old retry refuses before removing Undo and the new draft remains. The full UI group passed 32 tests. Actual physical memory pressure, all modes, and physical desktop interaction remain unverified. The user approved this scoped guidance update.

Final Sphere points entered as three finite comma-separated numbers now participate in recovery. Four fixtures using the default construction plane cover Center, TwoPoint, ThreePoint and FourPoint, checking OCCT volume, placement and Undo after simulated pressure was released. A separate FourPoint fixture kept a 64 MiB managed reservation active and succeeded after reclaiming a history-only mesh. The recovery key includes the command's captured construction plane. Other coordinate syntax, arbitrary planes, desktop and real physical RAM exhaustion remain unverified. The user approved this scoped guidance update.

Box with two base corners now offers recovery for final numeric height, `Height=`, or a viewport height point. Three fixture paths checked OCCT volume 6000 mm³ and Undo after simulated pressure was released. A 64 MiB fixture kept its reservation active, reclaimed one history-only mesh and completed with the same volume. A further exhaustion fixture kept the pending corners, document and Redo and succeeded after releasing pressure; a replaced Box session is rejected before history removal. The full UI group passed 40 tests. Other Box modes, arbitrary planes, actual mouse and physical RAM pressure remain unverified. The user approved this scoped guidance update.

Cube, Center, 3Point and Vertical Box completion now also retain their stage, points, dimensions and plane for recovery. Eight fixture cases (four modes, text or viewport point) checked OCCT volume and Undo after simulated pressure was released. A Center fixture reclaimed a history-only mesh under a held 64 MiB reservation; a changed-width fixture refused stale recovery before deleting Undo. The affected UI group passed 43 tests. These are limited fixtures, not proof for all geometry, planes, mouse interaction or physical RAM exhaustion. The user approved this scoped guidance update.

Direct commands `Box 10 20 30` and `Sphere 5` now retain their command text on recognized memory refusal and offer the same consent-based retry. Both simulated-pressure cases preserved document and Redo, refused a changed draft before history removal, and produced the expected OCCT volume and Undo after pressure was released. With a held 64 MiB managed reservation, each reclaimed one oldest history-only mesh and completed. The affected UI group passed 45 tests. Other command forms, physical RAM exhaustion and desktop interaction remain unverified. The user approved this scoped guidance update.

Actual low-memory desktop interaction, physical menu clicks, other commands, and errors converted to another type remain unverified. Each individual retry still runs synchronously and may be slow for expensive geometry. Managed/native/shared ownership can keep bytes alive, so deleting an entry cannot promise that retry succeeds. This change does not implement source-only restoration or display eviction.

The automatic loop is now split across UI frames: each frame performs at most one retry and removes at most one oldest Undo step. A Cancel press stops subsequent attempts and reports how many steps were already removed; those removed steps cannot be recovered. One egui test keeps simulated pressure active, observes one removal per frame, cancels, and verifies the document and Redo remain intact. A second test changes selection between retries and verifies stale-context refusal before another removal. The full affected UI test group passed 36 tests. Each individual kernel call is still synchronous; real desktop responsiveness and physical RAM pressure remain unverified. The user approved this scoped guidance update.

## DupEdge selected-edge memory recovery

For a pending DupEdge edge selection, or a preselected edge invoked by menu or
CMD, a recognized managed-memory refusal now opens the existing consent-based
retry flow. The captured authority, document epoch, selection revision,
DupEdge session, scene publication and draft are checked before each retry or
history removal. A selected source-edge adapter callback now preserves the
original typed admission error; allocator refusal is distinguished from
structural limits. The proof-backed edge adapter path preserves callback
errors as well.

In a simulated 64 MiB managed-pressure fixture, the dialog was left open for
three frames without removing history; after its confirm button was clicked,
the oldest history entry holding a mesh was removed and retry produced the
selected Box edge. One Undo restored the prior document. A second pending
fixture released pressure before consent, retried without deleting Undo, and
refused changed draft or selection before history removal while preserving
Redo. Separate preselected fixtures exercised menu and CMD. A focused adapter
callback test distinguished managed-memory failure from an injected overflow.
This is not a physical OOM, desktop click, every edge/alias, or an end-to-end
proof that every possible native allocation is classified.

Validation: 64 UI recovery tests, 87 wireframe tests (8 diagnostics/3DM tests
ignored), 24 DupEdge-filtered tests (1 desktop-result inspection ignored),
the focused adapter test, a no-default-feature check, and the existing debug
build passed. Some groups overlap; counts are tests, not distinct fixtures.

### DupEdge command ID and temporary-edge follow-up

The preselected retry offer now recognizes both `DupEdge` and the existing
`OM9-CURVE-040` command ID, as well as menu invocation. In the command-ID
fixture an egui click on the consent button retried and cleared the submitted
draft. A separate synthetic paired-display fixture gave the selected edge a
temporary topology identity, simulated memory pressure, retained that pick
and Redo on refusal, then retried and undid the result. This fixture did not
exercise a real save/reopen source-view worker or reclaim Undo while the
temporary view was active. The full recovery group passed 65 tests; a
no-default-feature check and the existing debug build passed. Physical memory
exhaustion and desktop clicks remain unverified.

### Reopened DupEdge source view with history reclamation

A Box was saved and reopened, then a synthetic large mesh was added to the
reopened document snapshot solely to construct a history-owned allocation.
After the mesh was deleted and a separate edit undone, a real asynchronous
DupEdge source-view worker was requested and published; the fixture selected
a temporary edge using the controller's viewport-hit path. Under sustained
64 MiB managed-pressure simulation, Enter offered recovery without changing
the document, Redo or selected edge. Three egui dialog frames made no removal;
a simulated confirm click led to removal of one oldest Undo entry and a
successful retry. The resulting curve contains the picked midpoint, and
Undo/Redo restored the pre-command document and result.

The original attempt put the large mesh into the archive itself and failed
before reopening at `archive preflight bytes`; the revised fixture reopened
only the Box and inserted the history mesh in its in-memory snapshot. The
full UI recovery group passed 66 tests after this revision. This covers one
Box edge and an explicitly requested worker, not automatic mismatch-triggered
view creation, actual viewport clicking, arbitrary CAD files, physical OOM or
native peak.

### Automatic old-view recovery and consent under pressure

An authored cylinder with a deliberately coarse old displayed chord was saved
and reopened. In a real offscreen GPU viewport with simulated egui input,
clicking the chord and completing DupEdge automatically requested and
published a source view; clicking a regenerated circular segment selected the
temporary source edge. A synthetic history-only mesh was added to the
reopened snapshot and then deleted before the command. Under sustained 64 MiB
managed pressure, the next Enter retained the document, Redo and temporary
pick and opened recovery. No Undo was removed before simulated consent;
following the consent click, one oldest Undo was reclaimed and retry
completed. Four samples on the output matched the radius-2 circle at Z=5, and
Undo/Redo restored one/two objects. This uses the real worker and offscreen
GPU with simulated clicks, not physical desktop input or real OOM. The GPU
adapter was available: the focused `--nocapture` run emitted no skip. The
DupEdge-filtered library group passed 27 tests with one desktop inspection
ignored. The mesh was not part of the saved archive.

### DupEdge source-view start and publication memory recovery

A recognized memory refusal while starting a source-view worker is now
returned as its original typed error, so the existing explicit-consent
recovery can retry. For preselection the original session identity is restored
if the worker could not start, preserving the retry context. Recognized memory
failures reported while polling or preparing/publishing a worker also offer
recovery when the original session, selection revision and scene publication
still match. A worker memory error with a changed context is converted to stale
and cannot offer retry against the newer selection. Fixed internal worker
limits remain distinct and do not prompt history removal.

On a reopened old-chord cylinder using an offscreen GPU and simulated input,
three fixtures covered worker-start refusal from an active DupEdge session,
worker-start refusal after preselection/menu, and a managed-memory refusal
during publication. In each, the existing drawing and Redo were retained;
simulated consent reclaimed one oldest Undo, restarted the source view, and
the reselected output matched four samples of a radius-2 circle at Z=5 with
Undo/Redo. A test-only probe confirmed the old chord reached source mismatch
and the active-session worker reservation itself failed before retry; a
separate injected worker-result fixture checked both unchanged and changed
selection contexts. Synthetic history mesh was added after reopening.

Successful restart now reports that the source view started, rather than
claiming the whole DupEdge finished; the acknowledged memory error is cleared
only when that exact recoverable status is current. Earlier focused testing
observed the old error incorrectly remained visible despite successful
restart. A later worker error may still stop recovery; fixed budget limits,
unknown native failures, physical OOM and actual desktop were not proven.

Validation: 31 DupEdge-filtered library tests passed (one desktop-result
inspection ignored), 66 recovery UI tests passed, 87 wireframe integration
tests passed (eight ignored), focused stale/error classification tests and a
no-default-feature check passed. These groups overlap; counts are not unique
fixtures. Existing debug executable built successfully without a new target.

### One consent across asynchronous DupEdge worker retries

The DupEdge recovery dialog now remains active while its source-view worker is
running. The original consent covers later recognized worker memory refusals:
after rechecking authority, document epoch, session, selection, scene and
draft, the UI removes at most one oldest Undo for that failure and retries on a
subsequent frame. It does not ask for consent again. Successful worker
publication closes recovery and asks for edge reselection. Cancel stops the
worker without further history removal; changed selection/draft cancels and
reports stale before publication or another removal. A fixed worker limit, an
unrecognized error or exhaustion stops rather than discarding extra history.
Previously the dialog closed as soon as the worker started, so a later
recognized failure could cause another consent prompt.

A reopened coarse-chord cylinder was exercised with offscreen GPU and egui:
simulated pressure first required one oldest Undo removal, and a second worker
memory error was injected after its actual worker completed. Without a second
consent click, one more old Undo was removed and the retry produced the
verified radius-2 circular edge with Undo/Redo. Separate fixtures canceled
while waiting, changed draft, changed selection, injected a fixed-limit error,
and exhausted history after repeated injected errors; they verified no extra
removal, drawing and Redo preservation where applicable. The injected errors
are not physical OOM. DupEdge-filtered library tests passed 37 with one
desktop inspection ignored; all 66 UI recovery tests, no-default check and
existing debug build passed. Physical desktop input, real OOM, every worker
error and arbitrary geometry remain unverified.

### No unnecessary Undo removal before source-view retry

A saved/reopened coarse-chord cylinder was preselected with simulated GPU
viewport input and invoked through the existing `OM9-CURVE-040` command ID.
Managed pressure made the initial source-view start refuse; after pressure was
released but before the simulated consent click, both pre-existing Undo steps
remained. Consent retried the worker, kept the recovery active until the view
was published, removed zero Undo steps and cleared the submitted command ID.
The fixture then reselected the source edge, verified four samples of the
radius-2 curve at Z=5 and passed Undo/Redo. The history meshes were inserted
into the reopened in-memory snapshot. The focused test used an available
offscreen GPU (no skip was emitted); an existing old-view selection regression
also passed. This is not a physical desktop or real OOM test.

### Retry after worker release before pruning more history

A failed asynchronous worker releases its reservations when its result is
consumed. The existing one-consent flow now retries once using that released
memory before removing another Undo; if that retry also fails, it discards at
most one oldest entry and repeats under the same consent. A synchronous
retry-start failure still leads directly to one removal, because it has
already retried after release. This supersedes the earlier description that a
recognized worker failure always removed an Undo immediately.

On a reopened old-chord cylinder with offscreen GPU/egui, managed pressure
required one initial removal. After releasing pressure and injecting a worker
memory failure, retry completed without discarding the second Undo. A separate
fixture injected repeated failures and verified a second entry was removed
only after the no-removal retry also failed, with one consent. An exhaustion
fixture verified safe stop, retained drawing/Redo and no unbounded retries.
These are injected worker failures and managed-pressure fixtures, not real
physical OOM or a general native-memory proof. DupEdge-filtered library tests
passed 39 with one desktop inspection ignored; 66 UI recovery tests,
no-default check and the existing debug build passed.
### Root selection change during source-view worker

An offscreen GPU fixture with a reopened coarse-chord cylinder reproduced a
stale source-view publication when root object selection changed while the
consented recovery worker was pending. The worker receipt and recovery key now
include the root selection revision. With the same simulated change, the view
is rejected, the drawing and Redo remain intact, and no further Undo is
discarded. A separate worker test verifies rejection outside memory recovery;
another checks stale worker errors. The selection change was applied through
the controller and an egui frame, not physical desktop input. Pressure and
worker errors were simulated; this does not establish behavior under physical
OOM or for every geometry.

DupEdge-filtered library tests: 41 passed, one desktop inspection ignored;
UI recovery: 66 passed. The no-default-features check and existing debug build
also passed.
