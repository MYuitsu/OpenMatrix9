---
id: RCORE-07
title: NURBS editing và phép hình học dùng chung
priority: P0/P1
date: 2026-10-09
evidence_level: source_inspected
runtime_validation: not_run
evidence_scope: initial_source_audit_2026-10-09
---

# RCORE-07 — NURBS editing và modeling

## Cập nhật triển khai P0 — 2026-10-09

Metadata/source observations bên dưới là baseline audit ban đầu. Rust kiểm basis
trên đường publish spline đã nối với native adapter: finite poles/knots,
degree/cardinality/multiplicity/domain và periodic conventions. Invalid basis
bị reject trước dựng output. Đường này tạo spline không có rational weights;
validator Basis riêng có unit tests cho weights/homogeneous, chưa phủ mọi native
rational consumer. Context units mới được chụp cho curve construction input; ngưỡng
solver không bị âm thầm thay bằng absolute/relative tolerance chung.

Đây là validation tích hợp của một slice, không phải generic NURBS editor hay
shared geometry kernel. Rust/native evidence, mixed geometry 56 checks và
cold restore 28 checks đã qua, được dẫn tại [báo cáo P0](../../../docs/validation/2026-10-09-rhino-core-p0.md);
không nâng các phép knot/retrim/continuity chưa chạy thành pass.

**P0 còn thiếu:** solver contracts/capability và diagnostic chung; trim/topology
validation xuyên operation; semantic parameter mapping sau reverse/conversion;
tolerance/revision snapshot thống nhất, failed solver rollback và stale worker
guards xuyên family. Generic grips/Rebuild/Refit/G0–G2 workflow đầy đủ vẫn cần
P1/evidence riêng, không chứng minh bằng sampled deviation hoặc shape validity.

## Mục tiêu và phạm vi

Chuẩn hóa khả năng đọc/sửa basis NURBS, lifecycle grips và các phép dựng hình dùng chung của OM9.
Phạm vi gồm `OM9-POINTS`, `OM9-SURFACE`, `OM9-EDIT`, `OM9-TRANSFORM`, Curve/Surface/Solid và `OM9-TOP11-001/003`.
P0 là contract solver/tolerance/topology; P1 là editor CV/edit point, Rebuild/Refit và continuity workflow.
Giữ exact feature ID và supported slices; file này không tự mở khóa các command hiện disabled.
Baseline audit nguồn ban đầu là **source-inspected / not-runtime-validated**; lúc đó chưa chạy so sánh với Rhino 5 hoặc native FreeCAD.
FreeCAD HEAD `21d36cfa1eb110a1d0667050ff31706298805bbd`; OM9 HEAD `52e887ab706dcd5d80e23979fb1f1b62d8d869b0`.
Checkout có thay đổi làm việc; dùng hashes trong [baseline nhóm](README.md), không coi HEAD là snapshot sạch.
User's Guide PDF 53–71 / trang in 45–63 được định vị trong [audit](../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md).

