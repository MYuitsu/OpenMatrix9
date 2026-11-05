# Snapshot metadata and Windows system commit — 2026-11-05

## Implementation

Viewport snapshot admission now estimates only the metadata that will actually be copied. The source scene's lazy spheres/BVH and unused vector capacity are not inherited by the fresh snapshot. The estimate includes the enclosing tracked owner and conservative Arc control storage. After copying/filtering objects, actual metadata capacity is reconciled with the shared reservation; excess is admitted before publication, unused allowance is returned, and metadata is marked allocated after the owner exists. Shared geometry keeps its existing buffer lease. This does not reserve unknown allocator excess before it occurs.

Background hover similarly separates snapshot metadata from future picking scratch. `HoverSnapshot` owns the scene, tracked document, shared buffer owner and reservation in that drop order. Metadata materializes after construction; picking scratch stays pending until the worker finishes. Normal completion and unwind destroy the snapshot before releasing its reservation. A completed worker sends only its object-ID result after dropping these owners. Current-scene lazy cache ownership is not implemented by this bundle.

The platform adapter now reads system commit through `K32GetPerformanceInfo`, converts page counts with checked multiplication and rejects invalid/overflowed values. CommitTotal changes on commitment even before pages are touched; it is distinct from physical Available and process working set. [Microsoft PERFORMANCE_INFORMATION documentation](https://learn.microsoft.com/en-us/windows/win32/api/psapi/ns-psapi-performance_information). The selected Kernel32 entrypoint and structure/size requirements are documented by [Microsoft GetPerformanceInfo](https://learn.microsoft.com/en-us/windows/win32/api/psapi/nf-psapi-getperformanceinfo).

Rust owns `CommitMemory` and policy. The necessary platform exception is one short Win32 FFI call in the existing `om9-native-owner` adapter using pinned `windows-sys` 0.61.2, with initialized storage and no retained pointer. No C++ adapter, dependency version or geometric algorithm changed.

Windows Controller initialization installs the commit probe on the shared reservation registry, so adopted/detached peers use it too. Admission/growth, remaining-headroom sizing and recheck apply the commit gate under the same lock as pending/allocated transitions. Pending allocations must fit `CommitLimit - CommitTotal - 256 MiB`; the 256 MiB reuses the project's minimum safety margin and is not a document/export cap or a percentage of pagefile size. Existing allocations remain in managed usage and are not deducted again as pending commit. A configured but failed probe returns unsupported; an absent portable probe remains explicitly unverified through `commit_sampled()`. No pagefile settings are changed and no future pagefile extension is assumed.

The 60%-of-physical-RAM managed ceiling and Undo policy are unchanged. A valid physical sample tightens that ceiling even when the independent commit gate refuses the request. This prevents a failed commit admission from allowing a later unsampled request to bypass a learned physical ceiling.

## Verification

Named batch: **R1 snapshot metadata / system commit integration**. Windows default features, existing OCCT 8.0.1/TBB/jemalloc runtime, external `E:\om9-surface-snap-validation`, debug without incremental/debug info, Cargo `-j 2`; tests serial except deliberately concurrent account fixtures. No desktop OM9 window was opened or closed.

| Check | Result |
|---|---|
| Native system-memory adapter | 5 pass: actual Windows physical/process/commit probes and checked commit-page conversion |
| Core memory budget | 4 pass |
| Core retained/account lifecycle | 101 pass, including commit versus physical pressure, simultaneous commit requests, adopted peers, grow refusal, recheck, missing/invalid probe and retained physical ceiling after refusal |
| App presentation snapshot ownership | 10 pass |
| App hover filter | 14 pass |
| App scene pressure / replacement budget / STL lifecycle | 7 / 7 / 44 pass |
| egui selection/hover overlay | 14 pass, 1 opt-in release-responsiveness test ignored |
| B-rep picking precision | 19 pass |
| Gumball interaction / viewport | 7 / 7 pass |
| Offscreen GPU publication | 28 pass, no no-adapter/skip messages |
| Document contract / file lifecycle | 12 / 20 pass |
| Ordinary interactive RAM | 9 pass; its normal invocation ignores 6 large tests |
| Isolated real-resource fixture | 1 pass: actual 270 MiB resource (283,115,520 payload bytes), MatrixApp startup and retained usage above 256 MiB; 5 other large tests remain unexecuted |
| Full app STL integration | 60 pass, no skips; 109.00 s test elapsed |

The integration matrix and 270 MiB fixture passed before the final physical-ceiling retention adjustment, which only changes failed-admission handling. After that adjustment, the five native, four budget, 101 core/account, ten presentation, fourteen hover, seven scene-pressure, seven replacement and 44 STL-lifecycle checks were rerun and passed. No STL geometry/encoder code changed. Final app library build took 48.15 s; final core library build/test 14.86/5.74 s. The isolated 270 MiB fixture took 5.18 s. These timings describe tests/builds, not navigation performance or measured RSS peaks.

The new Controller fixture prepares a real mesh insertion, injects zero commit headroom while physical RAM is ample, proves scene construction was not entered, and checks document/scene/Redo/charges stay unchanged. An admitted retry publishes and clears Redo normally. Metadata fixtures populate the source's picking caches, confirm they are absent from the worker snapshot charge, check metadata is allocated while hover scratch remains pending, and retain snapshot owners across New until final drop.

Fixture corrections were necessary: a New test initially left an unsaved document awaiting confirmation, so it now starts from a clean baseline and asserts New really happened. Old hover oracles expected zero CPU-cache bytes despite persistent live scene buffers; they now compare the captured live-buffer baseline. Pressure fixtures are bounded to tens of MiB rather than assuming the whole machine's managed RAM ceiling fits current commit. An attempted use of the private app constructor in an integration fixture failed compilation; the fixture now uses existing public admission APIs and the constructor remains private. No failed or zero-test run is counted as a pass.

Final unit logs: `build/standalone-validation/ui-artifacts/commit-metadata-20261105-020505`. Integration/STL/real-resource logs: `build/standalone-validation/ui-artifacts/commit-integration-20261105-020117`. Earlier metadata passes and fixture failure logs: `build/standalone-validation/ui-artifacts/snapshot-metadata-20261105-014826`, `build/standalone-validation/ui-artifacts/snapshot-metadata-20261105-014726`, `build/standalone-validation/ui-artifacts/commit-metadata-20261105-015151`, `build/standalone-validation/ui-artifacts/commit-integration-20261105-015910`.

Final source fingerprint: `d5e07ea7c07adf1d0c5fd74c4152466d2c2df017113078cdc713c84b0f3bfc99`. SHA-256 input is concatenated UTF-8 repo path then raw file bytes, in order: core `src/budget/memory.rs`, `src/transaction.rs`; native-owner `src/system_memory.rs`, `src/lib.rs`; scene `src/scene.rs`; app `src/session.rs`, `src/gpu_viewport.rs`, `src/ui.rs`, `tests/selection_overlay.rs`.

## Limits and next work

Public-source audit passed with 15,173 source files, 594 icon bindings, 593 SVG files and no errors. Artifact pruning removed zero runs and retained cited evidence. The ledger and three changed milestone JSON files parse, the new ledger ID is unique, and reviewed plan/report local links resolve. Native checks used the pinned Windows bindings and actual Kernel32 entrypoint; existing OCCT runtime loading was exercised by the integration corpus.

This closes selected viewport/hover snapshot metadata phase/lifetime checks and the Windows interactive-account commit-admission gate. R1 remains open for persistent metadata of the main scene, direct lazy spheres/BVH ownership, query allocations, other worker/staging/native owners, stale high-water/auxiliary estimates, reclamation and full diagnostics. Picking scratch is still an estimate and some cache builders still allocate infallibly; this batch does not establish a process RSS/native-memory upper bound or protection from real OS OOM. Commit and physical probes are point-in-time observations, not Windows reservations. Other processes and driver/native allocations can still change pressure.

No current-source release navigation benchmark, WPR presentation or physical-pointer test was run. The single >256 MiB resource test is not a complex CAD model, mesh, file roundtrip or measured peak-RAM proof. Remaining future batches across the full plan are not fully enumerated, so their total is unknown. No new 3DM/openNURBS fidelity measurement package is closed by these admission regressions. P1/v5, GPU batching, complete STL/3DM accuracy gates, UI background export, staged Save/STL budgets and checkpoint remain open. Next: give current-scene metadata and lazy picking caches admission and last-owner leases, then remove whole-headroom reservations from Save/STL. No reusable business-rule skill text changed.
