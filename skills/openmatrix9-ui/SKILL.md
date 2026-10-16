---
name: openmatrix9-ui
description: "Implement or review the Matrix9 interface described by OpenMatrix9 specs v2: menus, sidebar, F6, CMD, Layers, Builder, Gumball and four viewports in standalone OM9."
---

# Giao diện Matrix9, implementation OM9 độc lập

Đọc [đích v2](../openmatrix9-workflow/references/spec-v2-target.md), `OpenMatrix9_Codex_Spec_v2/MATRIX9_UI_CONTRACT.md` trong checkout và [workspace contract](../openmatrix9-workspace-contract/SKILL.md). QymCAD chỉ cung cấp code mẫu/routine bên trong; UI/layout/style/workflow phải đúng Matrix9 như mô tả hiện tại. Chuyển ownership không cho phép redesign.

1. Workflow `route ui-menu`/`ui-viewports`, rồi exact ID. Đọc spec v2 và `Resources/menu/MainMenu.ini`; [ui-evidence](references/ui-evidence.md) chọn nguồn đúng component.
2. Rust sở hữu UI/session state và dispatcher; UI/scene adapter hiển thị snapshot, gửi event. Menu/CMD/F6/control dùng cùng schema, giữ names/options/units/Enter/Esc/focus/lifecycle; command chưa kiểm chứng visible-disabled với lý do.
3. Own camera/scene/picking/document và bốn view phải thao tác thật. Tái dùng routine QymCAD/framework sau rà dependencies/license, không đem workflow upstream vào sản phẩm hoặc gọi FreeCAD views trong target.
4. Dùng authored assets và palette/meaning đã duyệt; không trích thương mại để ship. Runtime không phụ thuộc ref/manuals/đường dẫn máy này. Missing reference tra `docs/openmatrix9-reference-locations.json`, không tự thay bằng screenshot QymCAD.
5. [Build-validation](../openmatrix9-build-validation/SKILL.md) kiểm actual app/screenshot/pointer/DPI/lifecycle. Old Qt/Coin/FreeCAD fixtures chỉ là baseline nguồn port. Sync v2 và ledger đúng evidence.

[Command/grid theme](references/command-grid-theme.md) và workspace references giữ Command xanh, canvas đen, finite world grid, titles/dropdowns, pinned copyable CMD và Line/Polyline interaction đã duyệt. Defaults/unknowns không lấy từ upstream. UI giống hình không chứng minh solver đã hoạt động.

## Cập nhật nghiệp vụ sau khi code

Sau code và kiểm tra, nếu nghiệp vụ đổi, trình bày nội dung trước/sau và hỏi có lưu vào skill không. Yêu cầu cập nhật skill đã có cho chính thay đổi là đủ, không hỏi lại. Xem [quy tắc chung](../openmatrix9-workflow/references/business-rule-updates.md). Ghi evidence kỹ thuật riêng, không coi nó là đồng ý nghiệp vụ mới.
