# Command-line suggestions — 2026-11-01

## Hành vi
- Khi ô `Command:` đang focus và có chữ (chưa có khoảng trắng), app hiện popup **Available commands**
  ngay trên dòng lệnh.
- Nguồn gợi ý là `assets/features.json`, chỉ lấy feature có `Controller::availability(feature.id)`
  thành công; command trùng được gộp. Không bịa command ngoài catalog.
- Xếp hạng: khớp chính xác → prefix → chứa chuỗi; trong cùng hạng, tên ngắn hơn trước, rồi alphabet.
  Tối đa 8 dòng.
- Điều khiển:
  - `↑`/`↓`: chọn dòng (không còn nhảy lịch sử khi popup mở).
  - `Tab`: điền command đã chọn kèm dấu cách.
  - `Enter`: nếu dòng chọn khác draft thì điền; nếu draft đã đúng command thì submit như cũ.
  - Click chuột vào dòng: điền command.
- Popup tự ẩn khi có khoảng trắng/đối số, khi đang chạy command có options, hoặc mất focus.

## Kiểm chứng
```text
cargo test -p om9-app --lib typed_command_offers_available_suggestions_and_enter_completes
1 passed
```
Test khẳng định:
- gõ `Bo` → danh sách có `Box` đầu tiên và mọi tên đều có trong features catalog;
- `Enter` điền `Box `;
- gõ đúng `Box` rồi `Enter` vẫn submit command thay vì tự điền lại.

`om9-app --lib`: **320 passed, 3 failed**; 3 fail là icon-size oracle và Split widget có sẵn,
không liên quan command line.

## File
- `crates/om9-app/src/ui/command.rs`
- `crates/om9-app/src/ui.rs`
- `crates/om9-app/src/session/command_options_tests.rs` (charitable solid Cancel exception:
  prompt solid chỉ quảng bá bốn mode; Cancel/Esc do input dùng chung xử lý)

## Giới hạn
- Chỉ gợi ý tên command cấp đầu; gợi ý giá trị option vẫn dùng token/options hiện có.
- Popup dùng con trỏ/nội dung egui; acceptance OS-mouse cho riêng autocomplete chưa có.
- Catalog hiện 444 command; command chỉ tồn tại dạng cú pháp đặc biệt (ví dụ `Gumball Snappy`)
  không nằm trong danh sách gợi ý cấp đầu.
