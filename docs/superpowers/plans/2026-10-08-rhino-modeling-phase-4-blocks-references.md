# Phase 4 — Blocks, copy độc lập và geometry phụ thuộc: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Dựng tiếp trên nested/shared blocks và reference curves bằng current owning geometry với semantics copy/edit rõ.

**Architecture:** Thêm materialization cho modeling mode, giữ structural preservation riêng. Exact reference normalization giải quyết owner/domain trước khi tạo CAD độc lập.

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

Structural graph/copy/remap được test; geometry owner edits và standalone/mixed PolyEdge còn nhiều compatibility guards. Không dùng preservation success để claim modeling edit.
Bắt đầu sau gate Phase3; API của phases trước định nghĩa trong spec và plans tương ứng.

## Review Focus

- Shared definition và copy độc lập: sửa copy không đổi instance/reference gốc.
- Nested nonuniform scale/reflection: đúng CAD/matrix parity và mesh winding.
- Reference chỉ khớp endpoints: sai interior mapping vẫn phải fail normalization.
- Deleted owner hoặc mixed archive UUID collision: không restore snapshot hoặc trỏ nhầm owner.
- Mixed CAD+mesh/cloud block: snap không enumerate point của member mesh/cloud.

## Task file map và bước thực hiện

Các file ghi Create là đề xuất mới, chưa tồn tại. API ghi ở Interfaces là thiết kế mới trừ khi chỉ rõ reuse.
Tất cả relative paths lấy H:/FreeCAD-src/build/om9-dev làm root; file tích hợp từ checkout chính được ghi absolute path riêng.

### Task P4.1: Materialize block và explicit copy semantics

**Files:** Create: Gui/ModelingBlocks.h, Gui/ModelingBlocks.cpp; Modify: ThreeDmModeling.py, Gui/ThreeDmModeling.cpp, Gui/ThreeDmPython.cpp, Gui/CMakeLists.txt; Test: tests/native/three_dm_modeling_blocks.cpp, tests/modeling_blocks_smoke.FCMacro; Modify: tests/native/CMakeLists.txt.

**Interfaces:** ModelingBlockResult materializeModelingBlock(const ModelingBlockRequest&); request includes root IDs, current definition members, world matrix, copy_scope(independent/shared) and output mode(placed/flatten). No implicit shared-definition editing.

- [ ] **Step 1 — Viết regression cho P4.1.** Các assertions bắt buộc: modeling_block_copy_is_independent: nested rigid/nonuniform/reflected/sheared CAD+quad mesh+cloud controls; exact world bounds and attributes; modify copy leaves original current digest unchanged. Missing external-only member/cycle fails before transaction.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy cmake --build H:/FreeCAD-src/build/3dm-modeling-native --target ThreeDmModelingBlockTests --parallel 4`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Reuse current native graph resolver and affine transforms; place CAD with native transformations when Placement cannot encode full matrix. Preserve native mesh as mesh; promote editable member with independent identity instead of editing shared graph implicitly.
- [ ] **Step 4 — Chạy lại build, rồi chạy native test.** Sau command Step2 chạy `rtk proxy ctest --test-dir H:/FreeCAD-src/build/3dm-modeling-native -R "OM9-MODELING.Blocks" --output-on-failure`. Đăng ký đúng test name trong tests/native/CMakeLists.txt. PASS = named assertions đạt và exit0; compile/link không thay thế behavioral test. Sau native PASS chạy macro phase bằng runner để chứng minh host lifecycle.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-4/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P4.2: Normalize reference curve thành owning geometry

**Files:** Create: Gui/ModelingCurveNormalization.h, Gui/ModelingCurveNormalization.cpp; Modify: Gui/ThreeDmNativeReferences.cpp, Gui/ThreeDmCurveOnSurface.cpp, Gui/ThreeDmModeling.cpp, Gui/CMakeLists.txt; Test: tests/native/three_dm_modeling_references.cpp, tests/modeling_references_smoke.FCMacro; Modify: tests/native/CMakeLists.txt.

**Interfaces:** NormalizedCurve normalizeModelingCurve(const ON_Curve&, const NativeReferenceGraph&, double tolerance); fidelity=exact-converted/unsupported; independent curve owns all data and contains no unresolved external owner pointer. Optional approximation remains a separate future request.

- [ ] **Step 1 — Viết regression cho P4.2.** Các assertions bắt buộc: reference_normalization_checks_full_geometry: reversed/domain-reparameterized PolyEdge and supported CurveOnSurface compare full basis or certified mapped-locus proof; independent failure control matches endpoints but wrong interior. Unknown mapping/unsafe userdata affecting geometry returns unsupported, no output pretending editable.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy cmake --build H:/FreeCAD-src/build/3dm-modeling-native --target ThreeDmModelingReferenceTests --parallel 4`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Resolve referenced owner before extracting underlying curve/edge3D and exact domain/reversal. Type-specific CurveOnSurface maps admitted only when exact native/analytic conversion available; don't sample polyline and label NURBS exact.
- [ ] **Step 4 — Chạy lại build, rồi chạy native test.** Sau command Step2 chạy `rtk proxy ctest --test-dir H:/FreeCAD-src/build/3dm-modeling-native -R "OM9-MODELING.References" --output-on-failure`. Đăng ký đúng test name trong tests/native/CMakeLists.txt. PASS = named assertions đạt và exit0; compile/link không thay thế behavioral test. Sau native PASS chạy macro phase bằng runner để chứng minh host lifecycle.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-4/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P4.3: Member editing và current export lifecycle

