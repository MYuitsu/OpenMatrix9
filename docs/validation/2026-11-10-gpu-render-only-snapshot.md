# GPU render-only snapshot

The production GPU preparation jobs now receive only render object identities,
placement, colour, visibility and shared render buffers. The presentation
snapshot no longer owns the document, picking state, display receipt, or a
remeshed pick display. The current document and scene are checked before the
snapshot is created; the existing document/epoch/publication key and GPU
publication fence continue to reject stale results. Hover picking still uses
the separate document-backed scene snapshot.

The object vector is admitted and charged for its actual capacity, while
shared buffers keep their existing leases. The presentation allocation stays
charged while a cached Weak exists. The last render snapshot releases its
vector charge independently. A pressure refusal test and shared-owner tests
exercise these lifetimes.

With the default OCCT feature, 15 presentation snapshot tests passed. In the
Sphere fixture, New was completed after the unsaved-changes decision: the old
document and its B-rep display were released while the old GPU snapshot still
held usable, charged render buffers. The complete offscreen GPU publication
suite passed 29 tests, including Sphere selection, preview visibility,
Undo/Redo, old-document rejection and resource lifetime. The no-default
presentation suite passed 14 tests and a no-default old-document offscreen
test passed. Builds used `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`.
Public-source audit and `git diff --check` passed.

This change does not remove display ownership from a live document or Undo,
implement eviction/remesh for document history, establish native or process
peak memory, or validate the running desktop. The M0/M1 gates remain open.
