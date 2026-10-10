# Phase 2 — Phân loại object, snap nhẹ và sửa curves: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Snap CAD native có chi phí giới hạn; mesh/cloud mặc định không được quét points; sửa curve nhập và xuất đúng kết quả.

**Architecture:** C++ phân loại geometry không materialize points; Rust quyết định eligibility. Spatial/picking broad-phase và cache theo geometry/view thay cho document scan mỗi hover; integrate curve commands có review.

**Tech Stack:** Rust state/policy + C++23/Qt/OCCT/openNURBS + Python FreeCAD binding; Windows matching MSVC/Qt/Python SDK.

**Spec:** [Thiết kế chung](../specs/2026-10-08-rhino-modeling-five-phase-design.md).

**Status:** accepted_scoped on 2026-10-09; P2.1–P2.4 and actual FreeCAD/Rhino5 SaveAs reread passed on matching final source/runtime. One final review, one Important fix verified RED→GREEN; two Minor findings deferred. Evidence: docs/validation/modeling-phase-2/requirements.json.

## Global Constraints

- Source thực hiện: H:/FreeCAD-src/build/om9-dev; H:/FreeCAD-src/Mod/OpenMatrix9 chỉ là nguồn tích hợp có đối chiếu, không ghi đè nguyên checkout.
- openNURBS pin: eb92af3ba1806b0a34a99aba0d3bda83e3d46083; Rhino5/V5 là target baseline hiện tại.
- Phase 1 ưu tiên Import/Export Selected qua file 3DM; hai chiều Ctrl+C/Ctrl+V đã được bổ sung và nghiệm thu ở Phase1; Phase2 giữ hồi quy clipboard và worker mặc định60%.
- History Rhino/Matrix, materials, textures, lights, rendering và layouts không thuộc workflow dựng tiếp; Undo/Redo FreeCAD, chọn cạnh/mặt và Wireframe/Shaded thuộc phạm vi.
- CAD/NURBS có tessellation hiển thị vẫn là CAD; geometry kind, host representation và display-mesh state là ba thuộc tính riêng.
- Snap mặc định không enumerate mesh vertices, PointCloud points hoặc SubD display-mesh vertices; không biến CAD thành mesh để chỉnh sửa.
- Geometry hiện tại và geometry mới là nguồn export modeling; không phục hồi snapshot cũ để ghi đè sửa/xóa/copy của người dùng.
- Import/modeling là chế độ mới tách khỏi preservation; không làm yếu các guard của chế độ preservation đã có.
- Giữ ngưỡng nhẫn: bounds 0.001 mm; area/volume max(0.001, 0.0001 * abs(reference)); fixture analytical dùng ngưỡng riêng ghi trong oracle hiện hữu.
- Mỗi phase nghiệm thu trên cùng source/binary với hashes, fixture và kết quả rõ ràng; test lịch sử không được tính là lần chạy mới.
- Không tự commit/push, ghi đè thay đổi của tác vụ khác, thay SDK hoặc sửa skill trong nhiệm vụ lập kế hoạch này.

## Baseline và dependencies

CoreSnaps hiện duyệt mọi object mỗi pick; CoreSnapGeometry lấy CAD vertices/edges. Chưa có mesh-aware policy/index budget. CurveGeometry/CoreRebuild/spline mới nằm ở checkout chính.
Bắt đầu sau gate Phase1; API của phases trước định nghĩa trong spec và plans tương ứng.

## Review Focus

- BRep có mesh hiển thị: không bị phân loại thành Mesh; snap trỏ vào CAD vertex thật.
- Mesh/cloud một triệu points: zero-read trong snap mặc định, không cấp phát mảng size bằng vertex count.
- Nested Link, reflection và hidden parent: transform đúng một lần và không đưa hidden member vào snap.
- Shape/placement/camera đổi sau Undo/Redo: không dùng stale candidate cache.
- Vượt candidate budget hoặc dense coincident geometry: result incomplete được báo rõ, không trả snap sai như chắc chắn.

## Task file map và bước thực hiện

