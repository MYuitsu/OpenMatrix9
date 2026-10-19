# Box/Sphere — nghiệp vụ OpenMatrix9 đã được duyệt

Áp dụng cho `OM9-SOLID-012` (Box) và `OM9-SOLID-014` (Sphere). Đây là các lựa chọn của OpenMatrix9 đã có bằng chứng native; không coi chúng là defaults hay solver được khôi phục từ Matrix/Rhino. Các lệnh Sphere/Circle, SubD Sphere và Gem/Pave Sphere có hợp đồng riêng.

## Tra cứu phạm vi hiện hành

Trong checkout đang làm việc, tra exact ID qua workflow router. Nền tảng document/BRep/tessellation theo M01 L2-01-03/01-05/01-06; đọc exact feature trong milestone sở hữu. Phân biệt supported slice với hợp đồng mục tiêu và tài liệu code mẫu.

- Box: `specs/04-solid/om9-solid-012-box.md`.
- Sphere: `specs/04-solid/om9-solid-014-sphere.md`.
- Bằng chứng: `docs/validation/2026-10-09-solid-box-sphere-options.md`, ledger `docs/openmatrix9-progress.json`; fixtures `rust/tests/solid_options.rs`, `tests/solid_options_smoke.FCMacro` cùng các test core.
- Box hiện hành: `docs/validation/2026-10-12-box-documented-workflow.md` và `.json`; `rust/tests/box_documented_workflow.rs`, `tests/box_documented_workflow_smoke.FCMacro`. Nội dung chọn phương pháp bắt buộc của báo cáo này đã được supersede bởi correction Box mặc định theo video; xem báo cáo hiện hành dưới đây.

Trước khi thay đổi hoặc tuyên bố thêm capability, đối chiếu code/spec và chạy fixture liên quan trên đúng native module. Kết quả validator/code mẫu không thay thế nghiệm thu hình học trên OM9 standalone. Những status/supported families dưới đây là scope FreeCAD lịch sử; catalog v2 vẫn not_implemented_on_standalone_om9 cho đến evidence mới.

## Box: luồng mặc định, các option và frame

Người dùng làm rõ video ngày 2026-10-12: **video là Box mặc định, không phải một trong bốn option**. Box mở ngay ở `First corner of base ( Diagonal 3Point Vertical Center ):`; **không bắt buộc chọn phương pháp trước điểm đầu**. Điều này thay thế quy tắc chọn phương pháp bắt buộc đã lưu trước đó. Áp dụng correction này cho quy tắc Box đã được người dùng đồng ý cập nhật skill.

| Luồng | Quy tắc |
|---|---|
| Mặc định | Chọn góc đáy thứ nhất, chọn góc đáy đối diện, chọn hoặc nhập Height: **3 lần chọn**. Hai góc xác định cả Length và Width trong CPlane đã chụp. Không nhận số Length thay góc đối diện. `Corners` là tên enum nội bộ của luồng mặc định, không phải option thứ năm hiển thị. |
| Diagonal | Hai góc đáy đối diện rồi Height; không hỏi/nhận side Length. Có Cube trong nhánh này. |
| 3Point | Option riêng: chọn hai đầu của cạnh đầu tiên, chọn điểm phía đối diện hoặc nhập signed Width, rồi Height. Không đổi option này thành luồng mặc định 3 lần chọn; không nhận số Length thay điểm cuối cạnh. |
| Vertical | Chọn hai đầu cạnh trong CPlane đã chụp; Width theo pháp tuyến CPlane để đáy vuông góc CPlane, Height vuông góc đáy đứng. Không nhận số Length thay điểm cuối cạnh. |
| Center | Chọn tâm rồi góc để xác định cả Length/Width và chuyển sang Height; hoặc nhập Length toàn phần, rồi Width và Height. Hai extent tâm–góc phải nhân đôi. |
| Cube | Hai góc đối diện 3D xác định cạnh bằng nhau, mỗi cạnh dài `diagonalLength / sqrt(3)` và hướng khối; chọn trong Diagonal hoặc shortcut mode. |

Bốn option Diagonal/3Point/Vertical/Center dùng click hoặc phím tắt D/3/V/C trước điểm đầu; chúng là nhánh tùy chọn. Right-click icon Box chọn option 3Point riêng và vẫn nhận hai đầu cạnh rồi Width/Height.

`0` là điểm gốc CPlane tại bước nhập điểm, không phải chiều dài zero. Ví dụ mặc định: `Box → 0 → 10,20,0 → 5`. Option 3Point: `Box → 3 → 0 → 10,0,0 → 20 → 10`. Center nhập số: `Box → C → 0 → 10 → 20 → 10`. Không suy diễn video thành chuỗi nhập Length/Width cho Box mặc định.

Box hover vẽ outline đáy sau góc đầu và wire khối ở bước Height, cùng các marker, trong cả bốn view. Preview không pick được và không tạo object/transaction. Trong Top/Bottom view, khi ray song song trục Height, kéo theo hướng lên màn hình với tỉ lệ world; default dùng góc đáy đối diện làm anchor. Side/perspective view giữ phép closest-ray/axis. Height âm đặt đỉnh dưới đáy; marker phải nằm trên đáy gốc và đỉnh thật, không dịch hai lần do normalized origin.

Frame chụp tại điểm định nghĩa đầu tiên; đổi viewport không đổi frame, đặc biệt pháp tuyến Vertical. Cube quay tối thiểu body diagonal của frame sang diagonal pick; half-turn chính xác dùng trục đơn vị `cross(capturedBodyDiagonal, capturedX)`. Diagonal gần đối hướng vẫn phải đến đúng điểm pick.

Enter tại Width dùng Length, Enter tại Height dùng Width. Typed Length dương chỉ dùng ở bước góc-hoặc-Length của Center; signed Width/Height và picked extent âm dịch origin để dimensions dương, giữ frame thuận tay phải. Chuột Width của Vertical chiếu lên mặt phẳng đáy đứng. Bằng chứng hiện hành trong checkout: `docs/validation/2026-10-12-box-default-video.md`, `rust/tests/box_default_video.rs`, `box_wire_ffi.rs`, `tests/box_default_video_smoke.FCMacro`. Native evidence ở báo cáo chọn phương pháp bắt buộc trước đó là historical.

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

Rust giữ input state/pure construction math; typed OM9 geometry/selected OCCT adapter dùng owned references; OM9 document/journal publish một typed root geometry object, valid/closed BRep, đúng một solid, volume dương trong một transaction. Không đánh giá CMD như mã Python. Metadata lưu exact ID/command/mode, dimensions/frame đã chuẩn hóa và tên references.

Đơn vị mm, dimension suffix mm/cm/in, coordinates/final extents trong +/-1e9 mm; dimensions không zero, từ 1e-7 đến 1e9 mm, Sphere radius dương. Preview là OM9 scene node không pick được, không tạo document object. FitPoints preview fit cùng điểm hover; Tangent giải sau constraint cuối, không có live preview mọi nhánh.

Revalidate BRep và global placement của references trước commit; source đổi/xóa hoặc lỗi không để output dở dang. Giữ nguồn; kết quả là snapshot độc lập, không regenerate theo nguồn sau commit. Cancel/Esc, đổi/đóng document hoặc đóng/deactivate workspace dọn input/preview. Input Undo tách khỏi OM9 document Undo/Redo; save/reload giữ geometry và metadata. Associative History, active-layer/Builder/Styles và toàn bộ foundation compatibility vẫn chưa được chứng minh.
