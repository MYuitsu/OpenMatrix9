# Phím tắt Matrix9 trong OpenMatrix9

Nguồn: các mô tả tại `ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs/01-core`. Bảng này ghi các phím mặc định được nêu rõ trong specs và trạng thái thực tế của bản thử ngày 2026-10-05.

| Phím | Lệnh / tác dụng gốc | Trạng thái bản thử | Spec |
|---|---|---|---|
| F2 | `CommandHistory` | Mở/ẩn lịch sử nhập CMD và đầu ra Report view; chưa đủ toàn bộ tùy chọn lịch sử Rhino | OM9-INFO-004 |
| F4 | Gốc mặt phẳng dựng hiện tại | Nhận điểm gốc trong Line, Polyline, Distance, Angle, PictureFrame; chỉ F4 không kèm Ctrl/Alt/Shift, ngoài hộp thoại | OM9-VIEWPORT-001 |
| F5 | `CenterViewport` | Đưa camera của viewport đang dùng về tâm, giữ mức zoom | OM9-VIEW-012 |
| F6 | Menu ngữ cảnh | Có 8 nhóm gốc; nội dung đọc từ ContextMenu.xml gốc theo lựa chọn. Chỉ lệnh đã có handler được bật | OM9-F6-001 |
| F7 | Lưới viewport hiện tại | Bật/tắt riêng khung đang dùng, kể cả khi CMD có focus | OM9-DISPLAY-004 |
| F8 | `Ortho` | Bật/tắt trạng thái lưu, dùng cho chuột Line/Polyline và PictureFrame | OM9-SNAP-014 |
| F10 | `PointsOn` | Đã giữ đúng phím/tên; công cụ chỉnh control points chưa có | OM9-TOP11-001 |
| Ctrl+T | `Properties` | Mở bảng thuộc tính FreeCAD của đối tượng được chọn | OM9-INFO-001 |
| Ctrl+Q | `Group` | Đã giữ đúng phím/tên; Group chưa có handler hình học | OM9-UTIL-012 |
| Ctrl+W | `UnGroup` | Đã giữ đúng phím/tên; UnGroup chưa có handler hình học | OM9-UTIL-015 |
| Ctrl+Alt+C | `gvCenterObjects` | Đã giữ đúng phím/tên; thao tác đưa đối tượng về tâm chưa có | OM9-UTIL-011 |

Giữ Shift đảo tạm Ortho khi đặt điểm bằng chuột. Theo OM9-SNAP-014, Main O-Snap phải On; khi Off, Ortho không ràng buộc điểm. Tọa độ nhập CMD được giữ nguyên. Distance/Angle chưa dùng trạng thái F8 để ràng buộc điểm chuột.

Ctrl + kéo chuột trái zoom viewport hiện tại. Enter xác nhận CMD hoặc kết thúc Polyline trong viewport; Esc hủy công cụ đang chạy. Các tổ hợp sửa văn bản như Ctrl+A/C/V/Z vẫn theo ô nhập liệu.

F10, Ctrl+Q/W và Ctrl+Alt+C hiện báo lệnh chưa khả dụng. Khi dùng OpenMatrix9, Ctrl+Q/W không gọi thao tác thoát/đóng tài liệu của host. Bảng phím chỉ hoạt động khi workbench OpenMatrix9 đang bật; hộp thoại và menu đang mở nhận phím của chính chúng.

F6 hiện phân biệt không chọn, một Curve, Surface, Polysurface và lựa chọn mặc định. Chưa đủ điều kiện Gem, grips, các tập lựa chọn đặc biệt, chỉnh menu, builder, Report và Materials. F2 chưa có toàn bộ lịch sử theo thứ tự thời gian hoặc đầy đủ các tùy chọn gốc.

Specs hiện có chưa đủ bằng chứng để gán F1/F3/F9/F11/F12 hoặc tái tạo toàn bộ keymap Rhino được người dùng tùy biến. Không coi bảng này là bằng chứng đã hoàn tất mọi phím/chức năng Matrix9. Kiểm tra bản thử được ghi tại `docs/validation/2026-10-05-core-keyboard.md`.
