# Curve conversion parameter queries — independent native review, 2026-10-10

Requested under `superpowers:requesting-code-review`, against baseline `e5633b7`.
Scope: the six source files fingerprinted below, the direct raw NURBS entry
preflight, and persisted native/SDK execution evidence. Rust portable policy
has a separate review. Only this report was edited by this reviewer; no build,
native/GUI process, production source, test, acceptance matrix or Git mutation
was performed.

## Assessment

No blocking defect remains in the finite, synchronous, full-domain native
parameter-query slice. The resolved P1 early-read finding and its valid
RED/GREEN proof are recorded below. This conclusion does not close the full
RCORE-01 conversion, ownership, memory or durable dependency contracts.

Safe Rust owns query limits, status/domain/tolerance validation and point
observation policy. C++ is a necessary permanent native adapter for openNURBS
conversion/mapping, OCCT construction/evaluation and Qt serialization. It
reuses the existing converter rather than introducing a new geometry solver.

## Lineage, mapping and ownership

- `GetNurbForm` status 1/2 is retained. The independently owned prepared
  `ON_NurbsCurve` is validated before and after `ClampEnd(2)`. That exact
  prepared object supplies both the OCCT pole/weight/knot factory and the
  intermediate-point/basis report; a second conversion is not used for mapping.
- Forward and inverse native bool results are checked separately. A false
  result discards even a finite output sentinel. Nonfinite or out-of-domain
  successful output rejects before it can be used for downstream native
  evaluation. An inconsistent inverse is rejected and never used as an
  evaluation parameter. A failed batch returns no partial shape/report.
- Requested source parameters and actual target-domain parameters are checked
  by Rust. The raw diagnostic must also lie in the target domain; this slice
  deliberately rejects unsupported extrapolation. It does not implement
  subdomain, reversal, transformation or export correspondence.
- Reports retain copied scalars, arrays and basis/query vectors. The by-value
  C ABI contains no native pointer, allocator or document owner; the native
  objects remain live on the synchronous C++ stack. A fixture checks the copied
  report after its original native owners leave scope. C++ kernel exceptions
  are handled outside Rust calls, and Rust statuses become native query errors.
- The source CRC is checked before/after conversion; fixture mutation counters
  are intentionally outside the native geometry payload CRC. This is source
  geometry evidence, not durable identity, a cryptographic immutability proof,
  or certification of arbitrary concurrent mutation.
- Frame is explicitly `native_conversion_local`. General `ON_Curve` length
  units and source identity remain unknown; the independently authored circle
  fixtures explicitly supply `fixture_length_unit=mm`. Host unit context,
  world placement, generation/revision, lock and commit guards are not added.
- Form 1 permits raw parameter reuse only at validated requested rows. Form 2
  never permits it. Native nonlinear rows are not converted into an affine or
  callable global map. JSON keeps global locus/parameterization, parameter map,
  topology, attributes and global deviation unknown. The saved basis convention
  is explicitly the openNURBS expanded structural view after clamping, not a
  claim of identical OCCT canonical periodic representation.

## Resolved P1: raw Domain/CRC before storage preflight

The initial new entry called `Detail::preflight` and `DataCRC` before the shared
raw NURBS guard. In the pinned openNURBS source,
`ON_NurbsCurve::Domain` reads indexed knots through `ON_GetKnotVectorDomain`,
and `DataCRC` traverses CV/knot buffers using native counts/strides. Malformed
counts, owned capacities or alignment could therefore cause an early read.

The final `importCurveObserved` first casts a direct `ON_NurbsCurve` and calls
`BasisContract::preflightBasis`. Count and storage-envelope checks precede
native validity/scans, Domain and CRC. Full shared basis value validation still
runs before `GetNurbForm` copying and OCCT output construction in the existing
preparation path. The first SDK validity traversal follows the count/storage
envelope, rather than the later Rust value checks. The guard does not mutate or
allocate an extraction snapshot of the source.

`GuardedBadNurbs` starts from a valid independently converted basis, temporarily
sets `m_cv_count=1000001`, and traps virtual Domain/CRC access with an exception
and a flag. It restores the count before destruction. Removing the entry guard
therefore produces a deterministic failed assertion without an unsafe read;
restoring the guard rejects before either trapped metadata call.

