# Detached occlusion snapshot

`Scene::into_occlusion_snapshot` validates each nonempty triangle buffer's
entity binding while the document is present, then consumes the scene into a
`PickSnapshot` with only a weak document identity. The occlusion query uses
the same intersection and fallback callback implementation as the live scene.
The caller remains responsible for admission and shared-buffer leases, capturing
any source needed by a fallback proof, and revalidating the scene publication
before using an answer. This is an internal building block, not a worker or UI
integration.

The targeted Rust fixture compares live-scene and detached results for an
authored NativeBrep triangle: front/behind targets, miss, excluded object and a
simulated fallback callback. It drops the scene and document, checks that the
document and native payload are no longer live, and runs the detached query on
a worker thread. A second fixture corrupts the geometry revision and verifies
that capture refuses the stale binding. These tests do not establish general
OCCT topology attestation, actual fallback-source proof, GPU/desktop behavior,
worker admission or stale-result rejection at a product publication boundary.
With the configured OCCT runtime, `cargo test -p om9-scene --lib -j 2 occlusion`
passed both snapshot fixtures, and `cargo test -p om9-app --lib -j 2 occlusion`
passed three existing fallback/Gumball tests. The public-source audit passed.

At the snapshot-only stage Line still computed shaded occlusion on the UI
thread; the later integration below changes that for ordinary scenes.
Canonical BRep/NativeBrep and Undo still retain display; C1–C5 remain
open. The existing curved-trim full-face integration failure remains open.

## Line worker integration

For a Line Surface Snap request, the scheduler now attempts to capture an
admitted detached occlusion snapshot and sends it to the source refinement
worker. After refining the candidates, that worker checks the chosen exact
point against the captured display buffers. The receipt contains only the
boolean result, and the snapshot and its shared-buffer reservation are dropped
before receipt publication. The existing receipt checks current source, scene
publication, selection, preview, camera, viewport and cursor before the UI
uses that result. Scene snapshot construction and binding validation still
occur on the UI thread.

The worker snapshot is used only if there are no visible trim fallback
buffers. If such a buffer exists, physical-memory sampling is unavailable, or
snapshot admission/capture fails, the existing UI-thread occlusion path remains
in use; it retains its conservative source proof. This preserves behavior but
does not complete the background requirement. Native peak, physical OOM,
desktop/GPU and general topology remain unverified.

The later fallback-source integration below supersedes the restriction on
visible trim fallback buffers; the admission-failure fallback remains.

An OCCT sphere plus foreground mesh fixture checks that the scheduler's worker
receipt reports occlusion and rejects a changed camera. A separate admitted
snapshot fixture verifies New releases the old Document, the snapshot remains
queryable, and dropping it reduces the managed CPU-cache reservation. The
headless Line click fixture verifies the product receipt includes the worker
occlusion result and its existing stale-input cases. `cargo test -p om9-app
--lib -j 2 occlusion` passed five tests; `surface_snap` passed 23 and ignored
one diagnostic; the Line UI fixture passed separately. The no-default-feature
library check passed with warnings. These checks do not establish bounded
native allocation peaks, all memory-pressure paths, or full desktop behavior.

## Source-only trim fallback on the worker

The occlusion capture now copies eligible visible fallback NURBS sources and
their trim extensions into independently admitted Rust storage. A separate
Kernel reservation covers the source copies and the maximum serial proof
scratch. The worker rechecks pending admission and uses the same conservative
proof as the UI, keyed by captured occurrence path; failed or uncertain proof
still treats the fallback as occluding. Missing/unsupported sources cannot
prove clear. If capture or admission fails, Line retains the previous UI path.
Snapshot preparation and copying still occur on the UI thread. The later
admission-refusal change below replaces the UI fallback on capture failure.

For one affine cylindrical fallback with 1,025 trim vertices, a worker snapshot
matches the live-scene proof; an OCCT sphere behind it is also refined and
cleared through the actual scheduler. The copied source gives both clear and
not-clear answers after New releases the old Document, and its managed Kernel
reservation drops with the snapshot. Simulated Kernel exhaustion rejects capture
and a retry succeeds after release. The five app occlusion tests, the 23-test
Surface Snap group (one diagnostic ignored), and the no-default-feature library
check passed. These fixtures do not prove all trim topology, native peak,
physical OOM, UI latency, desktop/GPU, or other commands. C1–C5 and the known
full-face curved-hole verification failure remain open.

## Refuse Line occlusion when admission fails

Line no longer runs synchronous UI-thread occlusion when the worker snapshot
cannot be admitted or captured. The scheduler returns the error before queuing
a request; the viewport clears any old Line job, rejects the point and keeps
the command active. A successful shaded result must have the worker's
occlusion receipt. This intentionally changes the earlier admission-failure
fallback: the user can retry after resources become available.

In the sphere-and-foreground-mesh scheduler fixture, simulated CPU-cache
exhaustion after reserving the source worker budget refuses snapshot capture,
leaves the transaction snapshot unchanged and leaves no job queued; after
releasing pressure, retry reports the expected occlusion. A separate egui
fixture exhausts managed quota before a Surface Snap click and verifies the
command phase, document/transaction snapshot and pending-click state remain
unchanged; after pressure is released a worker receipt is obtained. These are
managed-pressure fixtures, not physical OOM, desktop input or a proof for all
geometry. Source/display ownership, C1–C5 and the curved-hole failure remain
open; other commands and Gumball retain their own synchronous paths.
