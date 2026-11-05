# Snapshot/Master and first-edit scene ownership — 2026-11-05

## Scope

Snapshot decoding uses `DocumentSnapshot::restore_with_admission` before chunk-table/JSON/hydration allocation stages. Snapshot and Master replacement build a candidate Controller under the current shared reservation, then use the same locked publication as New/Open. Candidate buffers receive a persistent split lease before publication. Observers move and the old STL session invalidates only inside the infallible replacement callback; a late admission refusal keeps the existing Controller. A first incremental edit from an empty scene now also splits a persistent buffer lease before commit instead of waiting for viewport preparation. This does not reserve the caller's earlier Master-document clone or all workspace/path allocator excess before it occurs.

The offscreen integration harness was stale after the production viewport API gained a Controller argument. Its fixtures now retain an actual Controller, render that Controller's current scene, and use the normal Gumball preview method. No production API was weakened to accept an unrelated scene. All original geometry/upload/publication assertions remain.

## Checks

Windows default features, OCCT 8.0.1/TBB/jemalloc runtime paths, `E:\om9-surface-snap-validation`, debug without incremental/debug info, Cargo `-j 2`, serial tests:

| Check | Result |
|---|---|
| App library `presentation_snapshot_tests` | 9 pass, including initial incremental scene and both restore paths retaining old worker snapshots |
| `session::incremental_scene_memory_tests` | 6 pass |
| `session::files::replacement_budget_tests` | 7 pass, including refusal after candidate construction retaining observers/history |
| `session::stl::cancellation_tests` | 44 pass |
| Archive `document_snapshot` | 11 pass |
| App `document_contract` | 12 pass, including Snapshot/Master pressure, selection, history and change notifications |
| App `file_lifecycle` | 20 pass |
| App `interactive_memory_limits` | 9 pass, 6 large fixtures ignored |
| App `gpu_navigation_publication` | 28 pass, no no-adapter/skip messages; 10.03 s test elapsed |
| App library `hover` filter | 13 pass; 8.81 s test elapsed |

The integration contract build took 22.22 s, file lifecycle 2.74 s, corrected RAM build 2.76 s, and final GPU harness build 3.24 s. The initial test command used the nonexistent target `interactive_memory` and ran no RAM tests; corrected target results are above. The first GPU invocation failed compilation on eight missing Controller arguments; only the corrected harness run is counted. The optional RTK bootstrap path was absent; direct RTK-proxied PowerShell scripts performed the checks.

Logs: `build/standalone-validation/ui-artifacts/r0-snapshot-master-20261105-012035`. Commands are in ignored validation scripts `build/standalone-validation/run-r0-20261105.ps1` and `run-r0-followup-20261105.ps1`; logs alone are not the durable record. R0 source fingerprint before subsequent RAM-accounting changes: `85b862542d9af345aeb4668e551f0af7bfe4fd356a5167cd55fa0cb2643fd4af`. SHA-256 input is concatenated UTF-8 repo path then raw file bytes, in order: app `session.rs`, `session/contract.rs`, `session/files.rs`, `session/project_state.rs`, `gpu_viewport.rs`; core `transaction.rs`; archive `snapshot.rs`; app tests `gpu_navigation_publication.rs`, `document_contract.rs`.

## Conclusion and limits

Public-source audit after the R0/R1 evidence batch: 15,172 source files, 594 icon bindings, 593 SVG files, no errors. Artifact pruning removed zero runs and preserved cited evidence. The ledger and three changed milestone JSON files parse; reviewed plan/report local links resolve.

R0 closes the selected scene-buffer admission/publication verification, including initial scene, ordinary candidate/history paths already documented, Snapshot/Master and first incremental edit. It does not close R1/R2. Scene metadata, lazy caches, partial overlap of unrelated buffer owners, stale auxiliary estimates, pending/live physical accounting, native peaks and a process RSS cap remain open. Pressure fixtures control the managed account, not actual OS exhaustion. No new release, physical-pointer, presentation-time or user-model benchmark was run; headless success does not prove desktop smoothness. No business command semantics or skill rules changed.
