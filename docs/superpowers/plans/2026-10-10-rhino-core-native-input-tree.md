# Rhino core native NURBS input-tree preflight

> Execute the authorized all-P0 plan with superpowers:executing-plans. This package
> continues T07; an increment is not a waiver of its remaining contracts.

**Goal:** Validate reachable original NURBS data before existing native converter
copies/GetNurbForm/output constructors, including proxy, nested curve and BRep
surface/3D-curve/2D-trim inputs. Reuse the current shared Rust basis validator and
the existing SDK conversion; do not replace native curve/surface solvers.

**Observed gap:** `ThreeDmCurves::nurbs` validates only a direct ON_NurbsCurve
before GetNurbForm. A proxy or nested owner can hide that data. `importBrep`
copies the entire source before child basis checks. `importSurface` has the same
direct-only gate for a NURBS surface/revolution profile. Prior T07 Hatch evidence
covers owned loop JSON but explicitly leaves these native owner paths open.

**Architecture:** Safe Rust owns bounded traversal/count/checked aggregate input
policy, with no retained host pointer. C++ extracts live borrowed SDK relationships
and native storage envelopes; every reachable rational basis reuses checkedBasis.
Run the entire preflight before copy/evaluation/construction in each root consumer.
Existing conversion topology, periodic conventions, source ownership and working
V5 dispatch remain authoritative. Unsupported owner classes fail with a precise
reason if safe extraction cannot be established.

**Boundary:** Incoming native owners must already be initialized, readable and
immutable during the synchronous call. This does not validate dangling pointers
or claim archive decoding allocated no native source object. Input extraction
buffers, native kernel output constructors/copies, raw archive/process memory and
aggregate worker budgets must be recorded separately. Avoid interpreting logical
count bounds as an instrumented certificate of every allocator.

## Task1 — exact consumer/owner audit and policy regression

- [ ] Enumerate actual supported native curve/surface/proxy/BRep/cage/member and
  trimmed OCCT entrypoints from the current source; distinguish preserved opaque
  objects from geometry consumed by a solver. Record actual SDK child accessors
  and capacity semantics before traversing them.
- [ ] Reuse native basis counts/values and write safe Rust policy regressions for
  finite/weights/knots/product overflow, bounded depth/node count, cycles and
  aggregate logical bytes. Keep generated spline policy separate.
- [ ] Define copied/borrowed ABI ownership, alignment, one-call exclusive state,
  failure behavior and native-exception boundaries; test layout and bad inputs.

## Task2 — native preflight before copies and output allocation

- [ ] Author failing native fixtures with valid live input owners but invalid
  reachable NURBS values: direct, CurveProxy, nested PolyCurve, SurfaceProxy,
  revolution profile and BRep support/trim/edge storage. Restore injected fields
  before source destruction when a fixture would violate SDK owner cleanup.
- [ ] Instrument the actual relevant copy/GetNurbForm/kernel-construction entries
  for fixtures; distinguish those entries from input setup/extraction allocation.
- [ ] Add minimal traversal adapters using the Rust policy and existing basis
  validator. Preflight roots before source native copy/CRC/domain/proxy inspection
  wherever those calls can read nested raw storage or create output.
- [ ] Add positive rational, periodic, proxy reversal/subdomain and trimmed BRep
  controls proving no change to source bytes, working output or parameter policy.

## Task3 — host mutation boundary and checkpoint

- [ ] Invalid host requests preserve exact object/property inventory, source bytes
  and Undo/Redo; include curve/surface grid overflow and malformed owned data.
- [ ] Build exact SDK/native targets; run full Rust and appropriate existing
  native/FreeCAD controls on frozen source and record process/module fingerprints.
- [ ] Obtain independent review of actual native preflight ordering, ownership and
  limits. Keep unproved archive/member/worker/allocation clauses open explicitly.
- [ ] Synchronize earned acceptance/report/spec/README/manifest/live ledger and
  save a local checkpoint, then continue remaining P0 requirements.
