# Viewport selection and published scene receipt

## Gumball context changes during Perspective drag

Two egui regression fixtures exercise a Perspective Gumball drag after its preview has moved: one switches the active viewport and the other changes the Perspective camera while the button remains down. Both verify that the next frame clears the preview, the subsequent release leaves canonical placement and Undo untouched, and a fresh drag can commit a 2 mm translation with one Undo entry. Both passed without a runtime change. The full `gesture_regressions` suite passed 11 tests on the default-feature E: build with `-j 2`. These tests programmatically change workspace state and run headlessly; they do not prove the corresponding desktop event sequence or physical pointer behavior.

The viewport previously picked from the current scene even while the last published image could represent an earlier document/scene revision. A combined MatrixApp/real-offscreen-wgpu regression reproduced selection against the newer geometry while the old scene receipt was displayed.

`GpuViewport` now records the document ID and scene epoch associated with its published image. At primary press, the UI captures that receipt. A new object click, selection band, grip or Gumball gesture is admitted only if the published receipt and the press receipt match the current scene; an already captured Gumball/grip gesture retains its existing cancellation guards. A rejected click gives a status message. Once a matching image is available, a fresh click can select normally.

## Validation

- `cargo test -p om9-app -j 2 --lib ui::gumball_gpu_tests -- --nocapture --test-threads=1`: 4 passed on a real offscreen GPU adapter. The new test covers an edited scene while the old receipt remains visible, then selection after a current receipt; it also presses while the receipt is stale, publishes the new scene while the button remains held, and checks release does not select.
- Red/green: bypassing the current-scene guard made the first stale-click assertion fail; restoring it passed. Separately bypassing the press-receipt check made the post-publication release assertion fail; restoring it passed. Both temporary changes were reverted.
- `cargo test -p om9-app -j 2 --test gpu_navigation_publication --test navigation_lifecycle --test gesture_regressions --test preview_drag --test gumball_viewport`: 40 passed. Added receipt assertions cover edit, Undo, Redo and opening another document with the same epoch. These suites ran after the scene receipt was added; the final press-receipt guard was subsequently checked by the four combined GPU tests above.

## Limits

The previously documented release desktop trial predates this scene-receipt change; a new release/desktop run is still needed. Receipt identity does not by itself prove GPU completion, actual monitor presentation or that a shared target texture retained old pixels while being overwritten. Selection/preview/visibility and framebuffer identity, builder handles, OSnap, exact kernel refinement, small features, and all command-specific click paths require further audit. This change does not certify general B-rep picking accuracy or close P2. UI tests use synthetic egui input; no physical mouse test was performed.

## Preview and input-basis follow-up

An offscreen GPU/egui Line test demonstrated that scene epoch alone was insufficient: a transient Gumball preview moved the live scene while the displayed image still showed the earlier preview, and the command accepted a point from the not-yet-published position. The initial attempt to reproduce with `clear_selection` was invalid because that call itself clears the preview; the corrected Line fixture failed before the fix. The viewport now retains the actual published render key and compares its document, scene, selection, preview, CPlane, hidden-source state and target size with the current content. The primary press also captures its render key and full camera, so a later publication does not make an old press valid. Once a Gumball/grip gesture has started, its own capture path remains active.

An initial full-key comparison incorrectly treated a generation change with identical input content and camera as a different image, breaking a Gumball-after-orbit regression. The input-basis comparison now excludes generation but retains the full camera equality check. Five combined GPU/egui tests pass, including the new stale-preview Line test, press-before/publication/release rejection, and immediate Gumball drag after orbit. Eleven GPU publication, fourteen snap UI and nine gesture regressions also pass. This is offscreen synthetic input; the test does not prove physical desktop presentation, complete framebuffer isolation, other command overlays, or safe asynchronous worker publication. It rejects a stale click rather than queuing it. The desktop executable predates this change.

## Display mode and published input basis

