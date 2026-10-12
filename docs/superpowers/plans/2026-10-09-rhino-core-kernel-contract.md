# Rhino core kernel contract implementation plan

> For agentic workers: execute the authorized P0 contracts with
> superpowers:executing-plans; request an independent review before acceptance.

**Goal:** Reuse existing geometry converters and add missing shared Rust basis,
representation and operation contracts for their supported branches.
**Architecture:** Rust owns portable validation, descriptors, operation policy and
conversion facts. C++ extracts native data and calls existing OCCT/openNURBS
converters. Existing object identity guards and transactions are reused.
**Tech stack:** Safe Rust, bounded borrowed C ABI, existing FreeCAD/OCCT/openNURBS,
Python application fixtures and existing CMake targets.
**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`.
**Baseline:** `fcdb0ac`; all P0 remains in progress.

## Constraints and reuse

Preserve command IDs and disabled unsupported commands. Do not replace kernel
solvers, importers, topology classification, transactions or the identity registry.
Reuse `core_objects::Stamp`, `captureObject/verifyObject`, existing rational curve
and surface converters, seam pairing and shell/cavity classification. Keep the
generated spline limits distinct from native import policies. Native ownership is
a permanent adapter exception; Rust retains independent data, never host pointers.
The archive parser already allocates native input data. Record that boundary
separately from rejecting invalid basis before output construction; narrow gates
cannot close the literal all-consumer T07 allocation requirement.

The mapped guide PDF25–31 and exact Rhino5 help were read during reuse preparation.
Documented representation definitions do not prove solver or runtime parity.
Rhino5 currently crashes on isolated startup; no new runtime parity is claimed.

## Task 1: exact fixtures on existing converters

Files: `tests/native/rhino_core_kernel_contract.cpp`, `tests/native/CMakeLists.txt`,
`tests/rhino_core_kernel_contract_smoke.FCMacro`.

- [x] Integrate independently authored CircleR10, square20×20/holeR3 and open
  cylinderR4/H7 fixtures. Record basis, trim roles, distinct edge identities,
  ordered use occurrences and connected free-boundary components.
- [x] Compile/run against matching libraries. Require analytic radius/circumference,
  adaptive area `400−9π` and `56π`, hole exclusion and two circular free boundaries;
  investigate failures before changing implementation or oracles.
- [x] Verify source immutability, one import transaction, Undo/Redo and separate
  cold FCStd reopen. Record sampled deviation as sampled; do not certify a global
  maximum, parameter map, full T09/T11 or all ownership from these observations.

## Task 2: reusable basis validation

Files: `rust/src/nurbs_basis.rs`, `rust/src/spline.rs`, `rust/src/lib.rs`,
`rust/tests/nurbs_basis.rs`, `rust/tests/spline_basis.rs`.

Interfaces: borrowed `Axis`, `CurveBasis`, `SurfaceBasis` and explicit
`BasisPolicy`; `validate(policy) -> Result<(), BasisError>`; checked count/byte
preflight before buffer allocation. Existing `spline::Basis::validate` remains a
wrapper under its current generated policy and supported messages.

- [x] Write/run RED tests for general rational curves and U-major surface grids,
  periodic/unclamped axes, nonfinite/mismatched data, overflow, domain and policy
  bounds; prove native policy can accept valid data beyond generated limits.
- [x] Extract existing structural axis/weight checks rather than duplicate them.
  Preserve generated spline behavior and test all old spline/Circle consumers.

## Task 3: representation and operation policy

Files: `rust/src/geometry_contract.rs`, `rust/tests/geometry_contract.rs`.
Interfaces: versioned owned descriptors, typed representation/topology facts,
`GeometryRef` containing the existing `Stamp`, `ConversionReport` distinguishing
locus/parameterization/topology preservation, and a typed operation/options/backend
registry. Unknown/uninspected facts are explicit and unknown option combinations
reject; preserve the legacy coarse capability API while adding detailed gates.

- [x] RED/GREEN inspection and existing working-exchange operation policies.
- [x] Test that periodic/closed surface facts alone never imply a solid, and that
  conversion permission does not invent preservation or a verified parameter map.

## Task 4: native gates and companion extraction

Files: new small native contract adapter and Rust FFI module, existing
`ThreeDmCurves.cpp`/`ThreeDmSurfaces.cpp`/CMake lists.

- [x] Gate raw/converted rational basis before OCCT arrays and export objects;
  validate after clamping/convention changes. Keep native validity and geometry
  operations in their current adapters and record conversion facts honestly.
- [x] Add read-only companion extraction with generation/unit/world provenance;
  bound all arrays and document pointer lifetime/alignment/thread requirements.
- [x] Audit rational consumers and keep uncovered allocation/consumer branches open.

## Task 5: review, evidence and synchronization

- [x] Run focused/full Rust tests, matching SDK and relevant native trim/seam/shell
  regressions, exact GUI/cold fixtures and public-source audit.
- [x] Independently review numerical/topology oracles, gate placement, ownership,
  ABI bounds and preservation claims. Store hashes and exact unsupported clauses.
- [x] Synchronize only demonstrated spec/catalog/ledger/acceptance scope; commit
  locally and continue remaining P0 contracts.
