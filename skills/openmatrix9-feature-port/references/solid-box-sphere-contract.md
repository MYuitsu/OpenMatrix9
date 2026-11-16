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


### Sphere / Boolean Union viewport detail (OM9 subset, 2026-11-14)

The user approved a manual DISPLAY parameter for strong/weak machines and selected Light as the default. Light uses a separate 0.1 mm / 0.5 rad viewport representation; High detail uses 0.01 mm / 0.3 rad. Both retain canonical BRep geometry for CAD; viewport quality must not replace computation/export tolerances. Changing a supported root BRep scene prepares the representations on workers and publishes the combined scene once, without a geometry Undo entry; cancellation, stale scene or quota refusal retains the previous effective profile and scene. Unsupported/unattested geometry is refused explicitly.

Verified scope is two radius-5 spheres at (0,0,0) and (5,0,0), Boolean Two Objects Union with DeleteInput=Yes, source-linked Surface Snap at one analytic point, and the recorded Undo/Redo/failure fixtures. Nine integration tests and eight native-window release runs passed. This is not general NativeBrep/legacy support, full source/display separation, v5 persistence completion or a guarantee of higher FPS. See `docs/validation/2026-11-14-viewport-detail-sphere-boolean.md`. This records the tested Solid scope under the existing authorization to synchronize Solid guidance after verification.

Light navigation now uses transient 70-percent target dimensions and one MSAA sample, restoring full-size four-sample rendering after settling without changing canonical geometry or tessellation. On the same two-sphere/Union fixture, 11 integration tests and six paired native-window pointer runs passed; moving GPU time fell from 0.641 to 0.139 ms, while both variants remained near 30 encoded frames/s under the configured cap. Handled navigation events are consumed after native widgets while keyboard/IME/focus/primary clicks remain. The earlier 10.6 FPS was not reproduced in the final release counterfactual; do not claim a threefold FPS gain, a proven IME-call timing cause, general scene coverage or measured RDP display FPS. Evidence: `docs/validation/2026-11-14-light-navigation-smoothness.md`. These are scoped OM9 optimization choices, not recovered original Matrix behavior.

Many-sphere timing evidence (2026-11-15): product High/Light completed six release runs at 64 spheres (three per mode) and two at 128 (one per mode), using independent intersecting pairs and one multi-input BooleanUnion. Light reduced observed total Boolean wait by 55–56%, while both modes stayed near the configured 30 encoded FPS and retained roughly 2.1 s / 5.1 s final UI publication stalls. Native source volume, bounds, closedness and solid counts passed in all eight runs. Separate active gaps from idle repaint waits; these results do not establish client displayed FPS, a publication-stall fix or global geometry/picking parity. See `docs/validation/2026-11-15-many-sphere-light-high.md` and its JSON data; CAD behavior was unchanged.

Boolean publication timing (2026-11-15): committed single-body worker results now prepare source isocurves/render buffers off UI and install them into a private candidate scene only after source-stamp, immutable representation/fingerprint, pin and unique-root checks. Canonical BRep, tolerances and atomic publication are retained. Sixteen focused tests and ten release runs passed (eight optimized at 64/128 spheres, two same-binary former-path runs at 64). Final UI polls fell from roughly 2.1 s to 57/42 ms High/Light at 64 and from historical 5.1 s to 158/94 ms at 128. Source measures/triangles remained unchanged; request-time pauses and variable 10.6–30 encoded FPS remain. Do not claim general FPS gains, all Boolean-path coverage, client-present timing or full migration/persistence completion. Evidence: `docs/validation/2026-11-15-boolean-publication-worker-buffers.md`.

RDP pacing evidence (2026-11-15): session-aware surface presentation preserves the CAD frame clock and exact BRep/tolerance. Ten frozen-release runs passed (two VSync controls at 64, six optimized at 64, two optimized at 128); steady encoded Perspective FPS changed from approximately 10.6 in the matched controls to 29.987–30.010, with p95 encoded intervals 35–36 ms and no steady interval over 100 ms. Eighteen focused tests passed. Creation/request/publication pauses remain: largest active gaps 64 High/Light 181.811/88.221 ms and 128 601.296/453.035 ms. Source measures and display triangle counts remain unchanged; 128 has only one sample per mode. UI callback rates are not CAD FPS, and RDP client display/video FPS remains unmeasured. See docs/validation/2026-11-15-rdp-viewport-pacing.md and its JSON. This is technical evidence under the existing Solid synchronization authorization, without a CAD business-rule change.

