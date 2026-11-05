# STL capture owner accounting

Follow-up: the capture registry now transfers across controller replacement; see [replacement evidence](2026-11-03-capture-across-replacement.md). The limitations below reflect the earlier delivery.

Follow-up: retained Undo versions no longer sum their point counts in the tested transaction paths; see [point-limit evidence](2026-11-03-undo-point-limit.md). Remaining limitations below describe the earlier delivery.

P0 partial: `TransactionCoordinator::capture_retained_document` registers weak references for externally held immutable document captures. STL captures use this entry point. The coordinator's existing retained-owner admission counts a document allocation once when it is shared with the current world or Undo/Redo; if it is no longer internally retained but an external capture still holds it, its estimated document bytes remain charged. Registry storage is also included. Weak registration itself does not keep the document alive.

Windows, `CARGO_TARGET_DIR=L:/om9-target`, Cargo `-j 2`: four retained-validation unit tests passed, including same-Arc deduplication, detached capture accounting, drop and byte-limit refusal. The existing atomic-transaction, transaction-scope and Undo suites passed 17, 9 and 7 tests respectively; a new atomic-transaction test separately passed after a real non-recording move left the capture as the old document's only strong owner. All 27 STL integration tests passed. These use accounted estimates, not measured RAM allocation peaks.

This is coordinator-local accounting only. A capture that outlives its coordinator during document replacement still needs a session-wide owner ledger; nested shared payloads may be counted conservatively across different document allocations. Export scratch, caches, GPU/shared memory, kernel peak and concurrent reservations are not yet incorporated into one admission authority. The existing 256 MiB limit and aggregation of structural point counts across internal history remain; this does not establish the 60% RAM policy or authorize background UI export. Active OM9 was not touched.