The display-only full-sphere guide heuristic previously allowed a fixed `1e-4` steradian difference from a full sphere. A fixture refines one OCCT sphere triangle seven times on the sphere and removes a nondegenerate child: the old threshold incorrectly accepted the incomplete mesh for full-sphere guides. The threshold now scales with triangle count and floating-point epsilon; that fixture rejects the hole while its complete refined sphere and the existing full sphere, translated sphere, ellipsoid and box cases pass. Three guide tests and ten GPU/egui Gumball tests passed on the E: default-feature debug build with `-j 2`. An initial test removed a degenerate triangle and was corrected; one run also failed to start because of a mistyped runtime DLL path, then was rerun with the correct path. The numeric threshold is a heuristic, not a proven floating-point error bound or a topology test: still smaller holes, overlapping triangles and complex trimmed spheres remain unverified. No actual desktop was tested.

An additional offscreen GPU assertion compares a narrow strip across an authored quad edge with an otherwise identical empty Perspective viewport. At least one edge pixel differs, so a blank Wireframe target cannot satisfy this fixture merely because its interior differs from Shaded. The targeted test and all 10 `gumball_gpu_tests` passed on the default-feature E: debug build with `-j 2`, no adapter skip. This covers one mesh edge, not B-rep isocurves, all wireframe geometry, a real desktop or display timing.

A real offscreen GPU test of an authored quad returned the same filled-face pixel `[0, 93, 61, 255]` in Wireframe and Shaded before this change. The GPU document pass ignored the mode even though Surface snap used it for mesh occlusion. The document renderer now clears colour/depth and draws line overlays in Wireframe, and draws faces in Shaded. Display mode participates in the per-view render invalidation, completed-image key and primary-press input basis; an old-mode image is rejected for new picking until the corresponding image is published. Existing captured Gumball/grip gestures keep their established handling.

The same test now passes, checking different face-interior pixels and rejection of the old-mode receipt before publication. Six combined GPU/egui tests pass, including Perspective Gumball after orbit and stale scene/preview cases. Eleven GPU-publication tests and fourteen snap UI tests pass; the four-view navigation test was extended and rerun to check that changing one mode causes no scene preparation or geometry upload and no encoding of the other three views. These are synthetic, offscreen tests; they do not verify the desktop, all object types, arbitrary trimmed isocurves, hidden-line rendering or full occlusion correctness. The early gate remains open.

### Captured Gumball Surface snap during a mode change

The existing active-Gumball path deliberately continues while a viewport image is being prepared, but it used the newly requested display mode for mesh occlusion even when the published image still showed the prior mode. A direct call to the Gumball Surface snap path using an actual offscreen GPU image reproduced snapping through a shaded mesh after selecting Wireframe but before the wireframe image was published. The path now uses the mode in the published receipt for occlusion; with no published GPU image it rejects the snap. The test passes for both directions of mode change, as do seven combined GPU/egui tests, nine gesture regressions and fourteen snap UI tests.

The GPU/egui fixture additionally sends complete press, drag and release sequences in both directions. For Shaded to Wireframe it verifies the shaded receipt remains published at the occlusion decision and checks that the occluded target does not change the line or create Undo history. For Wireframe to Shaded it checks that the wireframe receipt remains published, the line's committed pivot lies on the canonical sphere within `1e-6` model units, and one Undo record is created. The latter case places pointer movement and release in the same synthetic egui frame. These tests do not establish physical display timing; desktop and full occlusion validation remain open.

### NURBS display occlusion

A headless Line/Surface snap fixture reproduced snapping through a visible shaded NURBS plane to the B-rep sphere behind it. The occlusion check now includes the rendered triangles of `Geometry::Surface` along with authored meshes. The fixture passes for the surface in front, behind, hidden, and in Wireframe; fifteen snap UI tests and seven GPU/egui Gumball tests pass. This check matches the current display mesh and does not refine intersections against the NURBS source or implement Surface snap onto that source. It scans triangles synchronously; cost on large surface scenes is not measured. Other occluders, tolerance edge cases and desktop presentation remain open.

