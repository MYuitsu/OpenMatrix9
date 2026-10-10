# Phase 5 — Geometry còn lại, tương thích phiên bản và bản cài cuối: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Đóng geometry capability trong phạm vi dựng tiếp và tích hợp workflow năm phase trên matching installed runtime.

**Architecture:** Complete a scoped per-type operation/version matrix; bổ sung adapter theo từng nhóm nhỏ. Explicit target profile và tested packaging tách khỏi legacy preservation/full-archive policy.

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

Cloud/Hatch/TextDot/Mesh đã có nhiều scoped adapters; SubD/cage/morph và combined primary installed runtime chưa hoàn tất. Target V5 không đại diện mọi geometry của SDK mới.
Bắt đầu sau gate Phase4; API của phases trước định nghĩa trong spec và plans tương ứng.

## Review Focus

- Native SubD và display mesh: không gắn nhãn mesh là editable NURBS hay tự xuất mất cage.
- Newer-only geometry target V5: không silently downgrade rồi báo thành công.
- Geometry missing adapter trong promised scope: không dùng retained snapshot hoặc rejection để đánh dấu full support.
- Clean installed runtime thiếu worker hoặc core patch: preflight fail, không copy proof từ sandbox khác.
- Large CAD+mesh+cloud và repeated import/save: memory/performance/caches không tăng theo toàn mesh vertices mỗi hover.

## Task file map và bước thực hiện

Các file ghi Create là đề xuất mới, chưa tồn tại. API ghi ở Interfaces là thiết kế mới trừ khi chỉ rõ reuse.
Tất cả relative paths lấy H:/FreeCAD-src/build/om9-dev làm root; file tích hợp từ checkout chính được ghi absolute path riêng.

### Task P5.1: Remaining geometry adapters và scoped matrix

**Files:** Create: docs/modeling-geometry-capabilities.json, Gui/ModelingNativeGeometry.h, Gui/ModelingNativeGeometry.cpp, tests/native/three_dm_modeling_remaining_geometry.cpp, tests/modeling_native_geometry_smoke.FCMacro; Modify: Gui/ThreeDmModeling.cpp, Gui/SnapObjectInfo.cpp, Gui/CMakeLists.txt, tests/native/CMakeLists.txt; Reuse: ThreeDmPointCloud.py, ThreeDmHatch.py, ThreeDmTextDot.py.

**Interfaces:** NativeModelingGeometry convertModelingNative(const ON_Geometry&, const ModelingOperationRequest&); result contains kind, native payload, host representation, operation capabilities and failure reason. Matrix rows use read/display/edit-operations/write-version evidence, not class-wide boolean.

