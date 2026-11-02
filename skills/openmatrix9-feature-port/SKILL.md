---
name: openmatrix9-feature-port
description: Implement or specify OpenMatrix9 standalone Rust features using milestone contracts, including Rhino 5 3DM, Copy/Paste, Box/Sphere, Gems, Builders, cutters, render, SubD and Emboss; retain Matrix9 UI behavior; include command icon bindings, default invocation and underlined parameter shortcuts.
---

# Exact OM9 feature trên nền độc lập

Đọc [đích milestone và giao diện Matrix9](../openmatrix9-workflow/references/milestones-target.md). Resolve checkout, dùng workflow `search` rồi `feature <exact-ID>` và `route feature-port`; đọc spec trả về, migration/backend requirements và options/unknowns của đúng feature.

- Rust trước; [ownership/FFI](../openmatrix9-workflow/references/rust-first.md). OM9 sở hữu model/session/transaction/history, UI đúng Matrix9; kernel adapter không giữ document. Chọn routine QymCAD và OCCT theo capability, không cả host/UI.
- Local milestone specs là nguồn triển khai người dùng đã chọn; không yêu cầu đọc lại PDF cho semantics đã mô tả. Chỉ đọc đúng manual/procedure khi có unknown thật sự ảnh hưởng quyết định. Giữ nguồn, inference và quyết định OM9 riêng; không bịa defaults hoặc bật nhánh chưa có solver.
- Viết record `docs/features/<ID>.md` theo [feature-contract](references/feature-contract.md). Inputs/units/geometry/ownership/error/preview/commit/cancel/History/Undo/Redo/.om9 và UI routes phải có evidence theo đúng effect.
- Builder/History/Styles chỉ đọc khi workflow thực cần. T-Splines khác Clayoo/SubD; MatrixArt khác Emboss. ID/alias/context giữ nguyên, menu/CMD/F6/control dùng cùng dispatcher.
- Các native C++/FreeCAD slices là nguồn port adapter và replay intent; không phải target đã hoàn thành. Dùng build-validation cho executable mới khi đã có target/build config thật; chưa có thì ghi chưa triển khai, không dùng plugin load để đóng gate standalone.
- Đồng bộ [progress](../openmatrix9-workflow/references/progress.md): milestone runtime_implementation/status/traceability, README và ledger đúng phạm vi đã kiểm. Không ghi kết quả standalone vào v1.

## Tích hợp command từ icon đến input

Khi implement/port/sửa command, bắt buộc làm đồng bộ binding icon đúng feature/biến thể, thao tác mặc định khi gọi lệnh và tham số có phím tắt gạch chân theo mẫu Polyline. Đọc [hợp đồng icon/default/options](references/command-integration-contract.md) trước khi sửa handler hoặc UI command; chỉ kết luận command hoàn tất khi các trục này và kết quả thực đã được kiểm.

## Nghiệp vụ đã duyệt cần đọc theo feature

- Curve `OM9-L2-03-16`: với lệnh Curve đang chạy, nhấp phải không kèm phím bổ trợ trong viewport gửi input theo cùng đường xử lý như Enter; kéo phải vẫn điều hướng. Lệnh giữ kiểm tra dữ liệu và không hoàn tất khi chưa đủ input hợp lệ. Quy tắc áp dụng cho phiên đang chờ; lệnh đã thực thi xong không có bước xác nhận giả. Người dùng duyệt 2026-10-28; đây là quy tắc OM9, không tự chứng nhận hành vi Matrix gốc. Kiểm cả Enter/nhấp phải, kéo phải, input thiếu và kết quả thực theo từng command/stage; evidence một phần không chứng minh toàn bộ command.
- Lệnh nhận điểm/hình học có hướng hoặc sửa parser/preview: đọc [hướng kéo, nhập điểm và regression](references/interactive-geometry-regressions.md). Giữ dấu theo CPlane, chung phép dựng preview/commit, kiểm bốn hướng kéo và tọa độ CMD thực; baseline xanh thiếu các case này không đủ để nghiệm thu.

