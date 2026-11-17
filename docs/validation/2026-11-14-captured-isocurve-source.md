# Captured-source Isocurve construction

## Root-only command start on a worker

MatrixApp now also resolves UV bounds on the source worker when a B-rep or
NativeBrep root is selected without a face or viewport ray. The receipt is
bound to its source stamp, scene publication, selection revisions and command
run. The UI rejects changed command input; managed admission and asynchronous
recovery use the existing face-worker path. This does not change the underlying
single-face requirement or the default headless Controller path.

Headless egui fixtures exercise one-face planar B-rep and NativeBrep through
start and build workers, sample the resulting middle line on the source plane,
and check Undo/Redo. Separate NativeBrep box fixtures reject a multi-face root,
changed CMD, simulated quota refusal, and root selection changed away and back,
while keeping the document and Redo. The 28 matching app library/UI tests and
57 integration tests passed, with four integration tests ignored. This does not
prove desktop/GPU, native peak, physical OOM, opaque payload independence or
C1–C5. In-flight native calls remain non-interruptible.

## Viewport-ray command start on a worker

For MatrixApp face picks with a stored viewport ray, command start captures the
editable B-rep source or NativeBrep payload and resolves the source hit and UV
bounds on the existing face-verification worker. NativeBrep conversion for this
step also runs there. Publication rechecks the selection, session, scene and CMD
draft; the local hit is checked against the original screen and world witness
before the pending input opens. Managed source conversion, scratch estimates
and the receipt are admitted; native internal peak is outside that accounting.
The initial display witness selection remains on the UI thread. A root-only
B-rep without a viewport ray and the default headless Controller retain their
existing synchronous paths.

Headless egui sphere fixtures cover editable B-rep and NativeBrep from ray pick
through both workers, sampled radius and one Undo/Redo. A NativeBrep box fixture
checks that edited CMD, changed selection or simulated Kernel refusal at start
keeps document and Redo; it does not establish actual desktop/OOM behavior.
The 28 matching app library/UI tests and 57 integration tests passed, with four
integration tests ignored. Opaque payload independence, arbitrary topology,
native peaks, and C1–C5 remain open. An in-flight native call cannot be stopped.

## NativeBrep construction on the worker

MatrixApp now captures the immutable NativeBrep payload for Isocurve construction.
The worker converts that payload to OCCT source and builds the curves, accounting
converted source and outputs through managed admission. Publication uses the
same session, scene, selection, draft and layer guards. Initial source conversion
and face/bounds resolution when starting the command are still synchronous; the
worker conversion at completion is a separate operation. The payload is opaque;
this check does not prove that every imported payload excludes cached display.

Headless UI fixtures exercise worker startup and Undo/Redo on a NativeBrep sphere.
A reopened, translated box exercises two picked faces, refusal under simulated
Kernel pressure followed by retry, retained pending input and Redo, and stale
selection refusal. A sheared/scaled cylinder is sampled under orthographic and
perspective picks with Undo/Redo. The 27 matching app library/UI tests and 57
integration tests passed, with four ignored. Native allocation peaks, physical
OOM, desktop/GPU, other imported payloads, and C1–C5 remain unproved; native
calls already in progress cannot be interrupted.

## Viewport-ray B-rep construction on the worker

MatrixApp now also starts the captured-source construction worker for editable
OCCT B-rep faces selected with a viewport ray. The worker re-resolves that ray
against the captured source; the controller checks session, scene, selection,
parameters and layer before committing. Face-hit preparation and initial bounds
resolution are still synchronous on this route. NativeBrep remains separate.

A headless egui sphere fixture confirms the product path starts the worker and
commits one Undo/Redo transaction. A reopened drilled-block fixture checks that
the resulting two segments end at the hole boundary and that a changed face
selection rejects a running result without losing Redo. The 27 matching app
library/UI tests and 57 integration tests passed; four integration tests were
ignored. These do not establish physical desktop/GPU behavior, real OOM, native
peak limits, arbitrary topology, or C1–C5 completion. The synchronous kernel
hole verifier remains a known failure.

## Product UI worker polling

This is the earlier non-ray checkpoint; see the ray checkpoint above.

