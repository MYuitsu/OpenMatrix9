# Box/Sphere — nghiệp vụ OpenMatrix9 đã được duyệt

Áp dụng cho `OM9-SOLID-012` (Box) và `OM9-SOLID-014` (Sphere). Đây là các lựa chọn của OpenMatrix9 đã có bằng chứng native; không coi chúng là defaults hay solver được khôi phục từ Matrix/Rhino. Các lệnh Sphere/Circle, SubD Sphere và Gem/Pave Sphere có hợp đồng riêng.

## Tra cứu phạm vi hiện hành

Trong checkout đang làm việc, lấy `spec_path` theo exact ID từ catalog/feature lookup. Package có thể ở `OpenMatrix9_Codex_Spec_v1/` hoặc `ref/matrix9/OpenMatrix9_Codex_Spec_v1/`; dùng vị trí tồn tại và phù hợp checkout. Đọc implementation notes của spec đã chọn, phân biệt supported slice với hợp đồng mục tiêu và code mẫu request-planner.

- Box: `specs/04-solid/om9-solid-012-box.md`.
- Sphere: `specs/04-solid/om9-solid-014-sphere.md`.
- Bằng chứng: `docs/validation/2026-10-09-solid-box-sphere-options.md`, ledger `docs/openmatrix9-progress.json`; fixtures `rust/tests/solid_options.rs`, `tests/solid_options_smoke.FCMacro` cùng các test core.

Trước khi thay đổi hoặc tuyên bố thêm capability, đối chiếu code/spec và chạy fixture liên quan trên đúng native module. Kết quả validator/code mẫu không thay thế nghiệm thu hình học FreeCAD. Catalog vẫn là `partially_implemented / validated_supported_slice` khi còn các giới hạn dưới đây.

## Box: định nghĩa kích thước và frame

| Mode | Quy tắc |
|---|---|
| Corners | Hai góc đáy rồi Height; hoặc góc đầu rồi Length/Width/Height nhập số. |
| Diagonal | Góc đối diện của đáy rồi Height; không hỏi/nhận side Length. |
| 3Point | Hai đầu cạnh đầu tiên chiếu lên CPlane đã chụp, rồi signed Width và Height. |
| Vertical | Cạnh đầu nằm trong CPlane đã chụp; Width theo pháp tuyến của CPlane đó, Height vuông góc đáy đứng. |
| Center | Điểm đầu là tâm đáy; Length/Width nhập số là kích thước toàn phần. Góc được pick cho hai extent tâm–góc phải nhân đôi. |
| Cube | Hai góc đối diện 3D xác định cạnh bằng nhau, mỗi cạnh dài `diagonalLength / sqrt(3)`, và hướng khối; chọn trong Diagonal hoặc dùng shortcut mode. |

Frame được chụp khi chấp nhận điểm định nghĩa đầu tiên. Đổi viewport sau đó không đổi frame, đặc biệt pháp tuyến dùng trong Vertical. Cube dùng phép quay tối thiểu từ body diagonal của frame đã chụp sang diagonal được pick; half-turn chính xác dùng trục đơn vị theo `cross(capturedBodyDiagonal, capturedX)`. Diagonal gần đối hướng vẫn phải đến đúng điểm pick, không được thay bằng half-turn gần đúng.

Enter tại Width dùng Length; Enter tại Height dùng Width. Typed Length dương; signed Width/Height và extent pick âm dịch origin để dimensions dương, giữ frame thuận tay phải. Chuột Width của Vertical chiếu lên mặt phẳng đáy đứng; chuột Height dùng trục pháp tuyến đáy và mouse ray. Ray song song cần đổi view hoặc nhập số. Right-click icon Box bắt đầu 3Point.

## Sphere: các nhánh dựng

