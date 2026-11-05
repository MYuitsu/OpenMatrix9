# Background hover snapshot admission

For scenes above 128 objects the optional hover worker previously cloned scene metadata and retained shared buffers and a document without registering that lifetime in the shared account. The UI now acquires a tracked document capture, checks it is the scene's exact document allocation, and reserves a conservative copy of scene cache usage alongside the caller's current held scene before cloning. The worker retains both owners until its snapshot is dropped. On refusal it skips this optional hover and marks the pointer position handled until input/view changes, avoiding repeated admission attempts each frame.

The reservation deliberately counts the snapshot's shared buffers in addition to the current scene while both live. This is conservative double counting, not the plan's eventual single-owner inventory. Its estimate also does not include all BVH construction/query scratch or allocator overhead, does not sample free RAM, and is not a physical peak bound. Registration and full scene inventory happen on the UI path; their cost and its effect on pointer latency have not been measured. Failure while the pointer remains still does not retry until the position/view changes. Ordinary picking and document operations remain separate; no desktop acceptance is claimed.

On E: with `-j 2`, 10 selection overlay tests passed, including a 130-object background hover that finishes and releases its reservation and a pressure fixture that declines hover without changing the document. Before the final retry-throttling branch, 19 B-rep picking, 11 gesture, navigation input and 3 startup tests passed. The fixture does not exercise a document replacement while the worker is paused or a real OS memory-pressure event.

## Nearest-only hover

Both foreground and background hover now call the existing nearest-object picker instead of requesting the complete list used by the ambiguity menu. The overlapping curve/mesh fixture checks that nearest picking returns the same first object and path as the full list; existing hover and picking regressions exercise the UI. This permits the picker's existing depth early-out but does not eliminate all candidate or hit allocations, prove equivalence for every geometry, or establish a measured latency gain. Ambiguity selection still requests all hits.

On E: with `-j 2`, 10 scene acceptance, 2 spatial precision, 10 selection overlay, 19 B-rep picking and 11 gesture tests passed. Public-source audit passed. No release or desktop check was performed.

## Stream per-object hit selection

Picking now maps B-rep topology and retains the best hit as each candidate is found, preserving the existing screen-distance then depth comparison and its first-hit tie behavior. It no longer stores a vector of every hit within one object before choosing its best. A 512-point cloud fixture with identical projected points checks that the closest depth and native point index are retained; existing mesh, B-rep, nested occurrence, snap, hover and gesture suites pass. Mapping still rejects candidates without a valid B-rep topology ID.

This removes the per-object hit vector, not all scratch: projected mesh vertices, ray candidates, B-rep mapping and the list of distinct object hits remain. The result has not been measured for peak memory or latency, nor verified on the desktop.

## One result for nearest-only picking

The nearest-only path now retains at most one object hit while traversing candidates and uses the same depth, root ID and occurrence-path ordering as the full disambiguation list. Equal keys retain the first encountered hit. The full-list path still sorts and deduplicates every matching root. A 150-object overlapping fixture with equal-depth ties checks that nearest and full-list first result agree; the scene, snap and subobject tests pass. BVH ray candidates and per-mesh transformation scratch still allocate. This is not measured latency, a complete geometry-wide equivalence proof or desktop acceptance.

On E: with `-j 2`, scene unit, admission, acceptance, snap, spatial precision and subobject tests passed; app B-rep selection and precision, nested occurrence, overlay and gesture suites also passed. Public-source audit passed. No release or desktop check was performed.

## Hover picking scratch estimate

The background hover reservation now adds a checked estimate for the fresh snapshot's occurrence spheres, BVH nodes/order, ray-query workspace and the largest per-object transformed mesh-vertex array. A scene fixture checks the estimate exceeds the actual retained sphere and index capacities for 200 points, and an overflow fixture rejects an unrepresentable object count. The estimate is applied before spawning the worker; it is not a guarantee of actual peak memory. Allocator overhead, sort implementation scratch, evaluator and other query allocations are not proved covered, and shared snapshot buffers remain conservatively double-counted. No physical RAM sample, release benchmark or desktop check was performed.

On E: with `-j 2`, scene unit, admission, acceptance, snap, spatial and subobject suites passed, as did 10 selection overlay, 19 B-rep picking and 11 gesture tests. Public-source audit passed.

## Physical RAM sample for hover

