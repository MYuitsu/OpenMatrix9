---
name: openmatrix9-ui
description: "Implement or review the Matrix9 interface described by OpenMatrix9 milestone specs: menus, sidebar, F6, CMD, Layers, Builder, Gumball and four viewports in standalone OM9."
---

# Giao diện Matrix9, implementation OM9 độc lập

Đọc [đích milestone](../openmatrix9-workflow/references/milestones-target.md), M01 L2-03-01/03-07/03-10 và các reference cục bộ trong checkout và [workspace contract](../openmatrix9-workspace-contract/SKILL.md). QymCAD chỉ cung cấp code mẫu/routine bên trong; UI/layout/style/workflow phải đúng Matrix9 như mô tả hiện tại. Chuyển ownership không cho phép redesign.

1. Workflow `route ui-menu`/`ui-viewports`, rồi exact ID. Đọc milestone spec và `Resources/menu/MainMenu.ini`; [ui-evidence](references/ui-evidence.md) chọn nguồn đúng component.
2. Rust sở hữu UI/session state và dispatcher; UI/scene adapter hiển thị snapshot, gửi event. Menu/CMD/F6/control dùng cùng schema, giữ names/options/units/Enter/Esc/focus/lifecycle; command chưa kiểm chứng visible-disabled với lý do.
3. Own camera/scene/picking/document và bốn view phải thao tác thật. Tái dùng routine QymCAD/framework sau rà dependencies/license, không đem workflow upstream vào sản phẩm hoặc gọi FreeCAD views trong target.
4. Dùng authored assets và palette/meaning đã duyệt; không trích thương mại để ship. Runtime không phụ thuộc ref/manuals/đường dẫn máy này. Missing reference tra `docs/openmatrix9-reference-locations.json`, không tự thay bằng screenshot QymCAD.
5. [Build-validation](../openmatrix9-build-validation/SKILL.md) kiểm actual app/screenshot/pointer/DPI/lifecycle. Old Qt/Coin/FreeCAD fixtures chỉ là baseline nguồn port. Sync milestone và ledger đúng evidence.

[Command/grid theme](references/command-grid-theme.md) và workspace references giữ Command xanh, canvas đen, finite world grid, titles/dropdowns, pinned copyable CMD và Line/Polyline interaction đã duyệt. Defaults/unknowns không lấy từ upstream. UI giống hình không chứng minh solver đã hoạt động.

## Viewport GPU đã duyệt

Người dùng duyệt ngày 2026-10-19: viewport desktop chỉ dùng GPU, không giữ nhánh render CPU/software dự phòng. Thiếu GPU phù hợp phải báo lỗi renderer. Model, picking và chuẩn bị dữ liệu vẫn do Rust xử lý; các lớp UI/Gumball qua egui không phải một renderer viewport CPU riêng. Bản mặc định phải dùng GPU mà không cần cờ bật; native OCCT là capability độc lập.

Khi sửa backend hoặc startup, xem bằng chứng trong checkout `implementation/evidence/M02/2026-10-19-gpu-only.md`; kiểm cả bốn viewport và trường hợp thiếu adapter, không dùng kết quả của renderer CPU cũ để nghiệm thu.

## Selection BRep đã duyệt

Người dùng duyệt ngày 2026-10-19 cho OM9: pick BRep giữ root, đầy đủ member path, UUID face/edge/vertex và SourceStamp/publication. Không chọn đường chéo tessellation như cạnh BRep. Publication mới làm pick cũ mất hiệu lực; Undo không làm pick cũ hợp lệ trở lại. Đây là quy tắc OM9 đã duyệt, không tự coi là hành vi Matrix gốc được chứng minh.

Khi triển khai hoặc kiểm tra selection BRep, đối chiếu bằng chứng trong checkout: `implementation/evidence/M02/2026-10-19-brep-selection.md` và các test/runtime được dẫn tại đó. Bằng chứng tự động không thay chữ ký reviewer UI.

## Cập nhật nghiệp vụ sau khi code

Sau code và kiểm tra, nếu nghiệp vụ đổi, trình bày nội dung trước/sau và hỏi có lưu vào skill không. Yêu cầu cập nhật skill đã có cho chính thay đổi là đủ, không hỏi lại. Xem [quy tắc chung](../openmatrix9-workflow/references/business-rule-updates.md). Ghi evidence kỹ thuật riêng, không coi nó là đồng ý nghiệp vụ mới.
