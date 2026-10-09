# RCORE-01.T10 — guarded host parameter dependency

This checkpoint implements the authored analytic Circle/Line dependency fixture.
It reuses FreeCAD Part features, native links/FCStd and the existing History policy,
ObjectSnapshot guard and openNURBS→OCCT observed conversion. It adds no menu command
and does not replace Line/Polyline or fit a replacement curve.

The native `CurveParameterPoint` retains its original source parameter, subelement,
tolerances, owned converted Shape, point and copied query report. C++ extracts the
original analytic OCCT support and world frame without first converting it to
NURBS. The existing converter builds the target and observes the actual forward
and inverse map from the same owned NURBS buffer. Safe Rust verifies both the
host→ON source binding and the host→selected output distance against the declared
length bound. Form2 rejects raw parameter reuse, including a coincident seam.
Form1 grants it only for a proven individual query.

The report records actual document UID, native object ID/name/subelement, runtime
guard, original/mapped/inverse parameters, distinct mapped/raw residuals, explicit
unit provenance, world placement, basis and the requested point result. Numeric
runtime tokens are process-local and are refreshed on recomputation; they are not
durable or globally unique identity. Native source links and recipe are persisted.

## Verification

| Check | Result |
| --- | --- |
| Full Rust | 556 passed; exit0 |
| Clippy all targets | exit0, existing repository warnings remain |
| Actual native converter controls | 159 passed; build/run exit0 |
| FreeCAD authored warm fixture | 205 passed; process exit0 |
| Fresh-process FCStd fixture | 33 passed; process exit0 |
| Existing History controls | 83 passed; process exit0 |
| Existing object lifecycle controls | 31 passed; process exit0 |
| Tools tests | 50 run,49 passed,1 skipped; tool output only |

The four accepted FreeCAD runs use module
`e9f49c9e5b78814aa39da86110243cca05e11574f75c784c226a72de0f9a6d4f`
and four distinct launcher PIDs, with persisted process/module/macro proofs.
They establish 238 authored assertions and114 existing controls,352 total.
Exact accepted folders and fingerprints are in
[the evidence](2026-10-10-rhino-core-host-parameter-dependency-evidence.json).

The R10 circle at original angle0.31 independently checks the world analytic point,
actual retained target at mapped parameter, rational weights and raw displacement.
The fixture covers rejected raw use, finite/domain/tolerance errors, explicit
unsupported extraction, source edits/Undo/Redo, source/output ancestor placement,
reparent Undo, units, History Update/Lock/Record, direct-edit detach/Undo, null and
nonnull errored sources, deletion under Lock, missing units and recovery. Creation
has one output and one Undo; each rejection retains exact native object/property
inventory, source BRep hashes and Undo/Redo counts. An Undo-held native record stays
unchanged during unrelated group/recompute activity before Redo.

Cold FCStd preserves the query/report basis and native source link/recipe/identity.
Source and point BRep bytes are exact. Actual converted degree, knots,
multiplicities, weights, domain and periodic state remain exact. FreeCAD native
ASCII serialization rounds some near-zero converted poles: the measured maximum
change is2.8493378257508002e-18mm, below the explicit1e-12mm serialization bound.
Both raw BReps are retained without normalization. Sixty-five independent target
samples have maximum radial error3.552713678800501e-15mm, below1e-7mm. A fresh guard
is produced after restore; Update Off retains owned geometry with an explicit
dirty flag until Update On refreshes it.

## RED and excluded attempts

- Missing native API baseline: `dba53aa53c1b48ab86772ca0d2ac422a`.
- Rust policy stub:1 passed,6 failed. A later two-leg tolerance counterexample
  failed before the direct host→selected bound was added.
- Actual restore defect: `56bb97536c634e3ba90bf064be05860e` reached195 checks and
  failed because restore lost the touched flag while HistoryDirty remained true.
  `mustExecute` now honors dirty records directly.
- Initial SDK attempts failed on missing complete OCCT type and explicit ON
  constructors. These are build failures, not accepted native runtime evidence.
- `d70d87fa2abe4553a624b1a08358da96` assumed a Box Edge was unsupported, although
  that edge is an admitted Line. `975dfbf058d847be9bb11661903b84e2` used an incorrect
  empty-link Python representation. These fixture errors were corrected.
- `70c8437f056b433b8eba9359a86821fb` passed205 assertions but left a restored
  document open under a different name and reached a Save prompt. Its exact
  smoke-owned process was stopped, exit−1; it is excluded.
- Earlier passing/basic or newline/basis variants and the two cold raw-byte
  expectation failures are retained as excluded artifacts. No failing run is
  counted as acceptance.

## Limits and next work

This is the authored full analytic Circle and Line extraction slice. Trimmed
circles, arbitrary original NURBS/other analytic families and composed
reverse/subdomain maps are explicitly unsupported. Point samples establish no
continuous or global deviation certificate. Native SDK/FFI are not certified
memory-safe, and this is no all-worker/thread/sanitizer/all-allocation proof.
Rhino5 runtime parity remains unavailable from the recorded executable crash.

Independent review:
[source/lifecycle/runtime review](../reviews/2026-10-10-host-parameter-dependency-review.md).
General T07 input/ownership/allocation boundaries and other P0 chapters remain
required. **All-P0 remains in progress.**
