# Initial scene and file-replacement buffer registration — 2026-11-05

## Behavior and correction

The previously interrupted changes register nonempty buffers before `Controller::new` returns. Private `new_with_held` candidates remain under the caller's construction allowance until they join the active account. New/Open split the admitted candidate reservation into a persistent buffer lease and the remainder used for atomic Controller replacement. `ExternalMemoryReservation::split_off` leaves the shared account total unchanged; dropping either portion releases only that portion. Empty scenes do not acquire a startup buffer lease.

The resumed audit corrected two test assumptions that buffer accounting begins at the first viewport/hover request. They now compare with the already registered startup baseline and independently check snapshot metadata increments. It also found an actual STL admission double charge: the detached capture's scene high-water value still included buffers already registered in the account. Capture, usage queries and incremental publication now track the unregistered scene portion, retaining buffer accounting in the shared account. The original competing-reservation STL assertion passes without being weakened.

The requested `PLAN-REVISED.md` path was restored as a link to the single canonical PLAN.md, which had replaced that draft in the preceding plan review.

## Evidence

Default features, E: debug target, OCCT 8.0.1 runtime, `-j 2`, serial tests:

- Core split-reservation primitive: passed before the pause, preserving totals and rejecting invalid splits.
- Five presentation snapshot ownership fixtures pass after startup-oracle correction. An additional real OM9 archive fixture opens a nonempty point document while retaining an old scene snapshot and lease: old plus new buffers stay charged before viewport work; old bytes release after the last old owner; a subsequent missing-file Open preserves document and charge.
- Six `session::files::replacement_budget_tests` pass, including New/Open competing reservation, physical-headroom refusal, archive input and Save lifetimes. An earlier `session::files::tests` filter ran zero tests and is not counted.
- Ten hover, 44 STL cancellation/admission, six incremental-scene memory, 11 document-contract, 20 file-lifecycle and nine ordinary interactive-memory tests pass. Six large interactive fixtures were intentionally ignored. The final STL build took 25.48 s and its tests 0.68 s; the subsequent integration build took 25.86 s.
- Twenty-seven serial offscreen GPU/egui Gumball/viewport tests pass without skips in 51.60 s. This does not measure physical desktop presentation or pointer latency.

## Open requirements

This closes selected startup and file New/Open buffer-registration evidence only. Other full-scene edit publications and snapshot/Master restore paths do not yet continuously register all persistent buffers/metadata/cache owners. Lazy picking cache, native OCCT, true OS pressure and release/desktop performance remain open. Historical high-water estimates can remain conservative after a later owner registration; auxiliary capture bytes can still become stale. The pending-versus-materialized reservation model is still incomplete, and detached export still reserves all currently admitted headroom. The R0 milestone and the full plan remain open pending the remaining path/lifetime checks.
