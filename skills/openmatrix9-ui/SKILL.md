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

**Quyền bàn phím của viewport và CMD (2026-11-18).** Khi không có lệnh đang chạy, viewport xử lý bàn phím theo cơ chế phím tắt. Phím chưa gán không đi vào CMD. Bấm `E` gọi End Snap một lần cho điểm kế tiếp, kể cả khi End thường trực đang tắt. Đổi nhóm icon, chuyển viewport, đóng menu tiêu đề và chọn đối tượng thông thường vẫn giữ phím tắt. Icon hoặc phím tắt khởi động lệnh đưa focus vào CMD theo prompt. Nếu lệnh yêu cầu chọn đối tượng rồi nhập tham số, chuyển focus sang CMD khi chọn xong đối tượng và tới bước nhập. Sau click nhận điểm trong viewport, kể cả điểm trả từ worker, trả bàn phím cho viewport trừ bước đang yêu cầu nhập như Elevator height. Khi lệnh kết thúc hoặc bị hủy, trả bàn phím cho viewport. Người dùng vẫn có thể bấm trực tiếp vào ô CMD để chủ động nhập lệnh hoặc tham số.

## Command: icon, mặc định và tham số

Khi sửa UI/input/preview/picking của lệnh CAD, áp dụng [hợp đồng nguồn và viewport](../openmatrix9-workflow/references/source-and-viewport-contract.md): đồng bộ đường tương tác với hình học nguồn và kiểm kết quả commit, không dừng ở hình vẽ hoặc handler mở được. Pick/snap phải gắn đúng nguồn và trạng thái; không dùng độ mịn hiển thị để thay dung sai CAD.

Khi kiểm tra lệnh qua UI, đọc cả mục “Kiểm tra từng lệnh” trong hợp đồng: tái dùng phần đã đúng, bổ sung preview nhẹ trước exact khi có chờ và kiểm việc thay preview/cancel/stale/Undo theo đúng nguồn. Cập nhật danh sách `docs/validation/command-source-viewport-checklist.md` của checkout; layout/icon thuần túy ghi căn cứ không ảnh hưởng thay vì tự mở sửa kernel.

Khi triển khai command hoặc binding/hành vi gọi command, đọc [hợp đồng icon/default/options](../openmatrix9-feature-port/references/command-integration-contract.md). Icon/menu/CMD cho cùng biến thể phải tới cùng owner; thao tác mặc định và các tham số gạch chân accelerator phải hoạt động thật theo mẫu Polyline. Không nghiệm thu command chỉ từ icon hiển thị hoặc handler mở được.

## Option hiển thị theo stage (người dùng duyệt 2026-11-02)

Option chỉ hiện **inline trong prompt**; mode/measure/giá trị hiện tại là chữ prompt (`Center of sphere`, `Diameter`), option chỉ là các lựa chọn thay thế còn lại của stage đó. Option của bước sau bị ẩn tới khi thao tác xảy ra (`Length` sau góc Rectangle, `Cube` sau góc chéo, `FromLastPoint`/`UseLast*` sau bản copy đầu); option mà parser từ chối ở stage hiện tại thì không quảng bá. Theo hướng Matrix đã chốt 2026-11-02: Line không có option `Length` (gõ số trần ở prompt cuối là length; `Length=` chỉ còn alias gõ tay), Rectangle không có option `Square` (giữ **Shift** khi pick góc để vuông; chữ `Square` chỉ còn alias). Mode construction giữ chữ trơn, flag/giá trị giữ `Name=Value` như chuẩn Polyline; Cancel là Esc trừ prompt mà contract đã có. Ẩn token không có nghĩa là bỏ đường gõ tay.

Nguồn hành vi là các extract Matrix đã nằm trong `OM9_Milestones/**/L3-*/reference/text` (book 1/2/addendum), không đọc lại PDF. Trước khi sửa một lệnh: đọc `Steps` + `Command Line Options` của ref, lập bảng stage (prompt, input bắt buộc, option khả dụng, giá trị hiện tại, chuyển bước, finish/cancel/undo), so với provider/`describe_command_option`/parser rồi mới sửa. Nếu Matrix khác contract cục bộ đã duyệt (M01 fixture, OWNER-CARDS), ghi cả hai và hỏi — không tự đổi. Sau khi sửa: cập nhật fixture stage, kiểm ba đường kích hoạt, geometry/Undo và ghi evidence.

Đọc [command-option-visibility](references/command-option-visibility.md) cho luật đầy đủ, workflow và các gap còn mở; [command-options-by-menu](references/command-options-by-menu.md) là file tra theo đúng 12 menu (Curve, Surface, Solid, Transform, SubD, Art, Builder, Tools, Gems, Setting, Cutters, Render): mỗi lệnh có option Matrix, trạng thái verified/auto/pending, ref trang và token OM9 hiện tại. Index thô từ 1130 ref L3: `docs/validation/matrix-ref-command-options-2026-11-02.md`; diff option với app: `docs/validation/matrix-ref-option-diff-2026-11-02.md`; ghi chú theo trang: `docs/validation/2026-11-02-matrix-command-prompts.md`.

Khi sửa input điểm, preview hoặc chuyển preview → scene, đọc [hướng kéo/CMD/redraw](../openmatrix9-feature-port/references/interactive-geometry-regressions.md). Chuyển điểm qua parser/owner chung; kiểm preview bám đúng phía con trỏ trong CPlane và hình sau commit. Đối chiếu model/scene rồi kiểm UI thật khi có triệu chứng chớp/mất hình; không suy ra GPU đã hiển thị từ semantic test.

