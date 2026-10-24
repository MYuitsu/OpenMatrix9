# Prompt Solid đã được duyệt

Người dùng duyệt ngày 2026-10-23 cho standalone OM9. Đây là quy tắc OM9 đã được
duyệt, không phải chứng nhận hành vi Matrix gốc hay nghiệm thu M03-UI.

## Text — OM9-SOLID-008

- Solid > Text hỏi nội dung, rồi Height. Giữ khoảng trắng và chữ số trong nội dung;
  không diễn giải chúng thành tham số kích thước ở bước nhập nội dung.
- Height sai cho nhập lại tại bước Height. Esc hủy mà không sửa tài liệu.

## Boolean Two Objects — OM9-SOLID-004

- Solid > Boolean Two Objects hỏi Operation, nhận `U`/`Union`, `D`/`Difference`,
  `I`/`Intersection`, không phân biệt hoa thường.
- Lựa chọn sai giữ prompt để nhập lại.

Menu và CMD dùng chung dispatcher và một transaction cho mỗi thao tác thành công.
Việc duyệt quy tắc prompt không đóng gate M03-UI; người dùng xác nhận chưa duyệt UI.
