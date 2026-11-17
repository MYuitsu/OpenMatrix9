# Light viewport navigation: two spheres and Boolean Union

The user requested smoother Light navigation after the manual Light/High profile implementation. This is a scoped optimization of the existing viewport and native UI event path. Canonical BRep, Boolean tolerance, display tessellation, document ownership and Undo behavior are unchanged by this optimization.

## Changes and diagnosis

Light uses the existing Q2 transient rendering tier while a viewport is moving: 70% of each target dimension, one MSAA sample. Existing lower-quality pressure tiers remain available. When navigation settles, the existing renderer restores full resolution and four MSAA samples. High detail retains its previous tier policy. No additional mesh pipeline or native geometry adapter was added.

Reducing GPU work alone did not resolve the low pointer-navigation cadence. The six-run release experiment in `build/standalone-validation/ui-artifacts/light-motion-20261114-ab/` reduced moving GPU time from approximately 0.68 to 0.13 ms, while both variants still encoded approximately 10.6 Perspective frames/s. This separates GPU cost from the native-window bottleneck in this fixture.

The installed `egui-winit` 0.36.2 platform tail (`src/lib.rs`, lines 1172–1187) calls `set_ime_cursor_area` when the IME rectangle changes **or any input events remain**. With CMD retaining text focus while orbiting, already-handled pointer movement requests native IME caret positioning repeatedly even when the caret rectangle is unchanged. Consuming those handled navigation events after all widgets have run was followed by approximately 30 encoded frames/s in a debug probe and is the additional change in the final release comparison below. However, the final release counterfactual also reached approximately 30 frames/s: the earlier low cadence was not reproduced. The source establishes a redundant native IME update path, but these measurements do not establish that it alone caused the earlier 10 FPS or that the new handling increases steady FPS. Its individual Windows call duration was not separately instrumented.

At the end of the native UI pass, only while Light navigation is moving, OM9 consumes handled pointer movement, wheel, middle and secondary-button events. Keyboard, text, paste, IME composition, focus and primary clicks remain available. Pointer state and the normal widget/navigation routing remain intact. A changed IME rectangle still reaches the platform tail. The application does not clear CMD focus, disable IME, patch a dependency or change Windows configuration.

## Reproduction and evidence scope

Build with `rtk proxy cargo build -p om9-app --bin om9 --release --target-dir L:/om9-target -j 2`.

Run `rtk proxy python tools/benchmark_sphere_viewport.py --motion-comparison --output build/standalone-validation/ui-artifacts/<new-run>`.

The six disposable native-window runs alternate baseline/motion/motion/baseline/baseline/motion on one executable. Both sides use Light, Shaded, two radius-5 spheres centered at (0,0,0) and (5,0,0), and Boolean Two Objects Union with DeleteInput=Yes. The baseline diagnostic flag `OM9_LIGHT_MOTION_BASELINE` disables both new optimizations. The final mode enables both. A separate earlier GPU-only experiment above isolates the insufficient first optimization.

`OM9_SPHERE_BOOLEAN_AUDIT_POINTER=1` injects secondary-button drags into the native raw-input router, rather than changing the camera directly. The time-based path, commands, window size and `OM9_FRAME_HZ=30` are matched. Each steady phase lasts five seconds; drag releases at command transitions, pauses while the Boolean worker publishes and ends with a one-second idle/full-quality check. Commands use the CMD dispatcher. This exercises the actual navigation/UI/render path with synthetic input, not physical mouse automation. The previous direct-camera High/Light measurements remain separate evidence.

Frame intervals come from the raw-input hook. Unique encoded Perspective frames are counted independently. Command-crossing intervals are included in the preceding stage. These measure application cadence and GPU submissions, not actual RDP client displayed FPS, presented frames or input-to-photon latency. The small fixture on one Windows session does not establish large-scene or general hardware performance.

## Final paired release results

