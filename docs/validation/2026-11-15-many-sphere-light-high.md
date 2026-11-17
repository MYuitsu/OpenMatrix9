# Many-sphere Light / High detail comparison

Follow-up: [worker-prepared Boolean publication buffers](2026-11-15-boolean-publication-worker-buffers.md) reduce the final UI stall and repeat both scene sizes. The timings below remain the historical pre-optimization measurements.

User request: increase the number of spheres, perform Boolean, and compare FPS and freezes between product Light and High detail. This extends the existing opt-in native-window audit, not product CAD behavior. Python only orchestrates Windows processes and analyzes Rust diagnostics; geometry, commands, UI routing and timing remain in Rust.

## Fixture and method

Each independent pair consists of two radius-5 spheres separated by 5 mm. Pair centers form a planar lattice with 20 mm column spacing and 16 mm row spacing, so different pairs do not intersect. For P pairs, columns = ceil(sqrt(P)), rows = ceil(P/columns). In creation order, pair j has left center `(20*(j%columns)-10*(columns-1), 16*floor(j/columns)-8*(rows-1), 0)`; its right center adds 5 mm in X. All inputs are selected in creation order to avoid random UUID ordering changing the Union fold.

One normal Sphere command runs per native raw-input frame. The audit fits all four cameras to the same authored bounds, then uses eight seconds of secondary-button drag input before and after BooleanUnion of all inputs with DeleteInput=Yes. It pauses drag while the background worker publishes. Actual widgets/navigation and GPU passes run in a native OM9 window; pointer events are synthetic, not physical mouse automation. After a one-second idle/full-target check, source geometry is measured outside the timed frame sequence.

Both variants use the current product presets: Light display tessellation 0.1 mm / 0.5 rad plus its transient 70% dimensions / 1-sample motion tier; High detail 0.01 mm / 0.3 rad and its existing scheduler policy. CAD source geometry and kernel tolerance remain independent of those display requests. This compares whole presets, not an isolated architectural change. Render cap is 30 Hz, Shaded mode, CMD dispatch, matching window/DPI/path and one release executable. Order is High/Light, Light/High, High/Light, with fresh disposable processes and no concurrent benchmark/build.

The pointer curve is time based with the same speed and amplitude. Its clock starts at audit startup, so different creation/Boolean durations shift the motion phase; these are not identical per-frame camera poses. Steady intervals exclude transitions and repeated runs show variation. Boolean source oracles do not depend on camera phase.

The source oracle is one result object with P closed solids, zero naked edges, volume `P*1125*pi/4 mm³` and independently computed lattice bounds. A result object may contain multiple disjoint solids; those are intentional here. Input/output triangle counts describe only the viewport representation.

## Timing definitions

- FPS counts unique encoded Perspective frames, independently of native callback cadence. Steady FPS excludes the first second and last 250 ms of each motion phase, so commands and camera-fit transitions do not distort the steady value.
- Raw-input callback gaps retain command crossings. `active_freeze` covers creation, active navigation and Boolean phases, with counts above 50/100/200/500/1000 ms and timestamps for gaps above 100 ms. `settling_gaps` separately records the final idle/full-quality phase; an idle repaint wait is not a freeze. The older `freeze` field retains all callbacks for traceability. A long active callback gap is a stall proxy, not measured client presentation; the multi-second final poll independently establishes a blocked UI-thread call.
- Boolean total runs from request to the next raw-input observation of completion. Final-poll time measures the UI-thread `poll_boolean_union_worker` call that receives and commits the result; it includes receive, validation and publication, and is not an isolated native kernel or commit timer.
- Source verification runs after the last timed raw-input frame, preventing measurement code from being mistaken for a product freeze. Input inventory inside timing reads only derived display counts.
- Encoded FPS, GPU duration and callback gaps do not measure swapchain presentation, RDP client displayed FPS or input-to-photon latency. Results apply to these fixtures and this Windows session.

## Reproduction

Build: `rtk proxy cargo build -p om9-app --bin om9 --release --target-dir L:/om9-target -j 2`.

