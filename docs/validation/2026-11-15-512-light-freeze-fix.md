# 512-sphere Light: retained-owner admission and UI freeze

Status: scoped native 512-sphere Light regression completed; measured results and limits below. The user requested fixing the previously measured 133.725-second Light callback gap while preserving exact CAD data and Undo.

## Baseline and scope

The prior native-window Light run created 512 radius-5 spheres in 256 independent intersecting pairs and committed one Union. At the transition, `boolean_worker_complete` measured 70392.950 ms in the successful receive/commit poll; the subsequent pane work measured about 63323.190 ms. Their enclosing callback gap was 133725.186 ms. GPU execution was only a few milliseconds. A later result-navigation callback gap was 32019.046 ms. Source bounds/volume, 256 closed solids and zero naked edges passed, with 513 retained Undo / 0 Redo. See [the unchanged baseline report](2026-11-15-512-spheres-memory-undo.md).

The baseline and new 512 regression use the explicit process-only object cap 200000. Default product cap 100000 remains unchanged and previously refused Sphere 447 due to cumulative retained document object slots. Point/RAM/Undo limits and exact CAD tolerances are not raised by this fix. The independent High snapshot-point refusal is outside this requested Light UI freeze fix.

## Rust changes

`TransactionCoordinator::validate_internal_retained_owners` used a duplicate-inclusive document bound to admit its temporary deduplication index. The bound charged before/after references to the same immutable document as additional logical objects. Crossing that bound selected the allocation-free repeated-scan fallback even when the exact retained objects fit. The allocation preflight now retains its full byte bound and per-document validation; logical object counts are checked with the original limits in the deduplicated inventory. Actual object/RAM refusals remain required.

Existence checks for external source/topology/display leases now search the same retained document set in reverse order. After a many-input edit, the newest Undo before-snapshot normally holds the old input set. This changes lookup order only, without a new cache, weakened allocation identities, altered accounting order or persistent state.

The existing opt-in `OM9_COMMIT_PROFILE` diagnostic now separates external versus internal retained-owner admission times for calls over 50 ms. No native adapter, source geometry, display density, history retention policy or CAD command behavior was changed.

## Initial measurements

An authored canonical-record timing probe creates 512 actual Undo versions without claiming OCCT geometry validation. It retains 131328 object slots and 195630618 managed bytes. On the old core, ordinary inventory measured 56.843 ms and inventory with 200000 object slots available measured 627.185 ms. Its setup took 114.545 s. The first index-only edit measured 60.087/314.388 ms; this was an intermediate diagnostic, not the final native UI outcome.

The initial object/byte guard and Undo/Redo regression passed before editing production code. The first index-only validation passed that semantic test, three allocator-peak tests and eight Undo-memory tests, plus the separate manual timing probe. The final implementation passed 13 core checks (2 new ownership/guard regressions, 3 allocator-peak tests and 8 Undo-memory tests). Twelve app integration checks passed in release: 5 Boolean publication, 6 viewport detail and 1 selection projection. They cover exact source measures, source-refined snap, source wires, stale/cancel/pressure refusal and atomic Undo/Redo in both profiles. The manual timing probe is ignored in the final semantic suite and is not counted as a passing acceptance test.

The first app release test launch could not run because the runtime DLL directory was absent from PATH (`STATUS_DLL_NOT_FOUND`). With `L:/om9-target/debug;L:/om9-target/release` prepended, all twelve app checks executed and passed without recompilation. Eighteen actual runtime DLL fingerprints and the focused results are recorded in `focused-validation.json`. Initial new-fixture capacity/history-loop mistakes were corrected before the final passing checks; they are not native product failures or additional passing tests. The product-default executable build and native-window confirmation completed; see the measured results below.

The product-default release executable built successfully. Native-window 64-sphere Light/High runs both passed using a stricter test object cap of 3000: final UI polls 21.662/51.420 ms, largest active gaps 68.393/111.132 ms, 65 retained Undo / 0 Redo each, and identical native source SHA `5de866c125bf898089a794deb9ba8bfbbfb7664a7f9dcd3305d9a869b325c1bc`. Encoded navigation FPS stayed about 30 before and after. These exercise the duplicate-inclusive object-bound threshold with exact owners still admitted; they are functional regressions, not a matched performance comparison against the earlier default-cap 64-sphere runs.

The 512-sphere Light run completed with the same 200000-object test cap as its baseline. Its geometry, display, resource and zero-drop checks passed.

