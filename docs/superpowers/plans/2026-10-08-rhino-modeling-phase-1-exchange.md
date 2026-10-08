# Phase 1 — Nhận geometry qua 3DM và tạo bản làm việc: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Import/Export Selected qua file 3DM cho bản geometry độc lập, không yêu cầu history/render.

**Architecture:** Thêm modeling mode riêng vào native reader/binder/writer hiện có. Giữ mode preservation và mặc định cũ; tạo runtime thử thống nhất ngay trong phase này.

**Tech Stack:** Rust state/policy + C++23/Qt/OCCT/openNURBS + Python FreeCAD binding; Windows matching MSVC/Qt/Python SDK.

**Spec:** [Thiết kế chung](../specs/2026-10-08-rhino-modeling-five-phase-design.md).

**Status:** draft để review theo yêu cầu chia5 phase; không phải authorization triển khai, chưa có task mới được chạy.

## Global Constraints

- Source thực hiện: H:/FreeCAD-src/build/om9-dev; H:/FreeCAD-src/Mod/OpenMatrix9 chỉ là nguồn tích hợp có đối chiếu, không ghi đè nguyên checkout.
- openNURBS pin: eb92af3ba1806b0a34a99aba0d3bda83e3d46083; Rhino5/V5 là target baseline hiện tại.
- Phase 1 ưu tiên Import/Export Selected qua file 3DM; Ctrl+C/Ctrl+V trực tiếp để sau và không chặn nghiệm thu năm phase này.
- History Rhino/Matrix, materials, textures, lights, rendering và layouts không thuộc workflow dựng tiếp; Undo/Redo FreeCAD, chọn cạnh/mặt và Wireframe/Shaded thuộc phạm vi.
- CAD/NURBS có tessellation hiển thị vẫn là CAD; geometry kind, host representation và display-mesh state là ba thuộc tính riêng.
- Snap mặc định không enumerate mesh vertices, PointCloud points hoặc SubD display-mesh vertices; không biến CAD thành mesh để chỉnh sửa.
- Geometry hiện tại và geometry mới là nguồn export modeling; không phục hồi snapshot cũ để ghi đè sửa/xóa/copy của người dùng.
- Import/modeling là chế độ mới tách khỏi preservation; không làm yếu các guard của chế độ preservation đã có.
- Giữ ngưỡng nhẫn: bounds 0.001 mm; area/volume max(0.001, 0.0001 * abs(reference)); fixture analytical dùng ngưỡng riêng ghi trong oracle hiện hữu.
- Mỗi phase nghiệm thu trên cùng source/binary với hashes, fixture và kết quả rõ ràng; test lịch sử không được tính là lần chạy mới.
- Không tự commit/push, ghi đè thay đổi của tác vụ khác, thay SDK hoặc sửa skill trong nhiệm vụ lập kế hoạch này.

## Baseline và dependencies

File/menu/API exchange, Wireframe và worker import có scoped evidence. Modeling mode và combined selected geometry mới chưa được implement.
Không phụ thuộc phase mới trước đó; matching SDK/source preflight bắt buộc.

## Review Focus

- Mixed source có geometry opaque: preflight phải báo object/class và không commit một phần.
- File đơn vị cm và block scale lồng: không scale lần thứ hai khi bind vào host.
- Chọn container đồng thời với member: không duplicate hoặc tự bỏ đối tượng.
- Current edited object cùng geometry mới: writer không lấy lại source geometry cũ.
- Worker/helper lỗi và FCStd thiếu nguồn: rollback hoàn chỉnh, không còn orphan process.

## Task file map và bước thực hiện

Các file ghi Create là đề xuất mới, chưa tồn tại. API ghi ở Interfaces là thiết kế mới trừ khi chỉ rõ reuse.
Tất cả relative paths lấy H:/FreeCAD-src/build/om9-dev làm root; file tích hợp từ checkout chính được ghi absolute path riêng.

### Task P1.1: Runtime thống nhất và báo capability theo object

**Files:** Create: tests/run_modeling_phase.ps1, tests/modeling_baseline_smoke.FCMacro; Create: rust/src/modeling_exchange.rs, rust/tests/modeling_exchange_policy.rs; Modify: rust/src/lib.rs, Gui/ThreeDmInventory.cpp, ThreeDm.py.

**Interfaces:** Script run_modeling_phase.ps1 -Phase [1..5] -FreeCADExe <absolute path> -DependencyPrefix <absolute path>; Rust GeometryKind tags1..9 và OperationCapabilities như spec. Script chọn đúng macro phase, xác minh module/script hashes rồi gọi run_menu_smoke.ps1.

