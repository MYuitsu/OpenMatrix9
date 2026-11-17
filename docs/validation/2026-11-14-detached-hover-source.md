# Detached hover picking source

The MatrixApp background hover snapshot now retains independently admitted
copies of exact curve inputs, scene buffers and B-rep display pins instead of
a captured Document. Capture validates each scene occurrence against the live
document before dropping the document owner. The worker uses the same picking
algorithm as the live scene, including curve evaluation and B-rep face mapping.
Point-cloud classification is captured separately. Other source payloads are
not needed for this object-pick query. Scene publication is now part of the
hover receipt, so an old result is rejected after a scene is republished even
when the document, epoch, camera and cursor are unchanged.

A headless fixture compares live and detached picks at orthographic and
perspective views for a point, cloud, line, circle, NURBS, polycurve, B-rep and
NativeBrep, including a miss. After New the old document, canonical B-rep source
and native payload are no longer alive; detached curve picking still works.
A simulated source-copy admission failure preserves the document. A separate UI
fixture rejects a delayed receipt from a different scene publication with the
same document, epoch and preview. Existing hover tests cover retry, buffer pins,
managed accounting and stale preview/document. The app hover group passed 18
and the scene library passed 51; world picking 8, instance admission 9, managed
picking cache 15 and navigation snapshot 2 passed. The no-default-feature app
library check and public-source audit passed with existing warnings.

At this earlier snapshot-only checkpoint, source-copy preparation and validation
ran on the UI thread; the worker integration below supersedes that limitation.
Multiple instances may still duplicate curve data. Existing scratch estimates exclude some
evaluator allocations. This evidence does not prove real OOM, native peaks,
all geometry/instances, desktop responsiveness or GPU behavior. Document and
Undo still own B-rep display, and cold restore and C1–C5 remain open.

## Source-copy preparation on the worker

For large-scene background hover, the UI admits a private scene snapshot and
starts one guarded worker. That worker validates occurrences and copies exact
curve sources into the detached snapshot, then releases its temporary captured
document before picking. Growth requests return to the UI for admission against
the current scene/account; cancellation is checked between source-copy steps.
A canceled or stale result cannot become current, including when the pointer
returns to its previous position. The active permit prevents replacement workers
from overlapping while the old worker exits. Scene metadata capture still runs
on the UI, and a single in-progress copy or pick cannot be interrupted.

A gated worker fixture changes to New while preparation is paused: the captured
document remains alive until the worker completes, cancels, or encounters a
simulated admission refusal; after that it is released and the managed CPU
reservation returns to the current-scene baseline. The successful path runs
on a different thread and returns a detached snapshot. The headless UI retry
fixture checks that an active worker prevents a replacement and that retry
succeeds after release. The hover group passed 19 tests, and the expanded retry
fixture was rerun. Physical OOM, thread-creation failure injection, native/GPU
peaks, desktop responsiveness, evaluator scratch estimates, and all geometry
remain unproved. Canonical Document/Undo and C1–C5 are unchanged.
