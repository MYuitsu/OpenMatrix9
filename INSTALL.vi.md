# Cài OpenMatrix9 0.0.2 cho FreeCAD chính thức

Bản thử nghiệm này dành cho **Windows x64, FreeCAD 1.1.4 chính thức** (Python 3.11, Qt 6.8.3, OCCT 7.8.1). Không cần source FreeCAD, Rust hay Visual Studio để sử dụng.

## 1. Cài FreeCAD

Tải bộ cài Windows x64 từ [release FreeCAD 1.1.4 chính thức](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4), cài và mở một lần. Vào Help → About FreeCAD để kiểm tra phiên bản **1.1.4**; tên thư mục cài có thể chỉ ghi `FreeCAD 1.1`.

Ví dụ chương trình đã cài: `C:\Program Files\FreeCAD 1.1\bin\FreeCAD.exe`.

## 2. Tải OM9 và đặt đúng thư mục

1. Đóng các cửa sổ FreeCAD trước khi thay plugin.
2. Tải [OpenMatrix9-0.0.2-FreeCAD-1.1.4-Windows-x64.zip](https://github.com/MYuitsu/OpenMatrix9/releases/download/v0.0.2/OpenMatrix9-0.0.2-FreeCAD-1.1.4-Windows-x64.zip).
3. Giải nén; bên trong có thư mục **OpenMatrix9**.
4. Nhấn **Win + R**, dán `%APPDATA%\FreeCAD\v1-1\Mod` rồi Enter. Nếu chưa có thư mục này, tạo nó.
5. Chép thư mục **OpenMatrix9** vào đó. Nếu đã có bản cũ, chuyển bản cũ ra ngoài thư mục `Mod` trước để có thể khôi phục.

Đường dẫn đúng có dạng:

```text
%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9\
    Init.py
    InitGui.py
    ThreeDm.py
    Resources\
    bin\OpenMatrix9Gui.pyd
    bin\OM9ThreeDmImportWorker.exe
```

`InitGui.py` phải nằm ngay trong thư mục `OpenMatrix9`, tránh giải nén thành `OpenMatrix9\OpenMatrix9\InitGui.py`. Không chép DLL/plugin vào `Program Files` và không cần mở FreeCAD bằng Administrator để dùng OM9.

## 3. Mở OM9

Mở lại **FreeCAD đã cài**, chọn **OpenMatrix9** trong danh sách workbench. Giao diện OM9 gồm sidebar, vùng Command, viewport và bảng Layers 32 màu. Các lệnh đã hỗ trợ được bật; các mục chưa port vẫn bị vô hiệu hóa.

Kiểm tra nhanh: tạo tài liệu mới, vẽ một Line, Undo rồi Redo; thử đổi màu, khóa/mở layer trong bảng Layers. Đối tượng bị khóa không cho chỉnh sửa/chọn theo luồng OM9 đã hỗ trợ.

## 4. Chuyển bản vẽ

Trong OM9, dùng lệnh **Copy/Paste** cho đối tượng đã chọn; **Copy Session** chuyển cả bảng layer, gồm layer trống, màu, khóa/ẩn và layer đang dùng. Cũng có thể import/export file `.3dm` và lưu dự án FreeCAD `.FCStd`.

Giữ clipboard yên khi trao đổi. Các script kiểm chứng Matrix trong source là công cụ phát triển; **không cần chạy chúng để cài hoặc mở OM9**, và OM9 là workbench FreeCAD. Bản release này không cài plugin `.rhp` hay adapter sản phẩm vào Matrix.

## Phạm vi bản thử nghiệm

- `0.0.2` bổ sung tương thích FreeCAD chính thức; bộ kiểm tra mới đo trực tiếp host 1.1.4. Bằng chứng Rhino/Matrix của các nhánh dev trước không phải chứng nhận lại toàn bộ bản stock này.
- Chỉnh curve, bảng layer, Copy/Paste, BRep trong các mẫu đã kiểm tra, worker import và thao tác viewport có báo cáo kèm release. Full openNURBS, mọi thuật toán trang sức và chuyển đầy đủ bảng màu/khóa bằng thao tác Matrix sản phẩm vẫn chưa được nghiệm thu toàn bộ.
- Làm việc trên **một tài liệu active** khi đang vẽ. FreeCAD 1.1 có thể commit draft của tài liệu trước khi bắt đầu transaction ở tài liệu khác; bản dev mới có API khác. Hãy hoàn tất hoặc hủy lệnh trước khi chuyển tài liệu.
- Các luồng nâng cao phụ thuộc bản sửa core FreeCAD dev, như copy dữ liệu Hatch dùng `PropertyFileIncluded`, chưa được chứng nhận trên host stock.
- Binary của `0.0.1` dành cho FreeCAD dev 27.1; không dùng chung với binary stock `0.0.2`. Hãy giữ plugin và host đúng phiên bản.

## Khi không thấy workbench hoặc có lỗi tải module

Kiểm tra lại phiên bản FreeCAD, đường dẫn versioned `v1-1\Mod`, cấu trúc giải nén và đủ hai file trong `bin`. Khởi động lại sau khi thay plugin. Mở View → Panels → Report view và gửi thông báo nếu vẫn lỗi. Không lấy DLL từ bản FreeCAD khác để chép đè.

Để gỡ OM9: đóng FreeCAD, chuyển `OpenMatrix9` ra ngoài `Mod`. Dự án đã lưu vẫn còn nguyên. Để kiểm tra tải xuống, so SHA256 với `SHA256SUMS.txt` của [release 0.0.2](https://github.com/MYuitsu/OpenMatrix9/releases/tag/v0.0.2).
