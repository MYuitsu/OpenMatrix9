# Parallel viewport Boolean and source-only Union

Latest native 512 comparison: Light passed; High published a preview but its exact worker failed before commit, retaining the original 512 spheres/Undo. Both attempts were executed. See [final attempt results](2026-11-15-speculative-boolean-512-comparison.md). Earlier pending statements below are historical checkpoints.

Status: implementation verified by 16 focused integration checks and four native-window runs at 64/128 spheres. Two native-window 512 profiles remain pending because desktop access failed.

`BooleanUnion` now runs a disposable approximate mesh preview concurrently with the exact canonical-source fold. Rust controls source ownership, ordering, cancellation and publication. Selected OCCT adapters return intermediate canonical source without tessellating placements or Union intermediates; the worker captures the requested final display/topology profile once and prepares source wires/render buffers before publication. The existing certified Fuse, validity checks and CAD tolerance remain unchanged. Native OCCT allocations are outside managed Rust callbacks, as in the existing snapshot adapter; the FFI is not certified memory-safe.

The mesh worker shares immutable render buffers, welds and simplifies to a 128-triangle target per input, combines overlapping components with bounded CSG, and prepares GPU-only transient buffers. It has a 64 MiB reservation, 1024-input/250000-output-triangle ceilings and bounded component work. A second, smaller classifier tolerance can resolve coarse-facet slivers; both tolerances apply only to the approximate display, with the same closedness checks. Unsupported/pathological previews can be refused visibly while exact work continues. These are not CAD geometry or Undo entries. Exact operations, persistence and authoritative measurements keep the original sources. The sphere preview diagnostic volume is about 89.8–89.9 percent of the analytic result: it is visibly provisional and must not be used as precise geometry.

The GPU cache keys include temporary identity, replace the displayed operands and keep accounting with each buffer clone. Hit testing/geometry editing against the provisional image is gated. Primary editing gestures defer exact publication; orbit/pan no longer defer it. Exact commit rechecks source/context/selection/publication guards and publishes source plus scene as one Undo step. Cancellation/staleness/failure preserves the document. Difference/Intersection/Split and Boolean2Objects mesh-preview coverage are not claimed.

## Native-window results

One run per size/profile. Layout: independent intersecting pairs of radius-5 mm spheres, followed by BooleanUnion with DeleteInput=Yes. Timings start at the Boolean request. FPS is distinct encoded Perspective frames, measured separately while waiting and after exact commit. GPU-frame gaps and UI callback gaps are distinct measurements. The exact time below is document/scene commit, not client-video delivery.

| Spheres | Detail | GPU preview s | Exact commit s | FPS waiting / after | UI Boolean max ms | GPU Boolean max ms | Resident peak / settled MiB | Undo |
|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 64 | light | 0.477 | 17.409 | 28.96 / 29.99 | 97.2 | 152 | 315.3 / 306.3 | 65 |
| 64 | high | 0.592 | 16.615 | 26.34 / 29.99 | 161.2 | 192 | 370.3 / 323.8 | 65 |
| 128 | light | 0.779 | 64.508 | 29.70 / 30.01 | 88.5 | 281 | 351.1 / 331.2 | 129 |
| 128 | high | 1.170 | 62.163 | 28.35 / 30.00 | 150.1 | 427 | 463.8 / 365.0 | 129 |

The source hashes are identical between Light and High, and match the earlier size-specific sources. 64: `5de866c125bf898089a794deb9ba8bfbbfb7664a7f9dcd3305d9a869b325c1bc`; 128: `6c58d8e0caf631b99761ae55ef3f827cdc010e8dde42b99210967d8bd3d65582`. Native analytic volume/bounds, closedness, solid counts and zero naked edges passed. Display densities remain 33024/320512 input and 28032/264448 result triangles at 64, and 66048/641024 input and 56064/528896 result triangles at 128 (Light/High). Undo is enabled, with 65 or 129 steps and Redo 0.

