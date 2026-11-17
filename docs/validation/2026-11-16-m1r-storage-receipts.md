# M1-R: immutable document storage receipts — 2026-11-16

Status: partial M1-R implementation, final native batch completed. All ten
16/64/128/512/768 Light/High configurations passed strict exact-source,
placement, per-sphere four-viewport publication and final Undo/Redo checks.
M1-R latency gates and the complete incremental owner registry remain open.
No Boolean; canonical geometry and ordinary Undo policy remain enabled.

## Final measured release

Release SHA-256:
`cf6c4d9c424abcac85f7fd0bc9e8a011f711f1fe981bd3774a12ac2da1291bd0`.
Build: `attribution-release.log`, passed. Named core checks: 36 passed
(`attribution-oracle-tests.log`, one ordinary-batch manual timing probe ignored);
scene checks: 31 passed (`attribution-scene-tests.log`); app library check:
passed (`attribution-app-check.log`). A separate release ownership micro probe
passed three repetitions. Broad library-test compilation is not claimed.
Full results: [JSON](2026-11-16-m1r-storage-receipts.json).

| Spheres | Detail | Orbit FPS | Selected orbit FPS | Max creation UI work ms | Max creation callback gap ms | Peak WS MiB | Undo steps |
|---|---|---:|---:|---:|---:|---:|---:|
| 512 | Light | 29.49 | 30.07 | 281.85 | 461.56 | 539.16 | 512 |
| 512 | High | 29.97 | 27.60 | 329.27 | 438.99 | 818.98 | 512 |
| 768 | Light | 29.60 | 29.78 | 728.50 | 730.05 | 718.83 | 768 |
| 768 | High | 29.98 | 20.30 | 821.12 | 824.36 | 1151.46 | 768 |

These are single paced physical-console runs, detailed commit profiling off,
four viewports, replay capped at 30 Hz. Each sphere must publish in all four
views before the next command. GPU publications are not proof of physical
scan-out. Callback gaps include scheduling/paint waiting; UI work is measured
separately. Neither metric is interchangeable with steady orbit FPS.

The selected High phases contain an initial hold of the previous published
image. The first new Perspective publication arrives 1,590 ms after phase
start at 512, and 3,195 ms at 768; the standard measurement discards only the
first 1,000 ms. This leaves 590/2,195 ms initial holds in the reported selected
FPS windows. After the first new publication, measured rates are 30.15/30.06
FPS for the remaining 6.15/4.55 seconds. **Keep the original 27.60/20.30 FPS
and initial delays**: the shorter follow-up rate does not erase the delay.
Camera revisions and UI input continue during those holds. The trace proves
delayed fresh publication at phase entry, not the exact worker cause or the
content's selection revision. Light's first-new delays are 439/639 ms.
Evidence: `selected-gap-diagnostics.json`, `selected-start-results.json`.

Total paced creation times are 148.63/280.73 seconds for 512 Light/High,
327.95/635.74 seconds for 768. Command maxima are 275.83/321.48 ms and
721.17/812.15 ms respectively. Command-to-all-views publication p95 is
500.42/917.20 ms at 512 and 1000.06/1767.39 ms at 768. At 768 there are
52 Light and 91 High creation UI callbacks above 500 ms; none above 1,000 ms.
Creation responsiveness still fails the M1-R latency objective.

Peak sampled private bytes are 782.19/1290.82 MiB at 512 and
1210.31/1617.98 MiB at 768 (Light/High). Sampling is 1 Hz; working-set peaks
and private allocations measure different things. GPU adapter memory is
recorded in JSON and includes other applications; it is not OM9-only VRAM.

The six small configurations passed before the large batch; orbit FPS ranges
29.35–30.08. Light/High exact-source fingerprint multisets match at every
size. All native bodies retain expected volume, one closed solid, zero naked
edges and correct placement. All ordinary creation Undo steps remain. The
desktop audit executes the final Undo/Redo pair, not every historical step;
core regression tests exhaust smaller histories.

The final fixtures explicitly allow **4,000,000 points** for every size and
profile; retained-version object slots use the previously recorded 100k/400k
allowances. Product default 2M points is unchanged. Undo retains its ordinary
1024 MiB soft quota/minimum one step, with no depth or mesh-quality override.
The earlier 2M High refusal is documented below and is not a 768 pass.

