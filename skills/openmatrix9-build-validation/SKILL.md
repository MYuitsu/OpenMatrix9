---
name: openmatrix9-build-validation
description: Build and validate OpenMatrix9 standalone Rust, selected OCCT/openNURBS adapters and Matrix9 UI against milestone contracts; use FreeCAD plugin checks only for explicit legacy maintenance.
---

# Kiểm chứng đúng target OM9

Với thay đổi CAD, kiểm theo [hợp đồng nguồn và viewport](../openmatrix9-workflow/references/source-and-viewport-contract.md): bằng chứng phải bao gồm hình học nguồn, viewport chịu ảnh hưởng và binding chọn nguồn; phân biệt semantic/scene với GPU/desktop. Hai profile display khi liên quan phải giữ cùng nguồn và dung sai tính toán, ngoại trừ thay đổi mà hợp đồng lệnh cho phép.

Kiểm tra một lệnh còn phải áp dụng mục “Kiểm tra từng lệnh” của hợp đồng: ghi phần nguồn/representation/preview sớm/commit-Undo đã có và còn thiếu vào `docs/validation/command-source-viewport-checklist.md` của checkout, bổ sung trong phạm vi được giao. Khi đo preview, phân biệt thời gian thấy hình tạm với exact commit và publish cuối; không gộp chúng thành một FPS hay suy ra tất cả đã đạt từ build pass. Yêu cầu chỉ chạy test hoặc chỉ đọc không tự mở triển khai.

Đọc [đích milestone](../openmatrix9-workflow/references/milestones-target.md) và [Rust/FFI](../openmatrix9-workflow/references/rust-first.md). Workflow `route build-validation` cung cấp policy/design, không chứng minh executable đã tồn tại.

Theo [validation-playbook](references/validation-playbook.md), resolve target/crate/build config thực trước khi chạy. Rust core hiện có và examples là portable/legacy component evidence, không thay executable standalone. Không reconfigure/build FreeCAD root cho v2. Toolkit dependencies, ABI, version và scheduler phải kiểm theo operation được chọn.

Kiểm tra phù hợp change: semantic Rust tests → native bridge/error/ownership/tolerance → actual app lifecycle/document/Undo/.om9/3DM → UI Matrix9 thật/screenshot/pointer → performance fixture nếu liên quan. Compile/load hay mẫu planner không đóng các gate khác.

Khi kiểm lệnh nhận điểm, frame hoặc preview, dùng [oracle hướng kéo và tọa độ CMD](../openmatrix9-feature-port/references/interactive-geometry-regressions.md): kiểm vị trí canonical, đủ dấu liên quan và preview/commit; phân biệt CMD invocation với nhập tọa độ, dispatcher với click UI, scene publication với GPU present. Lưu regression đỏ → xanh khi sửa; skill-only không đóng lỗi sản phẩm.

FreeCAD `OpenMatrix9Gui`/FCMacro chỉ cho task legacy được chỉ định: dùng shared `H:/FreeCAD-src/build/relWithDebInfo`, `build/build-om9-plugin.ps1`, `build/run-om9-plugin.ps1`, không clone runtime hoặc root rebuild vì thay OM9. Báo scope legacy rõ.

Lưu command/environment/source fingerprint/result và các phần chưa chạy ở `docs/validation/`; [sync progress](../openmatrix9-workflow/references/progress.md). Không tạo API/build target giả, không nâng trạng thái v2 từ native passes cũ, không publish ngoài yêu cầu.

## Xử lý ổ đầy

Chỉ đạo người dùng 2026-11-14: khi ổ workspace hoặc ổ build thiếu chỗ, chủ động dọn build cache cũ và tiếp tục thay vì dừng build chờ người dùng. Kiểm tra dung lượng trước khi ghi/build nếu có dấu hiệu thiếu chỗ; tránh ghi đè file khi ổ đã đầy. Khi build chưa chạy, dùng đúng script và phạm vi của ổ:

- Ổ H: chạy `rtk proxy python tools/clean-h-old-builds.py --apply` để dọn artifact Cargo cũ trong hai target H: được script chỉ định. Script mặc định giữ các mục dưới 7 ngày, bỏ qua đường dẫn chuyển hướng và từ chối xóa khi Cargo/rustc đang chạy.
- Ổ L: với cache incremental OM9, chạy `rtk proxy powershell -NoProfile -ExecutionPolicy Bypass -File tools/clean-l-rust-build.ps1 -Apply`; script mặc định chỉ dọn cache quá 30 ngày và không xóa dependencies/executable.
- Với run validation, dùng `rtk proxy python tools/prune_build_artifacts.py --apply`; giữ 10 run mới nhất và mọi run được `implementation/**` hoặc `docs/**` trích dẫn.

Đo lại dung lượng, sau đó tiếp tục thao tác/build bị gián đoạn. Nếu vẫn thiếu, có thể giảm ngưỡng tuổi của các script cache được phép xuống 1 ngày, sau khi kiểm tra không có build đang hoạt động; không tự dừng hoặc giết build. Giữ tối đa hai target build, tái dùng `L:/om9-target`, dùng `-j 2`; không tạo target mới để né thiếu chỗ. Không xóa source, model, tài liệu, SDK, artifact bằng chứng được tham chiếu, file người dùng hay dữ liệu ngoài phạm vi script. Chỉ báo cần người dùng xử lý khi không còn cache được phép dọn hoặc đã dọn mà vẫn thiếu chỗ.

## Cập nhật nghiệp vụ sau khi code

Sau code và kiểm tra, nếu nghiệp vụ đổi, trình bày nội dung trước/sau và hỏi có lưu vào skill không. Yêu cầu cập nhật skill đã có cho chính thay đổi là đủ, không hỏi lại. Xem [quy tắc chung](../openmatrix9-workflow/references/business-rule-updates.md). Ghi evidence kỹ thuật riêng, không coi nó là đồng ý nghiệp vụ mới.
