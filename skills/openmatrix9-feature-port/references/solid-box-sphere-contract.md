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

Các ID phục hồi `SolidBoxDiagonal`, `SolidBox3PointsHeight`, `SolidBoxVerticalBase`, `SolidBoxCenterofBaseCornerHeight` có icon riêng và cùng owner Box với các macro `_Box _Diagonal`, `_Box _3Point`/`_Box _P`, `_Box _Vertical`, `_Box _Center`. Kiểm thử world XY và Undo đã qua; các quy tắc CPlane và hướng dưới đây vẫn cần bằng chứng standalone cho từng biến thể, không suy hoàn tất từ binding. Xem `docs/validation/2026-10-29-solid-box-menu-variants.md`.

Trong OM9 world XY, Box mặc định/Diagonal và Center giữ dấu chiều cao khi pick; preview và commit dùng cùng đáy đã chọn, chiều âm xuống dưới đáy. Center nhập Length/Width cũng nhận Height âm; giá trị 0 bị từ chối và giữ phiên. Đã kiểm hình học, preview hai dấu và viewport mô phỏng; Vertical và 3Point còn phải xác minh riêng. Xem `docs/validation/2026-10-29-solid-box-signed-pick.md`.

Box mặc định/Diagonal và Center hiện chụp CPlane tại điểm đầu; preview đáy/chiều cao và commit dùng frame đó, giữ dấu chiều cao. CMD nhận `0` tại gốc CPlane và tọa độ cục bộ qua parser chung. Kiểm thử standalone đã đo hình học, preview hai dấu, tọa độ CMD và phép chiếu ray trên một CPlane xoay; chưa nghiệm thu viewport/GPU thật, đổi view giữa phiên hay các CPlane tổng quát. Xem `docs/validation/2026-10-29-solid-box-cplane.md`.

3Point và Vertical nhận điểm hoặc số Width có dấu sau hai đầu cạnh; 3Point dùng hướng trong CPlane vuông góc cạnh, Vertical dùng pháp tuyến CPlane. Height lấy theo pháp tuyến đáy tương ứng, preview và commit giữ cả hai dấu. Đã đo bốn tổ hợp dấu trên một CPlane xoay và kiểm viewport Vertical trên world XY; các góc nhìn edge-on, đơn vị và desktop thật còn mở. Xem `docs/validation/2026-10-29-solid-box-edge-width.md`.

Ở 3Point/Vertical, preview cạnh đầu chiếu theo CPlane đã chụp. Cube preview dùng cùng frame và phép quay với commit, kể cả đường chéo đối hướng; góc thứ hai hợp lệ commit ngay, điểm trùng bị từ chối và giữ phiên. Tại bước kích thước, `3` là số thay vì phím đổi mode. Đã kiểm controller, tọa độ CMD và menu mô phỏng; chọn điểm 3D qua viewport và desktop thật còn mở. Xem `docs/validation/2026-10-29-solid-box-cube-preview.md`.

Khi chọn góc Cube thứ hai bằng viewport, OM9 chiếu tia vào CPlane của viewport đang trỏ tới, trong khi giữ frame đã chụp ở góc đầu để định hướng khối. Đã kiểm mô phỏng góc đầu ở Looking Down, góc hai ở Through Finger và Side View, có đo vị trí/Undo; snap, CPlane tùy chỉnh và desktop thật còn mở. Xem `docs/validation/2026-10-29-solid-box-cube-viewport.md`.

Box chuyển Length/Width/Height nhập số từ đơn vị tài liệu sang mm đúng một lần; Enter dùng lại kích thước đã chuyển. Đường Box ba số trực tiếp cũng đổi đơn vị. Với Box mặc định/Diagonal/3Point/Vertical/Cube, không nhận một số thay điểm thứ hai; Center giữ nhánh nhập Length riêng. Lỗi hình học khi nhập Height giữ phiên để Undo và sửa. Đã kiểm cm, preview Center, kích thước có dấu và Undo; UI thật và các luồng còn lại cần kiểm riêng. Xem `docs/validation/2026-10-29-solid-box-units.md`.

Đã đo ba đường nhập inline, 3Point và Center trên cả sáu đơn vị tài liệu; tùy chọn Width trong tài liệu inch đã kiểm qua keyboard, chuột và nhập CMD mô phỏng. Phạm vi này chưa chứng minh mọi option, preview hoặc UI thật. Xem `docs/validation/2026-10-29-solid-box-unit-routes.md`.

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

Sáu ID menu phục hồi Sphere 2/3/4 Points, AroundCurve, Tangent và FitPoints có icon riêng và các macro `_Sphere` tương ứng dùng chung phiên Sphere. Sau khi chọn tâm, số `2`, `3`, `4` được xử lý là bán kính thay vì đổi mode. Đã kiểm menu mô phỏng, macro và các fixture Sphere hiện có; chưa coi mọi solver Tangent/FitPoints, đơn vị hay desktop thật là hoàn tất. Xem `docs/validation/2026-10-29-solid-sphere-menu-variants.md`.

Sphere Center nhận tọa độ CMD theo CPlane/đơn vị tài liệu và đổi Radius/Diameter nhập số sang mm; Sphere 2Point đổi Diameter và dùng trục X của CPlane chụp ở điểm đầu. Đường Sphere một số cũng đổi đơn vị; điểm world không đổi lại. Đã đo sáu đơn vị, CPlane xoay và tùy chọn Radius inch qua ba đường UI mô phỏng; các solver khác và desktop thật còn mở. Xem `docs/validation/2026-10-29-solid-sphere-units.md`.