**Files:** Modify: ThreeDmModeling.py, Gui/ModelingBlocks.cpp, Gui/ThreeDmModeling.cpp, tests/run_modeling_phase.ps1; Test: tests/modeling_block_edit_workflow_smoke.FCMacro, tests/modeling_references_smoke.FCMacro.

**Interfaces:** Consumes independent member map/materialized graph và modeling selection writer. Current geometry change emits invalidate for Phase2 snap index; preservation_request remains untouched for strict archival callers.

- [ ] **Step 1 — Viết regression cho P4.3.** Các assertions bắt buộc: block_edit_copy_delete_reopen: import twice with colliding native UUIDs; modify independent copied CAD member, delete other member, export selected; no extra roots/no resurrection. Source-deleted FCStd and Undo/Redo restore correct world geometry, identities and snap classification.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy pwsh -NoProfile -File tests/run_modeling_phase.ps1 -Phase 4 -FreeCADExe H:/FreeCAD-src/build/om9-modeling-sdk/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Normalize/materialize explicitly before commands so old retained owners don't block independent CAD. Keep provenance read-only; current selection controls output membership. Report rejected incompatible raw preservation separately from successful modeling conversion.
- [ ] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-4/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.


## Exit gate Phase 4

Blocks/member/reference operations trong capability matrix có exact geometry proof và failure semantics; generic unresolved references vẫn unsupported, không gọi phase full khi promised mapping chưa được thực hiện.

- [ ] Tất cả promised requirements có native/host/version evidence tương ứng.
- [ ] Không có source/binary mismatch hoặc unresolved critical finding.
- [ ] Failure/cancel/Undo/Redo/FCStd và current geometry export được chứng minh.
- [ ] Review Focus cases được exercised bởi regression trong các task trên.
- [ ] Báo phạm vi đạt, phạm vi chưa đạt và next action; không chuyển task count thành coverage percentage.

- [ ] **APP-GATE-P4:** FreeCAD/OpenMatrix9 + actual Rhino target ứng dụng đều đạt trên runtime của phase; Rhino SaveAs output được FreeCAD reimport; evidence theo [ma trận ứng dụng](../../validation/modeling-application-gates.md).

## Handoff

Phase5 đóng remaining geometry inventory và build/install acceptance.
Lượt hiện tại chỉ tạo tài liệu. Review thiết kế và plan trước khi chọn execution method; không suy ra task đã chạy từ checkbox/nguồn có sẵn.

