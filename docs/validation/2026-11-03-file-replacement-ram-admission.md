# New/Open replacement admission

## Scope

The app's New and Open construct a second controller before publishing it. A
separate coordinator sharing the external reservation account can reserve RAM
after the first controller's held-usage snapshot. The New regression with a
100,000-point cloud, retained Redo and a 64 MiB synthetic byte limit was red:
the replacement succeeded despite the competing reservation.

New and Open now adopt the shared account, then recheck the old retained
owners plus old auxiliary usage and the candidate's separately inventoried
usage while holding the shared external-document and reservation locks through
the controller swap. Contract state moves only after admission. The old
controller is dropped and observers notified after the locks are released.
The candidate is conservatively charged as a separate owner; this is not a
proof of deduplicated immutable geometry across controllers.

The two deterministic app tests inject a peer reservation after candidate
construction. Both return a budget error, preserve the coordinator snapshot
and Redo, and succeed when the reservation is released and the operation is
retried. New's earlier failing test is no longer ignored. Open uses an actual
temporary native archive. Tests use a synthetic budget and do not measure RSS.

Validation on 2026-11-03: both targeted app tests passed, all 20 app
`file_lifecycle` tests passed, all 247 `om9-core` library tests passed, and
`tools/public_source_audit.py` reported no errors. Tests used the external
`E:/om9-surface-snap-validation` target with `-j 2`; this is not a release or
desktop run. The artifact pruner retained cited runs and removed none.

## Limits

This closes the tested publication-time interleaving for these file paths, not
all managed RAM admission. The candidate is built before the final shared
check; its construction is not itself protected by a shared reservation. A
different coordinator can also reserve against its own retained world after
publication; there is not yet one authoritative account for all live
controller worlds and auxiliary caches. Native allocations, available physical
RAM, other replacement paths, background cancellation and 60% process-wide
enforcement remain separate work. The current 256 MiB general limit is not
removed by this change. No desktop or performance conclusion follows.
