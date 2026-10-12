# Phase 3 — Dựng tiếp trên surfaces, BRep và solids: Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Nghiệm thu workflow nhẫn nhập từ Rhino, tạo cutter/chi tiết mới, Boolean và xuất đúng CAD.

**Architecture:** Reuse openNURBS→OCCT converter; sửa converter chỉ khi fixture độc lập chứng minh lỗi. Tích hợp Surface/Edit từ checkout chính, snapshot current inputs và one native transaction.

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

Seam/pole/cavity và scale-edited sphere/cylinder V5 có proof; Loft/Sweep/Edit mới ở checkout chính chưa được chứng minh chung runtime với newest exchange.
Bắt đầu sau gate Phase2; API của phases trước định nghĩa trong spec và plans tương ứng.

## Review Focus

- Holes/seam/pole: Trim hoặc Boolean không chọn complementary surface hay mất singular trim.
- Open shell vs solid: Boolean chỉ nhận supported closed solid; không tự gọi solid constructor trên open shell.
- Nested/transformed App::Part: parent/global placement không bị nhân đôi.
- Input thay đổi khi dialog mở: stale signature phải cancel trước commit.
- Kernel failure/empty result: không xóa input, không xuất source geometry cũ để che lỗi.

## Task file map và bước thực hiện

Các file ghi Create là đề xuất mới, chưa tồn tại. API ghi ở Interfaces là thiết kế mới trừ khi chỉ rõ reuse.
Tất cả relative paths lấy H:/FreeCAD-src/build/om9-dev làm root; file tích hợp từ checkout chính được ghi absolute path riêng.

### Task P3.1: BRep fixture fidelity và edit applicability

**Files:** Modify only if reproduced failure: Gui/ThreeDmBrep.cpp, Gui/ThreeDmSolidShells.cpp, Gui/ThreeDmTrimMapping.cpp; Create: tests/native/three_dm_modeling_brep.cpp, tests/modeling_brep_smoke.FCMacro; Modify: tests/native/CMakeLists.txt, rust/src/modeling_exchange.rs.

**Interfaces:** Reuse importBrep(const ON_Brep&, double), exportBrep(const TopoDS_Shape&, double); capability_by_operation distinguishes curve/face/shell/solid and native-only source. Fidelity tests use original oracle tolerances.

- [ ] **Step 1 — Viết regression cho P3.1.** Các assertions bắt buộc: brep_edit_applicability: planar hole, capped cylinder seam, sphere poles, torus and cavity fixtures decode valid native CAD. Open-shell boolean unsupported; invalid topology no document mutation. Edited cylinder retains full common UV basis under target knot insertion.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy cmake --build H:/FreeCAD-src/build/3dm-modeling-native --target ThreeDmModelingBrepTests --parallel 4`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Register ThreeDmModelingBrepTests and ctest OM9-MODELING.Brep. Use independent analytic bounds/volume and decoded pcurve/basis checks; don't infer fidelity only from mesh/bbox/CRC. Passing existing converter receives no rewrite.
- [ ] **Step 4 — Chạy lại build, rồi chạy native test.** Sau command Step2 chạy `rtk proxy ctest --test-dir H:/FreeCAD-src/build/3dm-modeling-native -R "OM9-MODELING.Brep" --output-on-failure`. Đăng ký đúng test name trong tests/native/CMakeLists.txt. PASS = named assertions đạt và exit0; compile/link không thay thế behavioral test. Sau native PASS chạy macro phase bằng runner để chứng minh host lifecycle.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-3/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P3.2: Tích hợp Surface/Edit và input current geometry

**Files:** Integrate from H:/FreeCAD-src/Mod/OpenMatrix9: Gui/SurfaceController.{h,cpp}, Gui/SurfaceGeometry.{h,cpp}, Gui/EditController.{h,cpp}, Gui/EditGeometry.{h,cpp}, rust/src/surface.rs, rust/src/edit.rs and relevant tests; Merge: Gui/Command.cpp, Gui/Workbench.cpp, Gui/RustBridge.h, Gui/CMakeLists.txt, rust/src/ffi.rs, rust/src/lib.rs; Test: tests/modeling_surface_edit_smoke.FCMacro.

**Interfaces:** Reuse buildSurface(App::Document&, const std::vector<SurfaceInput>&, const SurfaceOptions&), commitSurface(...); editInput(...), verifyEditInputs(...), buildEdit(...), commitEdit(...). Keep existing native command IDs/options.

- [ ] **Step 1 — Viết regression cho P3.2.** Các assertions bắt buộc: imported_cad_for_surface_edit: Loft/Sweep from imported rational curves, Trim/Join from imported open faces, Explode polysurface, BooleanDifference on imported closed solid. Same result menu/CMD/mouse; readonly/task/stale/Cancel checks match existing contract.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk cargo test --manifest-path rust/Cargo.toml --test surface_session --test edit_session`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Merge controllers with dev main command routing instead of overwriting. Modeling inputs use world current geometry; preserve-mode retained owners remain protected. Add CV/fillet/offset capability as unsupported unless actual adapter exists; don't enable menu placeholders.
- [ ] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-3/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.