The parent reran this pair sequentially with the fixture frozen and the entry
guard removed/restored. The reviewer inspected both outputs and exit records:
RED built successfully, ran with exit 1, and failed only the ordering assertion;
GREEN built successfully, ran with exit 0, and passed all 159 checks. The final
regression query JSON is byte-identical to that GREEN report.

This closes the selected direct-NURBS early-read gap. It does not certify
wrappers/proxies, every native consumer, forged or dangling pointers, external
buffer lifetimes, aggregate budgets, or literal rejection before every native
allocation required by full T07.

## RCORE-01.T01 and T10 acceptance scope

Recommendation: the **numbered T01 R10 conversion fixture is verified under an
explicit sampled measurement scope**. Its literal row requires the declared
radius/deviation tolerance, weights not replaced by all 1, and a saved basis
and report. The new fixture provides all of these for its authored inputs:

- Analytic full circles of radius 10 mm have independent source domains
  `[0,2pi]` and `[2,7]`. Conversion retains a degree-2 rational basis with nine
  poles and nine weights, including four weights `sqrt(1/2)` and five weights 1.
  The exact prepared basis constructs the actual OCCT edge; both edges pass
  native validity checks. Basis and copied query reports are persisted.
- Eight independently calculated analytic points, including both boundary
  sides and nonboundary parameters, separately agree with the prepared NURBS
  and mapped actual OCCT points within the declared `1e-7 mm` length tolerance.
  Target points are not compared only against values obtained from the same
  target implementation.
- A separate 1001-point target-domain sweep checks radius residual against 10
  mm. Its maxima are `3.552713678800501e-15 mm` and
  `5.329070518200751e-15 mm`. These are sampled maxima, not continuous/global
  maximum-deviation certificates. The circle basis also retains zero Z poles.
- The largest corresponding native residuals are
  `1.4458497151625688e-14 mm` and `1.632896157031126e-14 mm`; inverse residuals
  are at most `1.7763568394002505e-15` in native parameter units, below the
  separately declared `1e-10` parameter bound. Raw-parameter point residuals
  reach about `0.16004411466232 mm`, demonstrating the dependency error.

This recommendation refines the earlier fixture's sampled-only evidence; it
does not reinterpret sampling as a global bound. Keep the scope beside the
numbered fixture status. All-family rational/periodic preservation, continuous
deviation, full conversion reporting and overall P0 remain open.

The native query portion of T10 has concrete correspondence evidence, separately
measured locus samples, rejected raw reuse for form 2, preserved query results
and native mapping provenance. **Full T10 remains partial**: no durable
FreeCAD/History dependency uses this report, and no complete host identity,
revision/stale-result/lock/transaction lifecycle is exercised by this slice.

## Execution evidence inspected

All paths below are under `H:/FreeCAD-src/build/`; exit records and logs were
read, not regenerated by the reviewer.

| Artifact | Result / SHA-256 |
|---|---|
| `p0-curve-parameters-native-preflight-rerun-red-exit.json` | build 0, run 1; `e9b280fb5f2263d7febca72aafc1c364b748ad577d8f0b8fbaffb5646ef84be4` |
| `p0-curve-parameters-native-preflight-rerun-red-results.json` | 159 checks, only early-read assertion failed; `a9ca4824be9c73e40ebb31ad71e155a9a61889ad44842808dedb2206fa00cad9` |
| `p0-curve-parameters-native-preflight-rerun-green-exit.json` | build 0, run 0; `ccb9d3dfeeccbeb2baa41bb286ae927980a06625e78c526b09ce8a58bea71c35` |
| `p0-curve-parameters-native-preflight-rerun-green-results.json` | 159/159; `e26fe3b7a29800f15144411bfe360da197549aa937b048b5ddb86688f191d471` |
| `p0-curve-parameters-native-final-results.json` | final 159/159, byte-identical to GREEN; `e26fe3b7a29800f15144411bfe360da197549aa937b048b5ddb86688f191d471` |
| `p0-curve-parameters-kernel-results.json` | 59/59; `f14f1b625aec416aace94ff52bd6d43efc4b4de8862280d3baf2169b5468a9c9` |
| `p0-curve-parameters-native-regressions-exits.json` | build/query/kernel/CTest all 0; `b1f4b5bf3f37f79adb80121de12766847d6d323bb92f613fe5cf5d143745019b` |
| `p0-curve-parameters-native-regressions-ctest.log` | Geometry, TrimInteriorMapping, MultiShellSolid and NativeBasisGate passed; `e57063584909bbf6c32701af143144a413b59d6a536340b2d4fa7c9401a9a270` |
| `p0-curve-parameters-sdk-exits.json` | configure 0, build 0; `b7c326b7db3d5c25dc367c8d81d5683469405523df279c416ff29d962ddfc7d1` |
| `p0-curve-parameters-sdk-build.log` | Rust/native adapter/worker/module rebuilt; `908a5b08ce0756219a069b58d7ad9db035770a4a48a6630d2a0b9a91acd9b712` |
| `rcore-native/ThreeDmCurveParameterTests.exe` | final native executable; `9157b8969c7c8ba8f3eac2ca6b3c02222699353e737d8578102e5cd52a8012bc` |
| `reusable-history-runtime/bin/OpenMatrix9Gui.pyd` | final SDK module on disk; `803bdc39d0877cf17e0bb5159c347be84f90e2cc740f9265b14847bcb1ee45a2` |

