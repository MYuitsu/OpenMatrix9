# Viewport navigation cache and timing

Scope: standalone M01 GPU viewport. A published texture whose document, camera, size, scene, CPlane, selection, preview and hidden-source state are unchanged now bypasses resource readiness work. Overlay cache matching also rejects stale hidden-source state during motion. Opt-in `OM9_INTERACTION_LOG` adds cumulative per-viewport target, scene/overlay preparation, upload, encode, cache and readiness counters. Input-to-submit timing now starts only when a navigation event is accepted and is retained until that view's camera is encoded and submitted. No frame rate, resolution, LOD, geometry or selection business rule was changed.

## Verification

- `cargo test -p om9-app -j 2 --test gpu_navigation_publication --test navigation_lifecycle --test navigation_deadlines --test navigation_precision --test navigation_commands --test ui_replay`: 31 passed across 6 suites on the final source.
- `cargo test -p om9-app -j 2 --test navigation_drag`: 7 passed, including stable BRep scene epoch and display-buffer identity while orbiting.
- `cargo test -p om9-app -j 2 --lib overlay_cache_tests`: 1 passed; `--lib frame_scheduler`: 4 passed. A separate `--nocapture` run of the GPU suite confirmed a working adapter and 10 passed, including four-view caching, DPI invalidation, selection, hidden preview, document replacement, resize/oversize and Box Undo/Redo.
- Final release `cargo build -p om9-app --bin om9 --release -j 2` passed. Builds/tests used Windows, default product features, targets outside the checkout (`L:/om9-target` or `L:/om9-nav-target`), and the pre-existing dirty worktree. The user's separate OM9 process was preserved.

## Paired measurements and limitations

Three synthetic native-window replays, hidden windows, 90 scripted steps apiece, four viewports, full-resolution Q0, 30/24 Hz targets, Vulkan RTX 3060 driver 610.88, 100% DPI, low-latency surface queue. Light = empty document; mesh = 400 independent repeated sphere meshes; instances = 1,000 instances of one authored tetrahedron definition. Each baseline/final pair was run consecutively; 67–70 Perspective navigation frame-interval samples per run. No records were reported dropped. The instance fixture SHA-256 is `f4e31f860348875bcb76606798b265c6660c010f81830a61bf540efc143d3534`.

| Fixture | Baseline frame interval p50/p95/p99 ms | Final p50/p95/p99 ms |
|---|---|---|
| Light | 93.504 / 97.586 / 99.054 | 93.965 / 98.583 / 99.673 |
| 400 meshes | 93.622 / 100.226 / 231.948 | 93.602 / 99.920 / 109.364 |
| 1,000 shared instances | 93.960 / 100.125 / 105.229 | 93.732 / 96.972 / 99.401 |

These short runs do **not** establish an FPS or latency improvement. An earlier baseline had a roughly 31 ms median before both baseline and final shifted to roughly 94 ms; the cause is not established. Hidden-window results are not actual user-visible desktop performance. Detailed frame intervals, late counts, GPU timestamp samples, frame/encode elapsed time, hook-to-submit samples and cumulative/delta viewport counters are in [aggregate JSONL](2026-11-01-viewport-navigation-performance.jsonl). The baseline and final input timing instrumentation differ, so those latency values must not be directly compared. Elapsed UI/encode times are wall times, not thread CPU measurements.

For the final 1,000-instance replay, after navigation began the active Perspective viewport encoded 72 frames; each of the three unchanged views encoded zero, performed zero further readiness checks, geometry uploads, scene or overlay preparations and target rebuilds, while reusing its cached image 93 times. The camera-only test also retains the already-published BRep display buffer, but no direct native tessellation-call counter was recorded.

Baseline binary SHA-256 `c423999c3889e6421661fcc607bc6c61bde46c4e2c8dc898cca059bb8f2b4157`; final `bb0b008ef18935dea9d06eb78e3744d0de271791bccee707a6b23c8172fc863c`. The final binary's logged navigation/frame-scheduler/UI/GPU-source fingerprint is `a9580caabdbb4b1d0cb6068adb6c22d88199bf4e80718e7934fcd3493e821af1`, matching those files on disk. The pre-existing baseline fingerprint is `d707b1953f40b528cc8e5d760a31552d08bd90a684bf114adb18867750819901`.

WPR is installed, but starting the DesktopComposition profile returned `0x80070005 Access is denied`; PresentMon/WPA exporter were not found on PATH. Therefore input-to-present and present-to-display remain unmeasured. The real OS pointer, visible window, human smoothness assessment, controlled CPU/GPU trace and proven 30/24 FPS are open. The shared resource pool still invalidates the GPU scene on selection/scene signature changes; incremental updates for those changes were not implemented. Next: profile a visible release with a permitted Windows present trace and a representative authored BRep/instance model, then target the measured bottleneck and evaluate selection-only scene reuse separately.