Sphere 3Point sau hai điểm nhận `Radius=value` hoặc `Radius` rồi số theo đơn vị tài liệu; bán kính nhỏ hơn nửa chord bị từ chối. Sau đó chọn hướng ngoài chord để xác định tâm, preview và commit dùng cùng phép dựng; Undo bỏ Radius trước khi bỏ điểm. Đã kiểm hai phía, một CPlane xoay, cm và ba đường tùy chọn/viewport mô phỏng; tương tác Matrix gốc và desktop thật còn mở. Xem `docs/validation/2026-10-29-solid-sphere-3point-radius.md`.

Các nhánh Tangent đang hỗ trợ nhập Radius số cho một, hai hoặc ba curve, một curve cộng một điểm, và hai điểm đổi số theo đơn vị tài liệu trước khi gọi solver. Đã kiểm cm bằng thể tích và khoảng cách ràng buộc, cùng ba đường tùy chọn mô phỏng cho một curve; đây chỉ là các fixture đường thẳng/điểm, không chứng minh solver tổng quát hoặc desktop thật. Xem `docs/validation/2026-10-29-solid-sphere-tangent-units.md`.

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


### Sphere placed curve sources (verified OM9 subset, 2026-10-29)

AroundCurve and existing curve-based Tangent sessions sample source curves in world coordinates by applying each entity placement. Captured sources must still match their source stamps when completing; a changed or deleted source rejects completion without a commit, while Undo/Cancel remain available. This is an OM9 implementation rule, not verified Matrix original behavior. The tested cases are translated lines, a rotated line and a deleted source; curve sampling is approximate, and general tangent geometry, arbitrary transforms, typed path-pick projection and desktop interaction remain open. See `docs/validation/2026-10-29-solid-sphere-placed-curve.md`.


### Sphere AroundCurve picked center (OM9 subset, 2026-10-29)

The center is projected onto the source curve in world space for both controller point input and `Path`/`Curve` coordinates. The bounded search evaluates points on the curve and accounts for placement. Translated line, endpoint and circle fixtures pass; coarse search plus local refinement does not prove a global nearest point for arbitrary curves. Desktop picking and original Matrix behavior remain unverified. See `docs/validation/2026-10-29-solid-sphere-aroundcurve-pick.md`.


### Sphere AroundCurve fraction and preview (OM9 subset, 2026-10-29)

`OnCurve` evaluates the curve at the requested parameter fraction in world placement. Once the center is set, the radius preview and point commit share that center and radius. A translated circle fixture checks exact center, preview, volume and Undo; real viewport/GPU, BRep-edge input, arbitrary curves and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-sphere-aroundcurve-fraction-preview.md`.


### Sphere AroundCurve straight BRep edge (OM9 subset, 2026-10-29)

`Path=Object.EdgeN` accepts a straight BRep edge through its native oriented endpoints. Fraction and point selection, preview and stale-source rejection share the curve session. A whole BRep requires one edge; non-circular curved edges are rejected. An identity-placement box controller fixture verifies input, invalid index, preview and Undo; BRep placement variants, other curved edges, direct viewport selection and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-sphere-straight-brep-edge.md`.


### Sphere AroundCurve circular BRep edge (OM9 subset, 2026-10-29)

Named BRep paths also accept circular edges, with OCCT providing the bounded oriented circle and Rust owning the resulting curve/session. A closed Cylinder edge fixture checks fraction, point, preview, source deletion and Undo. Trimmed arcs, periodic seam, other curved edge types, placed BReps, direct viewport and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-sphere-circular-brep-edge.md`.


### Sphere AroundCurve viewport source selection (OM9 subset, 2026-10-29)

In the simulated viewport, select a standalone curve or BRep edge, then choose the center; for BRep the second hit must be on the same edge and uses its world hit. Undo clears the center or, at source selection, the source. Ambiguous hits and instance edges are rejected. A line and a box edge are tested with simulated pointer events; overlapping-edge handling, real desktop/GPU, other BRep geometry and original Matrix behavior remain open. See `docs/validation/2026-10-29-solid-sphere-viewport-edge.md`.


### Sphere AroundCurve overlapping center hits (OM9 subset, 2026-10-29)

After a BRep source edge is captured, filter center hits by the captured source and edge before checking ambiguity. A simulated viewport fixture with two coincident boxes verifies the selected source wins; initial source selection with overlapping edges is still ambiguous. Desktop/GPU, instances and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-sphere-overlapping-edge.md`.


### Sphere AroundCurve ambiguous source selection (OM9 subset, 2026-10-29)

Overlapping BRep source-edge hits use the selection popup with object and EdgeN for each candidate. Chosen hits are revalidated against command run, scene, source and viewport; cancel/restart invalidates the old choice. Simulated coincident boxes verify a selected candidate and stale-session rejection; same-object overlapping edges, popup row click, desktop/GPU, instances and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-sphere-ambiguous-source.md`.


### Sphere AroundCurve standalone curve world hit (OM9 subset, 2026-10-29)

Viewport center selection uses the revalidated world hit of the captured standalone curve, rather than intersecting the CPlane first, and then projects to that curve. An elevated straight line in a simulated Perspective viewport verifies the center; general curves, snap, instances, desktop/GPU and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-sphere-curve-world-hit.md`.


### Sphere AroundCurve trimmed circular edge evidence (2026-10-29)

One open circular edge produced by a cylinder/box intersection was checked against native oriented vertices and an independent circle midpoint. Fractions 0, 0.5 and 1 and an off-arc path pick produce the expected center under a rotation and translation, with Undo. Seam-crossing arcs, both orientations, nonuniform placement, desktop and Matrix-original behavior remain open. See `docs/validation/2026-10-29-solid-sphere-trimmed-arc.md`.