Automatic full attribution inventories before final verification fall to
5 at 512 and 11 at 768; deferred refreshes are 1,019 and 1,525 respectively
for both profiles. The prior release performed 2N such censuses. End-of-audit
exact inventory is separate. The optional bound does not mark attribution
fresh and cannot authorize an over-budget admission or inexact trim.
The release micro probe's ordinary/bounded medians are 15.1107/0.0404 ms
(`release-micro-1.json` through `-3.json`); this synthetic ownership-only
fixture establishes neither native geometry performance nor desktop FPS.

Remaining: complete allocation-delta registry, removal of cold synchronous
inventories from interaction, unpaced burst and full pressure/alias matrix,
M1-R p95/p99 latency gates, selected-phase publication latency, and the earlier
maximize/resize issue. These runs began maximized at constant client size;
they do not prove that resize issue fixed. The old profiled 512 baseline is
not a controlled A/B speedup comparator.

Final maintenance: Python compilation passed for the four changed benchmark/
analysis tools; measurement/progress JSON parsed and all ten pass, Undo count,
4M allowance and Light/High source-parity records were checked. Public source
audit passed (16,114 source files, 594 icon bindings, 593 SVGs, no errors).
Artifact pruning ran after resolved-path validation: zero deletions; older
cited evidence runs remain protected by the repository retention rule.

## Implementation and earlier diagnostic attempts

The coordinator now reuses exact document storage Usage for immutable owners.
Each Undo entry contains two fixed receipts, charged through UndoEntry storage;
cloning an entry clears these transient receipts. The coordinator has two
weak-only slots for documents not owned by a journal entry. Their potential
Arc control blocks/inline Documents are conservatively charged even when the
slots are empty. No source payload or display is copied into these caches.

Hits require the identical allocation, immutable ownership and compatible
depth/degree/name limits. Aggregate Usage is checked against current limits
on every hit, independently of the limits that first computed it. Failed
computations are never cached. Weak slots prevent address recycling while a
receipt exists and force copy-on-write rather than mutation of the cached
allocation. The old structural validation proof remains separate.

Internal inventory uses the exact receipt instead of repeating document
storage walks; the conservative scratch admission still includes every
document occurrence before allocating an index. External inventory can reuse
receipts for journal/current candidates. Cross-document deduplication and
reservation guards remain active. Full owner-index/delta accounting (R2),
general worker preparation and amortized cleanup are not implemented here.
The new optional projection cache does use existing cache reclamation;
canonical source/display eviction and journal redesign are outside this change.

Debug cache hits execute the original storage calculation and compare its
result with the receipt. This is an independent slow-path oracle, excluded
from release. New integration tests cover warm calls, held-budget refusal,
rollback, Undo/Redo/branch, shared and independent synthetic canonical BReps.
The synthetic fixtures do not establish native geometry accuracy.

Profiling now records every retained validation call, including calls below
the old 50 ms logging threshold, and cumulative storage hit/miss counts.
Detailed logging may perturb latency; old thresholded call counts are not
directly comparable. Product validation remains active without profiling.

The console harness also samples native window visibility/client size,
maximized/hung state for its own process and stops its owned test tree if
Windows marks its visible window hung. It does not close user applications.

Evidence root: `build/standalone-validation/ui-artifacts/m1r-receipts-20261116`.
Baseline binary SHA-256 and prior measurement reference are in `baseline.json`.
Final results and remaining work are recorded above. The following sections
preserve earlier attempts; their binary identities and settings differ.

## First native attempt and follow-up

The first release (document receipts only) passed six console configurations:
16/64 Light/High with detailed profiling and 128 Light/High without detailed
profiling. Exact-source and final Undo/Redo assertions passed in each completed
run. Navigation stayed about 29.9 FPS. At 128, command maxima were 176.263 ms
Light and 186.036 ms High; peak working sets were 341.793 and 417.223 MiB.
These are not results for the subsequent aggregate projection cache.

The 512 Light diagnostic was stopped at 352 completed commands, with command
maximum 1,783.565 ms and final completed command 1,627.566 ms. It has no final
source/Undo audit or strict benchmark pass. High512/768 were not started in
that attempt. The owned stop receipt is `large/stopped-latency-gate.json`.
This failed the latency objective and triggered the next implementation step.

`transaction/retained_projection_cache.rs` now holds an optional admitted
projection for the exact internal/external owner lists. Weak handles prevent
allocation-address reuse and do not retain source payloads. A hit requires
matching capture-generation owner/value, unsaturated generation, live/dead
external states, candidate exclusion, external registry capacity and compatible
limits. Counters for external documents must not become stricter. Reservations
and journal/identity metadata are projected afresh. A miss retires the old weak
index, performs the original inventory and optionally admits a new index before
allocation; lack of cache headroom keeps the original validated operation.
Pressure on a warm projection also falls back after retiring optional cache.
Cache retirement/admission/install are serialized for read-only callers.

