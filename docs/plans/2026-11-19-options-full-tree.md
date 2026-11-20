# Kế hoạch Options cho OpenMatrix9

Kế hoạch bao quát hai nhánh **Document Properties** và **Rhino Options**, gồm trang **View** và cây **Display Modes**. Giữ cấu trúc danh mục và chức năng tham chiếu trong giao diện OM9. Ảnh chỉ cung cấp tên, thứ tự nhìn thấy và quan hệ cha–con. Giá trị mặc định, cách lưu và tác dụng của từng trường cần đối chiếu với mã hoặc hành vi của ứng dụng gốc.

Hiện icon Options và CMD đã mở cửa sổ có **General** và **View**. General có điều khiển Undo và ngôn ngữ. View đang hiển thị các điều khiển chưa kết nối. Menu viewport có 27 tên preset được khai báo, với hai chế độ có đường chạy được bật là Wireframe và Shaded. [Hiện trạng Options](../validation/2026-11-19-options-view-reference.md) · [Menu viewport](../../crates/om9-app/src/viewport_menu.rs) · [Trạng thái viewport](../../crates/om9-scene/src/workspace.rs)

Nguồn mã dịch ngược và mức xác nhận của từng trang được ghi trong [bảng đối chiếu Rhino Options](../analysis/matrix90/2026-11-19-rhino-options-sources.md).

## Cây danh mục quan sát được

| Nhánh | Trang và trang con |
| --- | --- |
| **Document Properties** | **Annotation** có Dimensions, Hatch và Linetypes. Dimensions chứa Dimension Style 01. Các trang còn lại gồm **Grid**, **Mesh**, **Notes**, **Render**, **Rendering** → Safe Frame, **Units** → Model và Layout, **Web Browser**. |
| **Rhino Options** | **Alerter**, **Aliases**, **Appearance** → Colors, **Clayoo** → License, Hotkeys, Update and statistics, **Context Menu** → Control Point Menu và Object Menu, **Files** → Search Paths, **General**, **Idle Processor**, **Keyboard**, **Libraries**, **Licenses**, **Modeling Aids** → Nudge, Smart Track, Cursor ToolTips, Gumball, **Mouse**, **Plug-ins**, **Rendering**, **RhinoScript**, **Selection Menu**, **Toolbars** → Size and Styles, **Updates and Statistics**, **View** → Display Modes và OpenGL. |

**Dimension Style 01** là tên style thấy trong tài liệu mẫu. Trang Dimensions sẽ liệt kê style thực tế của tài liệu. **View → OpenGL** là danh mục thấy trong ảnh. Trang tương ứng của OM9 cần hiển thị renderer thực tế và chỉ cung cấp các lựa chọn GPU có hiệu lực.

Có **34 tên Display Mode quan sát được**. Tách danh tính mode khỏi tên hiển thị để dùng cùng một danh mục cho Options và menu viewport.

| Nhóm triển khai | Các tên thấy trong ảnh |
| --- | --- |
| Cơ sở | Wireframe, Shaded |
| Bổ sung cách hiển thị | Rendered, Ghosted, X-Ray, Technical, Artistic, Pen |
| Chẩn đoán | Detect All, Detect Backfaces, Detect Naked Edge, Detect UV Normals |
| Phụ thuộc add-on | Clayoo Sculpt |
| Preset kiểu hiển thị | Art Color, Art Color Wires, Chalk, Floorplan, GVBlend, GVTech, Ice, Legacy, Machine, Matrix Blue, Matrix White, Pastel, Plastic, Presentation, Shiny Plastic, Simple Shade, Tech White, Vivid, Wire Render, Working Render, Working Shade |

Các mode lặp lại cấu trúc **Grid**, **Objects**, **Shadows**, **Other Settings**. Cây con của Objects được cấu hình theo mode. Ảnh cho thấy một số mode có Points, Curves, Surfaces và Meshes, còn một số có Points, Curves và Lines. Dùng cùng bộ trang thuộc tính và chọn các trường mà mỗi mode thực sự hỗ trợ. Bảy tên xuất hiện trong ảnh nhưng chưa có trong danh mục menu viewport hiện tại là **Artistic, Pen, Chalk, GVBlend, GVTech, Matrix Blue và Matrix White**.

## Thứ tự triển khai

