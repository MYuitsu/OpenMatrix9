# Cài OpenMatrix9 cho FreeCAD chính thức

Bản thử nghiệm này dành cho **Windows x64, FreeCAD 1.1.4 chính thức** (Python 3.11, Qt 6.8.3, OCCT 7.8.1). Không cần source FreeCAD, Rust hay Visual Studio để sử dụng.

## 1. Cài FreeCAD và mở OM9 bằng launcher

1. Cài Windows x64 từ **[release FreeCAD 1.1.4 chính thức](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4)**, rồi mở FreeCAD một lần. Kiểm tra Help → About FreeCAD: **1.1.4**; tên thư mục có thể chỉ ghi `FreeCAD 1.1`.
2. Mở **[trang Releases](https://github.com/MYuitsu/OpenMatrix9/releases)**. Chọn bản **0.3.5-202610121520**, mở **Assets** và tải **OpenMatrix9-0.3.5-202610121520-SetupLauncher-Windows-x64.zip**.
3. Giải nén ZIP SetupLauncher vào một thư mục. Giữ thư mục **OpenMatrix9**, **launcher-manifest.json** và các file **Start-OM9** cùng nhau, rồi bấm **Start-OM9.cmd**. Launcher đọc thư mục OpenMatrix9 đã giải nén.

Launcher tự cài/cập nhật trong `%APPDATA%\FreeCAD\v1-1\Mod\OpenMatrix9`, tạo icon **OpenMatrix9** trên Desktop và Start Menu rồi mở workbench. Những lần sau dùng icon Desktop; không cần giữ thư mục giải nén. Không cần Administrator, source FreeCAD, Rust hay Visual Studio.

Tải **SHA256SUMS.txt** từ cùng release để đối chiếu checksum của gói trước khi cài. ZIP SetupLauncher dùng để cài tự động; ZIP FreeCAD-1.1.4 dùng để cài plugin thủ công. Không cần tải Source code hay artifact trong Actions.

Cập nhật bằng Start-OM9.cmd từ gói mới. Bản cũ được giữ ngoài Mod trong `v1-1\OM9-backups`; gói không đổi thì không chép lại plugin. Nếu plugin đang được dùng và cần thay, lưu và đóng FreeCAD rồi chạy lại. Launcher không tự đóng ứng dụng; gói thiếu hoặc lỗi cache giữ lại bản đã cài. Log ở `v1-1\OM9-launcher`.

Launcher kiểm tra **release đã phát hành** ở tiến trình nền mỗi lần mở, báo một lần khi có phiên bản ứng dụng hoặc revision launcher mới. Artifact candidate không xuất hiện qua bộ kiểm tra release này. Mất mạng vẫn mở OM9; tải và cài bản mới là thao tác tường minh. Chỉ đổi launcher thì không chép lại plugin. Trạng thái ở `OM9-launcher\update-check.json`.

Ví dụ FreeCAD đã cài: `C:\Program Files\FreeCAD 1.1\bin\FreeCAD.exe`. Các bước sau dành cho cài thủ công.

## 2. Tải OM9 và đặt đúng thư mục

1. Đóng các cửa sổ FreeCAD trước khi thay plugin.
2. Tải **OpenMatrix9-0.3.5-202610121520-FreeCAD-1.1.4-Windows-x64.zip** trong **Assets** ở [trang Releases](https://github.com/MYuitsu/OpenMatrix9/releases).
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

## Phạm vi v0.0.3 lịch sử và giới hạn host còn áp dụng

Các mục dưới mô tả checkpoint v0.0.3 đã phát hành. Bản Gumball có bằng chứng kiểm thử cục bộ riêng; CI chỉ build/đóng gói theo yêu cầu. Xem [quy trình và bằng chứng](docs/validation/2026-10-12-local-tests-ci-build-only.md). Các ghi chú lịch sử không xác nhận toàn bộ bản mới.

- `0.0.3` bổ sung build công khai trên GitHub và launcher mới, trên nền tương thích FreeCAD chính thức của `0.0.2`. Xem [phạm vi CI và lỗi tooling lịch sử còn lại](docs/validation/2026-10-11-release003-ci.md). Bằng chứng Rhino/Matrix của các nhánh dev trước không phải chứng nhận lại toàn bộ bản stock này.
- Chỉnh curve, bảng layer, Copy/Paste, BRep trong các mẫu đã kiểm tra, worker import và thao tác viewport có báo cáo kèm release. Full openNURBS, mọi thuật toán trang sức và chuyển đầy đủ bảng màu/khóa bằng thao tác Matrix sản phẩm vẫn chưa được nghiệm thu toàn bộ.
- Làm việc trên **một tài liệu active** khi đang vẽ. FreeCAD 1.1 có thể commit draft của tài liệu trước khi bắt đầu transaction ở tài liệu khác; bản dev mới có API khác. Hãy hoàn tất hoặc hủy lệnh trước khi chuyển tài liệu.
- Các luồng nâng cao phụ thuộc bản sửa core FreeCAD dev, như copy dữ liệu Hatch dùng `PropertyFileIncluded`, chưa được chứng nhận trên host stock.
- Binary của `0.0.1` dành cho FreeCAD dev 27.1; không dùng chung với binary stock `0.0.3`. Hãy giữ plugin và host đúng phiên bản.

## Khi không thấy workbench hoặc có lỗi tải module

Kiểm tra lại phiên bản FreeCAD, đường dẫn versioned `v1-1\Mod`, cấu trúc giải nén và đủ hai file trong `bin`. Khởi động lại sau khi thay plugin. Mở View → Panels → Report view và gửi thông báo nếu vẫn lỗi. Không lấy DLL từ bản FreeCAD khác để chép đè.

Để gỡ OM9: đóng FreeCAD, chuyển `OpenMatrix9` ra ngoài `Mod`. Dự án đã lưu vẫn còn nguyên. Để kiểm tra tải xuống, so SHA256 với `SHA256SUMS.txt` của [release](https://github.com/MYuitsu/OpenMatrix9/releases).

Phiên bản có dạng `aa.bb.cc-yyyyMMddhhmm`. Launcher so sánh ba số trước, rồi thời gian build, rồi revision launcher của cùng phiên bản ứng dụng. Gói cũ dạng `aa.bb.cc` vẫn được hỗ trợ. Với candidate, chọn ZIP từ đúng commit nhánh đã build thành công; với gói đã phát hành, chọn thời gian khớp tag release. Thời gian dùng múi giờ Asia/Saigon; chỉ đổi `aa` khi người dùng yêu cầu, mỗi đặc tả hoàn tất tăng `bb`, mỗi tác vụ hoàn tất tăng `cc`.