### Hatch display occlusion

A headless Line/Surface fixture failed before the fix because a shaded Hatch mesh in front of a B-rep was ignored. The shared display-occlusion check now includes Hatch's rendered mesh. The fixture passes for a Hatch in front, behind, hidden and in Wireframe; a separate Snappy Gumball drag verifies that a Hatch in front prevents the move and creates no Undo entry. On the default-feature E: build with `-j 2`, 17 snap UI tests, nine gesture tests and seven combined offscreen GPU/egui Gumball tests passed. The GPU suite did not specifically inspect Hatch pixels. These checks use displayed triangles, do not cover every object or transparency case, and do not establish desktop presentation or performance on dense scenes.

### Published colour texture isolation

An offscreen GPU/egui regression switched a quad from Wireframe to Shaded and read a face-interior pixel after the new frame was encoded and submitted but before its metadata was published. Before the fix the retained texture pixel changed from `[13, 13, 20, 255]` to `[0, 93, 61, 255]` while the published mode still said Wireframe. The viewport now draws changed document frames to a separate colour texture and retains the old published texture until publication; the two same-size colour targets are reused in alternation. The regression checks the old pixel and mode, the new pixel after publication, and texture reuse on switching back. It passes on the available GPU without an adapter skip. The diagnostics' logical target byte count includes the spare colour target.

Eight combined GPU/egui Gumball tests and 11 GPU navigation/publication tests passed sequentially, as did 11 gesture and 7 Gumball viewport tests. This adds one single-sample full-resolution RGBA colour target per active viewport after the first change: approximately `width × height × 4` logical bytes, excluding driver allocation overhead and in-flight retirement. The GPU budget is not implemented and this change is not proof of safe behavior under GPU memory pressure. At that stage moving frames still used immediate publication; see the later follow-up below. Desktop presentation, OS compositor timing, picking on physical input, pressure/fence retirement and performance remain unverified.

Follow-up: a combined GPU/egui test resizes the window, verifies that the prior published texture and a read-back pixel remain unchanged after the resized frame is encoded, then changes the camera before accepting that frame. The obsolete frame is not published and the final texture is paired with the new camera. All nine combined GPU/egui tests passed serially with an adapter; no skip was reported. This synthetic test waits for GPU work and does not prove behavior under sustained GPU backlog or desktop presentation.

### Per-viewport submission backpressure

Two deterministic offscreen regressions withheld the call that acknowledges GPU submission. Before this change a moving viewport encoded a second frame anyway, both after another moving frame and after a superseded stationary frame. The viewport now retains a queue-completion receipt independently of publication and does not encode another document frame until the receipt completes; while waiting it requests a repaint and uses the latest camera on the next eligible frame. Both cases pass after the change. At that stage moving metadata still published immediately; see the following update. Queue completion is not desktop presentation, and the synthetic delayed acknowledgment is not a measured heavy-GPU workload.

### Moving-frame publication after queue completion

A delayed-acknowledgment GPU test reproduced the displayed camera changing as soon as a moving frame was encoded, before queue completion. Moving frames now also retain their colour texture and captured camera in a pending publication until the queue completes. If only the camera advances while that work runs, the completed image can be published and the latest camera is then rendered; document, scene, selection, preview, CPlane, mode, hidden sources or target size changes invalidate it. Published frame IDs now identify the encoded frame rather than the later UI poll. A separate test verifies that an old document's moving frame cannot publish after a document change.

The delayed-acknowledgment test passed after the fix. Fourteen GPU navigation/publication tests, nine combined GPU/egui Gumball tests, 11 gesture regressions and seven Gumball viewport tests passed with the default-feature E: build and `-j 2`; the additional old-document test passed separately. No GPU adapter skip was reported. GPU queue completion is not physical presentation. Desktop mouse behavior, actual input-to-display latency, saturated GPU workload, target retirement and GPU budgeting remain open; no frame-rate improvement is claimed.

