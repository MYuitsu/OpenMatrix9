# Builder candidate source and display reservation split

At the earlier two-owner checkpoint, a transient ring-builder B-rep candidate bound separate managed reservation
owners for canonical source/topology and display allocation. The source owner
also accounts shared topology; a display pin retains both owners so the shared
topology remains charged. A source-only transfer can release the display owner
once the last display or pin has dropped. The binding validates that owners
belong to the same account and memory class and that both charges are already
allocated and large enough. Candidate staging remains guarded by the original
preview; Document and Undo still reject transient owner-bearing geometry.

An actual SignetRing Builder fixture publishes a preview, holds a geometry clone,
cancels the builder, then keeps only its source. The display allocation dies and
its exact managed reservation is released; dropping the source releases the
remaining reservation. Core fixtures verify a live display pin keeps both
charges, and reject foreign-account or pending display owners. The ring-preview
app group passed 17 tests including apply, Undo/Redo, stale and retry; the focused
candidate-release fixture was rerun after the lock-order change. The core library
passed 389 tests with one ignored, and the two split-owner tests were rerun after
the lock-order change.

At that checkpoint this applied to the transient Builder candidate, not canonical document or
history storage. The source-side owner still conservatively covers some Builder
metadata after the preview is gone. NativeBrep, physical OOM, native peak,
desktop/GPU, cold Undo and C1–C5 are not established by these checks.

## Independent shared topology owner

The Ring Builder candidate now partitions the existing admitted reservation into
three owners: source, shared topology, and display. A source-only transfer keeps
source plus topology; a display pin keeps display plus topology and no longer
keeps the source owner. Each split guard also retains its own Arc/Mutex metadata
charge. Owner binding checks account, memory class and allocated capacity, and
locks the three guards in a stable order. Legacy single-owner candidates retain
their earlier conservative behavior.

Product-level SignetRing fixtures check both directions: retaining only source
releases display data and its owner charge, and retaining only a display pin
releases the source allocation and source-side charge while topology and display
charges remain. The last owner releases the remaining reservation. Core fixtures
reject foreign-account and pending owners. The Ring Builder group passed 18
tests, including staging, apply, Undo/Redo, stale and failure paths; core passed
389 with one ignored. This remains a transient-candidate result: canonical
Document/Undo, NativeBrep, cold restore, physical OOM and native peak are not
proved, and C1–C5 remain open.
