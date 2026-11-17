# MatrixApp Boolean Difference and Intersection workers

MatrixApp routes the two-set Difference and Intersection commands from menu or
CMD through the admitted Boolean worker. It captures source leases and local
placements for both sets, prepares checked group unions and performs the checked
binary operation on the worker. The original common-affine placement rule and
DeleteInput semantics remain: Difference removes targets even when DeleteInput
is No, while Intersection preserves them in that case. Empty results do not
commit. The previous disjoint-tool native shortcut is not used by this worker;
two separated cutters were checked, but its performance on large groups has not
been measured.

The controller rechecks selection revision, source stamps, command state,
layer, scene publication, epoch and session authority before publishing a single
transaction. Inputs, intermediate results and output use managed admission.
Explicit cancellation and changes to CMD draft or selection invalidate pending
work; an OCCT call already running cannot be interrupted.

Headless tests used overlapping boxes and two separated cutters: the seven
`session::boolean_union_worker::tests::` fixtures passed for Union and binary
commands, covering group results, both DeleteInput values, empty intersection,
stale selection, cancellation, managed budget refusal and Undo/Redo. The two
`ui_worker_` fixtures passed for menu and CMD Union, Difference and Intersection,
including draft-change cancellation and Undo/Redo. A separate source-lease
fixture checked two targets/two cutters and empty intersection. The app library
check without default features passed with existing warnings.

This does not certify every topology, group size, placement, shortcut behavior,
desktop/GPU, native peak or physical OOM. At that checkpoint Boolean Split and
Boolean Two Objects remained synchronous; the [subsequent Split worker](2026-11-14-boolean-split-ui-worker.md)
covers the MatrixApp route. Boolean Two Objects and the headless compatibility
controller path are still synchronous.
Document and Undo retain display; cold Undo, v5, NativeBrep and C1–C5 remain open.