Các file ghi Create là đề xuất mới, chưa tồn tại. API ghi ở Interfaces là thiết kế mới trừ khi chỉ rõ reuse.
Tất cả relative paths lấy H:/FreeCAD-src/build/om9-dev làm root; file tích hợp từ checkout chính được ghi absolute path riêng.

### Task P2.1: Classifier và Rust snap eligibility

**Files:** Create: Gui/SnapObjectInfo.h, Gui/SnapObjectInfo.cpp, rust/tests/modeling_snap_policy.rs; Modify: rust/src/modeling_exchange.rs, Gui/RustBridge.h, Gui/CoreSnapGeometry.cpp, Gui/CMakeLists.txt; Test: tests/modeling_snap_smoke.FCMacro.

**Interfaces:** SnapObjectInfo classifySnapObject(const App::DocumentObject*); fields kind, representation, display_mesh_state, preview, resolved_member, global_transform. om9_modeling_snap_allowed(uint32_t kind, bool native_cad, bool preview, uint32_t mode) -> bool.

- [x] **Step 1 — Viết regression cho P2.1.** Các assertions bắt buộc: mesh_and_cloud_default_snap_reads_zero_points: source inspection confirms no heavy-array reads; End/Mid/Point bounded extraction and million-point timing pass. Counters0/Proxy poisoning are not independent instrumentation; this proof limitation is a deferred Minor in final-review.md. cad_with_render_mesh_remains_cad: kind=CadBrep, representation=native-cad, display state=present, End eligibility true.
- [x] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk cargo test --manifest-path rust/Cargo.toml --test modeling_snap_policy`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [x] **Step 3 — Implement tối thiểu theo Interfaces.** Read cheap native runtime type/known representation before getSubObject or Python sequence. Simple native/nested/reflected Links resolve source classification; general mixed-block member normalization is Phase4. Source tags không override runtime geometry đã thay; preview/retained không mặc định CAD-editable.
- [x] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [x] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-2/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P2.2: Bounded snap query và index/cache

**Files:** Create: Gui/SnapQueryIndex.h, Gui/SnapQueryIndex.cpp; Modify: Gui/CoreSnaps.cpp, Gui/CoreSnapGeometry.h, Gui/CoreSnapGeometry.cpp, Gui/CMakeLists.txt; Test: tests/modeling_snap_smoke.FCMacro; Create: tests/modeling_snap_performance_smoke.FCMacro.

**Interfaces:** SnapQueryResult querySnapCandidates(Gui::View3DInventor*, const SnapQuery&); SnapQuery fields/caps từ spec; result.complete và counters bắt buộc. Index invalidate on camera/viewport, shape, placement, visibility, links, transaction and document close.

- [x] **Step 1 — Viết regression cho P2.2.** Các assertions bắt buộc: bounded_query_ignores_remote_objects: 10000 off-cursor CAD objects không bị enumerate per hover; 64-object/2048-per-object/8192-total caps thực thi. Mesh/cloud1000000 points có reads0;500 warm queries record p95<=16ms với hardware/hash. Budget exhaustion báo incomplete; manual pick/typed input còn dùng được.
- [x] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy pwsh -NoProfile -File tests/run_modeling_phase.ps1 -Phase 2 -FreeCADExe H:/FreeCAD-src/build/om9-modeling-sdk/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [x] **Step 3 — Implement tối thiểu theo Interfaces.** Broad-phase dựa native scene pick/screen tiles và cheap bounds được cache; geometric detail extraction chỉ cho eligible CAD gần cursor. Không rebuild whole index trên mỗi mouse move, không thêm worker thread đọc mutable native document. Duyệt topology CAD bằng iterator native có cap trước khi tạo Python/list/vector; không đọc toàn Shape.Vertexes/Edges rồi mới cắt xuống budget. Gate kiểm tra allocations/counters cho BRep topology lớn, không chỉ mesh.
- [x] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [x] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-2/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P2.3: Tích hợp curve commands trên cùng runtime

**Files:** Integrate selectively from H:/FreeCAD-src/Mod/OpenMatrix9: Gui/CurveGeometry.{h,cpp}, Gui/CoreRebuild.{h,cpp}, rust/src/spline.rs, rust/src/spline_ffi.rs; Merge: Gui/CurveController.{h,cpp}, Gui/RustBridge.h, rust/src/curve.rs, rust/src/ffi.rs, rust/src/lib.rs, Gui/CMakeLists.txt; Create: tests/modeling_curve_workflow_smoke.FCMacro.

**Interfaces:** Reuse publishedSplineShape(), createCurveFeature(App::Document&, PyObject*, const char*), sampleCurve(PyObject*, size_t) từ CurveGeometry.h; giữ command IDs, CoreSnaps::pick và modeled input adapters. Không copy nguyên lib.rs/FFI/controller của checkout chính.

- [x] **Step 1 — Viết regression cho P2.3.** Các assertions bắt buộc: imported_curve_command_routes: rational/periodic/imported transformed curves dùng Rebuild và curve creation qua menu/CMD/mouse chung; native output degree/poles/weights đúng trong scope; cancellation leaves document unchanged. Legacy Line/Polyline still passes.
- [x] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk cargo test --manifest-path rust/Cargo.toml --test spline_geometry`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [x] **Step 3 — Implement tối thiểu theo Interfaces.** Reconcile symbol/catalog changes với dev Core command families; resolve collisions explicitly. Import source interface/controller bằng diff, giữ dev native snap/placement policy. Thiếu tests/spline_geometry.rs thì tích hợp suite cùng module, không claim command test đã có.
- [x] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [x] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-2/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P2.4: Typed curve CV editing và lifecycle