Run 64 spheres: `rtk proxy python tools/benchmark_sphere_viewport.py --pairs 32 --output build/standalone-validation/ui-artifacts/many-spheres-64-20261115-ab`.

Run 128 spheres: `rtk proxy python tools/benchmark_sphere_viewport.py --pairs 64 --repeats 1 --output build/standalone-validation/ui-artifacts/many-spheres-128-20261115-ab`.

The initial proposed second level was 256 spheres. After the 64-sphere High runs took 52–55 seconds for Union and already showed multi-second commit stalls, the next completed-comparison target was adjusted to 128 spheres with one run per mode. The 256-sphere level was not run; it has no result or inferred timing here.

`--repeats 1` performs one pair for a pilot; the default is three. `--exe` selects the binary, and `--analyze-only` rechecks existing JSONL. The old two-sphere and Light-motion comparisons remain supported. Each many-scene process has a 600-second audit deadline and an outer 620-second process timeout; only that disposable process can be terminated on timeout.

## Validation before release measurement

An initial eight-sphere debug pilot exposed an audit assumption that Sphere selects its new output. The audit was corrected to recover the new document ID instead; no Sphere behavior was changed. The corrected pilot in `build/standalone-validation/ui-artifacts/many-spheres-pilot2-20261115/` completed both profiles with the expected four-solid source oracle. Its debug performance is not pooled with release measurements. The failed pilot remains in `build/standalone-validation/ui-artifacts/many-spheres-pilot-20261115/` and is not counted as a pass.

The focused Rust integration targets `viewport_detail`, `display_modes`, `display_persistence` passed 11/11 after the audit/timing extension. The analyzer also rechecked the previous eight-run profile batch and six-run Light-motion batch successfully. Existing library-test compilation failures from canonical migration remain outside these scoped targets.

## Release results

The 64-sphere batch completed **6/6** runs (three per mode), with zero dropped diagnostics and passing GPU configuration, completion, geometry and full-target-restoration assertions. Binary SHA-256: `19bd30c53cdd2a61f7ef914bb20916f8391f15a73006fcbd407e0faa62b72c4d`. Raw logs and source/binary receipts: `build/standalone-validation/ui-artifacts/many-spheres-64-20261115-ab/`. Durable per-run/aggregate data: [JSON record](2026-11-15-many-sphere-light-high.json).

The following are medians of the three per-mode runs. "Worst gap" is the median of each run's largest command-through-settle gap; the full observed ranges are retained below and in JSON.

| 64 spheres → 32-solid Union | High detail | Light |
|---|---:|---:|
| Encoded FPS, pre-Boolean camera motion | 29.996 | 29.996 |
| Encoded FPS, result camera motion | 30.001 | 29.997 |
| Moving GPU time, result p50 | 0.309 ms | 0.094 ms |
| Sum of foreground Sphere command times | 3.765 s | 2.517 s |
| Boolean request → completion observed | 52.653 s | 23.600 s |
| Final UI poll including publication | 2.116 s | 2.065 s |
| Worst raw-input callback gap | 2.125 s | 2.073 s |
| Active gaps >100 ms per run | 2 | 1 |
| Gaps >500 ms per run | 1 | 1 |
| Gaps >1,000 ms per run | 1 | 1 |
| Input viewport triangles | 320,512 | 33,024 |
| Union viewport triangles | 264,448 | 28,032 |

Light shortened total observed Union completion by **55.2% (2.23×)** and moving-result GPU time by **69.5% (3.28×)**. It did **not** establish a higher steady FPS under the 30 Hz cap or solve the multi-second publication stall. High worst gaps were 2.123–2.157 seconds; Light 2.067–2.100 seconds. Every run retained exactly one gap over a second, aligned with the final worker poll/commit. Active >100 ms gap counts were High 2/2/3 and Light 1/1/2. Remaining active gaps occurred around request/creation; settling callbacks are reported separately because they can include deliberate idle waits. All timestamps and stages remain in the JSON record. Waiting for a worker for 23–55 seconds is not itself a 23–55 second UI freeze.

