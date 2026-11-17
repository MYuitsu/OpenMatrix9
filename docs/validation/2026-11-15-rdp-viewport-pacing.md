# Remote Desktop viewport pacing

The authorized target is stable near-30-FPS navigation before and after a many-sphere Boolean, retaining canonical CAD data and comparing Light with High detail. This is a presentation/frame-loop optimization; it does not change Boolean tolerances, source geometry, display tessellation, picking or transactions.

## Diagnosis and change

Native timing instrumentation separates raw-input-to-UI-tail, UI-tail-to-painter (platform output plus egui tessellation), and painter-to-next-input (painting, presentation and event-loop wait). These broad brackets do not individually time `Present`, the RDP encoder or the client's display.

The original release four-sphere pilot reproduced about 10.6 encoded Perspective FPS with UI work around 2.3 ms and GPU work below 1 ms. Debug timing placed approximately 72 ms of a 94 ms cycle after the painter boundary. Switching Vulkan to DX12 or frame latency from one to two retained approximately 10.6 FPS. Disabling the VSync wait with `AutoNoVsync` in a later debug instrumentation build reduced the painter-to-next-input median to about 2.7 ms and restored 29.8–30.0 encoded FPS. The frozen-release same-binary counterfactual below establishes the surface policy as a causal factor in this remote session. This it does not identify an individual driver/compositor call or establish a general IME timing cause.

`om9-native-owner` now exposes the read-only scalar Win32 `GetSystemMetrics(SM_REMOTESESSION)` query. Safe Rust selects `AutoNoVsync` in a remote session and retains `AutoVsync` locally. The existing CAD frame clock still bounds scene submissions at 30 Hz; no busy-wait thread or system setting is introduced. wgpu selects a supported mode/fallback, so a nonblocking mode is not guaranteed on every adapter. `OM9_PRESENT_MODE=auto-vsync|auto-no-vsync` is a diagnostic same-binary override, not a stored product preference.

The optional native audit now requests at most a 30-Hz timer rather than flooding a nonblocking surface with 1-ms timer requests; navigation and worker completion can still wake normal UI processing. Its analyzer counts distinct encoded Perspective frame IDs and reports their interval percentiles separately from UI callback intervals. Window GPU configuration, actual remote-session detection, binary/source fingerprints, source measures and zero-drop log checks are retained.

## Diagnostic evidence

- `build/standalone-validation/ui-artifacts/pacing-dx12-20261115-probe/`: initial **Vulkan** release pilot; directory name was chosen before the backend override was applied. The GPU receipt, not the directory name, identifies the backend.
- `build/standalone-validation/ui-artifacts/pacing-phases-20261115-probe/`: debug timing, Vulkan/VSync.
- `build/standalone-validation/ui-artifacts/pacing-dx12-phases-20261115-probe/`: debug DX12/VSync counterfactual.
- `build/standalone-validation/ui-artifacts/pacing-latency2-20261115-probe/`: former release, Vulkan/VSync with frame latency two.
- `build/standalone-validation/ui-artifacts/pacing-no-vsync-20261115-probe/`: debug instrumentation build, Vulkan/AutoNoVsync; both presets returned near 30 encoded FPS.
- `build/standalone-validation/ui-artifacts/pacing-rdp-default-20261115-probe/`: session-detection and bounded-timer smoke test. A release compilation overlapped its latter part; its CPU/FPS figures are not accepted as release performance evidence.

System ETW profiling was unavailable (`wpr -start CPU -filemode`: 0xc5585011). No profiling policy or machine settings were changed; subsequent WPR status confirmed no recording. Instrumentation inside the app supplies the evidence instead.

## Validation

Eighteen focused integration tests passed: five Boolean publication/source/Undo/failure tests, six viewport-detail tests, three display-mode tests, two persistence tests and two frame-clock/presentation tests. The clock regression exercises frequent deliveries without allowing unrestricted CAD submissions. Public-source audit: 16,067 source files, 594 command bindings, 593 SVGs, zero errors. Broad library-unit compilation remains affected by the preexisting canonical migration errors documented in earlier evidence; it is not counted as passing.

## Frozen release results

All ten runs passed with SHA-256 `58507017a504dab1241e22b51f4d1b952b77f3934bf67437d8229c94674d1e77` and identical recorded runtime source fingerprints. Builds/tests had finished before these batches. Commands use `python tools/benchmark_sphere_viewport.py --pairs 32 --repeats 1 --present-mode auto-vsync` for the counterfactual; `--pairs 32 --repeats 3` and `--pairs 64 --repeats 1` for the default remote policy, each with its corresponding output directory below.