Debug projection hits are compared with the independent full inventory when
capture generation and external live-state permit a stable comparison. Thirty
integration checks passed, one manual timing test ignored:
`projection-final-tests.log` (startup-held 1, held preparation 6, retained
admission 2, receipt/concurrency 4, scopes 9, Undo memory 8).
This memoization avoids repeated work for an unchanged owner set; incremental
allocation-index updates across changed sets and worker preparation remain open.
No latency acceptance is claimed from these tests.

## Additional diagnostics and current implementation

`projection-small` and `independent-small` passed the six 16/64/128 Light/High
strict source/Undo/publication configurations for their respective binaries.
Their 512 Light runs (`projection-large`, `independent-large`) each committed
and presented all 512 spheres, then the watchdog stopped the owned application
during final verification. Neither has a strict final source/Undo pass.
The audit was calling the blocking Controller Undo convenience function, while
normal UI history already has an asynchronous worker. The audit now requests
Undo/Redo through that normal UI path, waits across frames and verifies exactly
one epoch publication per operation. Native source checks remain after timing,
one body per frame with a document/epoch guard. This is a harness correction,
not evidence that product Undo latency is solved.

The current implementation also:

- Reuses the internal projection when only external captures/reservations
  change, with fresh current limits/metadata checks.
- Searches newest retained owners first and skips external alias searches when
  the allocation is already owned internally. Resource multiplicity semantics
  remain unchanged. This removes an unnecessary growing prior-owner scan.
- Charges projection vectors and conservative Weak/control-block bounds to the
  shared admission account through RAII. Peers see cache growth, and release
  follows actual destruction. Header allocations and optional-registry growth
  are admitted through existing reservations, including caller-private held
  bytes during coordinator/peer startup.
- Registers a weak reclaimer using the existing optional-cache mechanism.
  Pressure may discard the derived projection without changing geometry or
  history. The original inventory remains the cache-miss/refusal path.

Tests compare debug fast paths with independent original inventories and cover
expiry without a tracked capture, tighter per-document limits, shared/distinct
synthetic canonical allocations, peer visibility/reclamation, account adoption,
held startup refusal and unchanged model/history. Picking pressure tests now
include projection-cache bytes in the pressure request and its independently
measured owner headers in accounting, while retaining pinned-reader and LRU
assertions. Full library-test fixture compilation still has pre-existing
CanonicalBrep migration errors; only named integration targets and app library
check are claimed here.

The current release log is `shared-release.log`. Native matrix `shared-small`
then `shared-large` uses detailed commit logging **off**, four viewports, normal
Sphere commands, full ordinary Undo and a 30 Hz replay scheduler cap. It is a
single paced run per configuration: each new sphere must publish in all four
viewports before the next command. It does not establish unpaced burst latency.
The fixture retains the prior benchmark's `OM9_MAX_OBJECTS=400000` for sizes
above 16 (`100000` at 16), allowing retained-version object slots; this is not
a claim for an unmodified default object-count limit. Undo remains enabled,
with its ordinary 1024 MiB soft quota and minimum one step. No Undo depth,
geometry point limit or mesh-quality override is used to shorten these runs.
The physical-console watchdog remains enabled. Results will be appended when
the matrix completes; earlier binaries are diagnostic history, not evidence
for current-source native acceptance.

## Earlier shared release: small and 512 results

Release build passed (`shared-release.log`, 9m27s), SHA-256
`8f428a4f1cfed18bd6f350178e8dc279cd8167a196e8a9d7e0bd5537ce25b7f0`.
All six 16/64/128 configurations passed strict exact-source, placement,
Undo/Redo and per-sphere publication checks. Fingerprint multisets match
Light/High at each size. The two 512 configurations have now also passed,
including identical source fingerprints and 512 retained Undo steps.

| Spheres | Detail | Orbit FPS | Selected orbit FPS | Max command ms | Max UI work ms during creation | Max callback gap ms during creation | Peak WS MiB |
|---|---|---:|---:|---:|---:|---:|---:|
| 512 | Light | 30.00 | 29.93 | 370.60 | 377.16 | 455.81 | 543.58 |
| 512 | High | 29.79 | 27.56 | 394.74 | 402.36 | 481.92 | 821.29 |