Artifacts: `build/standalone-validation/ui-artifacts/512-light-freeze-20261115/` contains the baseline source receipt, baseline core timing, index-only timing, scoped source diff and focused results. Its `native-64-cliff/` contains the two passing native runs; `native-512-light/` contains the completed primary regression.

## Completed native results

Frozen product release SHA-256 `8fdee846b5407f7f3814e00de4ef5aeba4b3187ba7b7f5da0eac5e71c830932e`; all 20 runtime/analyzer source hashes matched after measurement. Twenty-five focused checks and three native-window runs passed: two 64-sphere profiles and one 512-sphere Light. No compiler/test workload overlapped formal native timing. The 512 run retains the same test-only object cap 200000 as its baseline, with unchanged byte/point limits and loaded Undo policy. Diagnostic commit profiling was enabled and is recorded in the receipt. These are individual samples, not repeated statistical claims.

| Measurement, 512 Light | Before | After |
|---|---:|---:|
| Largest active UI callback gap, ms | 133725.186 | 6703.171 |
| Largest Boolean-phase callback gap, ms | 133725.186 | 1651.442 |
| Final successful UI receive/commit poll, ms | 70392.950 | 1514.707 |
| First post-commit frame pane work, ms | 63323.240 | 128.679 |
| Sphere creation commands, s | 1565.633 | 841.493 |
| Total Boolean wait, s | 1580.896 | 1312.139 |

The largest whole-scenario callback gap fell by 94.99 percent. Command/creation gaps remain included. Background Boolean wall wait is distinct from a continuous UI freeze; source/tessellation algorithms were not replaced. The pane measurement covers the first frame after the successful worker poll, matching the baseline event class.

Native source SHA remains `ae813c1a93bb232bccb9bb0057249423c6c70b74edd55d8d344b92bf90e80f19`; bounds, volume, 256 closed solids, zero naked edges and per-profile input/output display triangles match. Undo is still **513** (512 Sphere + 1 Union), Redo **0**, with policy `{'max_memory_mb': 1024, 'minimum_steps': 1}` and Undo enabled. Canonical data was preserved; no source was simplified, history cleared or point/RAM guard raised. The product default object cap remains unchanged and its 512 acceptance is not claimed.

scene steady distinct encoded Perspective FPS: 30.000, encoded p95 35 ms, encoded intervals >100 ms 0.

result steady distinct encoded Perspective FPS: 30.004, encoded p95 37 ms, encoded intervals >100 ms 0.

Windows process resident before Boolean / settled / timed peak: **461.6 / 575.7 / 599.0 MiB**; settled private commit 1078.7 MiB. MiB=1048576 bytes; resident/private commit differ and VRAM is not measured. The endpoint precedes native oracles and the full managed-history inventory. Full resource rows remain in JSON.

The authored core fixture is ownership/accounting evidence, not native geometry acceptance. Native source, scene, source-refined snap and Undo/Redo are covered by the app checks; actual native surface/frame timing is covered by the three window runs. Client-presented RDP video FPS, physical input acceptance, all retained Undo steps replayed, general geometry/all-overlap scenes and full migration/persistence remain unproved. The independent High 512 snapshot-point refusal was not fixed or rerun in this Light task.

Next inspect the remaining worst-gap phase and its `OM9_COMMIT_PROFILE` internal/external split before adding another optimization. Keep the exact source/point/byte/history guards. Raw phases and individual profile records are retained in the [machine-readable report](2026-11-15-512-light-freeze-fix.json).

## Final verification

Public-source audit passed: 16084 source files, 594 icon bindings, 593 SVGs, zero errors. Scoped Git whitespace checks and the new test's rustfmt check passed. All 20 frozen runtime/analyzer hashes and 18 runtime DLL hashes matched; the only changed runtime receipt paths relative to the baseline were the Rust transaction owner and benchmark receipt metadata. The inverse scoped patch reproduces the original transaction source hash, preserving unrelated existing changes. Repository and installed Solid technical references match; three milestone owner records, README and the live ledger carry the verified scope.

The remaining 6703.171 ms worst gap was measured at audit stage 1 (foreground Sphere creation). The corresponding Boolean-phase maximum is 1651.442 ms. These pauses are not hidden by steady FPS reporting. Artifact pruning was applied: zero runs removed, newest ten per root and 203 older cited runs retained.