## Viewport GPU đã duyệt

Người dùng duyệt ngày 2026-10-19: viewport desktop chỉ dùng GPU, không giữ nhánh render CPU/software dự phòng. Thiếu GPU phù hợp phải báo lỗi renderer. Model, picking và chuẩn bị dữ liệu vẫn do Rust xử lý; các lớp UI/Gumball qua egui không phải một renderer viewport CPU riêng. Bản mặc định phải dùng GPU mà không cần cờ bật; native OCCT là capability độc lập.

Khi sửa backend hoặc startup, xem bằng chứng trong checkout `implementation/evidence/M02/2026-10-19-gpu-only.md`; kiểm cả bốn viewport và trường hợp thiếu adapter, không dùng kết quả của renderer CPU cũ để nghiệm thu.

## Tốc độ điều hướng viewport

Người dùng đồng ý ghi quy tắc ngày 2026-11-01: orbit Perspective mặc định dùng π/600 radian mỗi điểm logic; một nấc bánh xe làm span nhân 0,81, nấc ngược nhân 1/0,81; pan theo span hiển thị tại độ sâu target. Giữ zoom neo con trỏ và input phân đoạn tỉ lệ. Đây là lựa chọn OM9 suy ra từ native Rhino 5, không phải chứng nhận giống Matrix9; chuột thật, DPI và cảm giác desktop còn cần kiểm tra. Xem `docs/validation/2026-11-01-rhino-inspired-navigation.md`.

## Selection BRep đã duyệt

Người dùng duyệt ngày 2026-10-19 cho OM9: pick BRep giữ root, đầy đủ member path, UUID face/edge/vertex và SourceStamp/publication. Không chọn đường chéo tessellation như cạnh BRep. Publication mới làm pick cũ mất hiệu lực; Undo không làm pick cũ hợp lệ trở lại. Đây là quy tắc OM9 đã duyệt, không tự coi là hành vi Matrix gốc được chứng minh.

Khi triển khai hoặc kiểm tra selection BRep, đối chiếu bằng chứng trong checkout: `implementation/evidence/M02/2026-10-19-brep-selection.md` và các test/runtime được dẫn tại đó. Bằng chứng tự động không thay chữ ký reviewer UI.

## Project Manager nhúng và workspace (người dùng duyệt 2026-11-17)

Khi workspace catalog và Project Manager nhúng cùng tồn tại, giữ nguyên dữ liệu nhúng khi lưu; chỉ nhập vào workspace qua thao tác rõ ràng, cấp lại ID cục bộ, giữ bag legacy inline và từ chối nhập nguyên lô nếu hủy, dữ liệu lỗi hoặc nguồn thay đổi. Đây là quy tắc OM9 do người dùng duyệt, không phải kết luận về định dạng Matrix gốc. Bằng chứng và giới hạn kiểm thử: `docs/validation/2026-11-17-project-manager-p07.md`; nghiệm thu desktop còn mở.

Khi Info của project có bản nháp đang mở, không đổi project qua sidebar và không bắt đầu import/refresh catalog; từ chối áp dụng kết quả import hoặc tải catalog đến trong thời gian đó. Bản nháp kết thúc khi Save, Cancel hoặc đóng cửa sổ. Nếu New/Open thay manager trong trường hợp không có workspace catalog, giữ bản nháp cũ để người dùng sao chép nhưng vô hiệu hóa Save và các thao tác phụ thuộc project; không gắn lại vào ID trùng của catalog mới. Người dùng duyệt quy tắc bảo vệ bản nháp ngày 2026-11-17; desktop và các thời điểm chuyển trạng thái thực vẫn cần nghiệm thu.

Khi đổi thư mục project, nếu catalog nguồn đã bị phiên khác sửa trên đĩa, từ chối chuyển bản cũ; sau khi tải lại catalog mới, người dùng có thể thử lại. Quy tắc OM9 được người dùng duyệt ngày 2026-11-17; kiểm thử Windows và giới hạn cạnh tranh hệ thống tệp được ghi tại `docs/validation/2026-11-17-project-manager-p07.md`.

Trên Windows, khi đổi thư mục project, có thể dùng lại catalog đích đã tồn tại nếu manifest và toàn bộ asset cần thiết khớp chính xác catalog đang chuyển, không ghi đè; từ chối nếu nội dung khác hoặc thiếu asset. Người dùng duyệt ngày 2026-11-17; bằng chứng thử lại sau tiến trình kết thúc và giới hạn tại `docs/validation/2026-11-17-project-manager-p07.md`.

Khi lọc Job Bag, giữ nguyên block lồng nếu các phần tử bên trong cùng một loại đã xác định; từ chối lọc block hỗn hợp hoặc chưa xác định để tránh lấy thiếu hoặc lấy dư. Khi Load, có thể xác định lại loại của block từ archive đã kiểm tra, kể cả khi metadata cũ ghi chưa xác định. Người dùng duyệt ngày 2026-11-17; bằng chứng và giới hạn tại `docs/validation/2026-11-17-project-manager-p07.md`.

## Cập nhật nghiệp vụ sau khi code

Sau code và kiểm tra, nếu nghiệp vụ đổi, trình bày nội dung trước/sau và hỏi có lưu vào skill không. Yêu cầu cập nhật skill đã có cho chính thay đổi là đủ, không hỏi lại. Xem [quy tắc chung](../openmatrix9-workflow/references/business-rule-updates.md). Ghi evidence kỹ thuật riêng, không coi nó là đồng ý nghiệp vụ mới.
