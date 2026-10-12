# Phase2 Rust-owned core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Move current Phase2 portable validation/state/data/index/cache/budget decisions into safe Rust, remove production Python PointsOn UI, and preserve accepted native CAD workflows.

**Architecture:** Extend the existing Rust static library with owned curve/session and snap-index modules behind a small typed C ABI. FreeCAD/Qt/OCCT/openNURBS/Coin adapters perform only native inspection, projection, kernel construction/checks, widget hosting and GUI transaction commits. Stage cache/session updates before publication; no native pointers are stored in Rust.

**Tech Stack:** Existing Rust2024/staticlib/std + C++23/Qt/OCCT/openNURBS/FreeCAD SDK; existing Python/Rhino application test macros remain. No new Qt or CAD-kernel stack.

**Spec:** [Approved design](../specs/2026-10-09-phase2-rust-owned-core-design.md).

Status: user approved this plan and inline/native execution on2026-10-09 (“Duyệt kế hoạch, triển khai inline”); implementation accepted_scoped after matching runtime gates and one independent final review.

## Global Constraints

- Source H:/FreeCAD-src/build/om9-dev; retain pre-existing broken Git ruling, no commit/push/primary product overwrite. Snapshot accepted product source/runtime/manifest before any code edit into H:/FreeCAD-src/build/phase2-rust-source-backup-20261009 (fail if already present; use a fresh timestamp suffix). Build migration module in H:/FreeCAD-src/build/om9-phase2-rust-module and runtime in H:/FreeCAD-src/build/om9-phase2-rust-sdk; historical om9-phase2-sdk and its evidence remain immutable.
- Safe Rust owns portable logic and copied data; C++ native objects remain native-side owned. No Rust raw Qt/document/kernel pointer storage, no worker mutable-document access, no panic/exception unwinding across ABI. Explicit borrowed-call versus owned-session buffer lifetimes and same-side free.
- CV finite values within1e9, degree1..25, poles/knots2..4096,3 coordinates per pole, positive weights, strictly increasing distinct knots, integer multiplicities1..degree+1, compatible ordered active domain. OCCT retains kernel constructibility/native validity checks.
- Join2..16 distinct open native inputs,64-edge cap, one nonbranching chain/cycle with every edge uniquely traversed; inputs preserved and atomic rejection/one Undo. Rebuild16 curves/256 poles, degree1..11, PointCount>Degree, default DeleteInput=true and dependent/stale protection.
- Snap radius8 logical pixels,64 objects/2048 candidates per object/8192 total, allocation caps before extraction. End/Mid/Point exclude heavy Mesh/cloud/SubD preview arrays; incomplete never certifies a picked point; manual/typed fallback remains.
- Preserve IDs/menu/mouse/CMD parity, local CV editor coordinates, Undo/Redo/Cancel/FCStd/currentV5, rational/periodic/placed geometry, clipboard and default60% workers bounded by tasks/RAM.
- Rhino5/V5/openNURBS pin eb92af3ba1806b0a34a99aba0d3bda83e3d46083. Ring bounds0.001mm; area/volume max(0.001,0.0001*abs(reference)); FCStd numeric1e-10 with exact discrete fields.
- Run shell commands through rtk. Existing docs/skill tools do not prove migration; Rust compile/tests do not certify native memory safety.

## Review Focus

1. Malformed degree/multiplicity/dimensions/null/alignment/length/overflow inputs reject before unsafe dereference or native mutation — Task1 FFI tests.
2. Cancel/doc close/delete/Undo/replaced object after draft edit invalidates Rust session and cannot commit stale/native-pointer results — Task2/5 real UI tests.
3. Native failure midway through camera index reconstruction cannot publish partial clean cache — Task3 staged-build abort tests, Task4 host invalidation.
4. Nonuniform rational/periodic curves and reflected/nested Links preserve geometry, projection and local/world frame exactly once — Task2/4 and actual Rhino gate.
5. Mode changes and remaining per-object/total budgets do not reuse candidates under an incompatible cache key, and Tie/HiDPI/manual fallback remain stable — Task3/4 combined-mode and scale2 tests.

