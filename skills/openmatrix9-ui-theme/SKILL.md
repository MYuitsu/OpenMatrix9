---
name: openmatrix9-ui-theme
description: Use when creating, changing, or reviewing an OpenMatrix9 popup, dock or floating panel, Builder, dialog, menu, tooltip, or the theme and readability of its controls. Covers shared egui styling and panel presentation. Use openmatrix9-ui for broader workspace and CAD interaction changes.
---

# Theme popup và dock OpenMatrix9

Dùng skill này ngay khi công việc OM9 liên quan đến popup, cửa sổ nổi, khung dock hoặc khả năng đọc của điều khiển. Khi thêm hoặc sửa hành vi lệnh, đọc thêm hợp đồng của [openmatrix9-ui](../openmatrix9-ui/SKILL.md).

## Điểm vào

- Tìm palette và style tại `crates/om9-app/src/ui/theme.rs`. Dùng các màu có vai trò rõ ràng cho nền, chữ, focus, chọn, cảnh báo và hành động chính. Tránh đặt một palette riêng trong từng lệnh.
- Bảng công cụ có phiên làm việc dùng `ToolPanel` và `ui/tool_host.rs`. Cùng một nội dung phải đọc được ở dock trái, dock phải và dạng nổi. Dùng ID ổn định qua hai dock, thả nổi và đổi profile để giữ trạng thái widget.
- Menu ngắn, tooltip và ComboBox bám điều khiển xuất phát. Dùng style phù hợp từ theme hoặc UI cha. Hộp xác nhận giữ modal và vòng đời focus. Hộp thoại hệ điều hành dùng style của hệ điều hành.

## Khả năng đọc

- Ưu tiên đồ họa có ích khi người dùng phải nhận dạng hoặc chọn hình học, đối tượng, mẫu hay cỡ. Dùng hình thu nhỏ từ dữ liệu hoặc hình học OM9 thật làm phần chính của ô chọn, đặt tên và số đo dưới hình, giữ nhãn truy cập bằng bàn phím. Ví dụ cả Gem Shape và Size đều hiện hình đá. Trên trang Size, giữ đúng tỷ lệ dài/rộng của viên đá, vẽ hình lớn dần theo cỡ trong cùng trang và giữ hình đủ rõ để nhận dạng. Ghi số đo trên ô và hiển thị đầy đủ kèm đơn vị khi rê chuột.

- Lấy các bảng Gem làm mẫu cho công cụ nhiều tham số. Tiêu đề 15 điểm logic, chữ chính và giá trị 13 điểm, nhãn phụ 12 điểm. Dùng màu chữ sáng trên nền tối, nút chính có nền mint và chữ tối. Dùng viền cho trạng thái được chọn để dễ phân biệt với hover.
- Khi sửa vòng chọn cỡ Gem Loader, áp dụng [quy tắc bảng cỡ, tab và Depth %](../openmatrix9-feature-port/references/feature-contract.md#gem-loader-size-catalog) của feature.
- Bố trí lại cột hoặc cho ngắt dòng ở dock hẹp. Kiểm chọn hình, nhãn, giá trị, nút đóng và nút Apply khi khung nhỏ. Dùng vùng cuộn cho tham số dài để phần tóm tắt và nút chính dễ tiếp cận. Chọn mật độ theo loại bảng, giữ dãy icon workspace theo hợp đồng hiện có.
- Kiểm text nhập liệu, trạng thái disabled và widget nổi thoát khỏi vùng vẽ của cha. Dùng style có phạm vi cho bảng cần chỉnh. Giữ theme CMD và viewport theo hợp đồng hiện hành.

## Kiểm chứng

- Mở bảng qua entry point thật. Kiểm bên trái, bên phải và dạng nổi. Dùng ảnh cửa sổ thật để đánh giá nhãn và tương phản ở chiều rộng hẹp và DPI liên quan.
- Khi bảng điều khiển lệnh CAD, kiểm chọn nguồn, xem trước, Apply, Cancel và Undo nếu phần trình bày có thể ảnh hưởng đến chúng. Di chuyển bố cục giữ tham số và preview trong cùng phiên; nút X và Esc tiếp tục theo vòng đời của lệnh.
- Ghi rõ phần đã quan sát trên Windows và phần đã chạy headless. Đối chiếu bản kế hoạch ở `docs/plans/2026-11-22-dock-popup-theme.md` và bản ghi kiểm thử tại `docs/validation/2026-11-22-dock-popup-theme.md` trong checkout.
