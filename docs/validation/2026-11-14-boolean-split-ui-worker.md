# MatrixApp Boolean Split worker

MatrixApp runs Boolean Split from menu and CMD on the admitted Boolean worker.
It captures source leases and placements for both sets, materializes working
B-reps under admission, and partitions every target against the target and cutter
sets. Duplicate canonical allocations with the same placement share the working
body; retained occurrences are conservatively counted individually. The worker
checks candidate pieces and uses two exact differences to identify duplicate
cells. Intermediate representations, vector growth, compounds and output pieces
are admitted; a native OCCT call already in progress cannot be interrupted.

The controller checks the original source stamps, selection revision, command,
scene publication, epoch, authority, layer and DeleteInput before publishing all
pieces in one transaction. Overlap between the two input sets is allowed, and
DeleteInput deletes each source at most once. Cancel, stale input or a managed
budget refusal leave the document and history unchanged.

Headless controller fixtures checked overlapping sets, both DeleteInput values,
three pieces and preserved total volume, common affine placement, Undo/Redo,
explicit cancellation, changed selection and managed memory refusal. A headless
MatrixApp fixture checked menu and CMD Split, piece volumes and Undo/Redo. The
kernel admitted-set fixture checked overlapping and repeated targets, duplicate
cell detection, two sequential cutters producing five pieces, output limits,
invalid surface input, cancellation and admission refusal. The worker test group
also reruns Union, Difference and Intersection regressions.

Validation after the final accounting change: 11 controller worker tests, three
headless UI worker tests, and nine kernel split-filter tests passed. The app
library also checked successfully without default features, with existing warnings.

These fixtures do not certify every topology, placement or large-group
performance, native peak, physical OOM, desktop or GPU behavior. At that
checkpoint Boolean Two Objects remained synchronous; the subsequent [preview
worker](2026-11-14-boolean-two-ui-worker.md) covers its MatrixApp route. The
headless compatibility Controller route remains synchronous.
Document and Undo retain display; cold Undo, v5, NativeBrep and C1–C5 remain open.
