# Cài OpenMatrix9 0.0.1 trên Windows

Đây là bản **thử nghiệm**, dành cho Windows 64-bit. Tải hai file tại
[Release 0.0.1](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.0.1):

1. **FreeCAD-OM9-27.1-Windows-x64.zip** — FreeCAD portable tương thích, đã build sẵn.
2. **OpenMatrix9-0.0.1-Windows-x64.zip** — plugin OM9.

## Cài và mở

1. Giải nén gói FreeCAD vào một thư mục có quyền ghi, ví dụ `C:\CAD`.
2. Giải nén gói plugin. Chuyển cả thư mục **OpenMatrix9** vào
   **`C:\CAD\FreeCAD-OM9-27.1\Mod`**.
3. Mở **`C:\CAD\FreeCAD-OM9-27.1\Mod\OpenMatrix9\Start-OM9.cmd`**.
   Launcher tự mở FreeCAD và chọn workbench OpenMatrix9.

Cấu trúc cuối cùng phải là:

```text
C:\CAD\FreeCAD-OM9-27.1\
  bin\FreeCAD.exe
  Runtime\
  Mod\
    Part\
    OpenMatrix9\
      Init.py
      InitGui.py
      Start-OM9.cmd
      Launch-OM9.ps1
      Resources\
      bin\
        OpenMatrix9Gui.pyd
        OM9ThreeDmImportWorker.exe
```

Không để lồng thành `Mod\OpenMatrix9\OpenMatrix9`. Không cần Python, Rust,
Visual Studio hoặc Pixi để sử dụng. Cấu hình OM9 nằm riêng tại
`%APPDATA%\OpenMatrix9\0.0.1`; cập nhật plugin bằng cách đóng ứng dụng rồi thay
thư mục `Mod\OpenMatrix9`. Các file bản vẽ do người dùng lưu được giữ ở vị trí
người dùng chọn.

## FreeCAD tải ở đâu?

Để dùng **binary OM9 0.0.1 này**, tải gói FreeCAD tương thích ngay trong release
OM9 phía trên. Host được đóng gói từ build dùng chung hiện có: FreeCAD 27.1.0dev,
revision `21d36cfa1eb110a1d0667050ff31706298805bbd`, Python 3.13, Qt6.

FreeCAD thông thường có tại [trang tải chính thức](https://www.freecad.org/downloads.php)
và [GitHub FreeCAD](https://github.com/FreeCAD/FreeCAD/releases). Binary plugin
này chưa được xác nhận với những gói đó. Muốn dùng host khác cần build OM9 với
SDK/ABI tương ứng; chép riêng `.pyd` vào FreeCAD khác phiên bản có thể không nạp được.

## Thao tác và phạm vi bản 0.0.1

- Chọn workbench **OpenMatrix9** nếu đổi sang workbench khác. Lệnh chưa hỗ trợ
  vẫn hiện nhưng bị vô hiệu hóa.
- Import/Export 3DM, Copy/Paste hình học và Undo/Redo trong OM9 đã có phạm vi
  kiểm chứng; không phải hỗ trợ toàn bộ openNURBS hoặc toàn bộ Matrix9.
- Bảng layer của OM9 có màu, khóa/ẩn và layer hiện hành. Kiểm chứng chuyển đủ
  palette hai chiều trong Matrix đã đạt 16/16 với adapter riêng; adapter đó
  chưa tích hợp vào giao diện chính Matrix và không nằm trong gói cài OM9 này.
  Copy/Paste mặc định của Matrix không được coi là đã mang đầy đủ palette.
- Undo/Redo nghiệm thu thuộc OM9. Không chuyển lịch sử Undo giữa hai phần mềm.
- Worker mặc định dùng 60% CPU logic, làm tròn xuống và tối thiểu một worker.

## Khi mở không được

Chạy đúng `Start-OM9.cmd`, không mở trực tiếp `bin\FreeCAD.exe` của gói này vì
launcher thiết lập Python và DLL của host portable. Kiểm tra cấu trúc thư mục
trên. Xem lỗi ở:

```text
%APPDATA%\OpenMatrix9\0.0.1\last-launch.stdout.log
%APPDATA%\OpenMatrix9\0.0.1\last-launch.stderr.log
```

Trong FreeCAD, bật **View → Panels → Report view** để xem lỗi workbench.
`SHA256SUMS.txt` trong release dùng để đối chiếu file tải; có thể chạy
`Get-FileHash -Algorithm SHA256 <đường-dẫn-file>` trong PowerShell.

## Source và giấy phép

Source OM9 được gắn tag `v0.0.1`. Source host tương ứng ở asset
`FreeCAD-OM9-27.1-source.zip` và
[repository FreeCAD của dự án](https://github.com/MYuitsu/FreeCAD/tree/21d36cfa1eb110a1d0667050ff31706298805bbd).
Giữ các giấy phép trong gói; xem `LICENSE`, `THIRD_PARTY_NOTICES.md`,
`Resources/licenses` và thư mục `licenses` của host.
