# Boolean publication: worker-prepared source wires

The user authorized reducing UI receive/commit freezes while retaining exact CAD data, then repeating the 64/128-sphere Light/High comparison. Baseline measurements and fixture definitions: [many-sphere comparison](2026-11-15-many-sphere-light-high.md). This task targets the committed-body Boolean worker path, not a change to geometry tolerance or tessellation quality.

## Diagnosis

An opt-in `OM9_COMMIT_PROFILE` trace separates registration, transaction preparation, scene capture and wire preparation. On the eight-sphere debug pilot, the former High path spent 85.171 ms registering output, 6.368 ms preparing the transaction and approximately 370.870 ms preparing source wires inside scene preparation, for a 470.777 ms final UI poll. These are diagnostic debug samples, not release benchmark numbers. Logs: `build/standalone-validation/ui-artifacts/commit-profile-before-20261115/`.

The old final poll called `prepare_missing_scene_wires`, which recovered the BRep, computed source-derived face isocurves, sampled them and built render buffers on the UI thread. This explains the similar long stalls in Light and High despite very different triangle counts.

## Change and preserved guarantees

- The existing Boolean worker now calls the same `prepare_source_buffers` routine for committed single-body results. It retains the buffers and reservation with its output; buffers contain immutable display identity/fingerprint receipts, without retaining a canonical/source clone across exclusive source transfer.
- UI still performs source registration, representation validation, normal transaction preparation, scene admission and final coordinator commit. A fresh root can install worker buffers only when source stamp, exact representation identity, fingerprint, pinned display and unique root/buffer ownership match. Existing display/source pins remain intact. No repeat materialization/native wire generation is needed for that root.
- The private candidate scene receives complete buffers before the transaction is published. Other missing wires use the existing preparation path. Existing authority/epoch/publication/command/selection/layer/source guards, cancel behavior, budget checks, source-reservation transfer, one Undo entry and Redo preservation remain.
- Worker buffers stay charged during preparation and receipt lifetime. UI admits their CPU-cache owner before releasing their kernel reservation; the handoff is conservative and can refuse near quota. Failure drops buffers before releasing the last lease. No validation is disabled and no partial scene/document is published on preparation failure.
- Canonical BRep, transforms, Boolean mathematics/tolerance, profile tessellation requests and source-derived wire sampling policy are unchanged. Split/multiple-output and Boolean Two Objects preview paths retain their existing handling; this is not a claim to optimize every command or finish source/display migration.

The final eight-sphere debug pilot showed source-wire installation around 1.5 ms instead of 371 ms. Full final polls were 100.845 ms High and 29.948 ms Light; output registration remained the largest High component. Logs: `build/standalone-validation/ui-artifacts/commit-profile-final-20261115/`. An intermediate worker-buffer prototype rematerialized a BRep a second time on UI; it was replaced by the checked fresh-root receipt path. That prototype's logs are `build/standalone-validation/ui-artifacts/commit-profile-after-20261115/` and are not final release evidence.

## Correctness checks

The focused targets `boolean_publication`, `viewport_detail`, `display_modes`, `display_persistence` cover 16 tests. New fixtures verify two independent intersecting pairs in both profiles through menu/CMD dispatch, analytic Union volume/bounds/closedness, source-linked Surface Snap, prepared interior wires and atomic Undo/Redo. Difference and Intersection have independent analytic volume checks in both profiles. Stale selection, cancellation and managed-budget refusal preserve the document/scene/history; the pressure fixture retains pre-existing Redo. A buffer receipt from another source or stale root is refused without modifying the original scene.

The existing full library unit-test target has unrelated canonical-migration compilation failures; they are not counted as passes. Existing BRep v5 save/reopen limitations and broader source/display work remain open. This change does not certify native FFI allocations as memory safe or fully account all native internal peaks.

## Release reproduction

Build in the reused target with `rtk proxy cargo build -p om9-app --bin om9 --release --target-dir L:/om9-target -j 2`.

Optimized 64 spheres: `rtk proxy python tools/benchmark_sphere_viewport.py --pairs 32 --output build/standalone-validation/ui-artifacts/commit-worker-64-20261115-ab`.

Optimized 128 spheres: `rtk proxy python tools/benchmark_sphere_viewport.py --pairs 64 --repeats 1 --output build/standalone-validation/ui-artifacts/commit-worker-128-20261115-ab`.

Same-binary 64-sphere counterfactual: add `--commit-baseline --repeats 1` and use output `build/standalone-validation/ui-artifacts/commit-worker-baseline-64-20261115-ab`. The diagnostic environment flag only restores the former UI wire-preparation path; it is not a product preference. `--profile-commit` emits phase diagnostics to stderr; it is disabled for formal timing.

The fixture, synthetic pointer input, independent source oracle, render cap and active/idle gap definitions remain as in the linked baseline report. Encoded Perspective FPS is distinct from callback frequency and RDP client presentation. Camera speed/amplitude are matched but wall-time phase is not pose locked. Release batches run sequentially without concurrent builds. Six optimized 64 runs and two optimized 128 runs are planned; 128 has one run per mode. Historical baseline results are labeled separately from the fresh same-binary counterfactual.

## Release results