| Batch | Spheres | Profile | Runs | FPS before Boolean | FPS after Boolean | Encoded p95, ms | Steady max, ms |
|---|---:|---|---:|---:|---:|---:|---:|
| VSync counterfactual | 64 | high | 1 | 10.588 | 10.608 | 96 | 97 |
| VSync counterfactual | 64 | light | 1 | 10.599 | 10.600 | 96 | 97 |
| Default RDP | 64 | high | 3 | 30.000 | 30.006 | 36 | 37 |
| Default RDP | 64 | light | 3 | 30.000 | 30.001 | 36 | 37 |
| Default RDP | 128 | high | 1 | 30.009 | 29.993 | 35 | 37 |
| Default RDP | 128 | light | 1 | 30.000 | 30.006 | 36 | 37 |

FPS and final polls are medians at 64 spheres; 128 has one sample per profile. Percentiles/maxima above are the worst observed across both steady phases and runs. No steady encoded interval exceeded 100 ms. This is native-window application encoding evidence, not RDP client displayed/video FPS.

| Batch | Spheres | Profile | Boolean total, s | Final UI poll, ms | Largest active gap, ms | Active gaps >100 ms | Moving result GPU p50, ms |
|---|---:|---|---:|---:|---:|---:|---:|
| VSync counterfactual | 64 | high | 51.338 | 55.052 | 181.467 | 1 | 1.877 |
| VSync counterfactual | 64 | light | 21.628 | 24.123 | 96.687 | 0 | 0.557 |
| Default RDP | 64 | high | 53.316 | 53.854 | 181.811 | 5 | 1.700 |
| Default RDP | 64 | light | 24.557 | 24.058 | 88.221 | 0 | 0.485 |
| Default RDP | 128 | high | 198.144 | 150.406 | 601.296 | 63 | 2.299 |
| Default RDP | 128 | light | 89.969 | 88.503 | 453.034 | 48 | 0.851 |

The matched 64-sphere counterfactual establishes that changing the surface policy removes the recurring approximately 94 ms cadence: painter-to-next-input medians fall from approximately 91 ms to below 1 ms. Frame-clock scene submissions remain near 30 Hz. This does not isolate an individual driver call. Light remains faster than High for this Boolean workload, but the pacing change itself does not speed geometry computation. The unthrottled surface and synthetic per-callback pointer replay generate more UI activity (roughly 300 callbacks/s at 64); these callbacks are not CAD FPS. The 64-sphere Boolean medians were 53.316/24.557 s High/Light versus 51.338/21.628 s in the VSync controls; this pacing fix makes no geometry-throughput improvement claim. Independent UI repaint/backpressure optimization remains open.

## Source guarantees and remaining work

Every run retains the existing independent native-source volume/bounds/closedness/solid-count/naked-edge oracle and final full-size four-sample targets. At 64 spheres, High has 320,512 input / 264,448 output triangles, Light 33,024 / 28,032. At 128 these counts double. Volume is 28,274.33388276626 mm³ at 64 and 56,548.667765580394 mm³ at 128. Each intersecting pair is separate from other pairs; the single result entity contains a BRep compound with 32/64 closed solids. This is not a worst-case all-overlapping Boolean workload.

Creation/request/publication pauses remain and are included in the active gap totals, even with steady navigation near 30 FPS. Inspect per-gap stages and timings in the JSON. The harness pauses drag while a Boolean worker is pending to admit guarded publication; it does not establish continued camera-drag performance during every worker operation. Local-console physical presentation, dynamic local-to-RDP session transitions, real mouse/client video latency, other hardware, general scenes and full canonical-source/persistence migration remain unclosed. No geometry, tolerance, Undo or pick behavior was changed by this pacing fix.

Frozen-binary batches:

- `build/standalone-validation/ui-artifacts/rdp-pacing-vsync-64-20261115-ab/`: 2 runs passed.
- `build/standalone-validation/ui-artifacts/rdp-pacing-64-20261115-ab/`: 6 runs passed.
- `build/standalone-validation/ui-artifacts/rdp-pacing-128-20261115-ab/`: 2 runs passed.

[Machine-readable aggregates, receipts and per-run summaries](2026-11-15-rdp-viewport-pacing.json).
