# Pending versus allocated RAM — production account, 2026-11-05

## Change

Both the portable `MemoryBudget` helper and production `Coordinator` reservations now retain separate pending byte counts. All reserved bytes, including allocated bytes, stay in the managed 60%-of-physical-RAM ceiling. Only allocations still to be made consume the physical admission headroom in a fresh OS Available sample. An absent sample does not mean allocated: an unsampled request remains pending until explicitly materialized or transferred into a registered candidate. The helper and production account use the same physical headroom calculation.

`ExternalMemoryReservation::mark_allocated` changes phase under the account lock without changing the managed total. Split/grow/shrink/drop preserve both phase totals; replacement publication and transaction-clone handoff remove the transferred pending allowance under the publication/account locks, with rollback retaining it on refusal. Retired Controller metadata is already allocated rather than a future request. Account adoption keeps the same registry and its phases across replacement.

The Controller registers existing render buffers as allocated, reserving only their new lease-node overhead as pending. Full/incremental/history/file/Snapshot/Master scene-buffer leases materialize after their buffers and Arc owner exist. Persistent buffer ownership and last-owner release remain unchanged. No geometry, tolerance, Undo settings or file-format change is introduced.

## Stale samples

The production account stores the short platform sampler installed by `Controller`. For sampled reserve/grow/remaining/recheck, it refreshes the sample while holding the same lock as phase transitions. A caller's stricter sample is retained, but its stale higher headroom cannot replace the fresh lower sample. The sampler follows the shared registry to detached peers. If a configured sampler cannot refresh a supplied sample, admission reports unsupported rather than trusting stale headroom. The sampler must not reenter the account. Portable/core fixtures without a platform sampler still use explicitly supplied point-in-time samples; these do not certify OS freshness.

This prevents a known race in a naive phase implementation: request B samples before request A allocates, A then materializes and removes its pending charge, and B attempts to spend its old Available value. It does not reserve physical RAM from Windows or stop other processes changing memory immediately after a probe. Allocation is not a residency certificate; paging, native allocation estimates and commit headroom remain separate requirements.

## Verification

Default-feature Windows build, existing OCCT 8.0.1/TBB/jemalloc runtime, external E: debug target without incremental/debug info, Cargo `-j 2`, serial:

| Check | Final result |
|---|---|
| Core `memory_budget` | 4 pass |
| Core `transaction::retained_validation_tests` | 98 pass |
| App presentation owners / scene pressure / replacement budget | 9 / 6 / 7 pass |
| App STL cancellation/admission | 44 pass |
| Archive snapshots | 11 pass |
| App document contract / file lifecycle | 12 / 20 pass |
| App interactive RAM | 9 pass, 6 large fixtures ignored |
| App offscreen GPU publication | 28 pass, no no-adapter/skip messages |
| App hover filter | 13 pass |

Controlled account fixtures test partial materialization without lowering managed usage, invalid phase changes, managed-ceiling refusal despite high Available, split/grow/shrink/drop, registration of known existing allocation, account adoption, remaining headroom, stale caller samples, sampler failure and recheck after a tighter sample. Existing concurrent tests retain the pending-reservation exclusion. Byte amounts in these account fixtures are simulated allowances, not actual allocations of that size or real OS exhaustion.

The first application run caught a missing phase update in the transaction-clone-to-candidate handoff: the published first-point scene had 24,506 stale pending bytes. Both success and rollback paths were corrected; the strengthened production-path test now requires zero pending CPU-cache bytes after publication, while its full buffer charge remains. The clone cancellation/success regression also checks zero leftover pending bytes. The former capture-metadata physical oracle was updated because registry metadata already exists before sampling; it remains counted in managed usage and is no longer treated as a future allocation. Assertions were not loosened to accept stale phases.

Final core logs and GPU/hover logs: `build/standalone-validation/ui-artifacts/r0-snapshot-master-20261105-013249`. Final Controller/archive/file logs after the sample-refresh change: `build/standalone-validation/ui-artifacts/r0-snapshot-master-20261105-013839`. The failed phase run is retained at `build/standalone-validation/ui-artifacts/r0-snapshot-master-20261105-013106`. GPU build/test elapsed 44.48/10.05 s; app library hover build/test 25.11/8.89 s; final app contracts 2.62/0.08 s, file lifecycle 2.71/1.57 s. These are build/test durations, not product performance benchmarks.

Source fingerprint: `2b03822fdd474043299a93bd530ab3b3868ea8ff9e8b78d0f98c11efd45fd0b8`. SHA-256 input is concatenated UTF-8 repo path then raw file bytes, in order: core `src/budget/memory.rs`, `src/transaction.rs`, `tests/memory_budget.rs`; app `src/session.rs`, `src/session/files.rs`, `src/gpu_viewport.rs`.

## Still open

Public-source audit passed: 15,172 source files, 594 icon bindings, 593 SVGs, zero errors. Artifact pruning removed zero runs and retained cited evidence. Four changed JSON files and local links in the two plans/two implementation reports passed validation.

R1 is incomplete. Snapshot metadata, lazy spheres/BVH and query scratch have not all been materialized/registered at their actual allocation boundaries; their reservations can remain conservative. Full source/metadata ownership, high-water/auxiliary estimates, allocation excess, cache reclamation and diagnostics remain to audit. The Windows system commit-headroom gate is not implemented. Save and STL still reserve all admitted remaining headroom (R2); export has not been connected to the UI worker. No process RSS cap, native OCCT upper bound, current-source release comparison, actual desktop presentation or user-model latency result is claimed. Six large RAM tests remain unexecuted. Checkpoint, v5, complete GPU batching and the other plan gates remain open. No reusable business-rule skill text changed.