MatrixApp now enables source-only construction on its non-ray OCCT B-rep
Isocurve route. CMD/menu share the command implementation; the UI polls the
worker and verifies the pending context before a single atomic transaction.
Draft edits or new keyboard input cancel the pending result; simulated memory
refusal keeps the pending session and Redo. The consented recovery loop waits
for completion, retries after release, and only then removes oldest Undo steps
as needed. The default headless Controller, NativeBrep, viewport-ray and other
surface routes still use their existing paths. Scene publication remains part
of the transaction.

The egui sphere fixture exercises successful background completion, Undo/Redo,
and preservation of an edited draft and pending session. A separate simulated
managed-pressure fixture exercises asynchronous recovery, retry and Redo;
26 matching app library/UI tests passed; the integration suite passed 57 tests
with four ignored, and a no-default-features library check succeeded with warnings.
This is not evidence of desktop/GPU
interaction, real out-of-memory handling, bounded native peaks, all source
geometry, or completion of C1–C5. The synchronous kernel hole verifier remains
a known failure.


The kernel can construct face isocurves from an admitted `CapturedBrepSource`
without retaining a document or display. It validates the canonical source,
counts source and output allocations through the caller's admission callback,
and checks cancellation at callback boundaries and after native work. The
caller remains responsible for retaining the output reservation and checking
the current document, selection and scene before publication. A native call
already in progress cannot be interrupted, and native peak is not bounded by
the Rust accounting.

The B-rep commit path without a viewport-ray pick uses this lease. MatrixApp
now runs the supported construction on a worker as described above; the
default headless Controller remains synchronous. The NativeBrep and viewport-ray
paths remain separate.

A gated worker fixture captures a trimmed planar source, drops the controller,
checks that the old Document and display are no longer alive, then creates two
curves whose sampled points avoid the hole. Its reservation remains live with
the output and is released afterward. Separate checks reject cancellation and
simulated admission failure during output construction. The Isocurve integration
suite passed 57 tests with four ignored; native allocation peaks, physical OOM,
desktop/GPU, all geometry and publication/stale checks for a future worker are
not proved by this fixture.

## Explicit controller worker and commit

This section records the earlier controller-only checkpoint; product wiring is
described above.

An explicit controller API can start and poll the source-only construction job.
It keeps the pending Isocurve session while the worker runs, rejects additional
command input, revalidates context before publishing, and holds the output quota
until the single transaction returns. Failure preserves the pending session;
Cancel and scene clearing drop the job. The normal CMD/menu/UI route does not
yet start or poll this API and remains synchronous at curve construction.

In a selected sphere fixture, simulated Kernel pressure rejects the worker
and retains the document, pending input and Redo. After pressure is released,
explicit retry creates an equatorial source curve, followed by one Undo/Redo.
The 24 matching app library/UI tests passed. The UI CMD-draft guard and
asynchronous recovery dialog are not covered, nor are physical OOM, desktop,
native peak, NativeBrep/ray paths or arbitrary geometry.

The next step is to wire a bounded, cancelable construction worker into the
actual command, preserve the pending parameter and memory-recovery lifecycle,
and validate source, selection, layer, document and scene before atomic commit.
Canonical Document/Undo still retain display and C1–C5 remain open.

## Source-only construction job

This section records the earlier job-only checkpoint; product wiring is
described above.

A controller-side construction job now captures a tracked B-rep source, runs
the admitted kernel operation on a worker, releases the source before sending
its accounted curve output, and checks authority, epoch, scene publication,
command run, selection revisions, source/parameter recovery key and current
layer when polled. A shared active permit prevents a replacement worker from
starting while a canceled native call is still running. The output reservation
outlives the curve vector. This job is not yet called by the product command;
in particular the UI must still guard CMD draft changes, poll and atomically
commit, and integrate asynchronous memory recovery before it can be enabled.

One controller fixture confirms the job runs off-thread and samples a radius-3
sphere equator, keeps output quota while curves live, rejects cancel, changed
parameters and changed selection, and preserves the transaction snapshot when
simulated Kernel pressure denies growth. The no-default-feature library check
passed with warnings. This does not prove desktop input, physical OOM, native
peak, worker publication, actual command execution or all geometry.
