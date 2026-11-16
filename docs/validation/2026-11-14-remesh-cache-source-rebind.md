# Reusing an attested representation after a placement edit

The existing optional remesh cache can now reuse its independently accounted
root representation when the current attested proof is rebound to the same
immutable source/topology after a document change. The fast rebind accepts only
identical canonical allocations. Before reusing a cached representation the
scheduler validates the current proof, requires the exact same proof owner,
profile and stable object/session identity, and retargets the old receipt to the
current stamp. Scene publication still checks the current document and source.
An independent source allocation follows the existing worker validation path;
a fingerprint or changed stamp alone is not proof.

One authored OCCT fixture creates two display profiles, moves the object, and
reuses the old buffer without a remesh using minimal kernel/display budgets. It
checks the new placement, stale old stamp, no additional Undo entry, then repeats
after Undo and Redo. An independently allocated source is refused a cache hit
and its worker refuses simulated insufficient budget without changing document,
scene or history. A mismatched session-generation receipt is rejected. The
application remesh test group passed 81 tests with one ignored, and the scene
library passed 51 tests. The expanded focused fixture was rerun after the group.

This is reuse of a still-live cached representation, not a cold-cache restore.
Document and Undo still retain B-rep display data; NativeBrep, block instances,
real OOM, physical desktop/GPU, arbitrary imports, and eviction after the last
reader are not proved by this fixture. C1–C5 remain open.
