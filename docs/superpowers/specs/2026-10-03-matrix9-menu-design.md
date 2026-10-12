# Thiết kế menu Matrix9 cho OpenMatrix9

## Mục tiêu và quyết định đã thống nhất

Người dùng muốn FreeCAD khi vào OpenMatrix9 có giao diện giống ảnh Matrix9 đã cung cấp. Người dùng yêu cầu tiếp tục dùng Rust và quy trình Superpowers. Rust quản lý dữ liệu menu, trạng thái nhóm đang chọn và lịch sử thao tác; C++/Qt tích hợp widget vào FreeCAD và thực thi những lệnh FreeCAD đã được ánh xạ.

Đặc tả này triển khai phần menu trước. Bố cục bốn viewport, lưới, vùng Command màu xanh và hệ thống tạo trang sức sẽ là các phần tiếp theo của giao diện đã thống nhất. Giai đoạn này không được nhận là hoàn thành toàn bộ màn hình Matrix9.

## Nguồn tham chiếu

- `ref/MainMenu.ini`: 18 nhóm, 11 nút nhanh; thứ tự, màu nhóm và tên icon là dữ liệu gốc.
- `ref/Matrix.rui`: tên macro, tooltip, GUID và tài nguyên bitmap dùng để đối chiếu icon.
- `ref/matrix9/vb6-lite/Matrix90/frmMenuTools.frm`: MAIN MENU, các control mẫu và vùng icon.
- Các form `frmMenuIconHistory`, `frmCMenuDisplay`, `frmMenuSnaps`, `frmMenuInfoSettings`, `frmMenuLayers`, `frmMenuProjects`: cấu trúc bảy khối sidebar.
- `ctrTitleBar.123` và `ctrTitleBar.ctx`: tiêu đề, font và nút đóng. Đuôi `.123` vẫn được đọc như dữ liệu UserControl; không đổi tên bản xuất của người dùng.
- Các `.frx/.ctx`: tài nguyên nhị phân; `OleObjectBlob` chưa được xem là PNG có thể dùng trực tiếp.
- Ảnh tham chiếu: `C:/Users/Admin/AppData/Local/Temp/codex-clipboard-5ec1c5b2-dd82-497b-b17c-3bd016400af5.png`.

Các file decompile là nguồn dữ liệu tham khảo. Không đưa mã assembly hoặc chương trình VB6 vào tiến trình FreeCAD. Tọa độ xuất từ VB6 phải được đối chiếu đơn vị, control cha và ảnh trước khi dùng; kích thước lúc thiết kế form có thể khác kích thước lúc chạy.

## Phạm vi phần menu

1. Một sidebar OpenMatrix9 bên trái, rộng mặc định 300 pixel logic Qt, có thể điều chỉnh và cuộn khi cửa sổ thấp.
2. Bảy khối theo thứ tự: ICON HISTORY, MAIN MENU, DISPLAY, SNAPS, INFO & SETTINGS, LAYERS, PROJECTS.
3. MAIN MENU là khối tương tác đầy đủ trong giai đoạn này: chọn nhóm, hiển thị lưới nút của nhóm, tooltip, 11 nút nhanh và Reset.
4. ICON HISTORY hiển thị tối đa 20 lần thực thi lệnh thành công trong phiên, lần mới nhất trước; nút Undo/Redo ánh xạ tới FreeCAD và phản ánh trạng thái có thể thực thi.
5. DISPLAY, SNAPS, INFO & SETTINGS, LAYERS và PROJECTS có tiêu đề và bố cục tham chiếu. Chỉ các thao tác có ánh xạ FreeCAD được kiểm chứng mới được bật. Quản lý project riêng, snapping hình học riêng và chức năng trang sức chưa thuộc giai đoạn này.
6. Giữ các menu chuẩn của FreeCAD truy cập được. Không dùng tên lệnh Rhino như lệnh FreeCAD và không chạy chuỗi macro Rhino.

## Ngoại hình và tương tác

