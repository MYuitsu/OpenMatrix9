# Mở OM9 bằng một lần bấm

1. Cài [FreeCAD 1.1.4 Windows x64 chính thức](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4), mặc định tại `C:\Program Files\FreeCAD 1.1`.
2. Giải nén **toàn bộ** gói launcher vào một thư mục.
3. Bấm **Start-OM9.cmd**. Script tự cài hoặc cập nhật plugin trong `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`, tạo shortcut **OpenMatrix9** có icon trên Desktop và trong **Start Menu → OpenMatrix9**, rồi mở đúng workbench.
4. Các lần sau, bấm icon **OpenMatrix9** trên Desktop hoặc tìm OpenMatrix9 trong Start Menu. Cả hai dùng launcher đã lưu trong AppData, nên anh có thể di chuyển hoặc xóa thư mục giải nén sau khi cài xong.

Khi có gói OM9 mới, giải nén gói mới và bấm `Start-OM9.cmd` của gói mới để cập nhật. Bản cũ được giữ trong `%APPDATA%\FreeCAD\v1-1\OM9-backups`, bên ngoài thư mục Mod. Nếu FreeCAD đang mở và cần cập nhật, hãy lưu bản vẽ, đóng FreeCAD rồi chạy lại; script không tự đóng ứng dụng của anh. Khi gói không đổi, script bỏ qua bước chép lại.

Khi mở OM9, launcher tự kiểm tra release trên GitHub ở tiến trình nền. Có bản mới thì báo link tải; mỗi phiên bản chỉ báo một lần. Mất mạng hoặc GitHub giới hạn lượt truy cập vẫn mở OM9 bình thường. Launcher chỉ đọc thông tin release, không tự tải/cài plugin. Bản thử nghiệm mới được ghi rõ trong thông báo. Cập nhật launcher cùng phiên bản plugin cũng được phát hiện theo số revision. Chạy lại script của gói mới để cập nhật; nếu chỉ đổi launcher, plugin không bị chép lại và không cần đóng OM9.

Không cần Administrator, source FreeCAD hay Rust. Bản này dành cho FreeCAD **1.1.4**; nếu cài FreeCAD ở đường dẫn khác, chạy PowerShell với `-FreeCADPath "D:\FreeCAD 1.1\bin\FreeCAD.exe"`. Có thể dùng `-SkipUpdateCheck` khi cần bỏ kiểm tra mạng cho một lần mở.

Log nằm ở `%APPDATA%\FreeCAD\v1-1\OM9-launcher\launcher.log`; mỗi lần mở có thêm file `start-*.json`. Kết quả kiểm tra phiên bản nằm trong `update-check.json`. Cấu hình và bản vẽ nằm ngoài thư mục plugin không bị thay thế. Icon dùng hình thương hiệu OM9 do chủ dự án cung cấp.

Phiên bản dạng `aa.bb.cc-yyyyMMddhhmm` được so sánh theo ba số, rồi thời gian build. Gói cũ dạng `aa.bb.cc` vẫn được nhận; revision launcher chỉ phân biệt các gói có cùng phiên bản ứng dụng. Xem phiên bản đầy đủ trên tag release và tên ZIP trước khi cài.

Đây là bản thử nghiệm. Hãy hoàn tất/hủy lệnh đang vẽ trước khi chuyển tài liệu. Xem hướng dẫn và giới hạn đầy đủ trong thư mục `OpenMatrix9/INSTALL.vi.md` đi kèm.
