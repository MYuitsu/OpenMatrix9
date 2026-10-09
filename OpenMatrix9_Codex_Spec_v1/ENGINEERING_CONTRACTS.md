# Hợp đồng kỹ thuật chung OpenMatrix9

Các mục dưới đây là yêu cầu thiết kế và nghiệm thu của OpenMatrix9. Chúng có điều kiện theo workflow thực tế; chưa được coi là hoàn thành nếu chưa có geometry/state evidence native. Mỗi feature liên kết các mục liên quan và giữ checklist áp dụng riêng.

Bổ sung chi tiết xuyên lệnh tại [specs/00-rhino-core](specs/00-rhino-core/README.md):
chốt representation/backend, document tolerance, parser/frame và capability
trước khi triển khai. Các yêu cầu RCORE không tự thay supported behavior,
default hoặc trạng thái native đã ghi của từng feature.

Nhóm `00` đối chiếu từng năng lực với API/UI trong source FreeCAD và ghi phần
OM9 cần viết. Chọn chương cùng mã RCORE và capability `.Cxx` liên quan;
không suy từ API geometry rằng editor, command session hoặc persistence của
workflow Matrix đã có sẵn.

## OM9-CMD - Điều phối và vòng đời lệnh

Menu, CMD và F6 phải đi vào cùng handler và capability gate. Tách prompt khỏi transcript; lưu trạng thái input bằng session. Kiểm tra hủy ở mỗi bước, repeat khi idle, hành vi sau Undo/Delete và không ghi successful history cho lệnh thất bại.

## OM9-OBJECT - Đối tượng và topology

Phân biệt point, curve, wire, surface, shell, solid và polygon mesh. Dữ liệu hiển thị không chứng minh hình học kín/hợp lệ. Phân biệt trim loop, edge và isocurve; không thay NURBS bằng mesh mà không nêu rõ supported slice. Kiểm tra output sau save/reload.

## OM9-SELECT - Selection và subelement

Kiểm tra chọn/add/remove, window/crossing và face/edge/control point theo policy UI đã chọn. Không lấy bounding box làm bằng chứng giao hình học chính xác. Resolve selection theo document identity và subelement; kiểm tra hierarchy/world placement và tham chiếu cũ.

## OM9-VIEW - Viewport và camera

Lưu projection, camera, target và CPlane theo viewport/document. Kiểm tra pan/zoom/rotate và modifier ở từng projection; điều hướng trong lệnh không được commit hoặc hủy input ngoài ý muốn. Đổi camera không thay geometry.

## OM9-DISPLAY - Chế độ hiển thị

Wire, shaded và render là chính sách hiển thị riêng. Đổi mode phải giữ document topology; kiểm tra visibility, màu và selection. Preview có owner rõ ràng và được dọn khi document hoặc workbench thay đổi.

## OM9-PICK - Điểm và ràng buộc tương tác

Marker preview phải đúng điểm sẽ commit. Grid, snap, Ortho và one-pick có trạng thái riêng; phân biệt cấu hình lâu dài với override tạm thời. Kiểm tra phối hợp các ràng buộc theo feature và không áp một thứ tự ưu tiên chưa được thiết kế.

## OM9-COORD - Hệ tọa độ và đơn vị

Tách tọa độ CPlane, tọa độ world và placement của object. Chuyển khoảng cách/góc tại ranh giới input có kiểu; ghi rõ đơn vị lưu trữ và hiển thị. Kiểm tra view có CPlane xoay, input tương đối, tọa độ âm và hierarchy lồng nhau.

## OM9-SURFACE - Dựng mặt từ đường

Giữ thứ tự rails/profiles, direction và seam. Adapter phải kiểm tra topology/validity/continuity và điều kiện tiếp xúc theo feature. Phân biệt output surface/shell với solid; dùng fixture có đáp án giải tích hoặc điều kiện hình học kiểm tra được, không lấy shaded preview làm bằng chứng.

## OM9-EDIT - Join, split, trim và chỉnh sửa

Định nghĩa riêng phần giữ, phần bỏ và ownership của input/output. Boolean/join/split có thể thất bại; báo lỗi trước transaction hoặc abort toàn bộ. Không suy ra tolerance hay DeleteInput từ tên lệnh. Kiểm tra số thành phần, topology và Undo/Redo.

## OM9-POINTS - Chỉnh control points

Phân biệt control point, edit point, solid grip và mesh vertex. Phải xác định loại geometry cho phép chỉnh và ảnh hưởng tới tham số/continuity. Không chỉnh polysurface như single surface. Kiểm tra tắt grips/hủy session và lưu lại hình học.

## OM9-TRANSFORM - Biến đổi

Rust có thể tạo ma trận/plan thuần, host áp vào geometry hoặc placement theo policy feature. Xác định pivot, trục, reference, copy ownership, reflection và scale. Kiểm tra tọa độ và kích thước thực; đổi camera không thay thế transform geometry.

## OM9-ANALYSIS - Đo và chẩn đoán

Giá trị đo cần có đơn vị và điều kiện hợp lệ. Dùng đối tượng có đáp án giải tích; phân biệt dữ liệu số, annotation và object chẩn đoán. Kiểm tra direction, naked edges, bad geometry và loại input riêng với kết quả hiển thị.

## OM9-ORGANIZE - Tổ chức document

Layer, group, hierarchy và block instance có semantics riêng. Không đồng nhất group với joined topology hoặc instance với bản copy geometry. Kiểm tra visibility/lock, ownership, tên, placement và khôi phục cấu trúc sau lưu/mở.

## OM9-ANNOTATE - Chữ và kích thước

Tách đơn vị model khỏi kích thước pixel của nhãn UI. Định nghĩa liên kết đo/geometry rõ ràng; sửa annotation không mặc nhiên sửa geometry. Kiểm tra style/text/font fallback, readability khi zoom và persistence, kể cả Notes không phải geometry.

## OM9-MAKE2D - Hình học chiếu và layout

Tạo projected curves theo projection/frame đã chỉ định. Quy tắc hidden/tangent edges, layer và page/detail scale phải rõ ràng. Screenshot là ảnh; không được dùng làm projected geometry hoặc bằng chứng kích thước in đúng.

## OM9-RENDER - Render và media

Tách material/light/camera/environment khỏi model geometry và trạng thái viewport. Adapter renderer phải có capability/options rõ ràng; không giả định backend độc quyền hiện diện. Kiểm tra export, resolution, alpha, hủy, lỗi dịch vụ và lỗi ghi tệp.

## OM9-SOLID - Dựng và xác nhận solid

Solid phải có volume kín và topology hợp lệ. Kiểm tra cap, Boolean, mặt mở, self-intersection và thất bại kernel; không tự đóng geometry ngoài policy feature. Fixture phải kiểm tra validity và thể tích có đáp án.

## OM9-PICTURE - Ảnh tham chiếu

Ảnh tham chiếu có đường dẫn, kích thước, placement và quy tắc scale riêng. Giữ aspect ratio theo option; kiểm tra relink/missing file và save/reload. Ảnh không tự chứng minh kích thước hoặc trở thành geometry vector.

## OM9-FLOW - Mapping và biến dạng theo UV

Định nghĩa frame, tham số hóa, seam, direction và policy rigid/deform riêng. Kiểm tra mapping ở biên, mặt trimmed, source/target lệch hướng và object nhiều thành phần. Không áp biến dạng hoặc tự chỉnh direction khi chưa xác định policy.