- Nền sidebar `#333333`; ICON HISTORY có nền `#646464` theo form xuất. Thanh tiêu đề dùng màu `#A19F94` từ `ctrTitleBar`, kiểm tra thứ tự màu BGR của VB6 khi chuyển sang RGB.
- Tiêu đề in đậm, font ưu tiên Verdana, khoảng 8.25 pt; font thay thế do Qt chọn nếu máy thiếu Verdana.
- Thanh tiêu đề khoảng 15 pixel logic; nút icon chính khoảng 25 pixel logic, hình 24 pixel. Dùng bố cục theo độ rộng khả dụng để không mất nút ở DPI 125%, 150% và 200%.
- Dòng nhóm lấy từ INI, giữ thứ tự. `Clayoo`, `Emboss`, `Settings` là tên trong bộ cài được giải mã; ảnh có `SubD`, `Art`, `Setting`. Giai đoạn đầu giữ tên dữ liệu gốc và ghi khác biệt khi kiểm tra ảnh, không giả định các nhóm đó tương đương về chức năng.
- Nhóm được chọn có dấu hiệu nổi bật và lưới icon bên dưới. Reset trở về nhóm đầu và trạng thái thu gọn mặc định; không xóa tài liệu hoặc dữ liệu FreeCAD.
- Tiêu đề khối có thể thu gọn; nút đóng ẩn khối trong phiên, thao tác Reset khôi phục các khối.
- Icon thiếu có ký hiệu thay thế nhất quán và tooltip chứa tên lệnh, không gây lỗi tải workbench.
- Lệnh chưa hỗ trợ có trạng thái vô hiệu hóa và tooltip nêu rõ chưa được triển khai trong OpenMatrix9; bố cục vẫn giữ vị trí để so sánh với nguồn.

## Kiến trúc

### Rust

Giữ crate `openmatrix9_rust`, edition 2024, loại `staticlib`. Sửa `lib.rs` và `commands.rs` để thực sự biên dịch Rust và cung cấp đủ các symbol trong `Gui/RustBridge.h`.

Tách bộ đọc INI, danh mục menu, ánh xạ lệnh và trạng thái giao diện. Bộ đọc không phụ thuộc Qt hoặc FreeCAD. Đầu vào là văn bản UTF-8 của INI; đầu ra giữ ID nhóm, tên, loại, màu, danh sách icon và 11 nút nhanh. Kiểm tra cấu trúc, số lượng được khai báo, khóa trùng và mục thiếu trước khi cung cấp dữ liệu cho UI.

Rust xử lý sự kiện chọn nhóm, thu gọn khối, Reset và ghi lịch sử. Lệnh chỉ được ghi lịch sử sau khi host xác nhận đã thực thi thành công. Danh mục và trạng thái có thể kiểm thử bằng `cargo test` mà không khởi động FreeCAD.

### C++/Qt

Thêm lớp sidebar trong `Gui`, được workbench tạo một lần rồi tái sử dụng. Host tạo widget từ dữ liệu Rust, tải icon, chuyển sự kiện về Rust và hiển thị trạng thái mới. Phần C++ chỉ ánh xạ thao tác được Rust yêu cầu tới API/CommandManager của FreeCAD.

Widget Qt chỉ được tạo và cập nhật trên GUI thread. Sidebar sở hữu rõ ràng các widget con; không dùng con trỏ widget phía Rust.

### ABI và vòng đời

Giữ ABI C hiện có để tích hợp workbench; mở rộng bằng các getter và hàm sự kiện cho sidebar. Chuỗi Rust trả về là UTF-8 có NUL cuối, địa chỉ ổn định theo thời hạn được API quy định; C++ sao chép trước khi dùng lâu dài. Không truyền kiểu Rust, STL hoặc Qt qua ABI.

Mọi chỉ số ngoài phạm vi trả về null/0/trạng thái lỗi. Rust không được unwind qua ABI. Việc gọi API trạng thái được giới hạn trên GUI thread; không tạo singleton có thể bị ghi đồng thời.

Khi kích hoạt OpenMatrix9, sidebar hiện ra. Khi rời workbench, sidebar ẩn và những dock FreeCAD mà OpenMatrix9 đã thay đổi được khôi phục đúng trạng thái trước đó. Bật/tắt nhiều lần không tạo widget, kết nối signal hoặc lệnh trùng.

## Tài nguyên và phân phối

Tạo dữ liệu menu và icon phân phối trong thư mục tài nguyên của module. Bản cài chạy được khi không có `ref`, không phụ thuộc đường dẫn tuyệt đối trên máy hiện tại. Bộ chuyển đổi tham chiếu đọc INI/RUI/form để tạo dữ liệu có manifest truy nguồn; bản xuất nguyên gốc vẫn giữ nguyên.

Ưu tiên icon có ánh xạ xác nhận qua tên macro/GUID. Bitmap có cấu trúc nhận diện được được trích và kiểm tra kích thước; tài nguyên OLE chưa giải mã có fallback, không đoán offset để tạo hình sai. Không chạy dữ liệu nhị phân được trích.