## Shared interfaces

Create rust/src/phase2_curve.rs for `Basis {degree:usize,periodic:bool,poles:Vec<[f64;3]>,weights:Vec<f64>,knots:Vec<f64>,multiplicities:Vec<usize>,first:f64,last:f64}` and `CurveError`; `Basis::validate(&self)->Result<(),CurveError>`, `validate_wire(vertex_degrees:&[usize],ordered_edges:&[u64],expected_edges:usize)->Result<(),CurveError>`, `join_options(inputs:usize,edges:usize)->Result<(),CurveError>`.

Create rust/src/phase2_session.rs for `Witness {document:u64,object:u64,generation:u64,signature:String}`, `CurveSession::new(witness:Witness,basis:Basis)->Result<Self,CurveError>`, `replace_draft(&mut self,basis:Basis)->Result<(),CurveError>`, `commit_request(&self,current:&Witness)->Result<Basis,CurveError>`, `cancel(&mut self)`. Returned request owns copies; canceled/stale sessions error.

Create rust/src/phase2_snap.rs for `ObjectToken=u64`, `Point=[f64;3]`, `ScreenPoint=[f64;3]`, `ViewKey {document:u64,view:u64,generation:u64,width:u32,height:u32,camera:[f64;16]}`, `Row {object:ObjectToken,bounds:[f64;4]}`, `Limits {objects:usize,per_object:usize,total:usize}`, `SnapIndex::begin_build(key:ViewKey)`, `add_row(row:Row)->Result<(),SnapError>`, `finish_build()->Result<(),SnapError>`, `abort_build()`, `invalidate()`, `near_objects(cursor:[f64;2],radius:f64,limits:Limits)->Result<Vec<ObjectToken>,SnapError>`. `SnapQuery` owns mode/caps/candidates/ranking state and exposes `remaining_budget(object:ObjectToken)->usize`, `consume(object:ObjectToken,mode:u32,points:&[(Point,ScreenPoint)],complete:bool,visited:usize)->Result<(),SnapError>`, `finish()->SnapResult`. Cache key includes object/mode/extraction budget and view/geometry generation. Results own points and explicit complete/picked/telemetry/error.

Create rust/src/phase2_ffi.rs and Gui/Phase2Rust.h as matching fixed-layout ABI. Tokens are u64, counts size_t/usize, numeric buffers borrowed for one call only; no Vec/String crossing ABI. FFI session/index registry owns Rust values, returns monotonic generation-tagged u64 handles. Input records expose scalars, pointer+count pairs and row-dimension validity; Rust rejects invalid typed scalar/shape/budget before copying. Native adapter may perform syntax decoding only.

Exports must include `om9_phase2_basis_validate(const Om9BasisInput*)->bool`, `om9_phase2_wire_validate(const Om9WireInput*)->bool`, `om9_phase2_session_create(const Om9WitnessInput*,const Om9BasisInput*)->u64`, `om9_phase2_session_replace(u64,const Om9BasisInput*)->bool`, `om9_phase2_session_check(u64,const Om9WitnessInput*)->bool`, `om9_phase2_session_drop(u64)`, `om9_phase2_snap_create()->u64`, `om9_phase2_snap_drop(u64)`, `om9_phase2_error(u64,char*,size_t)->size_t`. Snap build/query/budget/cache accessors map the safe interfaces above, using fixed-layout typed records and caller-owned output buffers; publish exact matching field/prototype declarations in Phase2Rust.h before any caller change. Keep Rust errors inspectable without borrowed String pointers. Native registry errors use status values, never fabricated success.

ABI record contracts, written as C++ declarations and mirrored with `#[repr(C)]` in Rust:

```cpp
struct Om9BasisInput { double degree; uint32_t periodic; const double* poles; size_t poles_len; const double* weights; size_t weights_len; const double* knots; size_t knots_len; const double* multiplicities; size_t multiplicities_len; double first,last; };
struct Om9WitnessInput { uint64_t document,object,generation; const uint8_t* signature; size_t signature_len; };
struct Om9WireInput { const uint64_t* vertex_degrees; size_t vertex_degrees_len; const uint64_t* ordered_edges; size_t ordered_edges_len; size_t expected_edges; };
struct Om9ViewKey { uint64_t document,view,generation; uint32_t width,height; double camera[16]; };
struct Om9SnapRow { uint64_t object; double bounds[4]; };
struct Om9SnapLimits { size_t objects,per_object,total; };
struct Om9SnapPoint { double world[3],screen[3]; };
struct Om9SnapResult { uint32_t complete,picked; double point[3]; size_t nearby_objects,visited_objects,visited_topology,generated,max_per_object; };
```

`poles_len` counts doubles and must be divisible by3; all basis list lengths must agree before allocation. Raw numeric degree/multiplicities stay double until Rust rejects fractional/out-of-range values. `periodic` accepts only0/1. UTF-8 signature maximum512 bytes. Invalid alignment/null/count/caps are rejected before slices; arbitrary address validity remains a caller obligation. Errors are copied from per-handle state, or thread-local state for handle0/failed creation, never a mutable process-global error. Monotonic handles are never reused and exhaustion errors; dropping an index invalidates its queries. Rust release retains panic=abort: validation removes ordinary panic paths; allocation failure/native dependencies are not certified safe.

Add exact snap exports: `om9_phase2_snap_begin(u64,const Om9ViewKey*)->bool`, `om9_phase2_snap_add(u64,const Om9SnapRow*)->bool`, `om9_phase2_snap_finish(u64)->bool`, `om9_phase2_snap_abort(u64)`, `om9_phase2_snap_invalidate(u64)`, `om9_phase2_snap_matches(u64,const Om9ViewKey*)->bool`, `om9_phase2_query_create(u64,const double* cursor_xy,double radius,uint32_t modes,const Om9SnapLimits*)->u64`, `om9_phase2_query_objects(u64,uint64_t* output,size_t capacity)->size_t`, `om9_phase2_query_budget(u64,uint64_t object,uint32_t mode)->size_t`, `om9_phase2_query_cached(u64,uint64_t object,uint32_t mode)->bool`, `om9_phase2_query_consume(u64,uint64_t object,uint32_t mode,const Om9SnapPoint*,size_t count,uint32_t complete,size_t visited)->bool`, `om9_phase2_query_abort(u64)`, `om9_phase2_query_finish(u64,Om9SnapResult*)->bool`, `om9_phase2_query_drop(u64)`. Object output returns required capacity and writes only with sufficient capacity; native read reports needed capacity before allocating within64. Cached consumption updates budgets/ranking once; duplicate consume is rejected. Query creation freezes selected tokens and caches; completion publishes new candidates only if the index generation still matches. Native exceptions abort query, preserving incomplete/no-picked semantics. Safe APIs additionally provide `SnapIndex::start_query(&mut self,cursor:[f64;2],radius:f64,modes:u32,limits:Limits)->Result<SnapQuery,SnapError>` and `SnapIndex::publish_query(&mut self,query:SnapQuery)->Result<SnapResult,SnapError>`.