All **6/6 native-window release runs** passed the geometry, GPU configuration, completion, target restoration and zero-dropped-record checks. Receipt: `build/standalone-validation/ui-artifacts/light-motion-ime-20261114-ab/binary.json`; SHA-256 `ff70b0d911ae96522ff84f0c79e493ada0a23db4c34828ab2194d1192d0fe0e6`. The session used NVIDIA RTX 3060, Vulkan, AutoVsync and a 30 Hz application render cap.

Values below are medians of three runs per variant, except the worst callback interval, which is the maximum across all three. The command-to-result interval includes the frame crossing the first Sphere command, second Sphere and Boolean transitions; startup is excluded.

| Metric | Baseline Light | Optimized Light |
|---|---:|---:|
| Encoded Perspective frames/s, two spheres | 30.167 | 30.148 |
| Encoded Perspective frames/s, Union result | 29.978 | 29.957 |
| Moving result GPU time, p50 | 0.641 ms | 0.139 ms |
| Result callback p95 | 31.797 ms | 31.739 ms |
| Request to preview observed ready | 153.967 ms | 123.005 ms |
| Commit on application thread | 69.608 ms | 68.836 ms |
| Worst callback interval, first command through result | 80.969 ms | 78.240 ms |
| Callback gaps >100 ms, first command through result | 0 | 0 |
| Perspective target while dragging | 649×410, 4 samples | 454×287, 1 sample |
| All four targets after settling | 649×410, 4 samples | 649×410, 4 samples |

The moving GPU median fell by **78.3%**, approximately **4.6× less GPU time** for this viewport workload. The final paired run does **not** demonstrate higher steady FPS or a meaningful reduction of callback stalls: both sides already ran at the configured approximately 30 encoded frames/s. Preview samples overlap (baseline 121.850–154.039 ms, optimized 121.990–152.805 ms), and polling cadence is quantized; the median difference is not a stable kernel-speed claim. The low 10.6 FPS in the earlier experiment remains a session-dependent observation with an unresolved exact cause, rather than the baseline for a claimed threefold gain.

Both variants retain 1,032 input triangles and 876 Union triangles. All six outputs are one closed solid with zero naked edges, bounds `[-5,-5,-5,10,5,5]`, and measured volume 883.5729338363053–883.5729338363054 mm³ against the independent analytic `1125π/4`. Geometry measures are native source checks; triangle counts describe the derived viewport representation. Source identity and Undo/Redo are additionally covered by the focused integration tests. The complete JSONL traces and per-run summaries remain in the cited run directories, which are protected from normal artifact pruning by this record.

## Validation and remaining work

The focused integration targets `viewport_detail`, `display_modes` and `display_persistence` passed **11/11**. New coverage checks Light's transient/idle tier and preservation of keyboard, Vietnamese IME, focus and primary-click events; the retained geometry tests cover menu/CMD Sphere/Union, source identity, analytic measures, Surface Snap, Undo/Redo, atomic remesh, cancellation, stale publication and budget refusal.

The full library unit-test target remains blocked by the previously recorded 319 canonical-migration test compilation errors. This optimization does not close broad source/display separation, opaque NativeBrep coverage or BRep v5 save/reopen. Next architecture reference: `implementation/design/M01/viewport-export-separation-20261102/PLAN_SOURCE_DISPLAY_COMPLETION_M01_M09.md`; next performance scope beyond this acceptance fixture is a separately specified larger scene and direct client presentation measurement.

Final scoped tests passed again (11/11). Both release and the default debug executable built successfully in the reused external target with `-j 2`. `tools/public_source_audit.py` checked 16,062 source files, 594 icon bindings and 593 SVGs with zero errors. Scoped `git diff --check` passed. Artifact pruning removed no runs and retained the newest ten plus cited evidence. The tested Solid guidance was synchronized to source and installed skills under the standing Solid authorization; global milestone acceptance and unrelated migration edits remain open.
