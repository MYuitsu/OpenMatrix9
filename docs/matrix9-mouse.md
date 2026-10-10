# Hành vi chuột trong OpenMatrix9

Phần này triển khai điều hướng bốn viewport, chọn đối tượng và xác nhận lệnh. Rust quyết định ánh xạ nút/phím và ngưỡng kéo; C++/Qt thao tác camera, selection và gọi cùng handler menu/CMD. Đây là phần triển khai có giới hạn, chưa phải toàn bộ hành vi Matrix9.

Nguồn ưu tiên là specs cục bộ `01-core`, đặc biệt `OM9-VIEW-004`: Ctrl + kéo chuột trái để Zoom Dynamic. Những hành vi điều hướng chung chưa mô tả trong specs dựa vào [tài liệu phím/chuột Rhino 5](https://docs.mcneel.com/rhino/5/help/en-us/user_interface/shortcuts.htm) và [Mouse Options Rhino 5](https://docs.mcneel.com/rhino/5/help/en-us/options/mouse.htm). Không đọc lại PDF. Cấu hình riêng Matrix9 của người dùng chưa được phục hồi đầy đủ.

| Thao tác | Hành vi hiện tại |
| --- | --- |
| Lăn con lăn | Zoom viewport đang trỏ |
| Alt + lăn con lăn | Di chuyển camera dọc hướng nhìn |
| Kéo chuột phải | Xoay Perspective; pan các viewport song song |
| Shift + kéo chuột phải | Pan |
| Ctrl + kéo chuột phải | Zoom |
| Ctrl + kéo chuột trái | Zoom Dynamic, giữ phiên Curve/Distance/PictureFrame đang nhập điểm |
| Ctrl+Shift + kéo chuột phải | Xoay viewport song song; thay đổi góc camera Perspective |
| Alt + kéo chuột phải | Dolly dọc hướng nhìn |
| Shift+Alt + kéo chuột phải | Nghiêng camera quanh hướng nhìn |
| Ctrl+Alt + kéo chuột phải | Xoay hướng nhìn tại vị trí camera |
| Ctrl+Shift+Alt + kéo chuột phải | Thay đổi zoom/góc camera |
| Nhấp trái | Thêm đối tượng vào selection; nhấp vùng trống xóa selection |
| Shift + nhấp trái | Thêm selection |
| Ctrl + nhấp trái | Bỏ chọn đối tượng; phân biệt với kéo để zoom |
| Ctrl+Shift + chọn | Chọn thành phần cạnh/mặt native |
| Kéo trái → phải | Chọn các đối tượng/thành phần nằm hoàn toàn trong khung |
| Kéo phải → trái | Chọn các đối tượng giao với khung |
| Shift + chọn vùng | Giữ selection trước đó và thêm kết quả |
| Nhấp phải nhanh | Xác nhận nội dung CMD/phiên đặt điểm; khi rảnh lặp lệnh gần đây khả dụng |
| Giữ chuột phải | Mở menu F6 dùng chung |
| Nhấp con lăn | Mở menu các lệnh gần đây theo cấu hình mặc định hiện tại |
| Esc khi kéo | Hủy thao tác và phục hồi camera trước khi kéo |

Ngưỡng phân biệt nhấp/kéo là 4 pixel logic, nên nhất quán giữa DPI 100% và 200%. Mất bắt chuột hoặc ứng dụng mất focus kết thúc thao tác; hover sau đó không tiếp tục điều hướng. Các task khóa camera/selection/document được tôn trọng. Chuột trái khi chỉnh sửa native và khi công cụ nhập điểm đang hoạt động đi qua handler hiện có, ngoại trừ Ctrl-kéo để zoom.

Preferences tại `BaseApp/Preferences/Mod/OpenMatrix9/Mouse`:

| Khóa | Giá trị |
| --- | --- |
| `ContextMenuDelay` | Mặc định 500 ms, giới hạn 100–5000 ms |
| `MiddleButtonAction` | 0: recent menu; 1: pan (Alt xoay, Ctrl zoom); 2: xoay (Shift pan, Ctrl zoom); 3: để host xử lý |

Recent menu là lựa chọn mặc định tạm thời vì cấu hình nút giữa của bản Matrix9 người dùng chưa rõ. Menu dùng lịch sử tối đa 20 lệnh thực thi thành công, không bao gồm toàn bộ lịch sử/favorites/custom macro của Rhino.

Theo yêu cầu mới nhất ngày 2026-10-05, tạm hoãn thay đổi mặc định nút giữa sang pan để người dùng kiểm tra trực tiếp trên Matrix9. Bản đang mở vẫn dùng cấu hình tạm thời trước đó; không coi đây là hành vi Matrix9 đã xác nhận.

Chọn vùng giao dùng kiểm tra hình học của FreeCAD; chọn vùng chứa lọc thêm bounding box chiếu lên màn hình, theo từng subelement. Phép chứa này có thể bảo thủ với hình học phức tạp. Các tay nắm builder, gumball/kéo biến đổi đối tượng, lệnh thay thế khi nhấp phải icon và mọi thao tác riêng của từng công cụ chưa hoàn thành trong phần này. Tài liệu kiểm chứng: `docs/validation/2026-10-05-core-mouse.md`.