Follow-up tests hold a moving frame's submission acknowledgment, then independently change selection or the Gumball preview while leaving the document epoch unchanged. In each fixture the earlier encoded frame ID is not published and a subsequent frame is published. All 17 GPU navigation/publication tests passed serially on the available adapter. These assertions cover frame identity and camera, not a pixel-level comparison of selection or preview, and do not model physical presentation or a sustained GPU backlog.

### Preview pixel publication in two viewports

An offscreen MatrixApp/egui test renders a selected planar mesh in Shaded mode in Perspective and Looking Down. A follow-up uses distinct perspective and orthographic cameras. It captures pixels inside the original and prospective moved face, changes the Gumball preview by 12 mm, and waits until each viewport has encoded its preview but has not accepted its publication. Both sampled pixels remain equal to the prior image. After publication, both viewports have changed pixels at the old and moved locations, while canonical placement and Undo remain unchanged. A first fixture run used Wireframe, so its face-interior assertion was invalid; the final fixture explicitly selects Shaded and checks the face and empty destination initially differ. All 10 combined GPU/egui tests passed sequentially on the available adapter.

This covers two sampled pixels per view for one planar mesh with synthetic preview and two camera angles. It does not inspect all pixels, simulate pointer capture in this specific case, or measure physical display and input latency.

### Hover preview freshness

Hover's cached result and background receipt now carry the exact document allocation, full camera, viewport rectangle, scene epoch, preview revision and pointer position. A stationary cursor is re-evaluated after a Gumball preview moves the object, and a completed old background hit is rejected when its basis no longer matches. A 130-object headless fixture submits a hover, changes the preview and checks that no old hit is presented while the worker drains; the test does not pause the worker at a deterministic barrier. Another fixture exercises the synchronous hover path. This is hover freshness, not proof that all picking or Gumball paths use the displayed GPU receipt. Physical presentation and desktop pointer behavior remain open.

On E: with `-j 2`, 12 selection-overlay tests passed serially, including the two hover fixtures; 19 B-rep precision and 11 gesture tests also passed. The separate 26-test GPU publication suite remained running after many cases reported over 60 seconds; no pass or failure is claimed for that run. Public-source audit passed. The user's running OM9 session was not closed.

Follow-up extracts the receipt comparison into one shared predicate used by the hover path. A deterministic delayed-receipt unit fixture checks each mismatch independently: preview, epoch, a different allocation with the same document ID, a different camera pose with the same revision, viewport resize and pointer movement. The matching basis is accepted. The fixture verifies the predicate using a queued result; it does not pause an actual worker inside `show`. On E: the unit fixture and all 12 selection-overlay tests passed; the earlier GPU publication process remained live and did not yield a terminal result during this follow-up.

The next fixture installs a synthetic worker channel and captured basis into `MatrixApp`, changes Gumball preview, sends the old hit and runs a real egui frame. The old hit is not published and the pending receipt is cleared. Both delayed-hover unit fixtures passed on E: with `-j 2`. This deterministically exercises UI receipt handling but does not pause a real worker or establish physical presentation. The separately running GPU publication suite remains without a terminal result.

A second synthetic-channel UI fixture replaces the document with another allocation carrying the same document ID, removes the old target, and then delivers the old hit. The egui frame rejects the receipt and clears the pending job. All three delayed-hover unit tests passed on E: with `-j 2`. This tests exact allocation identity at UI receipt time, not an actual background worker paused through New/Open. The parallel GPU suite was still running without a final result when checked.

### GPU suite resource follow-up (2026-11-04)

