# RCORE-01 — Surface preview and camera teardown, 2026-10-11T15:56:06+07:00

**Status: in_progress. Every original chapter requirement remains required.**

Surface error() reuses clearPreview before displaying the diagnostic: it removes
temporary Coin preview nodes and releases their root leases on the GUI thread.
Inputs/options remain available for explicit Preview regeneration. Rust policies,
state and native geometry solvers are reused. C++ is required for Coin/Qt ownership.

## Verified lifecycle

The unchanged48-case fixture exercises Sweep1/Sweep2/Loft, plain/History, native
before-close/close faults, early/stale rejection, Esc, Cancel, document close and
document switch. Runtime24 gives420/468 with48 assertions failing:24 error paths
retain a preview and an extra root lease on each of four native viewports.
Runtime25 plus the isolated camera-guard GUI gives468/468, actual exit0 and no
escaped Base exception. Sources, metadata, native identities and Undo stay intact;
early errors/cancellation preserve executable Redo and failed-close disposal keeps
the existing checked-publication contract. Preview regeneration then Cancel works.

Explicit test root leases permit safe post-close inspection. Matched no-preview
calibration contains identical native source types/shapes, Witness/Redo and four
viewers. Reference counts are5 before preview,6 during preview,5 after error or
cancel,3 after calibrated native closure. This measures actual viewer ownership,
not an assumption that all native references must reach zero.

The initial Runtime25 run passed468 assertions but emitted six unhandled camera
exceptions during no-preview calibration closure; it is retained and excluded
from the final clean GUI cohort. Native tracing locates getCamera through Command
testActive, CommandManager testActive and the MainWindow action timer. During
deleteSelf/deferred deletion the native view can exist after its camera is removed.
onHasMsg now admits OrthographicCamera/PerspectiveCamera only while a camera exists.
The final same-fixture camera RED is15/18; GREEN is18/18, both actual process exit0.
The three RED failures are exactly escaped-camera checks. Healthy perspective and
orthographic commands still switch cameras and retained document geometry is intact.

## Evidence boundaries

New GUI cohort: **486 assertions /2 actual exits0** (preview468 + camera18), with
actual loaded App and GUI DLL paths/hashes recorded. Earlier Runtime25 caller/cold
cohort: **649 assertions /23 actual exits0** (Builder/Surface faults,16 suites and
five independent cold FCStd hosts). Those earlier suites were not rerun against
the newly linked GUI DLL;1135/25 is the sum of distinct cohorts, not a claim of
1135 matching new-GUI assertions. The unchanged218 Rust source hashes reuse the
previous808 passing Rust tests; no fresh Rust execution is claimed.

The standalone OM9 plugin was built using the single SDK. Camera correction
required compiling only native View3DInventor.cpp and relinking a separate GUI
DLL against remaining SDK objects/resources; there was no root configure/full
FreeCAD build. Shared SDK executable/App/Gui/Base hashes are unchanged.
Plugin SHA256: `c081ed038273fed1259f962a9ed9f56fe83e9b139f738d179eabac64c2325a78`. New GUI SHA256: `8819fc482151dfef1964d7d3071ebd67ab3137a5a9d660b99a708ce13c395677`.

Early camera fixtures lacked pivy initialization or the checked command argument;
they are excluded. Only two verified owned failed test processes were terminated,
with PID/path/hash checks; the user's FreeCAD application was not stopped.
An earlier Builder launch failed before recorded bootstrap telemetry with ntdll
invalid-handle c0000008; the identical-binary retry passes17 assertions. Its root
cause is still unproven, and the failed artifact is retained separately.
Final camera RED6/GREEN6 and preview camera3 ran sequentially. Earlier overlapping
diagnostic launches and the rewritten diag2 receipt are not final proof.

## Remaining full chapter work

- RCORE-01 remains in_progress. Preserve every original geometry family, enabled consumer, eight typed errors, adapters1–7, C01–C12 and T01–T11; these caller witnesses do not certify the whole chapter.
- Global conversion deviation and complete parameter/topology maps, remaining source attributes/full GeometryResult consumers, rational/periodic basis/trims/seam occurrences, solid validity and truthful representation/read/edit/retain/export remain required.
- Worker-to-GUI dispatch, arbitrary/custom callbacks, all-family source/lock/owner gates, aggregate allocation/traversal/cancellation budgets, previews/cache cleanup in other enabled producers and independent persistence/lifetime contracts remain required. Surface proofs cover exercised synchronous error/cancel/close/switch paths; camera guard proves only the two action-state queries during deferred teardown, not every direct camera caller or partial allocation failure.
- Caller commit diagnostics/reference release are proven only for the exercised Builder Create/Restore/Match and Sweep1/Sweep2/Loft plain/History paths; other producers, partial Python allocation failures and additional native exception classes are not certified.
- Production deployment/ABI acceptance remains pending: the isolated GUI test host uses App19 and the separately linked camera-guard GUI DLL. Shared SDK App12 and GUI binaries are unchanged. Historical caller/cold suites below used the earlier GUI host and were not rerun against the new GUI DLL.
- One prior owned test launcher attempt terminated with ntdll invalid-handle c0000008 before recorded bootstrap telemetry; identical-binary retry passes, underlying startup cause remains unproven. Early new camera fixtures lacked pivy initialization or the checked-action argument; these attempts are retained and excluded from final proof.

Follow the [complete closure audit](../reviews/2026-10-10-rcore01-closure-audit.md)
and [enabled-consumer audit](../reviews/2026-10-10-rcore01-enabled-consumer-audit.md).
This synchronous Surface/camera slice does not close every T08 consumer, allocation,
native cache, worker, conversion map, geometry family, C01–C12 or T01–T11.