The four valid traces were rechecked for actual new encoded frames, nonzero visible geometry/draw calls, the expected input count, one preview/result object, and coverage near both edges of the FPS windows. They do not merely encode an empty viewport. First/last interior coverage lags are at most 44 ms. Remote-client video FPS remains unmeasured.

The rapid creation replay has a separate remaining limitation: it starts another Sphere before asynchronous GPU scene publication catches up. Whole-scenario largest encoded gaps are 2.840/4.160 s at 64 and 11.277/14.089 s at 128 (Light/High), even though largest UI callback gaps are 97.2/161.2 ms and 215.7/238.1 ms. These holds occur across batch creation. They are not a continuous main-thread Boolean stall, and this change does not fix that creation/publication scheduling.

## Invalid 512 desktop attempt

The throwaway Light attempt created 512 spheres and retained 512 Undo with the prior process-only object cap 200000. The test window became minimized; raw audit ticks continued while actual GPU/UI rendering did not. Computer Use confirmed the minimized window, and restoring it failed with GetCursorPos access denied (0x80070005); screenshot capture had also failed with monitor error 0x80070057. The owned test process was stopped after verifying its executable and audit-log environment. No output, first-preview timing, FPS, Boolean freeze, settled result RAM or successful 512 comparison is claimed. High was not launched in this invalid batch. The original trace is preserved.

The two 512 native profiles require an unlocked accessible desktop and a visible OM9 test window. The planned rerun keeps the same binary, normal Sphere/Union commands, 513 Undo expectation, loaded Undo policy and test-only object cap; production object/point/byte limits and CAD tolerance are unchanged. A kernel-only 512 source/final-display check passed separately; it cannot substitute for desktop/FPS/Undo/RAM evidence.

## Validation and provenance

Focused release integration batch: boolean_publication 5, boolean_speculative 3, viewport_detail 6 and navigation_deadlines 2, all passed. This covers native geometry, source-only operation under a tight display point budget, provisional mesh closedness/approximate volume, source immutability/serialization, cancellation/stale selection, pressure, snap and atomic Undo/Redo. The separate cloned-render-buffer geometry helper passed eight fixtures (4/64/128/512 at both profiles); this is not a desktop benchmark.

Earlier validation attempts are retained: initial cargo check had one new error conversion fixed; the app lib-test target failed compilation in existing canonical-BRep migration fixtures (318 diagnostics, no tests run); the first integration batch had 7 pass/1 preview failure, which led to the bounded classifier retry. Diagnostic 512 High initially exceeded the temporary output ceiling, corrected within the preview reservation before formal measurements. Full lib/workspace test acceptance is not claimed.

Final focused release build/test graph: 10m48s; default-feature product release build: 6m21s, both -j 2 in L:/om9-target. No compile/test workload overlapped the four valid native runs. Product executable SHA-256: `d7c750e73e2ed51fc30eacd6348698f136827e8ec03c462b430cd3989f2a80c4`. All 27 frozen runtime/analyzer source hashes and executable still match. Source baseline copies and scoped diff preserve preexisting dirty changes; nothing is staged or committed.

Artifacts: `build/standalone-validation/ui-artifacts/boolean-speculative-20261115/`, including native-64, native-128, invalid native-512, source receipts, GPU geometry coverage, diagnostic helper and test/build logs. Machine-readable data: [speculative Boolean results](2026-11-15-speculative-boolean.json).

Remaining: the two visible native 512 runs; client RDP presentation, screenshot/manual acceptance, repeated statistics, all-overlap/general geometry, all-Undo replay and full persistence/migration are unproved. Next restore desktop access and run only the two missing native profiles on the frozen binary before claiming the complete matrix.

## Completed 512 kernel proof

One authored 512-placement fixture shares a canonical radius-5 Sphere source and applies the same ordered lattice placements as the native test. The source-only fold completed in 710.256 s. Final Light capture/measurement took 5.552 s (135423 points, 224254 triangles); High took 12.049 s (1120000 points, 2115584 triangles), both below the unchanged 2000000-point cap. Both source hashes match the old completed 512 Light source: `ae813c1a93bb232bccb9bb0057249423c6c70b74edd55d8d344b92bf90e80f19`. Volume 226194.67106235237 mm3, bounds [-155,-125,-5,160,125,5], 256 closed solids and zero naked edges passed. This closes the final-only kernel point-budget check for this fixture, including High; native-window 512 acceptance remains pending. Shared source/no retained command history makes this a geometry/point-cap proof, not a production RAM, Undo, GUI freeze, first-visible or FPS benchmark.

