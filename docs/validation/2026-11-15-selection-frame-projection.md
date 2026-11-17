# Selection-frame UI: prepare immutable camera once

The user requested further UI/selection optimization after the [Boolean request batching experiment](2026-11-15-boolean-request-source-batch.md), preserving exact CAD data and retesting 64/128 spheres with Light and High detail.

## Cause, change and controls

The remaining Boolean request transition includes a four-pane `selection_frame` rebuild. Its loop projects every displayed vertex of every selected object. Previously each point invoked `Camera::project`, validating the same camera and viewport and normalizing the same camera basis repeatedly. The existing cache avoids unchanged frames, but does not reduce the first rebuild after selection/camera/preview changes.

Safe Rust now prepares a borrowed immutable `PointProjector` once per pane. It shares the original per-point projection arithmetic and retains finite-point/arithmetic checks, near/far clipping, exact transform validation, every selected vertex, pixel conversion, margin, clipping and the original selection-frame cache key. It introduces no source/GPU-buffer copy, worker, persistent cache, bounding-box approximation, sampling, precision change or CAD transaction. The canonical document, BRep, topology, source-based picking/snap, native Boolean operation/tolerance, display tessellation, request batching, publication and RDP presentation remain unchanged.

`OM9_SELECTION_FRAME_BASELINE=1` selects the former per-point camera preparation in the same executable. It is an opt-in diagnostic, not a product setting. With the existing interaction log enabled, each cache miss records its actual path, pane, vertex count, calculation duration and resulting frame. The native benchmark aggregates the four rebuilds within the Boolean request transition separately from the complete callback gap, synchronous request call, final UI poll, total background Boolean wait and steady encoded FPS.

## Validation design

Eighteen camera integration tests passed: the new projector regression compares bit patterns against the former point path over transformed cameras, analytic screen coordinates, clipping boundaries, pan and invalid camera/viewport/point cases; existing camera fit/grid/orbit fixtures retain independent geometric expectations.

Seventeen app integration tests passed across `selection_projection`, `boolean_publication`, `viewport_detail`, `navigation_deadlines` and `display_modes` (35 distinct tests including the camera batch). The egui regression checks exact frame rectangles in all four panes after pan, preview, changing the selected subset and clearing selection, while document/Undo remain unchanged. Boolean/profile tests retain native source measures, Surface Snap, source wires, stale/cancel/pressure guards, Undo/Redo, display modes and pacing. The final expanded UI regression was rerun successfully; that retry is not an additional distinct test.

The initial attempt to compile existing `selection_overlay` was blocked by two preexisting ignored hover fixtures that construct `Geometry::Brep(Brep)` after the canonical-source type migration. Those legacy fixtures are unchanged and not counted passing; the new focused egui regression is a separate integration target. Full legacy selection-test migration remains open. The release build and all sixteen native-window runs completed successfully; frozen results follow.

Completed native-window comparison: 16 runs, two alternating baseline/prepared pairs per profile at 64 and 128 spheres, on one frozen release executable. Both arms retain the production separated viewport, batched source request, worker publication, AutoNoVsync and 30-Hz CAD clock. Workload is the existing lattice of intersecting radius-5 sphere pairs followed by BooleanUnion DeleteInput=Yes, with synthetic pointer navigation before/after the worker and no drag during guarded publication.

Acceptance requires actual selection-path witnesses for all four panes, equal projected vertex coverage and display density per profile, identical native result source SHA across paths/profiles per scene, native volume/bounds/closedness/solid-count/naked-edge oracles, zero dropped diagnostics and restored four-sample full-size targets. Steady FPS counts distinct encoded Perspective frames; client video and physical input remain outside this evidence. All creation/fitting/request/commit gaps stay in active freeze accounting; final idle/settling is separate. Full migration/persistence and library-unit compilation remain open outside this scoped task.

## Frozen release results

All **16/16** native-window runs passed on release executable SHA-256 `0402499bb83298bfaf347dcc3b641b23e89401abef513fb27e224de61f14010c`. Both scene batches have identical executable/runtime-source receipts; no build, test or diagnostic probe overlapped formal timings. The actual path, four-pane rebuild and equal projected-vertex coverage assertions passed.

| Spheres | Profile | Four-pane selection calculation baseline → prepared, ms | Reduction | Full request/UI callback gap, ms | Worst active gap, ms |
|---:|---|---:|---:|---:|---:|
| 64 | high | 116.375 → 28.491 | 75.5% | 139.998 → 57.648 | 140.179 → 91.490 |
| 64 | light | 30.723 → 5.904 | 80.8% | 42.541 → 16.530 | 80.821 → 84.019 |
| 128 | high | 216.990 → 33.706 | 84.5% | 253.799 → 65.895 | 257.284 → 232.364 |
| 128 | light | 55.576 → 8.766 | 84.2% | 80.214 → 29.850 | 206.756 → 211.665 |

Calculation/transition figures are medians of two runs; active maxima are the largest gaps across both entire runs. Active accounting includes sequential Sphere creation, scene fitting, request, worker admissions, final publication and navigation; final idle/settling is separate. Selection timing covers every displayed vertex and includes transform/point validation. The complete transition also includes selection, source request and other UI work.