The Boolean samples were High 55.085/52.230/52.653 s and Light 23.757/23.218/23.600 s. Steady encoded FPS across every pre/post camera phase ranged only 29.993–30.009. Native callback cadence was much faster in this session (hundreds per second with the opt-in audit's repaint requests); it was deliberately not reported as viewport FPS. This differs from the earlier session-dependent 10 FPS probes.

The 128-sphere corroborating pair completed **2/2** runs with the same binary and all assertions passing. This is **one sample per mode**, not repeated performance coverage. Artifact receipt: `build/standalone-validation/ui-artifacts/many-spheres-128-20261115-ab/`.

| 128 spheres → 64-solid Union | High detail | Light |
|---|---:|---:|
| Encoded FPS, pre-Boolean camera motion | 30.004 | 30.000 |
| Encoded FPS, result camera motion | 29.993 | 30.006 |
| Moving GPU time, result p50 | 0.432 ms | 0.127 ms |
| Sum of foreground Sphere command times | 13.194 s | 10.402 s |
| Boolean request → completion observed | 199.627 s | 87.238 s |
| Final UI poll including publication | 5.115 s | 5.014 s |
| Worst active callback gap | 5.142 s | 5.040 s |
| Active gaps >100 ms | 65 | 47 |
| Active gaps >500 ms | 2 | 1 |
| Active gaps >1,000 ms | 1 | 1 |
| Input viewport triangles | 641,024 | 66,048 |
| Union viewport triangles | 528,896 | 56,064 |

Light completed this Union **2.29× sooner (56.3% less total wait)** and used **70.6% less moving GPU time**, while both modes still had a roughly five-second final publication stall. Of the >100 ms gaps, 61 High and 44 Light occurred during creation, one per mode at the full-scene transition, and three High/two Light during Boolean. The three final settling gaps per mode are excluded from active counts; their largest interval was approximately 484 ms and must not automatically be called a freeze. No >100 ms interval occurred in either steady camera-motion window at either scene size.

All **8/8 release runs** passed the source oracle. At 64 spheres, both profiles measured 28,274.33388276626 mm³, 32 solids, closed, zero naked edges and bounds `[-55,-45,-5,60,45,5]`. At 128, both measured 56,548.667765580394 mm³, 64 solids, closed, zero naked edges and bounds `[-75,-61,-5,80,61,5]`. These scalar/topology checks and matched authored parameters are not a global geometric-equivalence or general scene/picking proof. Every run restored all four targets to 649×410 with four samples; Light used 454×287 with one sample in motion, with full-quality targets also observed at a transition in its 128-sphere run. GPU configuration was NVIDIA RTX 3060/Vulkan/AutoVsync at 1.0 pixels per point.

The most useful remaining performance target is the UI-thread final-poll/publication path. Its 2.06–2.15 second and 5.01–5.11 second durations nearly account for the longest active gaps, whereas the measured GPU pass is below a millisecond. This identifies the blocking call, not its exact internal cause. Next step is to split timing inside `Controller::poll_boolean_union_worker` and `commit_solid_result_with_reserved_sources` into source validation, transaction and scene preparation before choosing a fix. This task changed only the opt-in audit and diagnostic timing, and does not claim to have fixed the remaining stall or completed source/display migration.

Final checks: both levels reanalyzed successfully after separating active and idle gaps; the durable JSON includes native source measurements, every long-gap timestamp, per-run data, source/binary fingerprints and the final analyzer hash. Both batches used the same binary and current Rust source fingerprints were verified against their receipts. Debug and default-feature release builds succeeded in `L:/om9-target` with `-j 2`. Public source audit checked 16,064 source files, 594 icon bindings and 593 SVGs with zero errors. Scoped whitespace and ledger/milestone JSON checks passed. Normal artifact pruning retained the newest ten plus cited evidence. README, live ledger, scoped M01/M03 runtime evidence and source/installed Solid measurement guidance were synchronized without changing CAD business behavior or global acceptance status.