### Task P3.3: Workflow nhẫn từ import tới xuất Rhino

**Files:** Create: tests/modeling_ring_workflow_smoke.FCMacro, tests/native/three_dm_modeling_workflow_oracle.cpp; Modify: tests/native/CMakeLists.txt, tests/run_modeling_phase.ps1; Create: docs/validation/modeling-phase-3/requirements.json during execution.

**Interfaces:** Consumes Phase1 prepare/export, Phase2 classify/query/curve edit và Phase3 native controllers. Oracle supplies fixture bounds/volume/topology and identifies intended cut/add result independently.

- [ ] **Step 1 — Viết regression cho P3.3.** Các assertions bắt buộc: ring_import_cut_add_roundtrip: import independent ring/representative torus; create cutter using CAD snap; cut specified hole; create Loft/Sweep addition; selection exports both modified and new geometry. Undo/Redo each step, delete original3DM, FCStd reopen; Rhino5 Open/SaveAs/reimport of result.
- [ ] **Step 2 — Chạy targeted check trước thay đổi.** Run: `rtk proxy pwsh -NoProfile -File tests/run_modeling_phase.ps1 -Phase 3 -FreeCADExe H:/FreeCAD-src/build/om9-modeling-sdk/bin/FreeCAD.exe -DependencyPrefix H:/FreeCAD-src/.pixi/envs/default/Library`. RED phải gắn với named behavioral assertion hoặc API mới đang thiếu, không phải sai SDK/path. Nếu existing implementation đã đáp ứng, record baseline và reuse; không tạo giả RED.
- [ ] **Step 3 — Implement tối thiểu theo Interfaces.** Macro performs actual commands and checks independent expected bounds/volume/topology, not just object count. Use user ring only as private supplemental fixture via env var; independently redistributable fixture is required baseline.
- [ ] **Step 4 — Chạy lại targeted check và relevant runtime.** Run cùng command ở Step2; PASS = tất cả named assertions đạt, process exit0; runtime có results.json ok=true và source/module hashes matching. Native target phải chạy executable/ctest sau build; compile-only chưa là PASS hành vi.
- [ ] **Step 5 — Review, evidence và scoped integration.** Record task requirement/proof/limits vào docs/validation/modeling-phase-3/requirements.json. Review diff và unresolved cases; chỉ commit files của task nếu execution context cho phép, không stage unrelated changes. Không push.


## Exit gate Phase 3

Trọn workflow cut/add/save/export và actual target proof đạt trên một binary. Đây là release thử phục vụ dựng tiếp; không suy ra mọi Boolean/fillet/offset của Rhino đều có.

- [ ] Tất cả promised requirements có native/host/version evidence tương ứng.
- [ ] Không có source/binary mismatch hoặc unresolved critical finding.
- [ ] Failure/cancel/Undo/Redo/FCStd và current geometry export được chứng minh.
- [ ] Review Focus cases được exercised bởi regression trong các task trên.
- [ ] Báo phạm vi đạt, phạm vi chưa đạt và next action; không chuyển task count thành coverage percentage.

- [ ] **APP-GATE-P3:** FreeCAD/OpenMatrix9 + actual Rhino target ứng dụng đều đạt trên runtime của phase; Rhino SaveAs output được FreeCAD reimport; evidence theo [ma trận ứng dụng](../../validation/modeling-application-gates.md).

## Handoff

Phase4 giải quyết mixed blocks và curves có owner references.
Lượt hiện tại chỉ tạo tài liệu. Review thiết kế và plan trước khi chọn execution method; không suy ra task đã chạy từ checkbox/nguồn có sẵn.

