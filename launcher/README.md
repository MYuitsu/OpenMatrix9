# Mở OM9 bằng một lần bấm

1. Cài [FreeCAD 1.1.4 Windows x64 chính thức](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4), mặc định tại `C:\Program Files\FreeCAD 1.1`.
2. Giải nén **toàn bộ** gói launcher vào một thư mục.
3. Bấm **Start-OM9.cmd**. Script tự cài hoặc cập nhật plugin trong `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`, tạo shortcut **OpenMatrix9** có icon trên Desktop và trong **Start Menu → OpenMatrix9**, rồi mở đúng workbench.
4. Các lần sau, bấm icon **OpenMatrix9** trên Desktop hoặc tìm OpenMatrix9 trong Start Menu. Cả hai dùng launcher đã lưu trong AppData, nên anh có thể di chuyển hoặc xóa thư mục giải nén sau khi cài xong.

Khi có gói OM9 mới, có thể dùng nút **Update** hoặc giải nén gói mới và bấm `Start-OM9.cmd`. Bản cũ được giữ trong `%APPDATA%\FreeCAD\v1-1\OM9-backups`, bên ngoài thư mục Mod. Khi gói không đổi, script bỏ qua bước chép lại.

Launcher **chỉ kiểm tra GitHub khi anh mở OM9** bằng icon hoặc `Start-OM9.cmd`, một lần cho lần mở đó. Không có lịch hay vòng lặp kiểm tra khi đang làm việc. Nếu có bản mới, nhấn **Update**: launcher tự tải gói SetupLauncher và `SHA256SUMS.txt` đúng release, xác minh checksum cùng toàn bộ file trong manifest, rồi tự cài. Không cần mở trang tải, tải ZIP hay giải nén bằng tay. Nếu OM9/FreeCAD đang mở, cửa sổ cập nhật chờ anh **lưu và đóng**; sau đó tự cài và mở OM9 lại với cùng host/profile. Nó không tự đóng hoặc buộc tắt ứng dụng. Có thể nhấn **Cancel** lúc tải/chờ đóng; bản đang cài vẫn được giữ.

Chọn **Later** để tiếp tục làm việc; lần mở sau vẫn có thể cập nhật. Mất mạng/GitHub giới hạn truy cập vẫn mở OM9 bình thường. Gói hỏng bị từ chối, cài lỗi thì khôi phục plugin và launcher trước đó. Bản thử nghiệm được ghi rõ; nhận cả revision launcher mới, không hạ phiên bản. Không dùng nút Update để cập nhật FreeCAD; bản launcher này vẫn yêu cầu FreeCAD **1.1.4**.

Không cần Administrator, source FreeCAD hay Rust. Bản này dành cho FreeCAD **1.1.4**; nếu cài FreeCAD ở đường dẫn khác, chạy PowerShell với `-FreeCADPath "D:\FreeCAD 1.1\bin\FreeCAD.exe"`. Có thể dùng `-SkipUpdateCheck` khi cần bỏ kiểm tra mạng cho một lần mở.

Log nằm ở `%APPDATA%\FreeCAD\v1-1\OM9-launcher\launcher.log`; mỗi lần mở có thêm file `start-*.json`. Kết quả kiểm tra/cập nhật nằm trong `update-check.json`; gói tải và staging nằm dưới `OM9-launcher\updates`. Khi cài xong, trạng thái `installed` xác nhận cài thành công và đã yêu cầu mở lại; xem `start-*.json` mới để kiểm tra workbench khởi động. Cấu hình và bản vẽ nằm ngoài thư mục plugin không bị thay thế. Icon dùng hình thương hiệu OM9 do chủ dự án cung cấp.

Đây là bản thử nghiệm. Hãy hoàn tất/hủy lệnh đang vẽ trước khi chuyển tài liệu. Xem hướng dẫn và giới hạn đầy đủ trong thư mục `OpenMatrix9/INSTALL.vi.md` đi kèm.