Final static checks: public-source audit passed (16089 source files, 594 icon bindings, 593 SVGs, zero errors); new-file rustfmt and scoped whitespace checks passed. The installed/source Solid references match. Pruning removed zero runs, retained the newest ten per root and 204 older cited runs. All frozen runtime/analyzer hashes and product executable still match. No test/build process remains active; the two native 512 runs await desktop access.

## Desktop retry awaiting user restart

The same frozen executable/source was retried in `native-512-visible` after the user restored RDP. The Light creation replay continued, but target captures remained black and activation still returned `GetCursorPos: Access denied (0x80070005)` after reinitializing the Computer Use connection. No fresh GPU frames were available for acceptance. The user chose to close/restart; the verified benchmark controller was stopped before it could launch High, leaving the OM9 window for the user. Its wrapper exited 15 after 664.622 s. This is a second invalid desktop attempt, not an accepted 512 FPS, preview, freeze or RAM result. Both 512 native profiles remain pending; the four accepted 64/128 runs and separate 512 kernel proof are unchanged. Receipt: `build/standalone-validation/ui-artifacts/boolean-speculative-20261115/native-512-visible-invalid-environment.json`.

## Audit recovery and rejected camera pilot

The interrupted pointer run continued briefly after its controller was stopped. Native UI callbacks still ran in about 5–8 ms while camera motion stopped after real pointer/focus input; absence of a new image did not establish a whole-UI deadlock. The direct-camera fallback was rejected too: per-raw-frame revisions encoded geometry but did not publish stable preview content. Neither pilot supplies accepted 512 FPS or first-visible results.

Only the opt-in Rust audit and Python analyzer changed: re-arm the synthetic orbit through normal input after gesture cancellation; require fresh frame IDs, rendered geometry and coverage at both edges of every FPS window. The release rebuilt successfully in 5m45s with -j 2, reusing L:/om9-target. All four prior 64/128 traces pass the stricter analyzer. Two 16-sphere native smoke profiles deliberately inject focus loss and verify resumed dragging/fresh encoded frames, visible preview before exact publication, geometry and 17 Undo; both pass, post-commit FPS 30.0, whole-active maximum callback gaps 37.607/61.481 ms (Light/High). Native 512 Light/High now run sequentially in `native-512-recovered`; acceptance remains pending. Boolean/kernel/source/renderer code remains unchanged relative to the prior frozen build.

## Completed native 512 Light, High running

Recovered-pointer Light completed with preview CPU-ready 2.911 s and published-content observation 3.830 s; exact commit 783.798 s, UI final poll 1511.571 ms, Boolean callback max 1658.170 ms, whole-active callback max 7514.473 ms (creation included). Resident peak/settled at performance_end 672.898/591.199 MiB, Undo 513/Redo 0. Analytic volume/bounds, 256 closed solids, zero naked edges and source SHA `ae813c1a93bb232bccb9bb0057249423c6c70b74edd55d8d344b92bf90e80f19` match the independent kernel proof and previous completed Light fixture. Input/final/preview triangles: 264192/224254/183296.

The initial strict analyzer rejected a 1411 ms trailing hold in the pending window, not geometry or a missing preview. FPS now counts fresh frames over the full fixed window, including first/last holds, rather than removing that time between the first and last available frame. Edge holds also enter the encoded-gap diagnostics. Light full-window pre/wait/result FPS is 26.304/29.323/29.930; frame-range-only waiting FPS was 29.398. All four earlier 64/128 traces were reanalyzed with the same denominator, retained separately as fixed-window-summary.json. High now runs in native-512-high-recovered on the identical runtime executable; only analyzer code changed after Light. The complete two-profile native 512 matrix remains pending High.
