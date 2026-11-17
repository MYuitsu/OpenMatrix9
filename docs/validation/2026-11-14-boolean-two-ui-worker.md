# MatrixApp Boolean Two Objects preview worker

MatrixApp now prepares Boolean Two Objects previews from captured B-rep sources
and placements on the admitted Boolean worker. The five modes are Union,
Difference, ReverseDifference, Intersection and InversionIntersection (XOR).
XOR admits both intermediate differences and their final union; empty outcomes
remain valid previews. The previous preview and its own reservation remain
alive until a replacement is ready, and the reservation is released when the
preview is discarded. Publication into the document still requires the normal
explicit commit and one transaction.

Before replacing a preview, the worker checks session, epoch, scene publication,
source stamps, selection revision, command, previous preview mode, layer and
DeleteInput. Cancel, changed input or selection, and managed budget refusal
leave the existing preview and document unchanged. Cancellation is observed
between native calls and during admitted copying, not inside an active OCCT call.

Headless controller fixtures checked all five modes on overlapping boxes,
including the bounds of the two difference directions, volume, old-preview
retention, preview replacement, commit and Undo/Redo. Equal boxes checked an
empty XOR, disabled commit and reservation release. Other fixtures checked
stale selection and managed pressure while preserving the old preview. A
headless MatrixApp fixture checked menu and CMD, XOR replacement, changed draft,
commit and Undo/Redo. After the change, 14 Boolean worker controller tests and
four UI worker tests passed. The kernel Boolean filter passed 18 tests with one
pre-existing ignored affine case; the app library check without default features
passed with existing warnings.

This does not prove all topology, placements, real OOM, native peak, desktop or
GPU behavior. The headless compatibility Controller path remains synchronous.
Document and Undo still retain display, and cold Undo, v5, NativeBrep and gates
C1–C5 remain open.