Rhino phân biệt CV với edit point trên curve; PointsOff tắt cả hai, PointsOn thông thường không cho sửa CV của polysurface chung.
SolidPtOn là loại grips khác và không được giả lập bằng cách di chuyển CV từng face độc lập. [Rhino 5 PointsOn](https://docs.mcneel.com/rhino/5/help/en-us/commands/pointson.htm) (truy cập 2026-10-09).
Rebuild chỉ định degree/CV count, có preview và deviation; surface có U/V và tùy chọn retrim.
Phép đo deviation bằng mẫu không được trình bày như một bound liên tục đã chứng minh. [Rhino 5 Rebuild](https://docs.mcneel.com/rhino/5/help/en-us/commands/rebuild.htm) (truy cập 2026-10-09).
Schema và quy tắc kiểm chứng bên dưới là contract OM9, không phải mô tả thuật toán độc quyền Rhino.

## Hợp đồng dữ liệu NURBS

`CurveBasis = {degree, poles, weights, knots, multiplicities, periodic, domain, direction, seam_parameter?}`.
`SurfaceBasis = {degree_u, degree_v, pole_grid, weights, knots_u/v, mults_u/v, periodic_u/v, domains_u/v}`.
Các arrays có length được kiểm; poles/weights/knots hữu hạn; số poles phù hợp degree/multiplicity/periodic representation.
Knot order và multiplicity phải thỏa backend đã chọn; không ép điều kiện clamped nonperiodic lên mọi periodic curve.
Weight hỗ trợ phải dương/hợp lệ theo backend; zero/negative/overflow phải được từ chối rõ nếu chưa có contract.
Index CV dùng quy ước Rust xác định; chuyển sang index native một lần trong adapter và kiểm bounds trước gọi.
CV là hệ số basis; edit point là giá trị curve tại tham số định nghĩa; knot marker là tham số, ba loại không dùng chung index.
Pcurve UV không được sửa bằng world transform của CV trừ khi có quy trình rebuild/retrim explicit.
Direction và seam có thể thay nhưng phải cập nhật reference mapping; không đổi world shape khi operation cam kết giữ shape.
Domain đổi phải chỉ rõ reparameterization hay trim/segment; hai thao tác này có semantics khác nhau.
Reverse phải lưu mapping tham số cùng direction: API doc Rhino nêu `[a,b] → [-b,-a]`, nhưng curve reverse và các overload surface reverse có ownership khác nhau.
Knot comparison cho preservation phải so cả domain và giá trị tuyệt đối khi contract yêu cầu; `NurbsCurveKnotList.EpsilonEquals` chỉ so chênh lệch giữa knot liên tiếp.

| Operation basis | Input/option bắt buộc | Output/invariant |
|---|---|---|
| Move/transform CV | Object/revision, CV indices, transform/frame, copy/replace policy | Basis mới; shape thường đổi; thông báo ảnh hưởng trim/History |
| Set weight | CV index, positive finite weight, supported normalization policy | Rational basis mới; không âm thầm ép toàn weights thành 1 |
| Insert knot | Parameter, multiplicity, add/target mode, parameter tolerance | Knot/pole count mới, giữ shape trong numerical bound đã công bố |
| Remove knot | Knot index, target multiplicity, geometric tolerance | Có thể từ chối nếu không đạt; không coi removal là luôn exact |
| Elevate degree | Degree đích, direction U/V nếu surface | Shape giữ trong numerical bound; không đồng nhất với Rebuild |
| Add/delete CV | Type editor và phương pháp tái dựng | Shape thay đổi; degree/knot/periodicity phải còn hợp lệ |
| Periodic seam | Periodic input, seam parameter, orientation | World curve/surface giữ; mapping tham số/edge cập nhật explicit |
| Rebuild | Degree/count từng trục, input retention, retrim, layer policy | Candidate mới và deviation report; trim có thể fail độc lập basis |
| Refit | Tolerance và phương pháp đo, degree/pole budget, continuity constraints | Candidate được chứng nhận trong phạm vi đo hoặc fail/budget exhausted |

Đường tương thích RhinoCommon phải ghi `requested` và `effective` degree/count: wrapper `Curve.Rebuild` chuẩn hóa count thành ít nhất 2 và kẹp degree trong 1–11 trước native.
Đó là contract API của export đã đọc, không phải chứng cứ default UI Rebuild hay yêu cầu sửa ngầm input của OM9.
Nhánh OM9 hiện hữu giữ validation/supported slice đã công bố; nếu áp compatibility normalization phải định danh policy và nghiệm thu riêng.
Không áp normalization curve sang surface: C# surface không kẹp tham số, nhưng native có rule riêng tăng count khi count không lớn hơn degree; curve có nhánh hạ degree theo count.

## Session grips và commit

`EditSession = {document_id, source_refs, source_revisions, basis_snapshot, grip_mode, selected_grips, frame, context_revision}`.
Các state: `idle → selected → grips_visible → editing_preview → validated → committed` hoặc `cancelled/failed`.
PointsOn/PointsOff điều khiển hiển thị/lifecycle; PointsOff không tự commit preview thay shape chưa được chấp nhận.
Grips giữ identity của object và basis revision, không tạo document point objects để thay CV.
Đổi selection mode, deactivate workbench hoặc document close phải bỏ callback và preview an toàn.
Ctrl/copy/transform chọn grips có policy riêng; copy object phải clone identity, không chia sẻ mutable basis ngoài chủ ý.
Sửa polysurface phải từ chối nhánh PointsOn thông thường hoặc yêu cầu Extract/Explode explicit trước khi sửa face.
Nếu có SolidPtOn riêng, solver phải giải tính liên tục/đóng kín liên face; không lấy tên command làm bằng chứng.
Trimmed surface cần contract: sửa underlying surface rồi refit/rebuild trim, giữ UV trim hay reproject 3D trim; không đoán.
Commit shape, metadata, dependent links và History cùng transaction; Undo phục hồi cả basis, trim và grips state theo policy.
Cancel không để object, source deletion hoặc undo entry thừa; lỗi solver không làm mất input.

## Contract các phép modeling dùng chung

Mỗi solver khai báo `operation × representation × options × backend × tolerance_rule × output_topology`.
`ModelingRequest = {sources, ordered_profiles, seam_choices, orientations, parameters, frame, tolerance_context, delete_input}`.
`ModelingResult = {outputs, source_to_output_map, topology_report, deviation_report?, warnings, backend_id}`.

| Family | Input và quyết định cần explicit | Kiểm output bắt buộc |
|---|---|---|
| Intersection | Curve/curve, curve/surface hay face/face; domain trim; tangency/coincidence policy | Point/curve/contact region phân biệt; không dựng giao giả theo view |
| Projection / pull | Direction hoặc closest-point rule; target surface nền hay trimmed face | Result nằm trên target domain; multiple solution/none được báo |
| Offset | Plane/normal convention, sign, corner join, distance, self-intersection treatment | Độ lệch và topology đúng supported branch; collapse phải fail/report |
| Join | Type compatibility, ordering, endpoint gaps, orientation, tolerance rule | Wire/shell/solid được phân loại; không heal hoặc force-close ngoài contract |
| Trim / Split | Cutter type, side/region retention, coincident contacts | Mảnh output có topology hợp lệ; trim loops và holes còn |
| Sweep / Loft | Rail count, profiles có thứ tự, reversal/seams, closed, frame law | Không twist do seam/order ẩn; cap/solid chỉ khi yêu cầu và hợp lệ |
| Revolve | Axis/frame, angle units, cap/singularity policy | Domain/orientation/topology đúng; full turn không đồng nghĩa mọi output solid |
| Fillet / Boolean | Radius/operands, tool retention, supported topology, tolerance | Không mất operands khi fail; structural validity và solid checks riêng |

G0 là sai lệch vị trí; G1 kiểm tangent/normal direction; G2 kiểm curvature theo contract đã chọn.
Không dùng cùng parameter derivative magnitude làm chứng cứ G1/G2 hình học nếu hai mặt khác parameterization.
Deviation report nêu đơn vị, domain, một/hai chiều, trim masking, sampling/bound, singularities và budget.
History và preview tái dùng cùng seam/profile-order/frame/tolerance policy; input đổi phải invalidate result cũ.

## Affine transform và biến dạng phi tuyến

Move/Rotate/Scale/Mirror/Orient/Array dùng contract transform riêng gồm frame, pivot/axis, matrix, copy/replace và source revision.
Rust resolve matrix world từ CPlane/local frame đúng một lần; Array trả danh sách transform có thứ tự và identity riêng cho copies.
Rotate ghi đơn vị góc; Orient ghi hai frames và scale policy; Scale ghi uniform/nonuniform và tâm scale, không ẩn trong placement.
Matrix phải hữu hạn và có affine last row hợp lệ; matrix singular hoặc gần singular ngoài miền hỗ trợ phải bị từ chối trước commit.
Mirror có determinant âm: kiểm orientation/normal/solid boundary sau phép biến đổi, không chỉ bounds hoặc dấu volume.
Nonuniform scale có thể đổi analytic representation: circle thành ellipse; sphere có thể cần surface/general transform khác và conversion report.
FreeCAD có `transformShape`, `transformGeometry/transformGShape`, `mirror` và `Base::Matrix4D`; adapter phải chọn nhánh theo geometry, scale và copy ownership.
Bend/Twist/Flow/Cage là biến dạng phi tuyến, không được mô tả như một matrix affine cho mọi điểm.
Nhánh phi tuyến cần domain/control field, mapping/inverse nếu dùng, jacobian/degeneracy policy, approximation/deviation và preservation topology riêng.
Chưa audit toàn bộ solver biến dạng của FreeCAD/addon; việc có matrix API không chứng minh full deformation và chưa rà không chứng minh không có.

## Ma trận capability theo checkout

| ID | Capability cần có | FreeCAD | Bằng chứng và giới hạn | OM9 riêng cần làm/đối soát |
|---|---|---|---|---|
| RCORE-07.C01 | Curve poles/weights/knots/degree | API | `GeomBSplineCurve`, getters/setters và native OCCT handle; S1/S2 | Validate owned basis Rust, conversion và typed errors |
| RCORE-07.C02 | Surface pole grid/weights U/V | API | `BSplineSurfacePy` có row/column setters, `buildFromPolesMultsKnots`; S3 | Editor grid/grip identity, trim propagation |
| RCORE-07.C03 | Knot insertion/removal và degree elevation | API | Curve `insertKnot/removeKnot/increaseDegree`; U/V variants của surface; S2/S3 | UI options và shape-preservation fixtures; không suy đã tương thích Rhino |
| RCORE-07.C04 | Periodicity, seam/origin và domain | API | `setPeriodic/setOrigin`, `setU/VPeriodic`, `setU/VOrigin`, `segment`; S2/S3 | Direction/domain map xuyên preview và History |
| RCORE-07.C05 | Curve/surface approximation và interpolation | API | `approximate/interpolate` ở S1/S2/S3 | Không đồng nhất với exact Rebuild count/deviation contract |
| RCORE-07.C06 | Generic PointsOn/EditPtOn cho mọi Rhino type | Một phần | Basis API có; Sketcher có B-spline display tools S9; chưa chứng minh editor chung | Rust session CV/edit-point; PointsOn alias trong keyboard không là completion |
| RCORE-07.C07 | Sweep/Loft/offset/Boolean BRep | API | `TopoShape` có makePipe/makeLoft/makeElementOffset/makeElementFuse/Cut; S4 | Per-operation options, tolerances, topology và failure mapping |
| RCORE-07.C08 | Surface filling và boundary curves | UI+API | `Surface_Filling`, `Surface_GeomFillSurface` tạo native features; S5/S6 | Map EdgeSrf/Patch theo supported inputs; không gọi mọi Patch/NetworkSrf đã có |
| RCORE-07.C09 | Projection lên surface | API | `Part::ProjectOnSurface`; S7 | Phân biệt projection với pull và trim domain; kiểm multi-result |
| RCORE-07.C10 | Pcurve/domain/normal/curvature access | API | Face domain và surface evaluation; S8 | Dùng làm validator và editor trim, không chỉ xem tessellation |
| RCORE-07.C11 | Full solid grips giữ topology chung | Chưa thấy trong phạm vi rà | Đã rà Part/Surface NURBS API và OM9 Curve/Surface/PointsOn entry points | Contract riêng; không di chuyển CV độc lập của faces |
| RCORE-07.C12 | Rhino Rebuild/Refit end-to-end | Một phần | FreeCAD approximation primitives S1/S2/S3; chưa chứng minh toàn workflow Rhino | S10/S11 có Rust rebuild + CoreRebuild/refit adapter; nghiệm thu count/degree/retrim/deviation riêng từng slice |
| RCORE-07.C13 | G0/G1/G2 certification xuyên trimmed joins | Chưa kiểm chứng | Có curvature/derivative nền, chưa chạy certifier cho fixtures | Nêu measure/bounds, xử lý seam/singularity và report uncovered region |
| RCORE-07.C14 | Grip cancel/Undo/redo/reload | Chưa kiểm chứng | Geometry API không chứng minh transaction lifecycle của UI OM9 | Chạy fixtures state/History/identity trước bật command |
| RCORE-07.C15 | Affine transform BRep và matrix analysis | API | `transformShape`, `transformGeometry/transformGShape`, `mirror`; Matrix determinant/analysis; S13 | Frame/copy/Array ordering, singular rejection, mirror orientation và conversion report |
| RCORE-07.C16 | Bend/Twist/Flow/Cage phi tuyến đầy đủ | Chưa kiểm chứng | S13 chỉ chứng minh transform matrix; chưa audit toàn bộ solver deformation của host | Contract riêng từng deformation family và deviation/topology fixtures |

## Điểm vào source đã kiểm tra

- S1 — [Geometry.h](../../../../../src/Mod/Part/App/Geometry.h), [Geometry.cpp](../../../../../src/Mod/Part/App/Geometry.cpp): `GeomBSplineCurve`, `GeomBSplineSurface`, `setPole`, `Save/Restore`, conversion lỗi `CADKernelError`.
- S2 — [BSplineCurvePyImp.cpp](../../../../../src/Mod/Part/App/BSplineCurvePyImp.cpp): knot/degree/weight/periodic/origin, `approximate`, `interpolate` gọi native APIs và chặn `Standard_Failure`.
- S3 — [BSplineSurfacePyImp.cpp](../../../../../src/Mod/Part/App/BSplineSurfacePyImp.cpp): U/V knot/degree/origin, `setPoleRow/Col`, `setWeightRow/Col`, `exchangeUV`, basis construction.
- S4 — [TopoShape.h](../../../../../src/Mod/Part/App/TopoShape.h), [TopoShape.cpp](../../../../../src/Mod/Part/App/TopoShape.cpp): modeling BRep, tolerances và output topology; không phải wrapper Rhino.
- S5 — [FeatureFilling.cpp](../../../../../src/Mod/Surface/App/FeatureFilling.cpp), [FeatureGeomFillSurface.cpp](../../../../../src/Mod/Surface/App/FeatureGeomFillSurface.cpp): `BRepFill_Filling`, `GeomFill_BSplineCurves`, boundary ordering/continuity inputs.
- S6 — [Surface Gui Command.cpp](../../../../../src/Mod/Surface/Gui/Command.cpp): `CmdSurfaceFilling::activated` tạo `Surface::Filling`, `CmdSurfaceGeomFillSurface`.
- S7 — [FeatureProjectOnSurface.cpp](../../../../../src/Mod/Part/App/FeatureProjectOnSurface.cpp): feature projection native; option parity cần fixture riêng.
- S8 — [TopoShapeFacePyImp.cpp](../../../../../src/Mod/Part/App/TopoShapeFacePyImp.cpp), [GeometrySurfacePyImp.cpp](../../../../../src/Mod/Part/App/GeometrySurfacePyImp.cpp): `isPartOfDomain`, `normalAt`, `curvature`, `uIso/vIso`.
- S9 — [CommandSketcherOverlay.cpp](../../../../../src/Mod/Sketcher/Gui/CommandSketcherOverlay.cpp): B-spline curvature comb display command; không là editor general 3D NURBS của Rhino.
- S10 — [curve.rs](../../../rust/src/curve.rs), [CoreRebuild.cpp](../../../Gui/CoreRebuild.cpp), [CurveGeometry.cpp](../../../Gui/CurveGeometry.cpp): Rust rebuild request và native construction của slice hiện có.
- S11 — [surface.rs](../../../rust/src/surface.rs), [surface_refit.rs](../../../rust/src/surface_refit.rs), [SurfaceGeometry.cpp](../../../Gui/SurfaceGeometry.cpp), [SurfaceRefit.cpp](../../../Gui/SurfaceRefit.cpp): session ordering, periodic origin, candidate và native bound.
- S12 — [core_keyboard.rs](../../../rust/src/core_keyboard.rs): mapping `PointsOn`; riêng mapping này không chứng minh có editor thực thi.
- S13 — [TopoShapePyImp.cpp](../../../../../src/Mod/Part/App/TopoShapePyImp.cpp), [TopoShape.cpp](../../../../../src/Mod/Part/App/TopoShape.cpp), [Matrix.h](../../../../../src/Base/Matrix.h), [Matrix.cpp](../../../../../src/Base/Matrix.cpp): transform wrappers, `BRepBuilderAPI_GTransform`, `mirror`, `Matrix4D::determinant/determinant3` và transform analysis.

## Chi tiết bổ sung từ code decompile

Phần này tách logic C# đọc được khỏi mô tả API đi kèm và suy luận từ mã giả native.
Nhánh wrapper xác nhận cách xử lý/tham số được chuyển tiếp, nhưng không tự chứng minh thuật toán kernel, default UI hoặc kết quả runtime.
Các chi tiết này giữ nguyên phân loại FreeCAD, supported slice OM9 và trạng thái `source_inspected` / `not_run`.
Đã đối chiếu body bridge với các overload native: thứ tự tham số, output tùy chọn và các nhánh normalization đơn giản có căn cứ rõ hơn.
Mã giả vẫn có giới hạn phục hồi kiểu/virtual call; kiểm chứng source không thay kết quả runtime hoặc chứng nhận sai số solver.

| Hành vi và yêu cầu OM9 | Giới hạn cần kiểm chứng |
|---|---|
| **`Curve.Rebuild`**. C# đưa point count lên ít nhất 2 và kẹp degree trong 1–11. Bridge đổi thứ tự pointCount/degree thành degree/count của overload native có `preserveTangents`, đồng thời yêu cầu output mới. Nhánh native kẹp count 2–1000, đưa count của curve kín lên ít nhất 3; nhánh không chọn xử lý tangent riêng hạ degree về count−1 nếu cần. Report giữ requested/effective options và phân biệt tạo candidate với thay source. | Mapping bridge và nhánh normalization đã có căn cứ source; output count/degree, knots và sai lệch vẫn cần đo. Nhánh tangent chỉ được chọn khi flag bật, count lớn hơn degree+1 và count lớn hơn 4; flag không chứng minh mọi đầu vào đều được giữ tangent. Không gán các bounds này thành UI default hoặc đổi supported slice OM9 tự động. |
| **`Curve.Fit`**. C# chuyển degree, fit tolerance và angle tolerance nguyên trạng. Bridge nối đúng thứ tự tới native và yêu cầu output riêng; native kẹp degree trong 2–11, dùng fallback cho fit tolerance không dương và angle âm. Angle zero không dùng fallback angular; mô tả API đi kèm nêu đây là yêu cầu xử lý mọi kink. Nhánh đã đọc còn phụ thuộc context Rhino trước khi giải. | Body xác nhận nhánh fallback nhưng chưa xác nhận danh tính mọi trường state hoặc numerical fitting bound. Không xem call này là hàm thuần có thể chạy tùy ý trong worker/headless. Kết quả curve khác null vẫn phải đo deviation và kiểm topology theo contract OM9. |
| **Domain và các overload `Reverse` của curve/surface**. Logic C# đọc được: curve domain setter/Reverse dùng mutable pointer; curve Reverse trả bool. Surface Reverse một tham số tạo object kết quả; overload `inPlace=true` dùng mutable pointer, trả chính instance khi native thành công và null khi fail. Mô tả API đi kèm nêu domain đảo dấu/đầu mút khi reverse. | Domain setter curve không expose kết quả thành công; không suy validation hoặc locus preservation từ việc setter chạy xong. Không áp chính sách copy của surface cho curve hay dùng reverse để thay trim. |
| **Knot indexer, `InsertKnot` và `EpsilonEquals`**. Logic C# đọc được: indexer kiểm index âm/quá Count; không thấy kiểm thứ tự/finite của giá trị knot tại wrapper. InsertKnot overload một tham số chọn multiplicity 1, overload đầy đủ chuyển thẳng native. EpsilonEquals yêu cầu cùng Count và so các khoảng knot liên tiếp, nên phép dịch toàn knot vector cùng hằng số không bị phát hiện bởi riêng comparator này. | Shape/parameterization preservation của insert là mô tả API đi kèm, không phải bound native đã chứng minh. Không suy add-versus-target multiplicity từ tên tham số. Comparator basis không thay kiểm domain/locus/weights. |
| **`Surface.Rebuild` và `Surface.Fit`**. Rebuild chuyển trực tiếp U/V degree/count qua C# và bridge, yêu cầu surface output mới. Native xử lý từng chiều: degree trong 1–11, count tối đa 1000, tăng count lên degree+1 nếu cần và ít nhất 3 cho chiều kín. Fit giữ thứ tự U/V degree, fit tolerance và con trỏ output double. Native ghi lại một giá trị tolerance qua con trỏ này sau khi gọi solver; import managed đặt tên nó là `achievedTol`, nhưng `Surface.Fit` công khai chỉ trả surface, không trả double. | Normalization surface tăng count khi thiếu poles; không được thay bằng rule curve hạ degree. Chưa chứng minh output double là sai số cực đại hoặc bound được chứng nhận: nó bắt đầu từ tolerance effective và đi qua solver chưa audit đầy đủ. Rebuild/Fit không bao gồm lời gọi retrim trong bridge; preservation trim và deviation phải kiểm riêng. |
| **Ranh giới bridge, kết quả và ownership**. Các bridge Rebuild/Fit kiểm input null; nhánh null trả null, nhánh có input trả kết quả native. Curve Rebuild/Fit và Surface Rebuild truyền null cho output tùy chọn của native; các callee đã đọc có đường tạo output và cleanup khi fail. Chuyển đổi NURBS toàn curve và theo subdomain đi qua entry point riêng với interval policy explicit. | Tin cậy cao hơn cho mapping/thứ tự tham số và nhánh xử lý đơn giản; mã giả native vẫn cần xác nhận runtime. Không suy copy độc lập mọi metadata, thread safety, atomic document commit hoặc đầy đủ cleanup của mọi solver từ bridge mỏng. Một số virtual-call/return-type vẫn phục hồi chưa đầy đủ. |

## Kế hoạch adapter tối thiểu, Rust-first

1. Rust sở hữu basis snapshot, CV/edit-point selection, editor/session state, validation và conversion report.
2. Expose native evaluation/shape construction có bounds rõ; không giữ document pointer trong Rust worker.
3. Tách operation exact-preserving, approximate và topology-changing; mỗi operation có capability ID/options riêng.
4. Native dựng curve/surface/trim bằng OCCT, trả shape clone cùng diagnostic và stable mapping dự kiến.
5. Rust áp policy deviation/continuity/budget; native certifier được dùng nơi phải truy cập exact OCCT geometry.
6. Preview-only shapes và grips overlay có owner riêng; GUI commit một transaction sau stale/lock checks.
7. Reuse các slice Rust/adapter hiện có sau regression; không viết lại solver OCCT hoặc thêm business logic Python vì tiện.

Lỗi cần phân biệt: `InvalidBasis`, `UnsupportedGripType`, `TrimRebuildFailed`, `NoIntersection`, `AmbiguousResult`, `DeviationExceeded`, `CertificationBudgetExceeded`, `StaleSource`, `KernelFailure`.
`NoIntersection` có thể là empty result hợp lệ theo operation; không luôn biến thành exception hoặc tự đổi solver.

## Phụ thuộc

RCORE-01 định nghĩa representation/ownership; RCORE-02 cung cấp effective tolerance và units.
RCORE-03/05 cung cấp frame/snap khi kéo grips; RCORE-04/06 quản lý session/selection/lock.
RCORE-08 cung cấp đo deviation/continuity/validity; RCORE-09 giữ topology identity và History.
RCORE-10 kiểm save/reload/3DM; RCORE-12 quy định overlay và invalidation display mesh.

## Acceptance fixtures — chưa chạy tại baseline audit nguồn

| ID | Fixture và thao tác | Expected invariant / evidence |
|---|---|---|
| RCORE-07.T01 | Rational quarter circle R=10 mm, insert knot rồi elevate degree | Radius và locus giữ trong numerical bound; weights rational còn; report degree/count trước-sau |
| RCORE-07.T02 | Periodic closed curve đổi seam, reverse và save/reload | World locus giữ; seam/direction mới có mapping xác định; không duplicate closing segment |
| RCORE-07.T03 | B-spline 6 CV, kéo một CV rồi cancel/commit/Undo/Redo | Cancel trả basis gốc không undo rỗng; commit một transaction; Undo/Redo khôi phục đúng poles/weights |
| RCORE-07.T04 | Cùng curve bật edit points và CV | Vị trí/identity khác đúng định nghĩa; edit point nằm trên curve, không gắn knot index giả |
| RCORE-07.T05 | Trimmed surface có lỗ, Rebuild với retrim | Lỗ còn và trims hợp lệ hoặc fail explicit; không xuất surface phủ kín lỗ mà báo thành công |
| RCORE-07.T06 | Hai faces shared edge trong polysurface, PointsOn | Từ chối nhánh không hỗ trợ; không tách ngầm hay làm hở seam |
| RCORE-07.T07 | Loft ba profiles kín có seam đảo thứ tự | Profile order/seams/reversal cố định giữa preview/commit/History; không twist bất ngờ |
| RCORE-07.T08 | Refit curve với spike hẹp nằm giữa các mẫu coarse | Không báo certified chỉ từ coarse sample; adaptive/bound bắt spike hoặc báo budget exhausted |
| RCORE-07.T09 | Ma trận mutation: Join/Boolean fail; Scale matrix singular; Mirror solid; nonuniform scale circle/sphere | Fail giữ input/links/layers; singular bị reject; mirror có orientation hợp lệ; circle thành ellipse và sphere báo representation thật, không giữ tag primitive sai; không heal/fallback mesh âm thầm |
| RCORE-07.T10 | Plane pair G0-only, G1 và mặt cong có G2 sai; thêm sphere pole singularity | Report tách G0/G1/G2 và singular/undefined; không suy continuity chỉ từ Zebra/shaded |
| RCORE-07.T11 | Reference Curve.Rebuild: degree 0/1/11/12, count 1/2/degree/1000/1001, open/closed và tangent flag | Ghi requested và normalization kỳ vọng theo nhánh managed/native đã đọc; đo output degree/count/closure/tangents để kiểm chứng boundary thực tế còn chưa chạy; lỗi không thay source |
| RCORE-07.T12 | Curve domain [2,7], surface U/V domain khác nhau, reverse theo từng overload | Reference kỳ vọng theo API doc là [-7,-2] cho chiều reverse; đo locus/tangent và identity source/result; OM9 giữ mapping đúng trong History và không phụ thuộc index CV cũ |
| RCORE-07.T13 | Hai knot vectors cùng intervals nhưng dịch toàn bộ +10; thử insert knot multiplicity 1 và explicit | Comparator khoảng không đủ để chứng nhận domain giống nhau; report absolute knots/domain riêng; đo locus và parameterization sau insert theo numerical bound |
| RCORE-07.T14 | Surface Rebuild counts không lớn hơn degree và vượt 1000; Fit với tolerances explicit | Ghi rejection/normalization thực tế; không giả Curve.Rebuild normalization được dùng lại; deviation/retrim phải được đo độc lập với API return value |

## Giới hạn và câu hỏi cần thêm evidence

FreeCAD có API NURBS rộng hơn những tên lệnh hiện được OM9 expose; adapter/UI/contract vẫn là phần riêng phải làm.
Chưa khẳng định Rhino Rebuild và FreeCAD `approximate` dùng cùng basis, knot distribution hoặc bound.
SurfaceRefit hiện thấy làm việc trên section-curve candidates; tên file không chứng minh refit mọi trimmed surface.
Chưa có runtime fixtures cho knot removal, degree elevation, periodic seams, singularities và retrim trong checkout này.
Search chưa thấy editor generic/solid grips chỉ giới hạn các entry points nêu trên; addon ngoài checkout không thuộc kết luận.
Lần audit nguồn ban đầu chỉ viết tài liệu và không chạy build/native tests; completion vẫn cần exact source/runtime version cùng kết quả từng fixture, tách với cập nhật slice ở trên.