Rebuild portable interface in phase2_session.rs: `RebuildOptions {degree:usize,point_count:usize,delete_input:bool}`, `RebuildInput {witness:Witness,has_dependents:bool}`, `RebuildSession::new(inputs:Vec<RebuildInput>,options:RebuildOptions)->Result<Self,CurveError>`, `replace_options(&mut self,options:RebuildOptions)->Result<(),CurveError>`, `commit_request(&self,current:&[RebuildInput])->Result<RebuildOptions,CurveError>`, `cancel(&mut self)`. ABI mirrors `Om9RebuildOptions {size_t degree,point_count; uint32_t delete_input;}` and `Om9RebuildInput {Om9WitnessInput witness; uint32_t has_dependents;}`; exports `om9_phase2_rebuild_create(const Om9RebuildInput*,size_t,const Om9RebuildOptions*)->u64`, `om9_phase2_rebuild_replace(u64,const Om9RebuildOptions*)->bool`, `om9_phase2_rebuild_check(u64,const Om9RebuildInput*,size_t)->bool`, `om9_phase2_rebuild_drop(u64)`. Cancel drops the handle; commit first checks unchanged witnesses/current dependencies, then performs native kernel/transaction work. Current dependency facts come from the host; Rust decides the DeleteInput guard.

### Task1: Owned curve/graph core and checked FFI

**Files:** Create rust/src/phase2_curve.rs,phase2_session.rs,phase2_ffi.rs; Gui/Phase2Rust.h. Modify rust/src/lib.rs,Gui/RustBridge.h. Tests rust/tests/phase2_curve_core.rs,phase2_ffi_contract.rs.

**Consumes:** existing modeling_exchange eligibility/60% policy and spline fit. **Produces:** Basis/CurveSession/graph and ABI interfaces above.

- [x] Write tests: rational basis preserves fields; degree0/26, negative/zero weight, unordered/repeated knot, noninteger multiplicity/raw degree, malformedXYZ, nonfinite/over1e9, incompatible counts/domain and4097-pole input rejected; exact4096 boundary does not panic. Closed/native constructibility remains kernel check.
- [x] Run `rtk cargo test --manifest-path rust/Cargo.toml --test phase2_curve_core --test phase2_ffi_contract`; observe missing-module/API or named behavioral RED before implementation.
- [x] Implement safe owned types, validation/branch+complete unique traversal rules, session stale/cancel ownership and minimal FFI. Checked counts/products/null/alignment before slices, checked handle/index access, no unwrap/panic paths. Keep error handling separate from numeric decisions.
- [x] Run same tests GREEN plus full cargo tests; FFI null/overbudget/alignment rejection tests must not dereference invalid memory. Verify copied input survives caller-buffer mutation, dropped/unknown handles fail, fresh handles cannot resurrect dropped sessions.
- [x] Record exact RED→GREEN/source changes/ownership contracts in migration ledger; preserve snapshot instead of committing under Git ruling.

### Task2: Native curve adapters and Qt PointsOn backed by Rust

**Files:** Modify Gui/ModelingCurveEditor.{h,cpp},Gui/CurveGeometry.{h,cpp},Gui/CurveController.cpp,Gui/CMakeLists.txt,CMakeLists.txt; create Gui/ModelingCurveEditorDialog.{h,cpp}. Remove production ModelingCurveEditor.py and its packaging/import path after equivalent behavior passes. Tests extend modeling_curve_editor_smoke.FCMacro/modeling_curve_editor_ui_smoke.FCMacro and add modeling_rust_ownership_smoke.FCMacro.

**Consumes:** Task1 Basis/session/FFI. **Produces:** native `TopoDS_Shape publishedSplineShape()`, `App::DocumentObject* createCurveFeature(App::Document&,const TopoDS_Shape&,const char*)`, native curve sampler `std::vector<std::array<double,3>> sampleCurve(const TopoDS_Shape&,size_t)`, original PointsOn menu/CMD route with same widget objectNames.