Associated sequential replay logs are
`p0-curve-parameters-native-preflight-rerun-red-build.log`,
`p0-curve-parameters-native-preflight-rerun-red-run.log`,
`p0-curve-parameters-native-preflight-rerun-green-build.log`, and
`p0-curve-parameters-native-preflight-rerun-green-run.log`. Final regression
query/kernel logs have the `p0-curve-parameters-native-regressions-` prefix.

Historical initial `native-red` built with exit 0 and failed at the unimplemented
stub. The first `native-green` built with exit 0 but failed one invalid-request
assertion: its fault-injection arc inherited the circle constructor's arc-length
domain while the test expected `[0,2pi]`. The fixture was corrected to author
that domain explicitly; `native-domain-green` then passed 157 checks. The
independently authored successful `[2,7]` circle did not need a tolerance change.
These historical runs are not final post-preflight proof.

**Excluded attempts:** `p0-curve-parameters-native-preflight-red-*` and
`p0-curve-parameters-native-preflight-green-*` were accidentally run concurrently
against one build directory. The first unexpectedly passed; the second failed
compilation with object-file permission/locking error and had no runtime result.
Their preserved artifacts are invalid as RED/GREEN causality evidence. Only the
sequential `native-preflight-rerun-*` pair above supports the resolved finding.
No application macro replay is claimed by this review; SDK success is a build
check, and the parent's host regression evidence has separate scope.

## Reviewed source fingerprints

Paths are relative to `H:/FreeCAD-src/Mod/OpenMatrix9/`. These are the final
implementation/fixture bytes read for this review and the plan bytes at review
time; later documentation/checklist synchronization is a separate mutation.

| File | SHA-256 |
|---|---|
| `Gui/CurveConversionContract.h` | `f70e2fff510ef7652a79345833c13eff055bd6b769654447668d248cb4db0d58` |
| `Gui/CurveConversionContract.cpp` | `5c950c807e3b7aa2537b9919d4589c6fb0b7521fb462a22ff2d76a6a2dbe0e03` |
| `Gui/ThreeDmCurves.cpp` | `4c65cb8baf9e194c5888125da7ce63841f132ada867767237cd136c6ea457669` |
| `cmake/OpenNURBS.cmake` | `71aa792005477d4461cafe71d8ffab8a66f2082db4da801052ba541bc5cecdbe` |
| `tests/native/rhino_core_curve_parameters.cpp` | `c4a239096f08f26f2fef1f214f1494eaf9c42cfcc9fbddf4d712a52f03a58926` |
| `docs/superpowers/plans/2026-10-10-rhino-core-curve-parameters.md` | `00c38a39364b9b6177b637fb6992771fe9885d93b26b4c024d3c994e68a6541c` |

Pinned primary local reference: openNURBS
`eb92af3ba1806b0a34a99aba0d3bda83e3d46083`, under
`H:/FreeCAD-src/build/dependencies/opennurbs/`. `opennurbs_curve.h` documents
separate locus/domain and parameterization success levels and forward/inverse
mapping bool results; `opennurbs_nurbscurve.cpp` and `opennurbs_knot.cpp` expose
the Domain/CRC reads used to identify the resolved ordering defect. No Rhino
runtime parity or whole-native-process memory-safety certification is claimed.
