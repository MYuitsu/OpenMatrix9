# STL scene high-water and per-document structural quotas — 2026-11-04

Detached STL capture admission previously used scene bytes fixed at capture. While a capture exists, the Controller now records a conservative high-water value at scene publication and when its scene usage is queried. The detached export reads this value before acquiring its existing shared-account reservation and rechecks it immediately afterward; an intervening increase reduces the export reservation before export work starts. When the last capture has gone, the next capture resets the high-water value to its current scene. The selected geometry and destination-preservation behavior remain unchanged.

A controlled 30,000-vertex authored-mesh test pastes another object while physical RAM is sampled. It verifies the scene grew by over 720,000 bytes, the export lease was reduced by at least that growth, cancellation leaves the existing destination unchanged, Undo does not prematurely lower the high-water value while a capture remains, and the next capture resets it after the old capture is dropped. This tests a selected interleaving, not all concurrency schedules.

Broader tests exposed two valid clipboard operations failing with `validation owner reservation`: the staged block document reported 12 nodes while the preliminary aggregate had 11. Nodes and points are per-document structural limits, whereas retained bytes are additive. Validation now supplies the full node/point limits for each document instead of subtracting other versions, without relaxing byte accounting. Existing separate limit-boundary tests still reject a single oversized document; nested-block and gem-in-block clipboard tests now pass.

## Validation

Default features, OCCT runtime, E: debug target, `-j 2`: 44 STL cancellation/admission tests pass including the growth/reset test, and six capture-filtered STL integration tests pass. Before the final reset-only change, 29 atomic-transaction tests, five job-lifecycle tests, 18 clipboard tests (three native exchange tests ignored), 20 file-lifecycle tests, three layer tests and 27 serial GPU/egui Gumball/viewport tests passed. The 27 GPU tests do not establish desktop presentation or performance. The reset-only change affects STL capture, not the viewport tests. The prior full 60-test STL integration run predates this follow-up; see the preceding [reservation report](2026-11-04-detached-stl-shared-reservation.md).

## Limits

This is not a registered live owner: auxiliary bytes remain a capture-time estimate; caches that grow without scene publication or a Controller usage query are not necessarily observed; scene can change after the second read. The high-water mark may overcount and reject work early, and the export still reserves all available managed headroom. Allocator overhead, staging, native OCCT allocations and other owners remain incomplete. These tests do not prove a hard 60% process limit, complete admission, desktop behavior, or background-export readiness.

## Native 3DM clipboard follow-up

The three clipboard integration tests previously ignored without a configured adapter were run with the native adapter `E:\om9-native-definition-release\om9-exchange-native.exe`, SHA-256 `0e159f1571d15e853c924e3ba2140480a816c77f1ab95845cc9cc7373b8d25ef`, the OCCT 8.0.1 runtime and E: debug Cargo target. `cargo test -p om9-app --test clipboard_om9 -j 2 -- --ignored --test-threads=1` passed all three in 8.12 seconds (2.62 seconds build): line archive copy/paste with one Undo, B-rep copy/paste at two nondefault document tolerances, and layer sidecar equivalence/conflict atomicity. This closes the three ignored cases for this adapter and these fixtures, not large 3DM files, global fidelity, native-memory bounds or desktop clipboard interaction.