The earlier default-parallel GPU test process stayed live for several minutes, consumed over 860 CPU seconds and about 687 MB working set, and many cases reported over 60 seconds without completion. The test harness contains both a polling loop and indefinite GPU waits; this observation does not identify a specific driver or test deadlock. To stop the ongoing resource load, the agent stopped only its own test process. Cargo reported abnormal exit `0xffffffff`; that run is not a pass.

The same 26-test `gpu_navigation_publication` binary was rebuilt and run with `-j 2` and `--test-threads=1`; all 26 passed in 8.88 seconds. An uncaptured serial verification passed all 26 in 9.09 seconds and printed no `SKIP: no GPU adapter` message. These are offscreen GPU tests, not physical presentation, latency or the user's desktop Gumball acceptance. Run this GPU suite serially for subsequent validations unless its concurrent behavior is investigated and fixed.

### Clear old hover while a worker is pending

An egui fixture first obtains a hover hit, holds a synthetic worker result pending, then moves the pointer away. Before the fix, the old hover remained visible while the result was pending, despite its input basis no longer matching. The hover path now clears the displayed hit as soon as its document, camera, scene, preview, viewport or pointer basis becomes stale. A completed result is still separately validated before publication. The fixture also delivers the old result afterward and verifies it cannot restore the old hit.

On E: with `-j 2`, the red/green fixture and six other hover-filtered unit tests passed, along with 12 selection-overlay, 11 gesture, seven Gumball viewport and 19 serial offscreen GPU/egui tests, with no reported GPU skip. This uses a controlled channel and simulated egui input, not a real worker paused at a barrier or a physical display. It does not close the Perspective desktop gate or establish general picking correctness.

### Hover state while the published scene is outdated

A GPU/egui fixture hovers a line, moves it in the document and renders one frame while the prior scene remains published. Before this change the exposed hover state retained the old object even though the hover overlay path was gated off. The active viewport now clears its hover object and cached input basis when the published content cannot be used for interaction, while navigating, or while a boolean preview suppresses hover. After the new scene is published, the fixture verifies the old position is not hovered and the moved line can be hovered again. The test passed after the fix on an available GPU; it exercises simulated egui input and one scene change, not physical presentation or general picking correctness.

On E: with `-j 2`, all 20 serial GPU/egui, 12 selection-overlay, 11 gesture and seven Gumball viewport tests passed; the GPU run reported no adapter skip. Public-source audit passed and artifact pruning removed no runs.

### Small pointer movements and hover freshness

The hover input-basis comparison formerly treated positions up to three logical points apart as equivalent. A point fixture placed the pointer four points from the target, then moved it 2.5 points: the new position exceeded the six-point picking tolerance but the old hit remained. The red/green fixture now passes because cached and worker hover results require the same pointer position; the receipt predicate also rejects a one-point difference. Stationary positions still reuse the cached result.

On E: with `-j 2`, 13 selection-overlay, eight hover-filtered unit, 11 gesture, seven Gumball viewport and 20 serial GPU/egui tests passed, without a reported GPU skip. These synthetic tests establish the tested tolerance boundary and stale-receipt predicate, not physical desktop behavior or all picking geometries. More pointer moves now cause fresh picking; responsiveness and worker turnover under sustained movement in a large scene have not been measured and remain open.

### Hover state on active viewport change

A simulated egui fixture hovered a point in Perspective, then activated Looking Down while the pointer remained outside that view. Previously, the active view only cleared a cached hover if the old pointer happened to lie inside its rectangle, leaving the Perspective hit exposed. The active view now clears hover and its cached basis whenever it has no hover pointer. The fixture failed before this change and passes after it; returning to Perspective restores the hover. This covers a controlled active-view change, not physical desktop interaction or all viewport transitions.

On E: with `-j 2`, 14 selection-overlay, eight hover-filtered unit, 11 gesture, seven Gumball viewport and 20 serial GPU/egui tests passed; the latter reported no adapter skip.

### Sustained hover motion probe