- [x] Write real-host RED assertions: portable basis validation uses Rust core, PointsOn works without importing production ModelingCurveEditor Python, invalid numeric requests cause no transaction, canceled/stale Rust sessions cannot commit after object replacement/doc/workbench switch. Keep legacy current native/periodic/rational/placement assertions.
- [x] Run dedicated owned host macro with current accepted binary and record RED for the missing Rust-backed path, not an SDK/path failure.
- [x] Decode typed UI/API inputs, send copied model/drafts to Rust, then use OCCT to construct/check and native FreeCAD to commit once. Qt widgets only display/update Rust model and relay native lifecycle. Replace Python Part spline publication/feature creation with native adapters and update all CurveController callers.
- [x] Build isolated migration runtime; run CV+CVUI+ownership+InterpCrv+Join host tests GREEN, plus Rust tests and targeted native periodic/seam regressions. Test Python editor absence by process module/import witness and packaged resource inventory, not string matching alone.
- [x] Record native exceptions/ownership and exact runtime/hash proof; no incomplete migration success claim.

### Task3: Rust-owned snap index/cache/query engine

**Files:** Create rust/src/phase2_snap.rs and rust/tests/phase2_snap_core.rs; extend phase2_ffi.rs,Gui/Phase2Rust.h,lib.rs.

**Consumes:** safe eligibility and typed owned buffers/handles from Task1. **Produces:** exact safe snap interfaces above and native-accessible index/query lifecycle, budget/cache/telemetry accessors.

- [x] Write tests:10k remote rows select only nearby object tokens,65 local objects return incomplete,2048-per/8192-total allocation budget before consume, combined modes preserve residual caps/cache-key distinctions, nearest/tie/depth/radius behavior, generation/camera/viewport invalidation, nonfinite/overflow/invalid limits reject.
- [x] Run `rtk cargo test --manifest-path rust/Cargo.toml --test phase2_snap_core` and observe RED.
- [x] Implement owned Vec/HashMap rows/tiles/candidates with staged publication. Begin/abort rebuild makes current generation incomplete; a partial build never becomes clean. Keep raw native pointers out; caller-owned typed output buffers report required capacity and do not overflow.
- [x] Same suite GREEN, including injected failed build after partial rows, failed native extraction after a prior best point, and retry under identical ViewKey. Incomplete clears picked. Run full Rust suite and ABI bounds/ownership tests.
- [x] Record Rust ownership and tests before native integration.

### Task4: Thin native snap/classification adapter

**Files:** Modify Gui/SnapQueryIndex.{h,cpp},SnapObjectInfo.cpp,CoreSnapGeometry.{h,cpp},CoreSnaps.cpp; extend modeling_snap_cache_smoke.FCMacro/modeling_snap_smoke.FCMacro and ownership macro.

**Consumes:** Task3 typed index/query ABI. **Produces:** existing querySnap3dm/classifySnap3dm/candidate endpoints, unchanged real mouse/CMD adapters and result fields, backed by Rust-owned state.

- [x] Write/extend real-host regression for Rust-backed index/query witness, all invalidation signals, partial-build incomplete behavior, remaining mode budgets and actual scale2 mouse snap. Preserve million Mesh/cloud and10000-edge topology cap fixtures; counters remain labeled uninstrumented unless independently observed.
- [x] Run matching current pre-migration binary for named ownership-path RED; use native/Rust injected failure proof for exceptional cache publication instead of claiming arbitrary native fault injection.
- [x] Keep native runtime type/Link/visibility/camera projection/OCCT extraction only; feed copied numeric snapshots/tokens to Rust and delegate row storage/tile lookup/cache/budget/ranking. Native failure aborts the Rust build/query; native identity→object lookup is checked on GUI thread and not retained in Rust.
- [x] Run snap/classifier/cache/heavy/performance/End/Point/scale2 GREEN on migration binary.500 warm queries p95<=16ms separately for10k remote CAD and1,000,002-point mesh+1,000,000-point cloud; record hardware/raw results and cold rebuild distinction.
- [x] Record adapter limits and source/runtime/projection proof; remove the old C++ long-lived row/geometry ownership path.

### Task5: Rust Rebuild decisions/session and native complete-curve sampling

