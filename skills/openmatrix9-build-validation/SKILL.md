---
name: openmatrix9-build-validation
description: Build and validate OpenMatrix9 standalone Rust, selected OCCT/openNURBS adapters and Matrix9 UI against milestone contracts; use FreeCAD plugin checks only for explicit legacy maintenance.
---

# Kiểm chứng đúng target OM9

Đọc [đích milestone](../openmatrix9-workflow/references/milestones-target.md) và [Rust/FFI](../openmatrix9-workflow/references/rust-first.md). Workflow `route build-validation` cung cấp policy/design, không chứng minh executable đã tồn tại.

Theo [validation-playbook](references/validation-playbook.md), resolve target/crate/build config thực trước khi chạy. Rust core hiện có và examples là portable/legacy component evidence, không thay executable standalone. Không reconfigure/build FreeCAD root cho v2. Toolkit dependencies, ABI, version và scheduler phải kiểm theo operation được chọn.

Kiểm tra phù hợp change: semantic Rust tests → native bridge/error/ownership/tolerance → actual app lifecycle/document/Undo/.om9/3DM → UI Matrix9 thật/screenshot/pointer → performance fixture nếu liên quan. Compile/load hay mẫu planner không đóng các gate khác.

FreeCAD `OpenMatrix9Gui`/FCMacro chỉ cho task legacy được chỉ định: dùng shared `H:/FreeCAD-src/build/relWithDebInfo`, `build/build-om9-plugin.ps1`, `build/run-om9-plugin.ps1`, không clone runtime hoặc root rebuild vì thay OM9. Báo scope legacy rõ.

Lưu command/environment/source fingerprint/result và các phần chưa chạy ở `docs/validation/`; [sync progress](../openmatrix9-workflow/references/progress.md). Không tạo API/build target giả, không nâng trạng thái v2 từ native passes cũ, không publish ngoài yêu cầu.

## Cập nhật nghiệp vụ sau khi code

Sau code và kiểm tra, nếu nghiệp vụ đổi, trình bày nội dung trước/sau và hỏi có lưu vào skill không. Yêu cầu cập nhật skill đã có cho chính thay đổi là đủ, không hỏi lại. Xem [quy tắc chung](../openmatrix9-workflow/references/business-rule-updates.md). Ghi evidence kỹ thuật riêng, không coi nó là đồng ý nghiệp vụ mới.