Before reserving a background hover snapshot the UI now samples total and currently available physical RAM where the native owner can supply it, and passes that sample with the caller-held usage to shared admission. A deterministic core fixture covers insufficient sampled headroom, unchanged document and reservation state, a successful sampled retry and release. This is a point-in-time observation, not an OS reservation: available RAM can change immediately afterward and the current accounting is conservative. If the platform sample is unavailable the existing unsampled fallback remains and RAM headroom is not verified; no user-facing missing-sample indicator is added here. Shared-buffer double counting, incomplete worker scratch, UI sampling cost and desktop behavior remain open.

On E: with `-j 2`, 268 core unit, 23 atomic transaction, 3 memory-budget, 10 selection overlay, 19 B-rep picking and 11 gesture tests passed. Public-source audit passed; no release or desktop test was performed.

## Required RAM sample before optional hover worker

The UI now refuses the optional background hover reservation when the physical-memory sample is unavailable, before capturing the source document or spawning a worker. A deterministic unit fixture passes an absent sample and an insufficient sample, checks that no CPU-cache reservation is retained and the document is unchanged, then verifies a sufficiently sampled reservation is acquired and released. Existing pressure and successful-background-hover fixtures exercise the UI path on Windows. This closes the earlier unsampled admission fallback for this worker only; the sample remains point-in-time, and native/allocator/query peaks and shared-owner accounting are not fully bounded.

The current platform provider returns no sample on non-Windows platforms, so background hover for scenes over 128 objects is currently unavailable there; ordinary foreground hover for smaller scenes is unchanged. On Windows, a failed sample likewise skips optional hover and the current retry policy waits for a changed input basis; no dedicated user-facing reason is displayed. Implementing a portable sample and clearer retry/feedback remains open. This behavior is not a complete P0 memory ceiling or desktop acceptance.

On E: with `-j 2`, nine hover-filtered unit, 14 selection-overlay, 11 gesture, seven Gumball viewport and 20 serial offscreen GPU/egui tests passed; no GPU adapter skip was reported. Public-source audit passed and artifact pruning removed no runs.

## Stationary hover retry after temporary refusal

Previously, a refused optional background hover was marked handled and could remain unavailable indefinitely while the cursor stayed still. On Windows, when admission fails or a RAM sample is temporarily unavailable, the active hover now schedules another repaint and retry after one second; a changed input basis can trigger a new attempt sooner. Platforms that cannot provide a sample do not poll indefinitely. The retry remains subject to the same current sample and reservation checks, and no geometry is modified.

A Windows egui fixture holds a competing reservation to force refusal, verifies that the retry is scheduled and no worker starts before its deadline, releases the reservation, then advances the stored deadline to now without moving the pointer. It confirms hover eventually returns and the worker's reservation is released. This controls the deadline directly rather than measuring actual wall-clock repaint scheduling or a real OS memory-pressure event. If memory stays unavailable, Windows continues retrying once per second while the same active pointer is present; no dedicated user-facing reason is shown. Complete process memory accounting, non-Windows sampling and desktop behavior remain open.

On E: with `-j 2`, 10 hover-filtered unit, 14 selection-overlay, 11 gesture, seven Gumball viewport and 20 serial GPU/egui tests passed; no GPU adapter skip was reported. Public-source audit passed and artifact pruning removed no runs.

### Retry repaint after intervening frames

The delayed hover retry initially requested a repaint only at refusal. Eframe discards repaint requests whose associated pass has become outdated, so intervening frames can consume that request before the retry deadline. While a retry is pending, each active hover pass now requests a repaint for the remaining duration; an earlier repaint remains allowed and the actual admission still waits for the deadline.

The controlled Windows egui fixture now attaches a repaint callback after the refusal, sets a short 100 ms deadline, advances three frames and checks a delayed request is registered, then verifies recovery after releasing the competing reservation. With this fixture the added assertion failed without the renewal and passed with it. An earlier assertion using a longer delay was invalid because an unrelated shorter egui repaint suppressed the callback; the short-deadline fixture isolates the request. This checks egui scheduling signals, not actual winit delivery, wall-clock latency, physical memory pressure or desktop presentation.

On E: with `-j 2`, 10 hover-filtered unit, 14 selection-overlay, 11 gesture, seven Gumball viewport and 20 serial GPU/egui tests passed; no GPU adapter skip was reported. Public-source audit passed and artifact pruning removed no runs.
