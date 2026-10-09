# Rhino core: owned Hatch NURBS input gate

Date: 2026-10-10. Baseline `32a72a6`, branch
`codex/rhino-core-object-foundations`. This is partial RCORE-01.T07 evidence;
**the complete T07 and all-P0 objective remain in progress**.

The existing Hatch editor, native constructors, metadata copying, closure and
topology validation, transactions and exporters are reused. The demonstrated
missing behavior was rejection ordering: malformed current-loop knot/CV input
could reach archive loading, native copies and `ON_NurbsCurve` construction first.
Safe Rust now validates the entire owned loop tree before those entries.

`rust/src/hatch_loop_input.rs` reuses the bounded owned JSON parser and shared
NURBS basis validator. It retains the boolean value for rational/nonrational CV
width, checks dimension/order/cardinality, finite positive weights, homogeneous
and dehomogeneous overflow, monotone knots/multiplicity and the exact active
domain. openNURBS' two omitted outer knots are restored for the structural view;
expanded ghost-pole data is interpreted as nonperiodic for this validation.
Compact periodic representation preservation is not claimed. Every NURBS child,
including a later nested child or loop, is admitted before native construction.

The synchronous byte ABI is called before source inspection in the public
boundary, before native staging in both current-field/loop applications, and in
the writer's selected, dependency, duplicate and independent-copy overlays before
source archive loading. Read-only Qt `.value()` access prevents adding optional
null keys to plain export requests. C++ remains the SDK/host adapter for real
source identity, loop class/domain construction, metadata and topology. Python
is only existing host/API and fixture glue. No new command or solver was added.

Limits are explicit: raw JSON32MiB, 3million JSON nodes, JSON depth256 for this
input only, native curve depth64, 16,384 curves/loops, 500,000 CVs, degree25,
1million knots, 2million NURBS numeric values and shared logical64MiB basis
budget. These bound owned data and extraction; they do not measure aggregate
native/process/worker allocations. Other parser consumers retain depth64.

## Evidence

The machine-readable [evidence file](2026-10-10-rhino-core-hatch-input-gate-evidence.json)
records exact source, module, macro, result, review and raw-log hashes.

| Verification | Result |
| --- | --- |
| Complete Rust suite | 541 passed, including18 Hatch input cases; exit0 |
| Clippy | exit0; one test-only manual-dangling-pointer warning retained |
| Python tools | 49 passed,1 skipped; exit0 |
| Native input gate | 27 checks passed; build/run0 |
| Retained native regressions | Merge, HatchCurrent, HatchLoops, HatchLoopEdit and HatchAffine:5/5 passed |
| Matching native SDK | configure/build0; current module `8b03c899…` |
| Fresh FreeCAD gate | 13 checks passed; process0 |
| Existing FreeCAD typed-loop edit | 132 checks passed; process0 |
| Existing shared Hatch edit | 12 checks passed; process0 |
| Existing instance isolation | 69 checks passed; process0 |
| Existing Builder History | 43 checks passed; process0 |

The five fresh FreeCAD processes provide269 checks on the same module. Actual
typed-loop edit exercises the existing degree-2 rational circle construction.
Source/archive/document geometry, object count, Undo and destination sentinels
remain unchanged on malformed requests. Valid basis controls continue to source
access and the supported native editor/export routes.

The original public-boundary RED is preserved at
`build/merge_replay-1/7ab0806026354619863b6b4bd0094a07`: wrong knot cardinality
reported missing source archive instead of input rejection on the prior module.
The portable stub RED had13 failed and3 passed cases; final18 passed.

Two introduced regressions were reproduced and fixed, with failed logs retained.
Mutable Qt optional-key access inserted null Hatch fields and broke plain/dependency
exports (native gate/retained Merge/Hatch tests failed). Read-only access fixes
this; the final27 checks and native regressions pass. Generic JSON depth64 also
rejected a native-supported63-parent PolyCurve (16 passed,1 failed); a Hatch-only
depth256 parser preserves that control while still rejecting native depth64.
The general parser default remains64. These are production regressions, not
fixture failures.

An excluded shared-Hatch replay stopped before any check because its authored
mm/cm archives had not been generated. Running the existing HatchAffine native
fixture produced both files in the harness' fixed temporary directory; the
unchanged host macro then passed12 checks. No product fix was inferred from the
missing fixture.

Independent [Rust review](../reviews/2026-10-10-hatch-input-rust-review.md) and
[native/host review](../reviews/2026-10-10-hatch-input-native-review.md) found no
remaining blocker in this scoped increment and verified recorded fingerprints.
The primary agent separately observed the three additional control process exits.

## Remaining scope

This proves rejection before the covered archive-read/copy/constructor entries,
not an instrumented certificate of every native allocation. Raw archive parsing,
proxy/member/trim consumers, other wrapper factories and aggregate worker budgets
remain full T07 work. Non-NURBS numeric/class/topology rules stay with existing
native validation. Expanded ghost-pole positive acceptance is an evidence gap;
this increment does not certify that convention for all native inputs.

The ABI caller must supply one initialized, immutable, readable live allocation.
Address/length checks cannot establish accessibility; no pointer is retained.
Rust panic-abort and allocation exhaustion are not recoverable geometry errors.
This is not a global memory-safety or native-owner lifetime certificate.

T08 delayed-result lifetime, T09 complete periodic/seam/hole persistence,
T10 guarded host dependency and T11 explicit subobject mutation owner remain
required. Continue with the existing FCStd persistence path and an authored true
periodic curve; there is no evidence that a replacement serializer is needed.
