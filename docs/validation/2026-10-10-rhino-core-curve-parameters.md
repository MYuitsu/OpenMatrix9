# Rhino core curve conversion parameter queries — 2026-10-10

Status: **validated bounded increment; all P0 remains in progress**. Parent `e5633b7`.

Existing openNURBS forward/inverse parameter APIs and the existing OCCT rational
curve factory are reused. The shared converter retains GetNurbForm return 1/2,
validates the owned NURBS before/after ClampEnd and constructs the target from that
same buffer. Legacy curve3d/curve2d/importCurve preserve their conversion path and
default native tolerance. No command or capability is newly enabled.

The internal synchronous `importCurveObserved` accepts explicit conversion,
length and parameter tolerances plus at most 4096 requested source parameters.
Safe Rust owns domain/status/budget/finite-value validation, point acceptance and
copied query semantics. The value-only ABI contains no pointers or allocator.
C++ performs required native calls, owns native lifetimes and serializes copied
validated observations/basis. This is a permanent native adapter exception.

Each forward output is checked before inverse native evaluation. False native
outputs are discarded, including finite sentinels. Source, prepared-NURBS and
actual-OCCT mapped points must agree within the declared length bound; inverse
parameters must meet the declared parameter bound. Form 1 raw reuse is allowed
only for the inspected row after target-domain/parameter/point checks. Form 2
never permits raw reuse, including coincident seam endpoints. Global locus,
parameterization, affine map, topology, attributes and deviation remain unknown.

Reports are copied values and remain readable after original native owners die.
ON_Curve supplies no document identity or length-unit context; report fields stay
null until a guarded host caller supplies them. Standalone fixtures explicitly
author millimetres and the tested domains. Native source CRC before/after remains
equal; this is a native geometry CRC observation, not a cryptographic byte proof.

## Exact authored fixtures

The R10 analytic circle is converted to rational degree-2 NURBS with nine poles
and non-unit weights, then to the actual OCCT edge. Eight query positions include
both seam sides, π/6, π/3, 5π/6 and asymmetric interior values. An independent
analytic point formula checks mapped points. The translated source domain [2,7]
repeats the same angular positions. Matching source/target domains do not make
this nonlinear mapping an identity or an affine map.

Raw source-parameter evaluation differs by up to **0.160044114662322 mm**. Mapped
point residuals are at most **1.633e−14 mm**, under the unchanged **1e−7 mm** bound;
inverse queries satisfy **1e−10** in the explicitly authored domain units.
Separate target-domain sampling uses **1001 samples per circle** and measures
radial residuals at most **5.33e−15 mm**. These are sampled observations, not a
continuous/global maximum certificate. Basis, per-query values and reports are
saved together for the exact conversion lineage.

Line and already-rational-NURBS controls retain native form 1 with row-specific
identity checks. Six fault-injection modes exercise false forward/inverse outputs,
NaN/infinity, out-of-domain mapping and inconsistent inverse. Fault injection
uses native virtual-call boundaries; it is rejection evidence, not successful
kernel geometry evidence. Empty batches return geometry with no mapping promise.

The numbered **RCORE-01.T01 fixture is verified** under the stated sampled oracle:
declared bound, retained non-unit weights, basis and conversion/query report.
**RCORE-01.T10 remains partial**: a standalone dependent-point query is demonstrated,
while durable History, host identity/units guards and later recomputation are open.

## Verification and corrections

- Full Rust **502 passed**; Clippy exit 0 with existing warnings.
- Python tooling **49 passed, 1 skipped** using its existing unittest runner.
- Matching FreeCAD SDK configure/build exit 0; module SHA256
  `803bdc39d0877cf17e0bb5159c347be84f90e2cc740f9265b14847bcb1ee45a2`.
- Native parameter fixture **159/159**; existing kernel fixture **59/59**;
  basis, Geometry, TrimInteriorMapping and MultiShellSolid four CTests passed.
- FreeCAD application **257 assertions / four exit-0 processes**:

| Suite | Assertions | Directory in build/merge_replay-1 |
| --- | ---: | --- |
| Modeling exchange | 32 | `48c178502eb847e1afd904f91a0933ec` |
| Kernel contract | 142 | `56f1204a6e864fa5b92cd63c538bee1f` |
| Spline | 41 | `40240fbddc974f08925e65658ebec4d8` |
| Circle | 42 | `1da6d128a0504e8bbafef296da3168e8` |

Independent review resolved two findings with real RED/GREEN regressions. Raw
reuse lacked target-domain membership; Rust now requires it. The new native entry
read raw NURBS Domain/CRC before its storage gate; storage preflight now runs first.
A guarded malformed-count fixture proves this order without invalid memory access.
Complete value validation still runs before IsValid/GetNurbForm.

Initial native GREEN exposed a fixture assumption: ON_ArcCurve(circle) defaults
to an arc-length domain. The fault fixture now explicitly authors [0,2π]; the
independent [2,7] circle had already passed. Failed evidence is preserved.
Two intermediate preflight attempts overlapped a shared build and are excluded
from proof; a source-frozen sequential rerun supplies the actual RED/GREEN pair.
An unavailable pytest runner, a unittest run against the wrong directory, and a
misspelled GUI macro are also excluded. The identified test-only GUI process was
stopped and the existing spline macro then passed. No unrelated process was used.

## Remaining scope

This API is a finite full-domain import query slice. Raw diagnostics outside the
target domain reject explicitly rather than extrapolating; broader cross-domain
queries need optional diagnostics. Original source and target owners must remain
live/immutable during the call. A report is evidence, not a retained native context,
live mutation permission, interpolated map or export correspondence. An arbitrary
later query needs a new valid conversion scope.

Host/History integration, stale/deleted/reopened source guards, surface UV maps,
transformed/reversed/subdomain/export lineages, wrapper storage safety, aggregate
memory/worker budgets and all-family conversion/ownership remain open. The raw
entry preflight closes direct ON_NurbsCurve ordering only; literal full T07 remains
open. Native exceptions and release panic=abort remain separate boundaries. There
is no new Rhino 5 runtime parity evidence.

[Fingerprint-bound evidence](2026-10-10-rhino-core-curve-parameters-evidence.json)
and independent [Rust](../reviews/2026-10-10-curve-parameters-rust-review.md) /
[native](../reviews/2026-10-10-curve-parameters-native-review.md) reviews record the
accepted and excluded runs, source hashes and exact limits.
