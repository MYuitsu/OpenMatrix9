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

- [x] Enumerate actual supported native curve/surface/proxy/BRep/cage/member and
  trimmed OCCT entrypoints from the current source; distinguish preserved opaque
  objects from geometry consumed by a solver. Record actual SDK child accessors
  and capacity semantics before traversing them.
- [x] Reuse native basis counts/values and write safe Rust policy regressions for
  finite/weights/knots/product overflow, bounded depth/node count, cycles and
  aggregate logical bytes. Keep generated spline policy separate.
- [x] Define copied/borrowed ABI ownership, alignment, one-call exclusive state,
  failure behavior and native-exception boundaries; test layout and bad inputs.

## Task2 — native preflight before copies and output allocation

- [x] Author failing native fixtures with valid live input owners but invalid
  reachable NURBS values: direct, CurveProxy, nested PolyCurve, SurfaceProxy,
  revolution profile and BRep support/trim/edge storage and readable cross-index
  references. Restore injected fields before source destruction when a fixture
  would violate SDK owner cleanup.
- [x] Instrument the actual relevant copy/GetNurbForm/kernel-construction entries
  for fixtures; distinguish those entries from input setup/extraction allocation.
- [x] Add minimal traversal adapters using the Rust policy and existing basis
  validator. Preflight roots before source native copy/CRC/domain/proxy inspection
  wherever those calls can read nested raw storage or create output.
- [x] Add positive rational, periodic, proxy reversal/subdomain and trimmed BRep
  controls proving no change to source bytes, working output or parameter policy.

## Task3 — host mutation boundary and checkpoint

- [ ] Invalid host requests preserve exact object/property inventory, source bytes
  and Undo/Redo; include curve/surface grid overflow and malformed owned data.
- [x] Build exact SDK/native targets; run full Rust and appropriate existing
  native/FreeCAD controls on frozen source and record process/module fingerprints.
- [x] Obtain independent review of actual native preflight ordering, ownership and
  limits. Keep unproved archive/member/worker/allocation clauses open explicitly.
- [ ] Synchronize earned acceptance/report/spec/README/manifest/live ledger and
  save a local checkpoint, then continue remaining P0 requirements.

## Task4 — archive member preflight before staging

Task3 follow-up: the exact property-value oracle and prepared-CAD mutation
gate are validated in Task7 below. Fake native count/grid overflow remains
separate initialized native-fixture evidence; no Python count-injection API or
global allocation/metadata coverage is claimed by the host fixture.

- [x] Add failing integration fixtures through `readArchive` for an invalid NURBS
  curve inside a transformed block and an invalid surface before `BrepForm`;
  instrument the actual archive `Duplicate`, `BrepForm`, and signed-placement
  orientation call sites; require the specific preflight error in each fixture.
- [x] Validate supported source curves, BReps, and surfaces before archive
  staging copies or surface-to-BRep construction, reusing the existing input
  tree and converters.
- [x] Run the focused native fixture and plugin-only/shared-host regression;
  record that raw `ONX_Model::Read` decoding and aggregate allocator coverage
  remain open.

## Task5 — Explode and NURBS cage preflight after archive decode

- [x] Add archive fixtures proving an invalid block curve rejects before
  `Duplicate`/`GetNurbForm`, and malformed cages reject before `IsValid` or
  `IsoSurface`, including finite knots whose adjacent difference overflows.
- [x] Validate supported Explode curve/BRep/surface owners before clone and
  add bounded cage count/product, knot/domain, stride/storage and CV/weight
  checks before SDK validity/evaluation; keep the existing converters.
- [x] Run RED/GREEN native proof, six related CTests, positive Explode fixtures,
  plugin-only build and selected-plugin FreeCAD host test; obtain independent
  review. Keep raw `ONX_Model::Read`, global allocation, worker budget and other
  analytic/cage-edit clauses open.

## Task6 — Mesh and PointCloud post-decode preflight

- [x] Add Rust-backed live-array count/capacity, logical-byte and supported
  value/index checks before Mesh SDK validity, archive staging copy and Explode
  clone; keep the existing Mesh and PointCloud adapters.
- [x] Add RED/GREEN fixtures for malformed archived mesh faces and PointCloud
  capacities, plus valid archive Mesh/PointCloud and unsupported PointCloud
  Explode controls.
- [x] Run six related CTests, positive Explode integration, plugin-only shared
  SDK build and fresh selected-plugin host smoke. Record that raw
  `ONX_Model::Read`, process-wide allocation and aggregate worker memory remain
  open.
- [x] Obtain independent review of array-envelope and nested owner checks; no
  code-level blocker remained for this bounded post-decode increment.
- [x] Refresh evidence with the current 92-check fixture, six focused CTests,
  and a host smoke tied to the current plugin hash; preserve raw decode and
  allocator limitations as open work.

## Task7 — prepared CAD before transaction opening

- [x] Reproduce a live invalid CAD batch entering the document: four extra
  objects, Undo1→2, Redo1→0. Keep the specific native basis rejection oracle.
- [x] Reuse Rust/native input-tree validation for all prepared CAD rows and
  preserved affine previews before BRepCheck/property assignment/transaction.
- [x] Reject worker entry before document lookup; verify safe RED→GREEN.
- [x] Compare exact persisted property member bytes/types/statuses for the
  document, every object and view provider without computed Shape getters;
  verify detached source bytes and executable Undo/Redo.
- [x] Independently review native ordering, wrappers and binding semantics.
  Fix actual storage vs overridden Python container length/iteration mismatch;
  observe safe container RED and final55-check GREEN.
- [x] Build only OpenMatrix9Gui using the shared SDK; final plugin host controls
  pass55 gate +32 modeling +19 preservation. Focused Rust47 and six native
  CTests pass; public-source audit passes.
- [x] Synchronize earned evidence/spec/README/manifest/ledger and retain the
  requirement-by-requirement chapter audit.

Ruling: require exact built-in prepared containers and intrinsic pair length —
the supported bootstrap returns those containers, while subclasses can change
the data consumed by binding or invalidate raw indexing — cost if wrong: custom
callers must normalize their containers before invoking the native import API.

Task3's local checkpoint and broader allocation/metadata/aggregate requirements
remain open. Task7 is a validated mutation-boundary increment, not completion
of RCORE-01. Continue common topology snapshots and the remaining P0 contract.