All **8/8 optimized release runs** passed (three per profile at 64 spheres, one per profile at 128). A further **2/2 same-binary 64-sphere counterfactual runs** passed. Each checked GPU configuration, zero dropped diagnostics, completion, analytic source measures and full-quality target restoration. Binary SHA-256 for all ten: `d7e824a1ecb7a59f7ea09dcb51eadd8079dbea63d89f53ae432619813553fcc1`. Raw paths are the three reproduction directories above. Durable data, actual geometry witnesses and per-run values are in [the JSON record](2026-11-15-boolean-publication-worker-buffers.json).

### 64 spheres: fresh same-binary comparison

The former path has one sample per profile; optimized values are medians of three unless labeled worst across runs.

| Metric | Former High | Optimized High | Former Light | Optimized Light |
|---|---:|---:|---:|---:|
| Final UI receive/validation/commit call | 2,126.757 ms | **56.631 ms** | 2,018.594 ms | **42.363 ms** |
| Call-duration reduction | — | **97.34%** | — | **97.90%** |
| Worst active callback gap across runs | 2,135.662 ms | 225.221 ms | 2,027.140 ms | 99.471 ms |
| Total Boolean request → completion | 52.419 s | 59.422 s | 22.478 s | 21.617 s |
| Encoded FPS, camera motion before Boolean | 10.637 | 10.603 | 10.632 | 10.608 |
| Encoded FPS, camera motion after Boolean | 10.740 | 10.698 | 10.572 | 10.593 |

Optimized UI-call samples were High 56.631/53.670/72.656 ms and Light 23.944/44.971/42.363 ms. Every optimized 64-sphere run had **zero active gaps above 500 ms or one second**. High retained one >100 ms interval per run around request setup (177–225 ms); Light retained none. Former-path runs still had a roughly two-second publication gap. The fresh counterfactual supports the causal improvement in the blocked UI call without requiring a comparison between different binaries.

### 128 spheres: historical reference and new single samples

Before values are the earlier default-path single runs, binary `19bd30c53cdd2a61f7ef914bb20916f8391f15a73006fcbd407e0faa62b72c4d`; the 128-sphere counterfactual was not rerun. These are one sample per mode, not repeated performance coverage.

| Metric | Before High | Optimized High | Before Light | Optimized Light |
|---|---:|---:|---:|---:|
| Final UI receive/validation/commit call | 5,114.914 ms | **157.735 ms** | 5,013.826 ms | **94.128 ms** |
| Call-duration reduction | — | **96.92%** | — | **98.12%** |
| Worst active callback gap | 5,141.795 ms | 659.448 ms | 5,040.332 ms | 439.802 ms |
| Total Boolean request → completion | 199.627 s | 207.139 s | 87.238 s | 80.951 s |
| Encoded FPS before Boolean | 30.004 | 10.591 | 30.000 | 10.594 |
| Encoded FPS after Boolean | 29.993 | 10.621 | 30.006 | 29.920 |

The remaining largest intervals end immediately after request setup: High request marker 29,746.616 ms, gap ending 30,401.428 ms (659.448 ms); Light marker 27,802.355 ms, gap ending 28,238.167 ms (439.802 ms). They are not the final receive/commit call. There were 71/48 active >100 ms gaps High/Light, mainly during foreground Sphere creation (62/44), plus one scene transition each and 8/3 during the Boolean phase. High had one >500 ms gap, Light none; neither had an active gap above one second. Request capture/admission and foreground creation remain optimization targets.

### Interpretation and precision

This optimization removes the multi-second source-wire calculation from final UI publication. It does not make the exact Boolean itself proportionally faster: the native/source work still runs, now with worker admission/communication, and total completion time varies. It also does not establish a steady FPS gain. The current same-binary former path also ran near 10.6 FPS; optimized 64-sphere result samples ranged 10.609–30.068 High and 10.590–10.682 Light. The 128 Light result reached 29.920 FPS in its single run. The earlier report's approximately 30 FPS and current cadence differ between runs/phases; the precise reason is unresolved. Do not infer a general threefold Light FPS advantage or blame the commit optimization from these observations. Present/RDP displayed FPS remain unmeasured.

Both profiles retained the original source measures: 64 spheres produced 32 closed solids, no naked edges, volume 28,274.33388276626 mm³ and bounds `[-55,-45,-5,60,45,5]`; 128 produced 64 closed solids, no naked edges, volume 56,548.667765580394 mm³ and bounds `[-75,-61,-5,80,61,5]`. Input/result triangle counts were unchanged from the former implementation: High 320,512/264,448 vs Light 33,024/28,032 at 64; High 641,024/528,896 vs Light 66,048/56,064 at 128. All four viewports restored full 649×410, four-sample targets after navigation. Scalar/topology checks and one source snap fixture are scoped evidence, not global geometric or picking equivalence.

Final checks: the final source passed all 16 focused integration tests, default debug/release builds, and the no-default-features library compile check. All ten release-run receipts matched the final binary and every recorded source fingerprint. Public source audit reported 16,067 source files, 594 icon bindings, 593 SVGs and zero errors. Scoped whitespace and ledger/three milestone JSON checks passed. Artifact pruning removed no runs and retained the newest ten plus cited evidence. README, live ledger, scoped M01/M03 traceability and source/installed Solid measurement guidance were synchronized under the existing Solid authorization. No unrelated changes were committed or published.