Controlled separation evidence (2026-11-15): 16 same-binary native-window release runs compare a reconstructed document-owned source/display pair (combined validation and serialization) with independent representations. Two alternating pairs ran per quality profile at both 64 and 128 spheres, retaining the same exact kernel, worker publication, GPU caches, motion quality and RDP policy. Exact native result source SHA and per-profile input/output mesh counts matched; 19 focused tests include coupled/separated source Snap and Undo/Redo. High final UI receive/commit medians fell 216.669 to 53.514 ms at 64 and 486.004 to 151.792 ms at 128 (75.3%/68.8% less); Light 41.305 to 24.584 ms and 125.533 to 91.826 ms (40.5%/26.9%). Both remain near the 30 encoded-FPS cap. Worst 128-sphere active gaps remain High 621 to 612 ms and Light 438 to 454 ms, and total Boolean wait is not consistently improved. This isolates the reconstructed document-coupling boundary, not the entire historical runtime, general source/display migration or client-presented FPS. The control is throwaway-only, not a persisted product mode. Evidence: `docs/validation/2026-11-15-coupled-source-display-ab.md` and its JSON. No CAD business rule changed.

Boolean request-source admission evidence (2026-11-15): 20 focused integration tests and 16/16 same-binary native-window release runs passed. 64 high request 47.5→2.5 ms (94.7% less), worst active 182.3→138.7 ms; 64 light request 46.2→3.4 ms (92.7% less), worst active 86.6→86.5 ms; 128 high request 365.8→8.9 ms (97.6% less), worst active 626.5→266.1 ms; 128 light request 365.0→8.7 ms (97.6% less), worst active 447.3→210.2 ms. Exact native result source SHA and per-profile mesh counts matched. Both paths remain near 30 steady encoded FPS. Foreground selection/redraw, creation/fitting, worker admission and publication pauses remain. Large-scene Union fixtures only; full migration/persistence, arbitrary geometry and client-presented FPS remain open. Optional debug probe was interrupted and not counted as a passed test. Existing grouped source capture replaces repeated single-source admission without changing CAD behavior or weakening managed memory/source/placement/publication guards. Distinguish measured request-call time from the full callback transition including selection/redraw. See `docs/validation/2026-11-15-boolean-request-source-batch.md` and its JSON. Technical evidence is synchronized under the existing Solid authorization.

Selection-frame projection evidence (2026-11-15): 35 distinct focused integration tests and 16/16 same-binary native-window release runs passed. 64 high selection 116.4→28.5 ms, transition 140.0→57.6 ms, active max 140.2→91.5 ms; 64 light selection 30.7→5.9 ms, transition 42.5→16.5 ms, active max 80.8→84.0 ms; 128 high selection 217.0→33.7 ms, transition 253.8→65.9 ms, active max 257.3→232.4 ms; 128 light selection 55.6→8.8 ms, transition 80.2→29.8 ms, active max 206.8→211.7 ms. Identical native result SHA, display density and projected-vertex coverage; both paths remain near 30 steady encoded FPS. Creation/fitting, other UI, worker admission and publication pauses remain; general scenes, client FPS, full migration/persistence and manual UI sign-off unproved. Existing selection_overlay ignored hover fixtures have two canonical-BRep compile errors and are not counted passing. Canonical BRep, Boolean tolerance, full vertex projection and selection visuals/validation remain. See `docs/validation/2026-11-15-selection-frame-projection.md` and its JSON. Technical evidence is synchronized under the existing Solid authorization; no business rule changed.