**Files:** Create: Gui/ModelingCurveEditor.h, Gui/ModelingCurveEditor.cpp, tests/modeling_curve_editor_smoke.FCMacro; Modify: Gui/CurveController.cpp, Gui/CMakeLists.txt, Gui/RustBridge.h, rust/src/modeling_exchange.rs, ThreeDmModeling.py.

**Interfaces:** CurveEditRequest {degree, poles, weights, knots, multiplicities, periodic}; CurveEditResult editModelingCurve(App::Document&, const std::string& objectName, const CurveEditRequest&). Typed numeric UI, no eval; request owns copied values và signature input.

- [x] **Step 1 — Viết regression cho P2.4.** Các assertions bắt buộc: curve_edit_preserves_rational_basis: move selected CV, keep weights/degree/knots unless field explicitly edited; invalid/negative weights and knot-order changes fail atomically. edit->Undo->Redo->FCStd->modeling export uses current curve; snap index refreshed.
- [x] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy pwsh -NoProfile -File tests/run_modeling_phase.ps1 -Phase 2 -FreeCADExe H:/FreeCAD-src/build/om9-modeling-sdk/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [x] **Step 3 — Implement tối thiểu theo Interfaces.** Editor mở CV chỉ cho selected native curve trong explicit edit mode; không tạo all-object point overlay. Validate basis/domain/periodicity và stale signature trước one transaction; CV source inspection/export là typed CAD operations.
- [x] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [x] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-2/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.


## Exit gate Phase 2

Classifier/counters, budget/correctness/performance và imported-curve lifecycle đạt trên binary Phase2. Các suite curve ở checkout chính không tự chứng nhận runtime dev.

- [x] Tất cả promised requirements có native/host/version evidence tương ứng.
- [x] Không có source/binary mismatch hoặc unresolved critical finding.
- [x] Failure/cancel/Undo/Redo/FCStd và current geometry export được chứng minh.
- [x] Review Focus cases được exercised bởi regression trong các task trên.
- [x] Báo phạm vi đạt, phạm vi chưa đạt và next action; không chuyển task count thành coverage percentage.

- [x] **APP-GATE-P2:** FreeCAD/OpenMatrix9 + actual Rhino target ứng dụng đều đạt trên runtime của phase; Rhino SaveAs output được FreeCAD reimport; evidence theo [ma trận ứng dụng](../../validation/modeling-application-gates.md).

## Handoff

Phase3 dùng curves nhập/sửa từ Phase2 và typed command adapters.
Phase2 đã thực hiện và nghiệm thu bằng báo cáo runtime có hash; nguồn, binary riêng và snapshot được giữ lại. Phase3 chưa được nghiệm thu. Hai Minor và các giới hạn chứng minh được ghi trong final-review.md và requirements.json.