Callback gaps include scheduler/paint waiting; they are not all CPU freezes.
These single runs do not establish stable percentile comparisons between
profiles. Neither Windows watchdog reported a visible hung window. The main
window was maximized from startup at a constant 1600x1000 client; this does not
resolve the earlier maximize/resize regression. Foreground varies: completed
GPU publication is measured, not physical scan-out or unoccluded visibility.

The selected 512 High orbit has a 583 ms gap between new completed Perspective
publications. UI callbacks remain below 37 ms there, and measured GPU queries
are below 9 ms; the missing-publication cause is not established. The result
does not meet the 28–30 FPS navigation target for this phase.

Creation work remains above the M1-R UI latency targets. The Light 512 command
p95/p99 are 270.03/326.11 ms. Source validation/admission is still active;
the implementation has not become an O(delta) allocation registry. The Undo
counter still reaches 1,024 full inventories at 512 creations, with the exact
end-of-audit census separate. Removing these capture-triggered repeated
inventories safely and avoiding cold aggregate rebuilds are remaining work.
No exclusive attribution of all current UI time to Undo is claimed without
new diagnostic phase measurements.

512 Light creation takes 160.80 s in the paced fixture. The prior 1,146.95 s
baseline had detailed profiling enabled and lacks final strict verification;
do not interpret the ratio as a controlled A/B speedup. The original 8.094 s
creation callback gap is historical diagnostic evidence, not a same-condition
counterfactual for this release.

## Follow-up: census attribution bound and point-ceiling diagnosis

The two 768 High attempts (`shared-large`, `shared-retry`) both stopped after
648 committed/presented spheres, at radius input for Sphere 649. The initial
generic GUI failure was independently reproduced by the direct Controller
probe as `Budget("points")`, with 648 objects and 648 Undo intact
(`native-admission-red.log`). The original probe's expectation that 768 High
would fit the default two-million point ceiling was incorrect. This is a
configured scene-complexity refusal, not evidence of source loss or a measured
768 High FPS result. The first failure's Close did not exit; its later idle
window must not be counted as a render freeze. Both failed attempts remain
preserved. The wrapper now tails complete new trace lines and stops its owned
process as soon as the audit reports failure; its partial-line/cursor check
passed (`check-audit-watchdog.py`).

[Point-ceiling research](2026-11-16-viewport-point-ceilings.md) distinguishes
Blender's declared per-mesh vertex ceiling, Rhino 5 RAM guidance and OM9's
whole-scene display points. The final stress matrix explicitly sets four
million points for **every** size/profile, in addition to the previously
recorded object-slot allowance. This preserves the application default and
all active configured validation, source precision, display quality and Undo
policy; it is not a default-ceiling benchmark or a fix obtained by lowering
mesh quality. The new manual native regression records both the expected
default-ceiling refusal and the full 4M-point fixture; its final revision has
not yet been executed, so it is prepared coverage only.

The next implementation avoids a capture-attribution census only when the
already admitted immutable document-byte summary plus fresh journal metadata,
all declared evidence/source upper bounds and overflow-vector capacity proves
history must still be below its soft quota. Publication may replace a current
world alias while Undo already owns both immutable versions: the byte-bound
helper verifies the unique allocation set on both sides, using Weak keys to
prevent address recycling. It makes no new admission decision or allocation.
Its normal aligned comparison is linear; many mismatches can require quadratic
membership comparisons. It is not a general allocation-refcount registry.

No exact attribution estimate is declared fresh by this shortcut; the existing
pending flag remains set. Growth at 100 MiB, an estimate that could exceed the
quota, an unavailable cache, saturated generation or an unproved bound still
takes the original census path. Trimming continues to require fresh exact
inventory. `deferred_attribution_refreshes` counts these avoided refreshes.
The new-baseline growth bound also reuses the immutable storage receipt.
Debug builds compare the bound against the independent original Undo inventory
when that oracle can complete; release excludes that oracle, not validation.

The follow-up passed 36 core checks (`attribution-oracle-tests.log`) including
48 create/edit/capture/drop/Undo/Redo steps and the existing quota/cadence/
rollback cases; 31 scene checks (`attribution-scene-tests.log`) and app library
check (`attribution-app-check.log`) passed. One manual timing probe remains
ignored in the ordinary core batch. The completed final release/native batch
is in `attribution-release.log`, `attribution-small`, `attribution-large`,
summarized at the start of this report. Earlier numbers in this historical
section belong to their named binaries. M1-R latency and other recorded gates
remain open despite completion of the final native matrix.
