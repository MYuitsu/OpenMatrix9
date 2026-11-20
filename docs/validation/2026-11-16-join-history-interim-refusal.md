# Join History: interim add-on refusal

The Tools add-on previously converted display samples to polylines and deleted the
selected parents. Its reply protocol does not create an associative recipe. Until the
host can create and replay that recipe from canonical curves, the add-on now refuses
the operation rather than publishing a non-associative result or deleting the inputs.

On 2026-11-16, `CARGO_TARGET_DIR=L:/om9-target cargo test -p om9-addon-tools --lib
-j 2` passed 11 tests, including a new add-on event regression asserting refusal and
empty create/delete lists for two touching lines. `python tools/public_source_audit.py`
passed (16122 source files, 594 icon bindings, 593 SVGs). This is add-on unit evidence
only: host routing, document transactions, Undo, UI, canonical geometry, dependency
recompute and save/reopen were not tested. No History implementation is claimed.

The initial test invocation failed to set the target environment variable because of
PowerShell quoting and began compiling in the pre-existing checkout target; it was
interrupted. The successful run above used the prescribed external target. Existing
target contents were not removed.

Next: implement a host-owned recipe and producer using canonical placed curves and
document tolerance, route icon and command aliases consistently, then verify parent
retention, recompute, failure atomicity, Undo/Redo and persistence through the host.

## Canonical producer foundation

The core registry now has a `JoinHistory` producer. It accepts ordered whole-curve
inputs and a recorded `tolerance_mm`, reuses the existing canonical join (including
entity placement), and rejects duplicate, mismatched or output-as-input identities.
The portable builder and OCCT registries both register it. The Tools add-on still
refuses the command: no host route yet creates the required recipe, and this producer
alone does not implement automatic propagation or a user-facing command.

The new core test first failed with `Unsupported("history producer")`, then passed
after registration. It checks two line parents, one translated, the child's endpoints
after a source edit and explicit recompute, and retention of the last good child on a
disconnected input. All 9 `om9-core --test producers` tests passed; the 7 matching
`om9-core --test top11 join` tests passed, and `cargo check -p om9-kernel -j 2`
passed. All used `L:/om9-target` and `-j 2`. These are core/compile checks, not host
transaction, display, persistence, automatic propagation, or desktop evidence. The
producer currently follows the existing registry's default-limits convention; host
admission and cancellation must be checked when the command is wired.

## Host transaction and explicit rebuild

The host now has an internal integration entry that builds from selected canonical
curves and inserts the child and its `JoinHistory` recipe in one transaction. It
records the document's absolute tolerance and source order, retains both parents,
and refuses disconnected curves before changing the document. The add-on route still
refuses the command, and the built-in menu/CMD route is not wired to this entry: it
does not yet arrange automatic updates or interactive preview.

One host fixture checks world placement, recipe contents, parent identity,
save/open, one Undo/Redo for creation, and explicit recompute after an endpoint grip
edit on the reopened document. A second fixture checks disconnected-curve refusal
and unchanged Undo. Both pass in the default OCCT-enabled build and in the portable
`--no-default-features` build. Initially the host recompute failed with a stale
source stamp: rebuilt results carried old recipe stamps even though the producer used
the current source. The shared recompute path now publishes the current stamps and
records the same stamps on the recipe. The 25 `associative_rebuild`, 9 `producers`,
and 2 `crv2view` tests pass. All runs used `L:/om9-target` and `-j 2`.

The test calls explicit recompute; it does **not** show that editing a parent
automatically updates its child, or that the parent edit and child update share one
Undo. Downstream propagation, error/cancel and budget behavior in that path, menu/
CMD/F6, GPU presentation and early preview remain open. No manual gate is signed.