**Files:** Extend phase2_session.rs/phase2_ffi.rs/Phase2Rust.h and tests; modify Gui/CoreRebuild.{h,cpp},CurveGeometry.{h,cpp},ModelingCurveEditor.cpp. Extend topology/workflow/ownership macros.

**Consumes:** Task1 graph/session and Task2 native shape/sampler; existing Rust spline::rebuild. **Produces:** Rust-owned option/draft/stale/cancel decisions with native selection/kernel/preview/transactions only.

- [x] Write Rust+host tests for stale/canceled options, PointCount<=Degree,16-curve boundary, dependent DeleteInput rejection, branched/incomplete traversal rejection, full-chain endpoint preservation/default DeleteInput/Undo/V5. Closed periodic native sampler must cover full domain.
- [x] Run new ownership/session tests RED before replacing existing Rebuild path.
- [x] Move portable options/state/budgets/graph decisions to Rust. Native selection and Shape lifetime/dependency witnesses stay GUI-bound; replace Python getObject/Shape/Edge.discretize/feature-creation calls with native FreeCAD/OCCT adapters. Keep Rust fitting and original IDs/menu/CMD/mouse/preview behavior.
- [x] GREEN Rust/full native/real topology+workflow+CV regressions; rational/periodic/placed source-independent FCStd and currentV5 outputs remain valid. Confirm no Python Part adapter remains in migrated product route.
- [x] Record ownership/native exceptions and preserve complete original input on failure.

### Task6: Packaging, complete application gate and final review

**Files:** tests/run_modeling_phase.ps1 and evidence tooling when needed; native CMake/tests only for additional ABI proofs; docs/validation/modeling-phase-2-rust/{summary,requirements,application-evidence,ownership-map}.json and human report; progress/roadmap/README/plan checkboxes. Migration-specific build/runtime helpers keep historical accepted reports immutable.

- [x] Write packaging/resource and ownership gate assertions and bind missing migrated-runtime paths as RED. Inspect matching source/runtime/SDK caches and register all prepared batches; no invented future total.
- [x] Build/capture/finish separate migration runtime; verify source/module/helper/UI/script hashes, native C ABI/layout and explicit removed Python UI packaging. Safe-core code has no unsafe; FFI/native unsafe boundary is documented and tested.
- [x] Run full Rust/Python/native suites and19 original host batches + new ownership batch +HiDPI2. Run actual owned Rhino5 rational/periodic/placed3 fixtures Export Selected→CV/weight/knot/new/Rebuild/Join→currentV5→Open/SaveAs, then actual FreeCAD saved reread. Exit0 and geometry/lifecycle parity required; historical tests do not count as new.
- [x] One fresh-context whole-branch review on the most capable available model under executing-plans; re-grade user effects, one Critical/Important RED→GREEN fix pass and complete fresh verification, no second reviewer. Defer true Minors explicitly; rule each declined scope.
- [x] Publish scoped acceptance only when every migration requirement and matching binary/application report is green. Retain snapshots/ledger under broken Git ruling; update docs and approved skill evidence without silently broadening business behavior. Full openNURBS/Phase3–5 and7 packages remain open; pending prepared count and total future unknown reported.

## Self-review and handoff

Coverage: owned curve/session/FFI→Task1; native CV UI/spline adapters→Task2; snap data/query/cache→Tasks3–4; Rebuild/Join decisions and Python Part removal→Task5; classifier/worker existing Rust policies retained→Tasks1/4; packaging/native/actual application gates and evidence→Task6. All five Review Focus classes have owning test steps. Self-review fixed missing raw-input records, snap/query and Rebuild signatures, error concurrency, caller pointer obligations and separate snapshot/build/runtime locations. Shared signatures are defined once and consumed consistently; native constructibility/projection/widget hosting remain explicit exceptions.

Plan was approved before product implementation; all six tasks and the single final review completed inline/native. No commit/push or primary product integration is included.