| Spheres | Profile | Path | FPS before / after Boolean | Encoded p95 / max, ms | >100 ms active gaps per run | Final UI poll, ms | Total Boolean wait, s |
|---:|---|---|---:|---:|---|---:|---:|
| 64 | high | baseline | 30.000 / 29.997 | 36 / 38 | [2, 1] | 53.851 | 52.827 |
| 64 | high | prepared | 30.004 / 29.999 | 36 / 38 | [0, 0] | 61.685 | 53.197 |
| 64 | light | baseline | 30.002 / 29.997 | 36 / 38 | [0, 0] | 23.843 | 24.773 |
| 64 | light | prepared | 30.000 / 29.995 | 36 / 37 | [0, 0] | 23.966 | 24.502 |
| 128 | high | baseline | 29.998 / 29.993 | 36 / 38 | [61, 57] | 154.157 | 199.783 |
| 128 | high | prepared | 29.996 / 30.006 | 36 / 37 | [57, 58] | 154.409 | 176.564 |
| 128 | light | baseline | 29.993 / 30.001 | 35 / 37 | [45, 44] | 88.381 | 70.190 |
| 128 | light | prepared | 30.004 / 29.997 | 36 / 37 | [45, 44] | 92.638 | 70.107 |

FPS, poll and total wait figures are medians; encoded intervals are worst across both runs/phases. The 30-Hz clock limits maximum-throughput inference. Total background Boolean wait is distinct from a UI freeze. Native mathematics, display quality and worker publication are common between A/B arms.

The first 128-High baseline had a 223.933 s total Boolean wait; the other High runs were 175.633–177.277 s. This variation is much larger than the measured selection/UI cost, so the median total-wait difference is not evidence of a native Boolean speedup from this change. The repeated selection/transition measurements are the scoped causal evidence.

Worst active gaps improved in High, but did not improve in Light: 80.821 → 84.019 ms at 64 and 206.756 → 211.665 ms at 128. The optimized runs' largest gaps occur in creation/fitting, which this patch does not alter. Do not report the 61–74% transition reduction as a reduction of every freeze or of the whole workload.

## Exact source, full selection coverage and remaining limits

At each size, all eight results have identical native source SHA across quality profiles and paths. Native volume, bounds, closedness, solid count and zero naked edges passed independently. All four settled targets restored to 649 × 410 and four samples; no diagnostic records were dropped.

| Spheres | Native solids | Native volume, mm³ | High projected points / input triangles / result triangles | Light projected points / input triangles / result triangles |
|---:|---:|---:|---:|---:|
| 64 | 32 | 28274.333882766 | 790016 / 320512 / 264448 | 204800 / 33024 / 28032 |
| 128 | 64 | 56548.667765580 | 1580032 / 641024 / 528896 | 409600 / 66048 / 56064 |

Per-point basis preparation is measured directly in the former control and replaced only at this boundary. The projector borrows the immutable camera, has private validated fields and retains finite-point/arithmetic/clip checks; existing per-object transforms and every point are retained. Camera math regression includes bit-for-bit comparisons, independent analytic screen coordinates and invalid-input/clipping oracles. Actual egui tests compare all four frame rectangles after pan, preview and selection changes, then verify clearing and unchanged document/Undo. Native pointer poses can vary slightly with event timing; no pixel-identical screenshot or physical-input claim is made.

Optimized 128 high runs retain [57, 58] active gaps over 100 ms, with [54, 56] attributed to sequential Sphere creation. Remaining UI/creation/fitting/worker-admission/publication pauses are not eliminated by this projection optimization.

Optimized 128 light runs retain [45, 44] active gaps over 100 ms, with [43, 42] attributed to sequential Sphere creation. Remaining UI/creation/fitting/worker-admission/publication pauses are not eliminated by this projection optimization.

Large native measurements cover independent intersecting sphere pairs and Union; arbitrary/all-overlapping scenes, other native families, physical RDP presentation, full persistence/source migration and manual UI sign-off remain open. The legacy selection_overlay compile failure and broader existing library-unit migration failures were not fixed or counted passing. No CAD business rule, selection geometry or GUI design changed.

## Commands and artifacts

Build: `cargo build -p om9-app --bin om9 --release --target-dir L:/om9-target -j 2`. Tests: `cargo test -p om9-scene --target-dir L:/om9-target -j 2 --test point_projector --test camera_grid --test camera_orbit --test camera_fit`; `cargo test -p om9-app --target-dir L:/om9-target -j 2 --test selection_projection --test boolean_publication --test viewport_detail --test navigation_deadlines --test display_modes`.

Reproduce with `python tools/benchmark_sphere_viewport.py --selection-comparison --pairs 32 --repeats 2 --present-mode auto-no-vsync --output <new-directory>`, substituting 64 pairs for 128 spheres. Python orchestrates Windows processes/log analysis; Rust owns commands, geometry, UI/rendering, assertions and timing witnesses.

- `build/standalone-validation/ui-artifacts/selection-frame-64-20261115-ab/`: 8/8 passed; raw traces, binary/source receipt and summary.
- `build/standalone-validation/ui-artifacts/selection-frame-128-20261115-ab/`: 8/8 passed; raw traces, binary/source receipt and summary.

[Machine-readable results, source/selection witnesses and receipts](2026-11-15-selection-frame-projection.json).
