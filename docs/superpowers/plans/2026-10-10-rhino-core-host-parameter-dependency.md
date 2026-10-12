# Rhino core guarded host parameter dependency implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Continue the authorized native execution and all-P0 scope.

**Goal:** Complete authored RCORE-01.T10 with a durable native dependent point evaluated at an original analytic source parameter through the actual observed NURBS conversion map.

**Architecture:** Reuse CurveConversion::importCurveObserved, shared Rust query validation, ObjectSnapshot guards, native FreeCAD property links/persistence and existing History update/lock policy. Add a narrow native Part feature that retains the original query recipe, owned converted Shape and copied host-bound report, then builds its point from the validated mapped query. Native extraction initially admits full-domain analytic Circle and Line with explicit unsupported reasons; it does not convert an analytic source to NURBS before extracting its parameter semantics. Safe Rust validates the host-source binding residual and selects mapped versus explicitly proven raw parameter use.

**Tech Stack:** Rust2024, FreeCAD/OCCT/openNURBS C++ host adapter, native FCStd persistence and macro fixture.

**Spec:** `OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md`, RCORE-01.T10; prior bounded native query proof `docs/validation/2026-10-10-rhino-core-curve-parameters.md`.

## Global constraints

- Rust owns portable numeric/binding/raw-use policy; C++ handles exact SDK geometry/frame extraction, native document properties/signals/lifetimes and serialization.
- Preserve source analytic type/domain/native parameter; same owned NURBS conversion lineage supplies target Shape and query map. No fit/reparameterization or approximate replacement solver.
- Capture/validate native source guard before any host commit; reports record actual document UID/object ID/name/subelement, runtime snapshot revision/session, explicit unit context and declared frame.
- Do not persist process-local snapshot capsules as durable identity. Native property link, source native ID/document UID and query recipe persist; fresh source guard/conversion observations are produced on recomputation.
- Point and converted geometry belong to the feature; read/query creates no source mutation or extra Undo. Creation is one native transaction; rejected raw query/invalid input creates none.
- Explicit positive length and parameter bounds; finite/domain failures reuse the existing Rust gate. Form2/raw-unproved use rejects even at an individually coincident seam sample.
- Keep all-P0 active and general conversion/worker/all-family clauses open.

## Review focus

- A correct standalone map bound to the wrong host frame/source must reject; compare ON source sample against actual original OCCT sample in Rust.
- A circle can preserve locus while raw source parameter gives a displaced point; persist distinct mapped/raw residuals and never label the nonlinear map identity/affine.
- Source edits/Undo/replacement/ancestor placement/units require fresh guarded conversion; no stale retained report may drive current output.
- Cold FCStd restore must retain source link/native identity, recipe, converted basis/query metadata and point; a new runtime session must revalidate rather than reuse old tokens.
- Invalid/unsupported recomputation must clear current point/converted outputs and mark dirty/error; History Update/Lock semantics must remain the established policy.

### Task1: Reuse native query validation for a host-bound dependent point

**Files:** modify `rust/src/curve_conversion.rs`, `rust/src/curve_conversion_ffi.rs`; create `rust/tests/curve_parameter_dependency.rs`; extend `Gui/CurveConversionContract.h` with the matching value ABI.
**Interfaces:** safe `dependency_point(input,observation,host_source_point,use_raw) -> Result<DependentPoint,QueryError>`; copied ABI `om9_curve_parameter_dependency(DependencyInput)->DependencySummary`. No retained pointer/host state.

- [x] Author RED cases for wrong host point, unproved raw form2, failed/nonfinite/outside-domain observations and valid mapped/form1-proven raw use. Include value ABI layout/invalid bool/reserved checks.
- [x] Add minimal Rust policy reusing `validate_observation`; return validated target parameter/point and host binding residual. Run full Rust tests and Clippy.

### Task2: Native durable host dependency

**Files:** create `Gui/CurveParameterPoint.h/.cpp`, `tests/rhino_core_host_parameter_dependency_smoke.FCMacro`; modify `Gui/CMakeLists.txt`, `Gui/AppOpenMatrix9Gui.cpp`, `Gui/HistoryFeature.cpp` for type/API registration and existing History integration.
**Interfaces:** `OpenMatrix9Gui::CurveParameterPoint` native Part feature with source link/subelement, original SourceParameter, UseRawParameter, explicit tolerances, owned ConvertedShape, persisted QueryReport and Recorded/HistoryDirty. `createCurveParameterPoint(document_name,source_name,subelement,source_parameter,length_tolerance,parameter_tolerance,use_raw=False)` evaluates/guards before its single creation transaction. No new menu command.

- [x] Native fixture RED exposes missing guarded durable query consumer.
- [x] Extract current original analytic support/frame directly, construct matching ON analytic source, use `importCurveObserved` for actual native map/target, validate actual OCCT host sample through the Rust dependency ABI, then create owned point. Bind copied report to actual source and units. Unsupported input rejects explicitly.
- [x] Connect native source/ancestor/unit changes, existing History update/lock/detach and Undo/restoration behavior. Recompute captures a fresh guard and native conversion; invalid recompute clears stale current outputs/reports and marks dirty/error.
- [x] Test R10 circle at asymmetric angle0.31: independent analytic point, mapped residual≤1e-7mm, raw displacement measured separately; raw reuse rejects with no objects/Undo. Include Line form1 positive reuse, source/domain/units/placement guards, invalid/unsupported inputs, edits/Undo/History update and native object inventory/source bytes.
- [x] Warm/cold FCStd persistence compares native source identity/domain, original query and mapping/basis/report, owned converted Shape/point and independent geometry; verify fresh-session re-evaluation. Run existing History/lifecycle/query controls and build the exact standalone SDK module.

### Task3: Independent review, evidence and checkpoint

**Files:** scoped review/report/evidence plus README/spec/live ledger/acceptance matrix/package manifest.

- [x] Independently review actual native lineage/host binding, Rust policy, mutation/History/error/persistence boundaries and final fresh-process artifacts.
- [x] Record exact source/module/macro/result/process/raw geometry fingerprints, actual RED and excluded attempts. Earn authored T10 coverage only; no whole-domain parameter-map/global deviation certificate.
- [x] Verify fingerprints/package manifest/diff and save a local checkpoint. Continue T07 and remaining P0 chapters/contracts.