1. **Khung Options và quyền sở hữu dữ liệu.** Dùng chung một cửa sổ cho icon và CMD. Cây hỗ trợ cuộn, mở và gập nhánh, chọn trang bằng chuột hoặc bàn phím và phân biệt mục khả dụng. Tạo bản nháp cho thiết lập mới. OK kiểm tra và áp dụng qua nơi đang sở hữu thiết lập, còn Cancel, Esc và nút đóng bỏ bản nháp. Thiết lập ứng dụng lưu theo hồ sơ người dùng. Thuộc tính tài liệu dùng codec và giao dịch của tài liệu. Thiết lập từng mode lưu theo ID của mode. Giữ các luồng General hiện có, kiểm riêng hành vi ngôn ngữ và Undo khi ghép giao diện.
2. **View và các điều khiển ứng dụng có sẵn.** Làm trước Zoom Scale và Single-click maximize. Với mặc định Zoom Scale 0.9, hệ số span theo nấc vẫn là 0.81 và giữ điểm neo con trỏ. Sau đó lần lượt xử lý Pan, Rotate, Linked viewports, tiêu cự, Wrap cursor và Dynamic redraw theo hành vi đã đối chiếu. Giá trị Pan Screen fraction 0.0625 trong ảnh là ca tùy chỉnh. Mã Rhino đã tra khởi tạo 0.125. Chọn mặc định OM9 khi chốt hợp đồng tính năng. Named Views phụ thuộc luồng lưu và khôi phục CPlane, projection và clipping planes. Tiếp tục nối từng trang General, Keyboard, Appearance, Toolbars, Modeling Aids và Plug-ins với chủ sở hữu tương ứng sau khi kiểm tra code. [Phân tích điều hướng](../validation/2026-11-01-rhino5-native-navigation-analysis.md)
3. **Document Properties.** Bắt đầu với các đường có sẵn cho Grid, Units → Model và Annotation → Dimensions. Tra riêng hai loại Grid, gồm thuộc tính lưới/CPlane và style hiển thị theo Display Mode. Units phải kiểm tra hồ sơ đổi đơn vị, dung sai và lựa chọn đổi kích thước hình học. Xác minh khả năng ghi, mở lại và Undo theo hợp đồng của từng trang. Sau đó bổ sung Layout, Hatch, Linetypes, Mesh, Notes, Render, Safe Frame và Web Browser khi đã có nơi lưu và chức năng sử dụng. Thuộc tính Mesh cho hiển thị và dung sai mesh xuất là hai nhóm cần kiểm riêng.
4. **Display Modes.** Dùng chung danh mục với menu viewport. Hoàn thiện vòng chỉnh, lưu và áp dụng **Wireframe** và **Shaded** trước. Dùng các trang Grid, Objects, Shadows và Other Settings chung với tập trường theo khả năng renderer. Mở rộng tiếp Ghosted và X-Ray, rồi chia các mode chẩn đoán, kỹ thuật, render và preset còn lại theo chức năng GPU thực tế. Mỗi mode mới cần fixture chứng minh khác biệt quan sát được. Các preset như Matrix Blue và Matrix White cần xác minh dữ liệu kiểu hiển thị, ngoài việc xác nhận tên từ ảnh. [Milestone Display Shading](../../OM9_Milestones/M02%20-%20Viewport%2C%20Gumball/L2-03-05%20-%20Febe%20Viewport%20Display%20Shading%20And%20Gems/L2.json)
5. **Các trang đặc thù và nghiệm thu toàn cửa sổ.** Kiểm các mục còn lại như Clayoo, RhinoScript, Licensing, Updates và lựa chọn renderer theo tính năng thực sự có trong OM9 hoặc add-on. Hiển thị các mục đang chờ cùng lý do. Kiểm danh mục dài và cây lồng trên Windows, mở qua icon và CMD, OK/Cancel, khôi phục sau khi mở lại, lỗi ghi, và hành vi khi có lệnh đang chạy.

## Bằng chứng cần có trước khi bật một mục

- **Thiết lập ứng dụng** áp dụng đúng cho từng thao tác và đọc lại đúng sau khi khởi động. Dùng ca hủy, giá trị sai và lỗi lưu để kiểm dữ liệu cũ.
- **Thuộc tính tài liệu** chạy thử với hai tài liệu độc lập. Kiểm đóng/mở lại và Undo/Redo cho những thay đổi thuộc giao dịch tài liệu.
- **Display Mode** kiểm trên cùng mô hình ở nhiều viewport. Đo thay đổi hiển thị và kiểm chọn đối tượng, snap, camera, lưới và khả năng quay về mode trước đối với các phần bị tác động. Dữ liệu nguồn và dung sai xuất được kiểm riêng khi trường có thể tác động tới chúng.
- **Giao diện** kiểm tương tác qua egui và review Windows thực tế. Ghi từng mục theo mức đã thấy trong ảnh, có code, đã kiểm tự động và đã review desktop. Bộ test từng bị chặn bởi hai bản add-on ABI nên kiểm lại lỗi này khi chạy, trước khi kết luận về kết quả.

Bước triển khai đầu tiên là xây cây danh mục đầy đủ, kết nối General và View với hai mục View chạy được, rồi hoàn thiện thuộc tính Wireframe và Shaded. Kế hoạch này ghi phạm vi và thứ tự, còn trạng thái runtime nằm trong bản ghi kiểm chứng của từng tính năng.
