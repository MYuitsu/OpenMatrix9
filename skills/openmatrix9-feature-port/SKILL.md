---
name: openmatrix9-feature-port
description: Implement or specify OpenMatrix9 standalone Rust features using milestone contracts, including Rhino 5 3DM, Copy/Paste, Box/Sphere, Gems, Builders, cutters, render, SubD and Emboss; retain Matrix9 UI behavior.
---

# Exact OM9 feature trên nền độc lập

Đọc [đích milestone và giao diện Matrix9](../openmatrix9-workflow/references/milestones-target.md). Resolve checkout, dùng workflow `search` rồi `feature <exact-ID>` và `route feature-port`; đọc spec trả về, migration/backend requirements và options/unknowns của đúng feature.

- Rust trước; [ownership/FFI](../openmatrix9-workflow/references/rust-first.md). OM9 sở hữu model/session/transaction/history, UI đúng Matrix9; kernel adapter không giữ document. Chọn routine QymCAD và OCCT theo capability, không cả host/UI.
- Local milestone specs là nguồn triển khai người dùng đã chọn; không yêu cầu đọc lại PDF cho semantics đã mô tả. Chỉ đọc đúng manual/procedure khi có unknown thật sự ảnh hưởng quyết định. Giữ nguồn, inference và quyết định OM9 riêng; không bịa defaults hoặc bật nhánh chưa có solver.
- Viết record `docs/features/<ID>.md` theo [feature-contract](references/feature-contract.md). Inputs/units/geometry/ownership/error/preview/commit/cancel/History/Undo/Redo/.om9 và UI routes phải có evidence theo đúng effect.
- Builder/History/Styles chỉ đọc khi workflow thực cần. T-Splines khác Clayoo/SubD; MatrixArt khác Emboss. ID/alias/context giữ nguyên, menu/CMD/F6/control dùng cùng dispatcher.
- Các native C++/FreeCAD slices là nguồn port adapter và replay intent; không phải target đã hoàn thành. Dùng build-validation cho executable mới khi đã có target/build config thật; chưa có thì ghi chưa triển khai, không dùng plugin load để đóng gate standalone.
- Đồng bộ [progress](../openmatrix9-workflow/references/progress.md): milestone runtime_implementation/status/traceability, README và ledger đúng phạm vi đã kiểm. Không ghi kết quả standalone vào v1.

## Nghiệp vụ đã duyệt cần đọc theo feature

- 3DM, clipboard, CAD/mesh/cloud, worker60%: [rhino5-3dm-contract](references/rhino5-3dm-contract.md). Giữ semantics đã duyệt; old native scope là historical, milestone có representation/block gates riêng.
- Box/Sphere `OM9-SOLID-012/014`: [solid-box-sphere-contract](references/solid-box-sphere-contract.md). Giữ correction Box mặc định, frames/Cube/FitPoints/AroundCurve/Tangent/chọn nghiệm; replay trên own geometry/identity.
- Text và Boolean Two Objects `OM9-SOLID-008/004`: [solid-prompt-contract](references/solid-prompt-contract.md). Nội dung/Height, lựa chọn U/D/I, retry, Esc và dispatcher/transaction chung; người dùng duyệt 2026-10-23.
- Document/cảnh `OM9-L2-01-03`: [document-scene-contract](references/document-scene-contract.md). Giữ định danh bền vững, envelope có kiểu, luật layer/khoá, group là lớp phủ, block dùng chung topology, extension slot có namespace, hợp đồng selection, batch thay đổi, snapshot/restore nguyên tử và provenance; owner duyệt 2026-10-17.

## Cập nhật nghiệp vụ sau khi code

Sau code và kiểm tra, nếu nghiệp vụ đổi, trình bày nội dung trước/sau và hỏi có lưu vào skill không. Yêu cầu cập nhật skill đã có cho chính thay đổi là đủ, không hỏi lại. Xem [quy tắc chung](../openmatrix9-workflow/references/business-rule-updates.md). Ghi evidence kỹ thuật riêng, không coi nó là đồng ý nghiệp vụ mới.