- [ ] **Step 1 — Viết regression cho P5.1.** Các assertions bắt buộc: remaining_geometry_capabilities: independent Mesh/cloud/Hatch/dot/SubD/PointGrid/NurbsCage/MorphControl/concrete surface-owner fixtures; owning payload survives clone/transform/FCStd; default snap reads0 mesh/cloud/display points. Unsupported field/edit has explicit status and leaves geometry unchanged.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy cmake --build H:/FreeCAD-src/build/3dm-modeling-native --target ThreeDmModelingRemainingGeometryTests --parallel 4`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Decompose adapters by geometry group if file grows; native SubD cage edit and limit display separate from exact-NURBS conversion. Enumerate concrete applicable classes from existing normalized catalog; helper/abstract/layout exclusions recorded with reason. Missing promised adapter stays open.
- [ ] **Step 4 — Chạy lại build, rồi chạy native test.** Sau command Step2 chạy `rtk proxy ctest --test-dir H:/FreeCAD-src/build/3dm-modeling-native -R "OM9-MODELING.RemainingGeometry" --output-on-failure`. Đăng ký đúng test name trong tests/native/CMakeLists.txt. PASS = named assertions đạt và exit0; compile/link không thay thế behavioral test. Sau native PASS chạy macro phase bằng runner để chứng minh host lifecycle.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-5/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P5.2: Version-specific selected writer và actual target acceptance

**Files:** Create: Gui/ModelingTarget.h, Gui/ModelingTarget.cpp, tests/native/three_dm_modeling_target.cpp, tests/modeling_target_smoke.FCMacro; Modify: Gui/ThreeDmModeling.cpp, Gui/ThreeDmPython.cpp, Gui/CoreThreeDm.cpp, Gui/CMakeLists.txt, tests/native/CMakeLists.txt, docs/modeling-geometry-capabilities.json.

**Interfaces:** TargetWriteReport writeModelingTarget(const ModelingRequest&, const std::filesystem::path&, int targetVersion); V5 default; targetVersion mới phải whitelist sau serialized native+host+application proof. Typed export choice labels show supported geometry profile.

- [ ] **Step 1 — Viết regression cho P5.2.** Các assertions bắt buộc: target_profile_never_silent_downgrade: V5 decoded geometry exact for baseline; unsupported SubD/new fields rejected atomically with class/action report. New target admitted only after captured actual application Open/SaveAs/reimport; no fake proof from matching registry alone.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy pwsh -NoProfile -File tests/run_modeling_phase.ps1 -Phase 5 -FreeCADExe H:/FreeCAD-src/build/om9-modeling-sdk/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Use pinned SDK archive writer by explicit target version, semantic reread and destination atomic commit. If modern Rhino target unavailable, record unverified and keep profile disabled; don't mark phase complete for the missing target requirement.
- [ ] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-5/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P5.3: Primary integration, installed runtime và end-to-end release gate

**Files:** Integrate reviewed diff into H:/FreeCAD-src/Mod/OpenMatrix9; Modify as needed: CMakeLists.txt, Gui/CMakeLists.txt, cmake/StandaloneSDK.cmake, tests/run_modeling_phase.ps1; Create: docs/validation/modeling-five-phase-acceptance.md, docs/validation/modeling-five-phase-acceptance.json; Test: all modeling phase macros and relevant existing regression macros.

**Interfaces:** Installed capability manifest binds source hash, native module hash, Rust library hash, Python scripts, helper hashes, FreeCAD build/version and target profiles. Runtime runner supports phases1..5 and refuses mismatched deployment.

- [ ] **Step 1 — Viết regression cho P5.3.** Các assertions bắt buộc: clean_install_full_workflow: real CAD ring workflow, dense mesh/cloud snap, mixed blocks, references, Undo/Redo, source-deleted FCStd, selected new+edited geometry target reread, missing helper/core patch and cancel/failure controls. 500 warm snap queries record counters/p95; phase matrix contains no promised unverified operations.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy pwsh -NoProfile -File tests/run_modeling_phase.ps1 -Phase 5 -FreeCADExe H:/FreeCAD-src/build/om9-modeling-sdk/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Review integration line-by-line; keep other concurrent Curve/UI changes. Rebuild/install from final reviewed primary source with required core patches/helpers, then rerun combined acceptance. Archive new reports separately; no push/publish unless separately instructed. Đóng mỗi phase theo requirements, không theo 16 task ticks hoặc tổng assertions. Không cộng checkpoint51 suite lịch sử với fast-import7 suite thành một run.
- [ ] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-5/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.


## Exit gate Phase 5

Promised scope matrix closed with per-operation/version evidence and installed source/binary bound. Out-of-scope history/render/clipboard remain excluded, not counted as missing full-archive features; missing geometry adapters remain open.

- [ ] Tất cả promised requirements có native/host/version evidence tương ứng.
- [ ] Không có source/binary mismatch hoặc unresolved critical finding.
- [ ] Failure/cancel/Undo/Redo/FCStd và current geometry export được chứng minh.
- [ ] Review Focus cases được exercised bởi regression trong các task trên.
- [ ] Báo phạm vi đạt, phạm vi chưa đạt và next action; không chuyển task count thành coverage percentage.

- [ ] **APP-GATE-P5:** FreeCAD/OpenMatrix9 + actual Rhino target ứng dụng đều đạt trên runtime của phase; Rhino SaveAs output được FreeCAD reimport; evidence theo [ma trận ứng dụng](../../validation/modeling-application-gates.md).

## Handoff

Sau review các kế hoạch, thực hiện Phase1 trước; clipboard và mở rộng bản vẽ kỹ thuật là roadmap riêng theo yêu cầu sau.
Lượt hiện tại chỉ tạo tài liệu. Review thiết kế và plan trước khi chọn execution method; không suy ra task đã chạy từ checkbox/nguồn có sẵn.