- Center dùng tâm và Radius dương hoặc khoảng cách world tới điểm radius. `Diameter=value` chia đôi giá trị. 2Point dùng trung điểm của hai đầu diameter; Diameter nhập số ở bước hai chạy theo X của CPlane đã chụp. Right-click icon Sphere bắt đầu 2Point.
- 3Point dùng circumcircle qua ba world points không thẳng hàng. Sau hai điểm, `Radius=value` hoặc `Radius` rồi giá trị và một hướng ngoài chord chọn mặt phẳng/phía của circle; radius ít nhất bằng nửa chord.
- 4Point dùng circle của ba điểm, hoặc nhánh Radius ở trên, rồi điểm thứ tư ngoài mặt phẳng circle để dựng circumsphere; điểm thứ tư đồng phẳng bị từ chối. Vertical dùng mặt phẳng pick radius vuông góc CPlane.
- FitPoints nhận 3..1024 điểm rồi Enter. `Selection` thêm selection; Enter khi session chưa có điểm nhập selection. Hỗ trợ Part vertices/VertexN, B-spline control poles, mesh vertices và point clouds, kể cả placement lồng trong group. Batch nhập phải atomic; Undo bỏ cả batch.
- FitPoints dùng normalized seed và geometric radial least squares. **Dữ liệu đồng phẳng nhưng không thẳng hàng fit một circle trong mặt phẳng đó; tâm và radius của circle trở thành tâm/radius sphere.** Dữ liệu thẳng hàng/singular bị từ chối. Không chọn một tâm sphere ngoài mặt phẳng tùy ý.

## AroundCurve: điểm world trên cạnh native

Chọn `Path=Object.EdgeN`; whole object chỉ hợp lệ nếu có đúng một edge. Edge của một Solid là input hợp lệ, không áp dụng gate kiểu của lệnh Trim cho parent object. Sau đó pick tâm trên đúng cạnh đã chọn hoặc nhập `OnCurve=0..1` (parameter fraction, không phải arc-length fraction), rồi Radius.

Tâm chuột lấy native world pick của cạnh rồi chiếu chính xác lên bounded edge; không chiếu xuống CPlane trước, kể cả curve ở trên CPlane. Parameter của periodic curve phải được chuẩn hóa vào interval của edge trước khi clamp, để cung đi qua periodic seam giữ đúng vị trí pick. Mặt phẳng radius vuông góc native tangent. Chọn path chưa yêu cầu giải nearest center; ambiguous nearest branches cần fraction, cusp/tangent suy biến bị từ chối ở bước chọn tâm.

## Tangent: đồng phẳng và chọn nghiệm

Ba constraints gồm native curves hoặc passing points; `Point` làm điểm kế tiếp thành passing point. Sau hai constraints, `Radius=value` hoặc `Radius` rồi giá trị giải fixed-radius. CMD curve pick dùng `Curve=Object.EdgeN@worldX,worldY,worldZ`.

Hỗ trợ bounded Lines/Circles và planar native NURBS với poles/weights/knots giữ nguyên, kể cả Solid EdgeN và global placement. Plane song song CPlane chụp tại constraint đầu; noncoplanar constraints bị từ chối. General nonplanar 3D sphere tangency chưa hỗ trợ.

Matching OCCT solver phải kiểm tra contact nằm trên bounded edges, radial residual và tangency. Xếp nghiệm theo khoảng cách từ contacts tới các vị trí pick. **Nghiệm cùng hạng không tự commit; yêu cầu `Solution=1..N`.** Multi-start NURBS hiện dùng chín seeds có bounds; không tuyên bố tìm đủ mọi nhánh. Links chưa hỗ trợ.

## Output, lifecycle và giới hạn dùng chung

Rust giữ input state/pure construction math; typed C++/Qt Part/OCCT adapter giữ native references và tạo một document-root `Part::Feature`, valid/closed BRep, đúng một solid, volume dương trong một transaction. Không đánh giá CMD như mã Python. Metadata lưu exact ID/command/mode, dimensions/frame đã chuẩn hóa và tên references.

Đơn vị mm, dimension suffix mm/cm/in, coordinates/final extents trong +/-1e9 mm; dimensions không zero, từ 1e-7 đến 1e9 mm, Sphere radius dương. Preview là Coin scene node không pick được, không tạo document object. FitPoints preview fit cùng điểm hover; Tangent giải sau constraint cuối, không có live preview mọi nhánh.

Revalidate BRep và global placement của references trước commit; source đổi/xóa hoặc lỗi không để output dở dang. Giữ nguồn; kết quả là snapshot độc lập, không regenerate theo nguồn sau commit. Cancel/Esc, đổi/đóng document hoặc rời workbench dọn input/preview. Input Undo tách khỏi native Undo/Redo; save/reload giữ geometry và metadata. Associative History, active-layer/Builder/Styles và toàn bộ foundation compatibility vẫn chưa được chứng minh.
