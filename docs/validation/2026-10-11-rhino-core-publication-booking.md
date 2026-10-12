# RCORE-01 — Builder/Surface publication and native booking

Updated: 2026-10-11T13:34:12+07:00. **RCORE-01 remains in progress; global publication acceptance is open.**

Builder Create/Restore/Match and Surface Sweep1/Sweep2/Loft keep their existing
solvers. They use FreeCAD's explicit preserving-Redo API and recheck source
snapshots immediately after booking. Surface failures include the actual
operation and source/subelement on the same typed diagnostic line. Transient
surface evidence carries that operation; Rust policy/solver logic is reused.
The same frozen fixtures on Runtime18 failed Builder6/18 and Surface18/30
(actual host exit0, failed checks/wrapper exit1); their failures are retained.
The original native output-created callback fixtures now pass Builder18/18
and Surface30/30 on Runtime21/API11, including executable prior Redo.

The wider unchanged Builder History fixture exposed a separate native fault:
Runtime19/API9 passed31/32 and failed Match, while the old Runtime18 passed43/43.
Deleting a previously journaled object creates a native Delete transaction with
no application booking. An explicit default edit or joined group then booked
a new ID but appended to the old deletion journal. Undo resurrected the deleted
RecordOff record. Native RED19 reproduced both cases (0/2). `setActiveTransaction`
now completes the preceding materialized journal before a different explicit
operation. Same-ID joins and temporary-name promotion keep their existing path.

A source-only review then identified a real callback race: completing that old
journal can close the requested group. RED20 reproduced an inactive ID being
returned/booked (0/1). The join now rechecks application activity after the
preceding commit and rejects that stale ID. Three new root Document tests pass
on API11. The original Builder History fixture remains unchanged and passes43/43,
including Match and FCStd save/reload.

Matching Runtime21/API11: **462 checks across16 serial host processes**, all
checks true and each actual host exit0. The structured report names every
fixture and its selected plugin hash: Builder publication/output/result/conversion,
History/groups; Surface publication/result/identity/frame/analytic/commands/History/
diagnostics; Edit callback and Python atomic API. **46/46 root transaction tests**;
the App executable has **602 passed,2 skipped,zero failed of604 executed** and
seven disabled cases. The two skips remain the zero-file backup policy tests.
Fresh Document/SDK bootstrap units plus cached other App units do not constitute
the whole FreeCAD project suite. Full Rust: **808 passed**, actual exit0.

## New native evidence contradicts full publication completion

The separate native publication-close unit ran three tests: **one passed,two
failed**, actual exit1. The default abort global-before/close pair passes.
`AbortSignalsOnceAfterAllParticipants` fails because the opt-in group abort
does not dispatch the global pair. `BeforeCommitCannotOverwriteValidatedSource`
fails because the global before-close callback can still change the active
source after validation and observes already-cleared booking/description.
These failures remain open acceptance evidence; they are not included in the
passing604-case executable or relabeled as successful tests.

The witness checks participant lifecycle, actual native identity and executable
Redo, source value through Undo/Redo and booking activity at the callback.
Before/close exception aggregation, counter recovery and deferred commit
retirement still need additional witnesses. [Native boundary review](../reviews/2026-10-11-atomic-publication-close-gap.md).

FreeCAD owns these journals and signals, so C++ is the necessary native adapter;
portable business logic remains Rust. Actual App source changes required only
a target-only `FreeCADApp` update in the single shared SDK. The standalone OM9
build reused its existing cache and14 workers. No root reconfiguration or
independent FreeCAD runtime was created. Current native/producer/Rust and
selected DLL/plugin fingerprints equal their matching captures.

## Remaining original objective

- Native global atomic abort must emit exactly one before/close pair with participant lifecycle and held Redo ordering. Current isolated witness fails; this is not a completed boundary.
- Native global before-commit observers must not overwrite validated sources; booking/descriptions must remain valid during before notifications. Current isolated witness fails.
- Before/close observer exceptions, exception-safe signal counters, deferred commit retirement and aggregate truthful verdicts require additional native witnesses and implementation.
- Worker-to-main-thread and arbitrary custom virtual callbacks, complete aggregate rollback postconditions and allocation/traversal/cancellation/worker budgets remain unproven.
- Every original RCORE-01 family, enabled consumer, full GeometryResult/source attributes, measured global conversion deviation and parameter/topology maps, representation preservation/export, persistence, ownership/cache/lifecycle, typed errors, adapters1–7, C01–C12 and T01–T11 remain required.

[Structured receipts and fingerprints](2026-10-11-rhino-core-publication-booking.json).
