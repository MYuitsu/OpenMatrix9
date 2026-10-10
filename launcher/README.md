# Mở OM9 bằng một lần bấm

1. Cài [FreeCAD 1.1.4 Windows x64 chính thức](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4), mặc định tại `C:\Program Files\FreeCAD 1.1`.
2. Giải nén **toàn bộ** gói launcher vào một thư mục.
3. Bấm **Start-OM9.cmd**. Script tự cài hoặc cập nhật plugin trong `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`, tạo shortcut **OpenMatrix9** có icon trên Desktop, rồi mở đúng workbench.
4. Các lần sau, bấm icon **OpenMatrix9** trên Desktop. Shortcut dùng launcher đã lưu trong AppData, nên anh có thể di chuyển hoặc xóa thư mục giải nén sau khi cài xong.

Khi có gói OM9 mới, giải nén gói mới và bấm `Start-OM9.cmd` của gói mới để cập nhật. Bản cũ được giữ trong `%APPDATA%\FreeCAD\v1-1\OM9-backups`, bên ngoài thư mục Mod. Nếu FreeCAD đang mở và cần cập nhật, hãy lưu bản vẽ, đóng FreeCAD rồi chạy lại; script không tự đóng ứng dụng của anh. Khi gói không đổi, script bỏ qua bước chép lại.

Không cần Administrator, source FreeCAD hay Rust. Script dùng plugin đi kèm, không tự tải từ mạng. Bản này dành cho FreeCAD **1.1.4**; nếu cài FreeCAD ở đường dẫn khác, chạy PowerShell với `-FreeCADPath "D:\FreeCAD 1.1\bin\FreeCAD.exe"`.

Log nằm ở `%APPDATA%\FreeCAD\v1-1\OM9-launcher\launcher.log`; mỗi lần mở có thêm file `start-*.json`. Cấu hình và bản vẽ nằm ngoài thư mục plugin không bị thay thế. Icon dùng hình thương hiệu OM9 do chủ dự án cung cấp.

Đây là bản thử nghiệm. Hãy hoàn tất/hủy lệnh đang vẽ trước khi chuyển tài liệu. Xem hướng dẫn và giới hạn đầy đủ trong thư mục `OpenMatrix9/INSTALL.vi.md` đi kèm.
