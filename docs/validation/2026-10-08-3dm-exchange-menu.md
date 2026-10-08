# Menu Import/Export 3DM — 2026-10-08

Theo yêu cầu ưu tiên giao diện, bản source phát triển cung cấp hai mục trực tiếp ở đầu menu **OpenMatrix9**: **Import 3DM...** và **Export Selected 3DM...**. Toolbar **3DM** dùng icon Import/Export chuẩn của FreeCAD. Menu File con và CMD tiếp tục gọi cùng command IDs và cùng handler.

Tạo hoặc mở project trước khi Import. Export bật khi chọn cả đối tượng trong project; hộp thoại mặc định bảo toàn dữ liệu nguồn, đích Rhino 5/V5. Geometry only vẫn là lựa chọn tường minh của handler hiện có. Không đổi chính sách tương thích, không bỏ guard chống mất dữ liệu.

Build/link và kiểm thử ứng dụng chạy tuần tự. Runtime preview riêng: `H:/FreeCAD-src/build/om9-ui-3dm-sdk`, liên kết ABI với SDK cũ; CMake thêm output root tùy chọn, mặc định giữ đường cũ. Không thay module đang được cửa sổ người dùng tải. Source chính chưa tích hợp.

## Bằng chứng

- RED trên SDK trước sửa: thất bại đúng kiểm tra hai command ở đầu menu.
- GREEN trên binary mới: **18/18 kiểm tra FreeCAD thật**, **75 Rust tests**, không thất bại.
- Hộp thoại Import/Export được mở và hủy; hủy không sửa document. Xuất hộp 2×3×4 thành V5 ON_Brep hợp lệ rồi nhập lại thành hình CAD chỉnh sửa được, thể tích 24 mm³. Chuyển workbench ba lần không nhân đôi toolbar.
- Screenshot menu và toolbar lấy trực tiếp từ Qt widget, đã xem kiểm chứng. Screenshot toàn cửa sổ không dùng làm bằng chứng viewport vì QWidget grab không chụp đúng OpenGL framebuffer.
- [Manifest hashes](3dm-exchange-menu-20261008/manifest.json), [FreeCAD results](3dm-exchange-menu-20261008/results.json), [menu](3dm-exchange-menu-20261008/3dm-menu-popup.png), [toolbar](3dm-exchange-menu-20261008/3dm-toolbar.png). Build và test logs giữ cùng thư mục.

Đây là kiểm chứng UI và luồng trao đổi đơn giản trên binary mới; không thay thế regression hình học đầy đủ hoặc Rhino 5 thực tế. Full openNURBS vẫn có 7 đợt chưa đóng; số bộ test tương lai chưa xác định. Không có bộ Rhino mới cần chạy chỉ vì thay menu này.

## Mở bản UI thử nhẫn

Win+R: `powershell -NoProfile -ExecutionPolicy Bypass -File "H:\FreeCAD-src\build\launch-om9-ring.ps1"`

Launcher dùng runtime preview và profile riêng, mở hai file nhẫn trong hai project, không ghi đè nguồn. Bản chứng minh khởi động ghi tại `H:/FreeCAD-src/build/om9-manual-ring-ui-20261008/startup.json`.
