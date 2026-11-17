# Remesh scene display owner

Before this change, a published remesh scene retained its complete worker
receipt, including a duplicate B-rep source and native topology descriptor
arrays, simply to keep the display and its memory charge alive. The private
scene now receives the kernel reservation and proof through its display owner.
After final validation and before publication, it drops the candidate receipt,
then reduces the reservation to the display and shared topology allocations.
Scene buffers and the in-process proof keep their own admitted owners.

The private receipt also keeps the owner alive during scene construction, so
the candidate is dropped before the transferred reservation on error or
cancellation. A core check refuses the display-only accounting operation when
the display topology and canonical topology are not the same owner. The
receipt-envelope allowance can overlap other metadata until publication;
allocator, native OCCT and channel peaks are not proved by these checks.

A fixture held weak references to the remesh candidate source and display.
After publication the duplicate source's strong owner was gone, the scene's
display remained alive, and the kernel reservation matched display plus
topology capacity. Undo released that display and reservation. A separate
fixture dropped a prepared private scene before publication and confirmed that
the candidate, reservation, current scene and coordinator state were preserved
or released as appropriate. Existing scene snapshot tests checked old snapshots
and picking across repeated remesh. The app topology group passed 40 with 2
ignored before adding the separate private-scene cancellation fixture, which
also passed. Core B-rep tests passed 5; one GPU offscreen remesh/picking test
passed. Builds used `CARGO_TARGET_DIR=L:/om9-target`, `-j 2`.

This separates the published remesh display from its temporary duplicate source;
it does not remove the original display from the document or Undo, provide a
reclaimable document display cache, establish legacy v4 topology provenance,
or prove native/process peak or desktop behavior. The main M0/M1 gates remain
open.

Follow-up: display and render-buffer leases were split, and the display
reservation now also covers its owner overhead. A buffer-only clone does not
retain the display charge; see `2026-11-06-remesh-buffer-display-leases.md`.