- 3DM, clipboard, CAD/mesh/cloud, worker60%: [rhino5-3dm-contract](references/rhino5-3dm-contract.md). Giữ semantics đã duyệt; old native scope là historical, milestone có representation/block gates riêng.
- Box/Sphere `OM9-SOLID-012/014`: [solid-box-sphere-contract](references/solid-box-sphere-contract.md). Giữ correction Box mặc định, frames/Cube/FitPoints/AroundCurve/Tangent/chọn nghiệm; replay trên own geometry/identity.
- Text và Boolean Two Objects `OM9-SOLID-008/004`: [solid-prompt-contract](references/solid-prompt-contract.md). Text/Height; Boolean chọn hai BRep, preview năm kết quả và ẩn tạm đầu vào, Next/nhấp viewport, Enter xác nhận, DeleteInput No/Yes; người dùng duyệt lưu 2026-10-28.
- Boolean Difference `OM9-SOLID-001`: [solid-prompt-contract](references/solid-prompt-contract.md). Chọn và hợp nhóm target/cutter, `DeleteInput` mặc định No, Undo/Cancel không tạo transaction; người dùng duyệt lưu 2026-10-28.
- Boolean Union `OM9-SOLID-003`: [solid-prompt-contract](references/solid-prompt-contract.md). Chọn ít nhất hai BRep, `DeleteInput` mặc định No, Undo/Cancel không tạo transaction; người dùng duyệt lưu 2026-10-28.
- Boolean Intersection `OM9-SOLID-002`: [solid-prompt-contract](references/solid-prompt-contract.md). Chọn hai nhóm BRep, hợp từng nhóm rồi lấy giao; `DeleteInput` mặc định No; người dùng duyệt lưu 2026-10-28.
- Boolean Split `SolidBooleanSplit`: [solid-prompt-contract](references/solid-prompt-contract.md). Chọn hai tập BRep, cho phép trùng đầu vào, tạo các phần riêng; `DeleteInput` mặc định No; người dùng duyệt lưu 2026-10-28.
- Extract Surface `OM9-SOLID-006`: [solid-prompt-contract](references/solid-prompt-contract.md). Chọn nhiều mặt BRep trên một hoặc nhiều nguồn; Copy=No tách mặt, Copy=Yes giữ nguồn; Undo/commit nguyên tử; người dùng duyệt lưu 2026-10-28.
- Pipe `OM9-SOLID-011`: [solid-prompt-contract](references/solid-prompt-contract.md). Trên đường tròn đầy đủ, bán kính không đổi, Cap=Round tạo cùng khối xuyến như Flat, kể cả Thick; người dùng duyệt lưu 2026-10-28, chưa xác minh Matrix gốc.
- Truncated Cone `OM9-SOLID-020`: [solid-prompt-contract](references/solid-prompt-contract.md). Đáy hỗ trợ tâm, 2Point, 3Point, FitPoints, AroundCurve và một nhánh Tangent + Point; các mode và hình học tổng quát còn mở.
- Solid đùn và Boss: [solid-prompt-contract](references/solid-prompt-contract.md). Boss đã kiểm polygon, tròn và ellipse NURBS với mặt trụ nguyên, kể cả đùn xiên; polygon, tròn, ellipse NURBS với biên phẳng nghiêng; hình học tổng quát còn mở.
- Document/cảnh `OM9-L2-01-03`: [document-scene-contract](references/document-scene-contract.md). Giữ định danh bền vững, envelope có kiểu, luật layer/khoá, group là lớp phủ, block dùng chung topology, extension slot có namespace, hợp đồng selection, batch thay đổi, snapshot/restore nguyên tử và provenance; owner duyệt 2026-10-17.
- Ring builder family + builder styles `OM9-L2-04-03`/`OM9-L2-03-12`: [builder-family-contract](references/builder-family-contract.md). Giữ bốn mode Eternity/Signet/Raised/Award với tag/command và mặc định đã duyệt, mesh solid kín từ section quay quanh trục, `MIN_WALL_MM = 0.3`, stone frames thay vì gem solid, preset `om9.builder_style.v1` và cập nhật history cùng owner id; người dùng duyệt 2026-10-27.
- Prong/metal/bead/mass settings `OM9-L2-04-15`: [settings-prong-metal-contract](references/settings-prong-metal-contract.md). Giữ góc 0° = ngang hàng và 90° = dọc hàng, `4 Prongs Close` sát hơn `4 Prongs 0` mà vẫn hai prong, prong của hai anchor trùng điểm thì gộp thành một prong chia sẻ (`merged_prongs`), `Metal Piece` `Border=On` bám border đã commit và `Mode=Curve` bám curve được pick, bead `Count=`/`Spacing=` loại trừ nhau, `SettingsMass` loại cutter; người dùng duyệt 2026-10-27.

## Cập nhật nghiệp vụ sau khi code

Sau code và kiểm tra, nếu nghiệp vụ đổi, trình bày nội dung trước/sau và hỏi có lưu vào skill không. Yêu cầu cập nhật skill đã có cho chính thay đổi là đủ, không hỏi lại. Xem [quy tắc chung](../openmatrix9-workflow/references/business-rule-updates.md). Ghi evidence kỹ thuật riêng, không coi nó là đồng ý nghiệp vụ mới.
