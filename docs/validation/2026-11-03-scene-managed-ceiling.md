# Scene planning and managed RAM ceiling

`Scene::capture_in_world` previously checked retained owners against the shared managed ceiling but planned the scene using the coordinator's unconstrained `Limits`. It now obtains the retained usage and effective limits from the same locked account snapshot and uses those limits for scene preflight and capture. A fixture with 500 authored points admits the document but rejects its scene under a tighter managed ceiling; capture with the independent default limits succeeds, and rejection preserves the coordinator snapshot.

On E: with `-j 2`, the instance admission, scene acceptance, navigation snapshot, object snap and subobject-picking suites passed, followed by 266 core unit, 23 atomic transaction and 3 memory-budget tests. Public-source audit passed.

The account lock is released before scene planning and allocation. This change does not reserve the planned scene bytes through the allocation or prevent a competing worker from consuming budget meanwhile. Actual allocation capacity, native/GPU owners, current free RAM, the 256 MiB cap and desktop verification remain open; this is not complete P0 admission or an RSS limit.

## Construction reservation follow-up

World scene capture now reserves the preflight estimate of its additional construction peak as CPU cache, with the caller's held usage checked alongside existing retained owners and outstanding reservations. The guard remains live through `capture_admitted` and releases on success or error. A constrained fixture verifies a caller-held allowance and an existing reservation jointly refuse a further request, with unchanged document and successful retry after release. Existing scene and picking suites pass. The fixture does not schedule simultaneous scene construction and export or measure a physical peak.

On E: with `-j 2`, 267 core unit, 23 atomic transaction, 3 memory-budget, 19 B-rep picking, 11 gesture and 3 startup tests passed, as did the scene suites above. Public-source audit passed and pruning removed no runs.

The preflight itself allocates its seen-set before this reservation, and the guard is released when scene construction returns: the resulting scene is not registered as a persistent shared-account owner. The estimate and temporary vector capacities are not a measured or allocator-enforced upper bound; there is no free-RAM sample on this path. Caller-held data is included in this admission but is not itself kept as a reservation. These gaps, unregistered native/GPU/cache owners and the 256 MiB cap remain open.

## Planning scratch reservation

World scene capture now reserves the estimated BTree seen-set before preflight traverses the document. That guard is dropped after planning and before the construction reservation is obtained; no constructed scene exists in the interval. The estimate uses the same conservative `Usage::btree` calculation already used in scene admission. Tests cover a refused scene and successful capture with no remaining temporary CPU-cache reservations; the existing constrained and scene interaction suites pass. No test directly pauses a real planner against a competing worker, and allocator overhead and other temporary allocations are not proven covered.

Persistent scene ownership, the free-RAM sample, the hard 256 MiB cap, native/GPU owners and a measured physical peak remain open.

On E: with `-j 2`, 3 scene admission, 19 B-rep picking, 11 gesture and 3 startup tests passed, in addition to the scene and core regression suites above. Public-source audit passed. No release or desktop verification was performed.

## Lazy picking cache inventory

Scene cache usage now includes actual retained capacities for the lazily populated occurrence-sphere vector and the picking BVH node and order vectors. A 200-point fixture verifies the increases, stable repeated inventory, a render snapshot without picking caches, and their removal when picking bounds are invalidated. Scene admission, acceptance, navigation, object-snap and subobject-picking tests pass.

This is inventory after allocation, not pre-admission: sphere and BVH construction still allocate lazily without a shared reservation, and their temporary centre/radius vectors, allocator overhead and concurrent construction are not accounted as a peak. Scene ownership remains unregistered after capture; a fresh scene usage must be passed as held by its callers. No physical-memory or desktop claim follows.

On E: `-j 2` app checks passed for 19 B-rep picking, 11 gesture and 3 startup tests. Public-source audit passed; no release or desktop check was performed.

## Direct BVH sphere input

The scene BVH builder now reads the already-retained occurrence spheres directly instead of constructing separate centre and radius vectors. This removes temporary payloads of 32 bytes per occurrence on this platform, excluding allocator capacity and overhead; no physical peak or speed measurement is implied. The original separate-array entrypoint remains available and uses the same builder. An independent 257-sphere comparison checks tree size and exact normal/fat-ray candidate sequences over 17 rays; the existing 27,000-object brute-force fixture, scene admission, navigation, snap and subobject picking suites also pass.

BVH nodes, ordering, sorting scratch and the spheres themselves still allocate. They are inventoried after construction but lack the required pre-allocation shared reservation and persistent owner registration. This does not remove the 256 MiB cap or establish desktop responsiveness.

On E: with `-j 2`, 19 app B-rep picking, 11 gesture and 3 startup tests passed; public-source audit passed. No release or desktop test was performed.

## Physical RAM samples for world scene capture

The application now supplies fresh physical-RAM samples before the shared reservations for scene planning and scene construction. The scene crate accepts a caller-supplied sampling function and retains the existing managed-only wrapper for callers without a platform provider; it does not depend on a Windows API. A controlled 200-point fixture independently injects low headroom at each of the two reservation stages, confirms refusal, unchanged coordinator snapshot and released reservation, then verifies successful capture when both samples have sufficient headroom.

This is a point-in-time check when sampling succeeds, not a guarantee of physical availability or complete scene ownership. A missing sample still permits managed-only admission, and validation scratch between the two reservations, lazy caches, allocator/native peaks, the existing 256 MiB limit and persistent scene accounting remain open. The test does not simulate real OS pressure or desktop presentation.

On E: with `-j 2`, six scene-admission, six replacement-budget, 20 file-lifecycle, 11 gesture, scene picking/snap regressions and 20 serial GPU/egui tests passed; no GPU adapter skip was reported.

## World scene document-validation scratch reservation

Between planner admission and scene-construction admission, world scene capture validates the document. That separate validation now holds a shared CPU-cache reservation for the document's existing conservative validation-scratch estimate, with a fresh physical-RAM sample when supplied. The document is already retained and is not added again to this temporary reservation. The existing estimate is now exposed as a read-only Rust API; it does not certify allocator or native overhead.

The controlled 200-point fixture now independently injects low sampled headroom at planner, validation and construction admission. Each case refuses and leaves the coordinator snapshot and temporary reservation state unchanged; an all-sufficient sequence captures the scene. On E: with `-j 2`, six scene-admission, six replacement-budget, 20 file-lifecycle and 11 gesture tests passed. This covers the explicit validation call in world scene capture: validation performed inside shared-account inventory/admission and `candidate_scene_peak` remains a separate accounting question. Missing physical samples, persistent scene ownership, lazy caches, native/allocator peaks and the 256 MiB cap remain open.

The same batch passed 26 atomic-transaction, eight Undo-memory and 20 serial offscreen GPU/egui tests; no GPU adapter skip was reported. Public-source audit passed and artifact pruning removed no runs.
