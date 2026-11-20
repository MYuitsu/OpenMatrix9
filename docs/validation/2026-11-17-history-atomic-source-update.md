# Atomic source and History update admission

`HistoryUpdate` previously validated every clean recipe against the pre-edit
document. A transaction containing both a source geometry/placement edit and a
rebuilt child therefore rejected the recipe's new source stamp before staging.
Preflight now defers checks only for objects explicitly edited by `Geometry` or
`Placement` in the same document and request. The complete staged world still
validates the final stamps, input and output types before publication. The temporary
pending-ID set is included in planning admission only for HistoryUpdate requests.

On 2026-11-17 the new regression first failed with `Stale("source stamp")`, then
passed. It covers a source geometry edit with its child geometry, recipe and result
in one transaction; a placement edit; a forged generation and a wrong typed input;
one Undo/Redo; and preserved Redo after a refused request. The 4 `history_mutation`,
7 `history_replay`, 25 `associative_rebuild`, and 2 default-build host Join History
tests passed with `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`.

The broader `atomic_transactions` test target did not compile: five existing test
fixtures use old `Brep`/`NativeBrep` types where the current geometry enum expects
canonical source types. That target is not evidence of a pass. This change supplies
transaction admission only; the host does not yet compute and append dependent
updates automatically. Worker/cancel/budget handling, scene publication, user
routes and end-to-end one-Undo propagation remain open.
