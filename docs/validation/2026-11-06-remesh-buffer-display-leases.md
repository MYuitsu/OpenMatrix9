# Independent remesh display and render-buffer leases

Remeshed scene objects now carry a display lease separately from the lease on
their render buffers. The display lease follows the B-rep pick display and
proof; a detached `SceneBuffers` clone retains only its buffer owner. Scene
snapshots and metadata updates clone both leases as needed. The two owner
allocations are admitted separately, including the overlap while the worker
receipt and new owners coexist. Candidate/display and buffers are dropped
before their respective charges on the checked error paths.

A fixture published a remesh, kept only a render-buffer clone, then undid the
object. Before the change, the display was gone but the buffer clone still
held 2,592 bytes of kernel reservation. After the change, the display charge
was released while the buffer remained usable; its CPU-cache charge decreased
only after the last buffer clone dropped. Existing snapshot/Undo, pressure and
stale-publication cases remained green. Scene unit tests passed 34; app
topology-owner tests passed 42 with 2 ignored; scene metadata and shared
instance tests passed 4 and 6; the offscreen GPU/picking fixture passed 1.
Builds used `CARGO_TARGET_DIR=L:/om9-target` and `-j 2`.

These are allocation-accounting and headless/offscreen observations, not a
measured process or native OCCT peak. The added owner allocations have separate
managed allowances; allocator over-allocation and all concurrent failure
interleavings are not proved. The original display is still retained by the
document and Undo; eviction and desktop acceptance remain open.
