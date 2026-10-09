# Rhino core polycurve partition preservation implementation plan

> **For agentic workers:** Use superpowers:executing-plans task by task and
> request independent portable/native reviews before accepting preservation.

**Goal:** Reuse existing native wires/reversal and preserve ordinary owned
polycurve segment order/direction/joints through working CAD exchange.

**Architecture:** Keep Line/Polyline and legacy single-edge importCurve unchanged.
Safe Rust validates owned segment/joint observations, limits and occurrence
coverage. Thin C++ resolves native owners, converts admitted children with the
existing curve factory, constructs a wire and exports oriented children safely.
Prepared conversion facts precede the existing atomic host commit; Python is
bootstrap/fixture/property glue only.

**Tech Stack:** Safe Rust/value ABI, FreeCAD/OCCT/openNURBS, native and fresh-host
FCStd/working V5 fixtures. No solver or native dependency is replaced.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`,
ordered segment contract and T05. Baseline `04d64cc`; reuse audit:
`H:/FreeCAD-src/build/p0-next-polycurve-reuse.md`. All P0 remains in progress.

## Global constraints

- Preserve command IDs and existing Line/Polyline/Arc-mode support status.
- ImportCurve stays single-edge for Hatch and other callers that require it.
- Exact owned ordinary ON_PolyCurve dispatch only; PolyEdge/reference graphs keep
  their existing explicit retention/rejection. Nested scope is not silently flattened.
- Preserve each source segment curve; no Join midpoint healing, AppendAndMatch,
  CloseGap, RemoveNesting, fitting or whole-polycurve GetNurbForm fallback.
- Domain/parent-domain relations, topological direction and supporting-curve
  parameterization are separate observations. No global/export parameter map.
- Native C++ owns live curve/wire handles and native calls. Rust owns numerical
  policy, budgets and copied facts. No retained host/native pointer crosses ABI.
- The fixture declares length tolerance 1e-7 mm; sampled geometry is not a global
  continuous bound. Native shared vertices do not establish original identity.

## Review focus

1. Ordered traversal must cover every edge occurrence, including reversed wires;
   Shape.Edges count/total length alone can conceal lost partition.
2. Parent segment intervals differ from child supporting domains; reversal can
   negate/reorder native domains without changing the geometric expectation.
3. Native containers/raw children need envelope gates before Domain/CRC/evaluation.
4. Append ownership transfers only on success; failure publishes no partial output.
5. Working exchange and preserve-mode replacement differ; History/userdata and
   nested/reference ownership stay unclaimed without their own evidence.

## Task 1: baseline the reusable host and expose exchange RED

**Files:** New `tests/rhino_core_polycurve_smoke.FCMacro`; ignored saved run helpers.
**Interface:** Warm report saves exact mixed native fixture, segment/basis ledgers
and FCStd path. Cold invocation receives explicit file and baseline paths; new
process compares native geometry/order/direction/joints before any exchange claim.

- [x] Construct line A=(-5,0,0)→J=(0,0,0), rational quarter R10 centered (0,10,0)
  J→B=(10,10,0); use existing Part APIs and source/sibling deep copies.
- [x] Inspect forward `[line,arc]`, reverse `[arc reversed,line reversed]`, joint,
  declared sampled radial/endpoint bounds and rational weights; save FCStd.
- [x] Reopen in a fresh process and compare ledger, CAD kind and copied basis.
- [x] Working V5 export/import must retain one Wire/two ordered edge occurrences;
  capture the actual missing-partition RED before production edits.

## Task 2: portable ordered segment/joint contract

**Files:** New `rust/src/polycurve_contract.rs`, `polycurve_contract_ffi.rs`,
focused integration tests; modify `rust/src/lib.rs`.
**Interface:** Typed copied segment rows, explicit joint/no-heal policy, checked
count/byte budget and owned report; value ABI supports preflight and sequential
row/coverage gates without native pointers. Parent/child domain evidence remains
separate; unknown global preservation stays unknown.

Portable interfaces: `Input {segment_count, parent_domain:Option<[f64;2]>,
length_tolerance, parameter_tolerance, closed, joint_policy}`;
`Segment {ordinal, family, child_domain, parent_domain, start_xyz, end_xyz}`;
`NativeSegment {ordinal, direction, start_xyz, end_xyz, shared_with_previous:Option<bool>}`.
`preflight`, `validate_source_step`, `validate_native_step`, `validate_finish`
return typed rejection/finite measurements. `report` validates complete borrowed
rows and returns independent owned vectors plus uninspected global conversion facts.
FFI is copied value-only source/native rows and summaries; no callbacks/pointers.
Declared first-slice policy: 4096 segments and 1MiB logical owned report data;
native copies/container/worker allocations remain separate and unclaimed.

- [x] RED/GREEN tests for sequence/ordinal/direction, finite domains/endpoints,
  source/native endpoint agreement, joints and full occurrence coverage.
- [x] Reject unknown flags/options, missing/nested/reference children, over-budget
  counts, invalid/gapped/branched sequences and malformed ABI values.
- [x] Preserve row-specific observations without claiming durable EdgeN identity,
  global parameter correspondence, tangent continuity or uninspected attributes.

## Task 3: native ordinary polycurve-to-wire and shared wire export

**Files:** New `Gui/PolycurveContract.h/.cpp`; modify `ThreeDmArchive.cpp`,
`ThreeDmMerge.cpp`, `ThreeDmArchive.h`, `ThreeDmStaging.cpp`, OpenNURBS CMake;
native fixture `tests/native/rhino_core_polycurve_partition.cpp` and native CMake.
**Interface:** A separate observed ordinary-owned import returns independent wire
and copied segment/joint facts. Shared checked wire export returns owned
ON_PolyCurve; original single-edge converter and reference routing stay intact.

- [x] Native RED requires two distinct oriented edge occurrences and unchanged
  source/sibling. Gate containers/children before metadata or conversion.
- [x] Convert each admitted child with the shared curve/basis factory, validate
  source endpoints/relations, MakeWire natively and revalidate actual traversal.
- [x] Export every oriented wire occurrence with existing exportCurve; keep
  unique_ptr ownership until Append succeeds, verify complete traversal/counts.
- [x] Carry copied prepared observations before host commit. Reread actual V5
  output and verify per-segment order/direction/endpoints rather than object count.
- [x] Reject/retain unsupported nested/reference/branched/gapped cases explicitly,
  preserving source/destination/transaction state and supported all-line controls.

## Task 4: acceptance, review, scoped sync and next P0

- [x] Run native fixtures, warm/cold host mixed fixture and real working exchange;
  existing Line/Polyline/edit/trim/working-exchange regressions on matching build.
- [x] Full Rust/Clippy, Python tooling, matching SDK/native build, independent
  portable/native review, public source audit and exact source/runtime fingerprints.
- [x] Synchronize T05 only after each stated assertion is established, retain
  global/nested/History/ownership limits, commit locally and continue all P0.