512-sphere RAM/Undo evidence (2026-11-15): one Light completed and one High was refused; this is not a fully passing profile comparison. Both use the existing process-only test object cap 200000 after the default cap refused Sphere 447 at 446 Undo (retained-version object slots). Light settled resident RAM 600.5 MiB, timed peak 616.4 MiB, retained Undo 513 / Redo 0, steady pre-Boolean encoded FPS 30.008, maximum active callback gap 133.725 s; 256 closed solids and analytic source oracles passed. High reached 512 spheres, then native CMD reported Boolean union: invalid creation input: BRep snapshot points; resident at refusal 710.5 MiB, peak 943.6 MiB, Undo 512 / Redo 0, pre-Boolean encoded FPS 29.995, active max 20.384 s before refusal. The effective snapshot point budget is reduced by held intermediate usage during sequential Union; the configured point cap remains 2000000, and no High result/source equality or final publication timing is claimed. Smaller High gaps cannot establish smoother successful publication. Light result FPS lacks uninterrupted samples after its post-commit stall; High has no result FPS. Twelve focused tests, release build and two separate eight-sphere resource smoke runs passed. Loaded Undo policy and exact CAD tolerances remain unchanged; no CAD business rule changed. Retained counts do not prove replay of every step; managed-history storage estimates differ from OS RAM, and client video FPS is unmeasured. Technical evidence synchronized under the existing Solid authorization. See docs/validation/2026-11-15-512-spheres-memory-undo.md and its JSON.

Retained-owner admission/UI evidence (2026-11-15): 25 focused checks and three native-window release runs passed (64 Light/High at stricter test object cap 3000; 512 Light at the prior test cap 200000). Light 512 largest active callback gap 133725.2→6703.2 ms; Boolean-phase max 133725.2→1651.4 ms; final UI poll 70393.0→1514.7 ms; first post-commit pane work 63323.2→128.7 ms. Native result source SHA and per-profile display triangles match the previous 512 fixture, with 256 closed solids, analytic bounds/volume, zero naked edges and 513 Undo / 0 Redo. The duplicate-inclusive allocation bound admits its index by bytes while exact logical object checks remain; external allocation-membership queries search the same owner set newest first. No source/tolerance, point/RAM/Undo policy or CAD business behavior changed. Individual samples; remaining creation/admission gaps, kernel wall wait, all-overlap/general geometry, all-Undo replay, client video FPS and High 512 snapshot-point refusal are not closed. Default-limit 512 acceptance is not claimed. Technical evidence synchronized under the existing Solid authorization. See docs/validation/2026-11-15-512-light-freeze-fix.md and its JSON.

Parallel provisional Union (2026-11-15, user-requested OM9 behavior): MatrixApp BooleanUnion may present a bounded disposable mesh CSG while the canonical OCCT source fold runs independently. Placement/Union intermediates return only canonical source; the requested display/topology profile is captured at the end. Provisional buffers never enter Document, export or Undo, cannot serve as exact geometry for hit testing/editing, and disappear on cancel/stale context/failure or exact publication. Exact sources/tolerances, freshness guards and one-step atomic source/scene commit remain authoritative. Primary editing gestures defer commit; orbit/pan does not. Sixteen integration checks and four native 64/128 Light/High runs verify this scope (first encoded preview 0.477/0.592 s and 0.779/1.170 s, identical native source hashes, 65/129 Undo). Three earlier 512 pilots were excluded. Recovery-audit native 512 Light passed: published preview 3.830 s, exact commit 783.798 s, full-window waiting/result FPS 29.323/29.930, 672.898 MiB resident peak and 513 Undo. High published preview at 5.360 s and measured 27.746 FPS while waiting, then its exact worker failed after 884.358 s, retaining 512 inputs/Undo; no final High source equality, result FPS or successful-publication freeze is claimed. The independent shared-source/no-history kernel 512 High capture passed at 1120000 points under the unchanged 2000000 cap, but does not establish the complete native worker path. Two native focus-loss recovery smoke profiles passed. Client video/manual acceptance remains unproved. See docs/validation/2026-11-15-speculative-boolean-512-comparison.md. Preview/classifier bounds can refuse pathological meshes while exact work continues. Other Boolean preview families, all-overlap/general geometry, manual/client-video acceptance and full Undo replay are unproved. Rapid batch creation can hold the viewport image longer than the UI callback gap; report both. This is an authorized OM9 preview policy and tested technical evidence, not recovered Matrix behavior. See docs/validation/2026-11-15-speculative-boolean.md and its JSON.