Không bổ sung framework giao diện Rust độc lập hoặc thay Qt của FreeCAD. Không chuyển logic menu sang Python. Python `InitGui.py` chỉ đăng ký workbench theo cơ chế hiện có.

## Ánh xạ lệnh và xử lý lỗi

Ánh xạ lệnh cơ bản gồm New, Open, Save, Save As, Undo, Redo và các góc nhìn/Fit All khi tìm được ID tương ứng trong mã nguồn FreeCAD hiện tại. Availability phụ thuộc tài liệu và CommandManager. Nếu command không tồn tại, nút chuyển sang vô hiệu hóa và host báo lý do trong Report view, không đăng ký lệnh giả là đã hoạt động.

Lỗi dữ liệu menu phải có thông báo chỉ ra nhóm/khóa bị lỗi; không làm sập FreeCAD. Thiếu icon không chặn việc dựng sidebar. Lỗi thực thi lệnh không được thêm vào lịch sử.

## Kiểm chứng và tiêu chí nghiệm thu

1. `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` và build release crate đều thành công.
2. Kiểm thử bộ đọc bằng dữ liệu tham chiếu thực tế, UTF-8/BOM/CRLF, khóa trùng, số lượng sai, mục thiếu và chỉ số ngoài phạm vi.
3. Kiểm thử trạng thái chọn nhóm, thu gọn, Reset; lịch sử giới hạn 20 và loại trừ lệnh thất bại.
4. Build/link target OpenMatrix9Gui với ABI đủ symbol và phân phối đủ tài nguyên.
5. Mở FreeCAD, kích hoạt OpenMatrix9: bảy khối hiện đúng thứ tự; MAIN MENU hiển thị 18 nhóm và 11 nút nhanh; tên và thứ tự icon khớp INI.
6. New/Open/Save và lệnh xem được hỗ trợ thực thi qua FreeCAD. Lệnh chưa hỗ trợ hiện rõ trạng thái vô hiệu hóa.
7. Chuyển workbench qua lại ít nhất ba lần; không tạo sidebar trùng và khôi phục dock đúng. Kiểm tra khi chưa có tài liệu và khi đã mở tài liệu.
8. Chụp sidebar và đối chiếu ảnh Matrix9; kiểm tra cửa sổ thấp và DPI 100%, 150%, 200%. Báo rõ trường hợp DPI chưa kiểm tra được trên máy.
9. Kiểm tra bản phân phối không cần thư mục `ref`; mọi trường hợp fallback icon được liệt kê. Không tuyên bố khôi phục đầy đủ icon hoặc hành vi gốc nếu còn fallback.

## Trình tự tiếp theo

### Điều chỉnh theo yêu cầu người dùng 2026-10-04

Yêu cầu mới nhất sau khi đổi màu: workspace sử dụng PNG Matrix90 RGB +5. Ưu tiên bộ derivative đã xác minh, không cộng màu lần nữa; ảnh vẽ lại và SVG là fallback. Bố cục, Rust state/ABI và nguyên tắc disabled giữ nguyên. Bằng chứng hiện tại: `docs/validation/2026-10-04-rgb-workspace.md`. Quy định ưu tiên ảnh vẽ lại dưới đây là lịch sử trước yêu cầu mới này.

Ưu tiên ảnh người dùng vẽ lại trong `OpenMatrix9_named_icon_crops`, giữ byte/nguồn manifest. Bốn ảnh MAIN MENU, DISPLAY/SNAPS, INFO/LAYERS và PROJECTS gửi sau đó là chuẩn bố cục và màu; không yêu cầu dùng artwork gốc. Thiếu icon riêng thì dùng SVG tự viết có nhãn fallback trong manifest. Icon disabled giữ màu, lệnh vẫn bị chặn. Các bảng phụ dựng đủ control trình bày theo ảnh nhưng chưa port hành vi. Xem `docs/validation/2026-10-04-modern-sidebar.md` cho bằng chứng hiện tại; hướng dẫn ưu tiên kho gốc trước đây được thay thế.

Sau khi người dùng duyệt đặc tả này, lập kế hoạch thực hiện theo task, kiểm thử và các interface cụ thể. Sau khi kế hoạch được duyệt, triển khai menu rồi tiếp tục phần viewport/vùng lệnh của màn hình khởi động.