An opt-in headless fixture creates 2,001 point objects and alternates the pointer 200 times between positions 4 and 6.5 logical points from the target, then waits for a final in-tolerance hover. It asserts that no out-of-tolerance sample exposes a stale hit and that the final hover completes. It measures the entire synthetic egui event/frame call, not just picking, and counts how many in-tolerance samples have already received a hit. Offscreen far points and unconstrained frame scheduling make this a stress fixture, not a typical scene or a desktop input-to-display measurement.

The debug run passed on E: with `-j 2`: p50/p95/p99 10.275/17.574/17.850 ms, 33/100 in-tolerance samples with an available hit, final wait 10.639 ms. Three release runs passed using the same fixture and hardware, with per-run p50/p95/p99 of 1.711/1.802/1.916, 1.653/1.771/1.877 and 1.592/1.692/1.766 ms. Each saw 0/100 immediate in-tolerance hits; final hover completion after the last pointer input was 2.836, 2.809 and 2.778 ms. The tighter unpaced loop can outrun the worker, so these figures do not predict hover availability at 30 or 60 Hz. These are separate processes without controlled background load, not a matched before/after baseline, desktop responsiveness or a broad performance guarantee. Measure paced input and representative B-rep scenes before choosing an optimization.

### Paced hover probes with points and B-rep instances

The fixture now also accepts 16 ms and 33 ms input intervals and polls synthetic frames during each interval. It separately counts hits available immediately at the input frame and hits available by the next input. A second scene contains one target point and 200 far-away instances of a shared sphere B-rep; the target being hovered remains a point. One release run per configuration on E: produced:

| Scene | Interval | Input-frame p50 / p95 / p99 (ms) | Immediate / before-next-input hits | Final hover (ms) |
| --- | ---: | ---: | ---: | ---: |
| 2,001 points | 16 ms | 1.599 / 1.741 / 2.034 | 0 / 100 of 100 | 2.739 |
| 2,001 points | 33 ms | 1.615 / 2.116 / 3.651 | 0 / 100 of 100 | 2.768 |
| Point and 200 B-rep instances | 16 ms | 1.193 / 1.342 / 1.646 | 0 / 100 of 100 | 2.298 |
| Point and 200 B-rep instances | 33 ms | 1.176 / 1.452 / 2.149 | 0 / 100 of 100 | 2.319 |

All four passed the stale-hit assertions and final recovery. These are one run each, with software sleeps and extra synthetic polling frames, and do not measure display presentation or a B-rep surface hit. The far-away B-reps do not represent a dense visible CAD model. No before/after speedup or guaranteed 30/60 FPS follows from them; OS scheduling and uncontrolled machine load affect the samples. Further work should measure direct B-rep picking, realistic visible models and actual desktop input-to-display.

### Paced hover of a B-rep target

The same probe now optionally uses an OCCT sphere B-rep as the target, with 200 additional distant instances. It first checks that the scene picking path hits the target at the chosen screen point and misses at the outside point. This tests hover against the B-rep's display mesh, not exact kernel surface refinement, silhouette accuracy or topology identification. Each run alternates 100 inside/outside pairs; no outside sample or subsequent polling frame may retain a stale hit.

On E: release `-j 2`, one 16 ms run measured input-frame p50/p95/p99 1.197/1.303/1.654 ms, 0/100 immediate and 100/100 before-next-input hits, and 2.914 ms final completion. One 33 ms run measured 1.182/1.304/1.640 ms, 0/100 immediate and 100/100 before-next-input hits, and 3.154 ms final completion. Both passed. A preceding debug 16 ms run passed the stale-hit and eventual-recovery assertions but saw 95/100 before-next-input hits. These single runs include synthetic egui frame work and polling, have uncontrolled system load, and use a simple visible sphere with distant other objects. They do not establish desktop latency, complex occlusion, exact picking or guaranteed responsiveness. Public-source audit passed and pruning removed no runs.