- [ ] **Step 1 — Viết regression cho P1.1.** Các assertions bắt buộc: test_modeling_scope_excludes_history_render_but_not_geometry: unknown geometry = preflight error; excluded history/render = report-only. modeling_baseline_smoke: matching native module và scripts cùng source; failure nếu runtime khác source. Worker failure giữ document object count/hash.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk cargo test --manifest-path rust/Cargo.toml --test modeling_exchange_policy`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Stage matching SDK vào runtime mới, không sửa runtime đang mở. Capture source/module/scripts hashes. Capability registry là một nguồn dùng chung cho UI/import/snap; unknown không mặc định editable.
- [ ] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-1/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P1.2: Modeling import và geometry làm việc độc lập

**Files:** Create: ThreeDmModeling.py, Gui/ThreeDmModeling.h, Gui/ThreeDmModeling.cpp, tools/tests/test_modeling_exchange.py; Modify: ThreeDm.py, Gui/ThreeDmPython.cpp, Gui/CoreThreeDm.cpp, Gui/CMakeLists.txt, CMakeLists.txt; Test: tests/modeling_exchange_smoke.FCMacro.

**Interfaces:** prepare_modeling(path: str, staging: str, scale: float) -> dict; prepareModelingArchive(const std::filesystem::path&, const std::filesystem::path&, double customUnitMm) -> QJsonObject. ThreeDm.import_file(..., mode='modeling') thêm mode mới.

- [ ] **Step 1 — Viết regression cho P1.2.** Các assertions bắt buộc: test_modeling_import_units_and_opaque_preflight: mm/cm control points, arc radius, trims và bbox đúng; opaque selected geometry từ chối trước transaction. test_display_mesh_does_not_change_cad_kind: tessellated BRep vẫn native-cad. Real runtime mở curve/BRep/mesh/cloud và xóa nguồn rồi FCStd reopen.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy python -m unittest discover -s tools/tests -p test_modeling_exchange.py`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Reuse ONX reader, typed converter và existing worker orchestration. Bind fresh working identity/provenance, không attach strict full-archive export dependency vào working geometry. Không bỏ unknown geometry âm thầm.
- [ ] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-1/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P1.3: Export selection từ geometry hiện tại

**Files:** Modify: ThreeDmModeling.py, Gui/ThreeDmModeling.cpp, Gui/ThreeDmPython.cpp, ThreeDm.py; Test: tools/tests/test_modeling_exchange.py, tests/modeling_exchange_smoke.FCMacro; Create: tests/native/three_dm_modeling_exchange.cpp; Modify: tests/native/CMakeLists.txt.

**Interfaces:** stage_modeling_selection(objects: list, staging: str) -> dict; writeModelingArchive(const QJsonObject&, const std::filesystem::path&) -> void. ThreeDm.export_file(..., modeling=True) giữ geometry_only=False mặc định của callers cũ.

- [ ] **Step 1 — Viết regression cho P1.3.** Các assertions bắt buộc: test_modeling_export_current_and_new_geometry: move imported curve + add native circle; decoded V5 contains current placement and circle exactly, no deleted original. test_failed_export_keeps_existing_destination: target bytes unchanged. Duplicate container/member selection bị báo rõ.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy pwsh -NoProfile -File tests/run_modeling_phase.ps1 -Phase 1 -FreeCADExe H:/FreeCAD-src/build/om9-modeling-sdk/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Build request từ current shape/mesh/cloud, geometry dependencies và basic layers; bỏ history/render theo scope. Stage V5, reread decoded geometry rồi QSaveFile atomic commit; xuất cùng file cho object nhập và object mới.
- [ ] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-1/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.


## Exit gate Phase 1

P1.1–P1.3 đạt: import, select, add geometry, selected export, unit fixtures, failure atomicity và source-deleted FCStd trên cùng runtime. Clipboard không thuộc gate.

- [ ] Tất cả promised requirements có native/host/version evidence tương ứng.
- [ ] Không có source/binary mismatch hoặc unresolved critical finding.
- [ ] Failure/cancel/Undo/Redo/FCStd và current geometry export được chứng minh.
- [ ] Review Focus cases được exercised bởi regression trong các task trên.
- [ ] Báo phạm vi đạt, phạm vi chưa đạt và next action; không chuyển task count thành coverage percentage.

## Handoff

Phase2 dùng GeometryKind/capabilities và modeling current writer từ Phase1.
Lượt hiện tại chỉ tạo tài liệu. Review thiết kế và plan trước khi chọn execution method; không suy ra task đã chạy từ checkbox/nguồn có sẵn.

