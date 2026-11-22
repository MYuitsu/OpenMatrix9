# Wireframe gần và xa trong viewport OM9

## Đối chiếu Rhino 5 và Matrix9

Tài liệu Rhino 5 mô tả đường đẳng tham số dùng để đọc hình dạng bề mặt trong Wireframe. Trong mã decompile cục bộ, Matrix9 chọn chế độ bằng `ShadeModeForm.SetView` rồi gọi display pipeline của viewport Rhino. RhinoCommon 5 chuyển màu và mật độ đường từ `DrawBrepWires` xuống `rhcommon_c.dll` tại RVA `0x44E70`. Hàm `CRhinoDisplayPipeline::DrawBrep` của Rhino tại RVA `0x4D33D0` chuyển tiếp việc chuẩn bị và vẽ. Thuộc tính độ dày của đường cong và của mesh có đường truy cập riêng. Các hàm đã đọc chưa xác lập công thức độ dày theo chiều sâu. [Rhino 5, Viewport display modes](https://docs.mcneel.com/rhino/5/help/en-us/seealso/sak_displaymodes.htm)

## Hiển thị trên Windows

Trước thay đổi, các cạnh trong viewport dùng cùng nét GPU một pixel. OM9 hiện nội suy bề rộng từ 2 px xuống 1 px và hệ số màu từ 1 xuống 0,65 theo chiều sâu của các đường hình học trong hướng nhìn. Đây là tham số OM9 chọn theo yêu cầu thị giác và ảnh tham chiếu. Nét theo kích thước pixel của render target. Mỗi viewport tính riêng với camera hiện tại. Khi toàn bộ đường cùng độ sâu, hệ số nội suy là 0,5.

GPU dùng lại cặp đỉnh đang có cho mỗi đoạn, cắt đoạn tại mặt phẳng gần trước khi chiếu và ghi độ sâu tại các pixel nét vẽ. Màu cơ sở là xanh lá cho vật thể bình thường và magenta khi được chọn. Lưới, trục, dấu điểm và chế độ Shaded tiếp tục dùng đường vẽ hiện có.

Hai lần chạy Gem Loader trong cửa sổ Windows tạo Round Diamond Ø3,10 mm và Oval Diamond 6 × 4 mm, mỗi lần một B-rep và một mục Undo. Đã kiểm tra Top, Perspective, Front và Right với ZoomExtents. Trên viên nhỏ, Top trước đó có thể giữ camera cách 100 mm trong khi đặt mặt phẳng cắt xa theo bán kính viên đá. Sau sửa, mặt cắt xa bao gồm khoảng cách từ mắt tới tâm cộng bán kính có lề. SHA-256 bản debug đã chụp là `5eae3998c0ad79e435315a3ba0812a8e6eb84e5fb81b4ffb31f3d78144027381`.

- [Round ở Top, crop đúng pixel](../../build/standalone-validation/ui-artifacts/wire-depth-20261122/Round-Diamond-Top.png)
- [Round, bốn viewport](../../build/standalone-validation/ui-artifacts/wire-depth-20261122/Round-Diamond-four.png)
- [Round được chọn, bốn viewport](../../build/standalone-validation/ui-artifacts/wire-depth-20261122/Round-Diamond-four-selected.png)
- [Oval được chọn, bốn viewport](../../build/standalone-validation/ui-artifacts/wire-depth-20261122/Oval-Diamond-four-selected.png)

Oval OM9 còn có chùm cạnh hội tụ lệch về một bên mặt bàn. Bộ dựng Oval hiện chia mặt bàn thành quạt tam giác trước khi chuyển thành B-rep. Đây là khác biệt hình học nhìn thấy được so với ảnh Matrix9. Việc đối chiếu hình học từng kiểu cắt và duyệt giao diện bằng mắt còn mở.

## Kết quả kiểm thử

| Phép kiểm | Kết quả |
| --- | --- |
| GPU, hai đường ở hai độ sâu | 8 cấu hình gồm 2 phép chiếu, 2 hướng camera và MSAA 1×/4×. Tích phân độ phủ cho nét gần 2,00 px và nét xa 0,99–1,00 px. Đường xa vẫn thấy khi đảo camera. |
| `antialiasing`, `render_loop`, `camera_fit` | 3 + 11 + 5 test đạt. Loạt render đầy đủ bật `om9-kernel/3dm-brep` như cấu hình ứng dụng. [GPU](../../build/standalone-validation/ui-artifacts/wire-depth-20261122/render-final.log) · [Camera](../../build/standalone-validation/ui-artifacts/wire-depth-20261122/camera.log) |
| Standalone Windows | Build `om9-app --bin om9` với feature mặc định thành công. Cả hai mẫu được chụp từ cửa sổ ứng dụng. |
| Kiểm thử app còn vướng | Bộ `om9-app --lib` và tích hợp `om9-app --test wireframe` bị chặn khi biên dịch các fixture B-rep cũ đang dùng API trước `CanonicalBrep`. Lượt lưu thử Round B-rep báo thiếu chứng cứ topology để lưu. Bằng chứng hình là từ phiên đang mở. |
