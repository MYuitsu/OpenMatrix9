# Rhino core curve conversion parameters implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:executing-plans task by task, and request an independent review
> before accepting the native mapping and preservation claims.

**Goal:** Preserve native curve-conversion lineage and expose validated copied
parameter-query reports so a dependent point never silently reuses an unproven
source parameter.

**Architecture:** Reuse openNURBS forward/inverse parameter APIs and the existing
OCCT curve factory. Safe Rust owns query budgets, domain/status validation and
owned observations. C++ keeps source/native buffers on its synchronous stack,
performs required kernel calls and serializes Rust-validated primitive results.

**Tech Stack:** Safe Rust, value-only C ABI, existing FreeCAD/OCCT/openNURBS,
native numerical acceptance fixture and existing application regression macros.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`,
conversion/ownership clauses and RCORE-01.T10. Baseline `e5633b7`; all P0 remains
in progress. Reuse preparation: `H:/FreeCAD-src/build/p0-next-parameter-map-reuse.md`.

## Global constraints

- Preserve command IDs and supported geometry; no new UI command is introduced.
- Locus preservation does not prove parameterization preservation.
- A false native mapping result must not publish its output argument.
- Use the exact owned NURBS buffer that constructs the target, rather than
  applying a mapping from a different conversion or backend.
- Existing affine `ParameterMap` stays affine. Native nonlinear query rows are
  observations for requested parameters, not a global interpolated map.
- Record units, domains, lineage and numerical bounds. Unknown topology,
  attributes, global deviation and future parameter correspondences stay unknown.
- Native ownership is a permanent adapter exception; no host/native pointer
  crosses the Rust value ABI or survives the synchronous call.

## Review focus

1. Circle mapping is nonlinear even when source/target domains and endpoints match.
2. Subdomains, translated domains and boundary parameters cannot be extrapolated.
3. Native false results can leave finite output sentinels; ignore them entirely.
4. Clamping and OCCT reconstruction require their own lineage and point checks.
5. A finite batch and a kernel-reported status are not a global accuracy certificate.

## Task 1: portable query policy and owned reports

**Files:** Create `rust/src/curve_conversion.rs`,
`rust/src/curve_conversion_ffi.rs`, `rust/tests/curve_conversion.rs`,
`rust/tests/curve_conversion_ffi.rs`; modify `rust/src/lib.rs`.

**Interfaces:** `ConversionInput` contains native form status, source/target
domains and explicit conversion/length tolerances. `ParameterObservation` contains
requested/mapped/inverse parameters, success flags and source/intermediate/native
point values. `validate_request` and `validate_observation` return typed errors or
owned query facts. A copied query report names the native mapping method and
scope; no general callable map is fabricated. The bounded value-only ABI mirrors
those records, returning validated fields/status with no borrowed native storage.

- [x] Write RED tests: native status1/2 distinct, status0/unknown rejected,
  nonfinite/reversed/overflowed domains and out-of-domain queries rejected.
- [x] Test false forward/inverse mapping with finite sentinels, nonfinite values,
  inconsistent inverse and source/intermediate/native residual failures.
- [x] Test nonlinear circle rows disallow raw source-parameter reuse; line rows
  retain distinct kernel-reported evidence without certifying unqueried values.
- [x] Implement checked query-count policy (OM9 limit4096), explicit tolerances,
  stable finite distance checks and independent owned report values.
- [x] Test C ABI sizes/offsets and unknown flags/statuses; run focused Rust tests.

## Task 2: exact-lineage native curve adapter

**Files:** Modify `Gui/ThreeDmCurves.cpp`; create
`Gui/CurveConversionContract.h/.cpp`; modify `cmake/OpenNURBS.cmake`.

**Interfaces:** `importCurveObserved(const ON_Curve&, tolerance, requestedParameters)`
returns independently owned native shape plus copied query/conversion facts.
Existing `importCurve` and `curve3d` reuse the same conversion/factory helper and
retain their supported geometry behavior. New query calls are synchronous and
bounded before result-vector allocation. C++ catches native mapping/construction
failures through the existing native error boundary; portable policy is in Rust.

- [x] Preserve GetNurbForm return1/2 instead of reducing the result to bool.
- [x] Refactor the existing owned NURBS → OCCT factory without duplicating solvers.
- [x] Keep exact converted basis lineage through clamping and target construction.
- [x] Use native forward/inverse APIs, checking bool results before consuming any
  output; evaluate mapped dependent points and raw-parameter diagnostics separately.
- [x] Pass copied scalars through Rust validation and return only validated query
  facts. Empty query batches preserve old geometry behavior, no mapping promise.

## Task 3: native acceptance, review and synchronization

**Files:** Create `tests/native/rhino_core_curve_parameters.cpp`; modify
`tests/native/CMakeLists.txt`; add scoped validation/review records and update
`docs/features/RCORE-01.md`, spec/catalog/ledger/manifest as actually demonstrated.

- [x] Run native RED/GREEN fixtures on independent R10 analytic circle (including
  domain[2,7]), line, rational NURBS and boundary/nonboundary query parameters.
  Save mapped/inverse/dependent-point results, raw mismatch and source CRC/basis.
- [x] Inject native false mapping with finite sentinels; no result is published.
  Test malformed parameters, explicit tolerances and query budget boundaries.
- [x] Require requested mapped point agreement within declared bounds, source
  immutability and no unsupported global/affine parameter claim.
- [x] Run focused/full Rust and Clippy, matching native/SDK build and applicable
  curve, trim mapping and working-exchange application regressions.
- [x] Independently review nonlinear lineage, statuses, limits, ownership and
  preservation claims. Synchronize exact verified clauses only, commit locally,
  and continue the remaining all-P0 contracts.
