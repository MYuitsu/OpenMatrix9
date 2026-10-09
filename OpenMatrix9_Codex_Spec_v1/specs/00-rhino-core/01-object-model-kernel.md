---
id: RCORE-01
title: Kiểu đối tượng, representation và ranh giới kernel
priority: P0
date: 2026-10-09
evidence_level: source_inspected
runtime_validation: not_run
evidence_scope: initial_source_audit_2026-10-09
---

# RCORE-01 — Kiểu đối tượng và ranh giới kernel

## Cập nhật triển khai P0 — 2026-10-09

Metadata `source_inspected/not_run` và các phát hiện nguồn bên dưới là baseline
audit ban đầu. Tiến độ triển khai sau baseline được ghi riêng trong
[báo cáo P0](../../../docs/validation/2026-10-09-rhino-core-p0.md); không dùng tổng
Rust tests để đóng toàn bộ fixtures của chương.

Đã thêm kiểm basis Rust trên đường publish spline thật: poles/knots phải hữu hạn,
degree/cardinality/domain/multiplicity và quy ước periodic phải hợp lệ trước
khi adapter dựng output. Spline được tạo hiện truyền `weights: None`; kiểm rational
weights/homogeneous có unit tests ở Basis, chưa là gate chung cho native rational
consumers. Export 3DM kiểm schema, tuple và index trước chuyển đổi
native; CAD và Mesh vẫn dùng representation riêng. Fixture hỗn hợp rational arc,
face có lỗ, mặt trụ mở, solid và mesh đã qua 56 checks, cold restore 28 checks;
native geometry target xác nhận thêm adaptive area và mẫu điểm độc lập.

**P0 còn thiếu:** descriptor/capability chung theo representation; identity có
generation/revision xuyên mọi adapter; ma trận chuyển đổi có bảo toàn topology,
parameter mapping và userdata; kiểm clone/subobject ownership cùng stale worker
trên mọi family. Sự hiện diện converter không chứng minh Rhino/OCCT semantic parity.

## Mục tiêu và phạm vi

Định nghĩa dữ liệu mà các lệnh OM9 được phép nhận, sửa và trả về trước khi bàn đến tên lệnh.
Phạm vi gồm `OM9-OBJECT`, `OM9-SOLID`, `OM9-DISPLAY`, mọi solver và trao đổi file.
Các yêu cầu kế thừa RCORE-01 cũ được mở rộng tại đây; đây không phải command mới hoặc feature thứ 608.
Baseline audit nguồn ban đầu là **source-inspected / not-runtime-validated**; ở lần audit đó chưa chạy FreeCAD, OCCT fixture hay Rhino 5.
FreeCAD HEAD được đối chiếu là `21d36cfa1eb110a1d0667050ff31706298805bbd`.
OM9 là repository lồng riêng tại HEAD `52e887ab706dcd5d80e23979fb1f1b62d8d869b0`.
Cả checkout có thay đổi làm việc; HEAD không đại diện toàn bộ nội dung source đã đọc.
Phải dùng baseline/hash của lần audit ở [chỉ mục nhóm](README.md) khi tái kiểm chứng.

Nguồn Rhino: User's Guide, PDF 25–31 / trang in 17–23 đã được ghi ở [audit nguồn](../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md).
Các schema, tên lỗi và policy bên dưới là thiết kế OM9 cần triển khai, không được gán là API Rhino.
openNURBS cung cấp trao đổi 3DM và dữ liệu/evaluation cơ bản; nó không bao gồm toàn bộ intersection, tessellation, interpolation, Boolean hay mass properties của Rhino SDK.
Vì vậy lựa chọn backend phải theo phép toán, không theo khả năng đọc một đối tượng NURBS. [Giới hạn openNURBS](https://developer.rhino3d.com/guides/opennurbs/what-is-opennurbs/) (truy cập 2026-10-09).

## Hợp đồng representation

| Loại logic | Dữ liệu bắt buộc và điều kiện | Không được đồng nhất với |
|---|---|---|
| Point / point cloud | Tọa độ hữu hạn, units, placement; cloud có thứ tự/index và thuộc tính point được hỗ trợ | Vertex của BRep hoặc các object point rời nếu chưa công bố conversion |
| Curve | Loại analytic/NURBS, domain, direction, trạng thái closed/periodic và giới hạn trim của curve | Polyline hiển thị đã tessellate |
| Polycurve | Danh sách đoạn có thứ tự, hướng và joint relation; bảo toàn phân đoạn nếu contract yêu cầu | Một B-spline fit qua mẫu hoặc mọi wire OCCT |
| Surface nền | Hàm theo UV, domain U/V, degree/knots/weights nếu NURBS, periodicity từng chiều | Face đã trim hoặc shell/solid |
| Trimmed face | Surface nền, vòng ngoài/vòng lỗ, pcurve UV, edge 3D, orientation và tolerance | Chỉ surface nền hoặc mesh phủ đầy lỗ |
| Polysurface / shell | Tập face với edge adjacency, orientation và thông tin biên hở/không manifold | Solid chỉ vì bbox kín hoặc shaded nhìn kín |
| Solid | Shell boundary hợp lệ, kín, orientation phù hợp và policy đối với nhiều shell/cavity | Surface tuần hoàn, face kín theo U hoặc mesh kín |
| Lightweight extrusion | Profile, path, cap, orientation và dữ liệu nguồn; conversion sang BRep phải được báo | Part Extrusion được dựng lại nhưng không còn cấu trúc dữ liệu nguồn |
| Mesh | Vertex/facet, winding, normals và thuộc tính được hỗ trợ; đơn vị rõ | BRep chính xác hoặc display mesh cache của BRep |
| Instance | Definition identity, transform và instance attributes; definition thuộc RCORE-06 | Shape đã bake/flatten tại vị trí world |
| Annotation | Loại annotation, style/reference, frame và units theo RCORE-11 | Curve outline hoặc shape trang trí |

Với NURBS phải giữ degree, knots kèm multiplicities, weights, periodicity, domain, direction và seam.
Với face phải giữ sự khác nhau giữa trim UV, edge 3D và isocurve: isocurve không mặc nhiên là cạnh topology.
Một cạnh seam có thể được cùng face sử dụng hai lần; đếm occurrence khác với đếm edge identity.
Representation analytic có thể đổi thành NURBS nếu policy cho phép, nhưng phải báo conversion và đo sai lệch.
Giữ locus không đủ để giữ parameterization: conversion report phải tách bảo toàn hình học khỏi mapping tham số.
`Curve.HasNurbsForm` mô tả hai mức thành công khác nhau cho hai yêu cầu này trong nguồn RhinoCommon đã đọc.
Không bắt buộc mọi representation tồn tại ở dạng native y hệt Rhino; bắt buộc công bố chính xác thứ được giữ và thứ mất.
Render mesh là cache dẫn xuất; thay mật độ mesh không thay CAD gốc hoặc chuyển routing sang mesh solver.

## Schema request, output và ownership

`GeometryRef = {document_id, object_id, subelement_ref?, geometry_revision, representation, local_to_world}`.
`GeometrySnapshot = {schema_version, unit_context_revision, geometry_data, topology_data, source_attributes}`.
`NurbsCurveData = {degree, poles_xyz, weights, knots, multiplicities, periodic, domain, orientation}`.
`NurbsSurfaceData` mở rộng theo hai trục U/V; dimensions và product của pole grid phải được kiểm trước allocation.
`FaceData = {surface_ref, loops:[{role, oriented_edge_uses}], pcurves, face_orientation}`.
`ConversionReport = {from, to, exactness, preserved_fields, dropped_fields, deviation_report?, warnings}`.
`OperationCapability = {operation_id, input_types, option_set, backend, constraints, evidence_id}`.
`GeometryResult = {status, output_snapshots, conversion_reports, topology_map, diagnostics}`.
Tên object/layer hoặc `EdgeN` đơn lẻ không phải identity bền; dependency phải theo RCORE-09.

Input phải được resolve sang world đúng một lần, kể cả placement lồng instance/part/group.
Native shape vẫn được owner native quản lý; Rust chỉ giữ opaque handle có generation hoặc snapshot do Rust sở hữu.
Không truyền raw pointer document, view provider hoặc OCCT object ra worker có lifetime không được ràng buộc.
FFI phải nêu allocator/free cùng phía, encoding, length/index bounds, cancellation và luồng được phép gọi.
Panic Rust và exception C++ không vượt ABI; chuyển thành lỗi có operation và input identity.
Safe Rust không chứng nhận toàn bộ kernel/FFI/native dependency là memory-safe.
Adapter không đổi shape trực tiếp trong phase preview: clone/snapshot → solve → validate → commit một transaction.
Commit kiểm lại revision, lock, ownership và existence của mọi source; stale request không được overwrite dữ liệu mới.
Lỗi hoặc Esc giải phóng preview/cache tạm, không xóa source và không tạo undo entry rỗng.

## Ma trận capability theo checkout

Nhãn FreeCAD mô tả bề mặt source đã thấy; mọi hàng còn phải nghiệm thu runtime và mapping OM9.

| ID | Capability cần có | FreeCAD | Bằng chứng và giới hạn | OM9 riêng cần làm/đối soát |
|---|---|---|---|---|
| RCORE-01.C01 | BRep object có placement và property shape | API | `Part::Feature::Shape`, kế thừa `App::GeoFeature`; S1/S2 | Adapter identity/revision và routing representation |
| RCORE-01.C02 | Curve NURBS rational/periodic | API | `GeomBSplineCurve`, persistence poles/weights/knots/degree; S3 | Bảo toàn dữ liệu qua 3DM/FCStd và supported edit slices |
| RCORE-01.C03 | Surface NURBS, face trim và pcurve | API | `GeomBSplineSurface`; `CurveOnSurface`, `isSeam`; S3/S4 | Mapping loop/orientation/domain có fixture lỗ và seam |
| RCORE-01.C04 | Phân biệt face/shell/solid/compound | API | `TopoShape` có type, `isValid`, `isClosed`; S5 | Quy tắc solid đầy đủ; không suy solid từ một boolean |
| RCORE-01.C05 | Point cloud và mesh độc lập BRep | API | `Points::Feature`, `Mesh::Feature`; S6/S7 | Type mapping và policy chuyển đổi, không dùng mesh cache làm master |
| RCORE-01.C06 | Shared instance/placement | API | `App::Link`, link placement và scale; S8 | Rhino definition/instance attributes và nested transform thuộc RCORE-06 |
| RCORE-01.C07 | Lightweight extrusion giữ nguyên representation | Một phần | `Part::Extrusion` có feature dựng hình; S9; chưa chứng minh serialization tương đương Rhino extrusion | Tách preserve payload khỏi BRep editable và cảnh báo conversion |
| RCORE-01.C08 | Native CAD và tessellation tách biệt | API | `TopoShape` giữ BRep; tessellation trong luồng shape/display; S5 | Cache generation/invalidation và đảm bảo export dùng CAD hiện tại |
| RCORE-01.C09 | Polycurve giữ phân đoạn/thuộc tính Rhino | Một phần | Wire/edge topology có ở S5; chưa chứng minh giữ semantics mọi polycurve | Conversion report và kiểm exactness theo từng kiểu đoạn |
| RCORE-01.C10 | Backend registry chung theo operation × type × options | Chưa thấy trong phạm vi rà | Đã rà Part geometry/topology và OM9 curve/surface/3DM entry points; chưa thấy contract thống nhất của nhóm này | Rust registry bắt buộc trước enable command |
| RCORE-01.C11 | Native lifetime qua worker và document close | Chưa kiểm chứng | Source shape/property không chứng minh lifecycle OM9 khi hủy/đóng document | Audit handle/generation/cancel rồi fixture runtime |
| RCORE-01.C12 | Annotation typed và structural 3DM roundtrip | Chưa kiểm chứng | Không audit toàn bộ annotation/3DM trong file này | Liên kết RCORE-10/11, không flatten rồi gọi là preserve |

## Điểm vào mã nguồn đã kiểm tra

- S1 — [PartFeature.h](../../../../../src/Mod/Part/App/PartFeature.h): `Part::Feature`, `PropertyPartShape Shape`, `getSubObject`.
- S2 — [GeoFeature.h](../../../../../src/App/GeoFeature.h): `Placement`, `getGlobalPlacement`; `globalPlacement` cũ có cảnh báo không xử lý App::Link đúng.
- S3 — [Geometry.h](../../../../../src/Mod/Part/App/Geometry.h) và [Geometry.cpp](../../../../../src/Mod/Part/App/Geometry.cpp): `GeomBSplineCurve`, `GeomBSplineSurface`, `Save/Restore`, OCCT handle và lỗi `CADKernelError`.
- S4 — [TopoShapeFacePyImp.cpp](../../../../../src/Mod/Part/App/TopoShapeFacePyImp.cpp), [TopoShapeEdgePyImp.cpp](../../../../../src/Mod/Part/App/TopoShapeEdgePyImp.cpp): `CurveOnSurface`, `isPartOfDomain`, `isSeam`.
- S5 — [TopoShape.h](../../../../../src/Mod/Part/App/TopoShape.h), [TopoShape.cpp](../../../../../src/Mod/Part/App/TopoShape.cpp): shape enumeration, `isValid`, `isClosed`, OCCT BRep và tessellation helpers.
- S6 — [PointsFeature.h](../../../../../src/Mod/Points/App/PointsFeature.h): `Points::Feature` kế thừa `App::GeoFeature`.
- S7 — [MeshFeature.h](../../../../../src/Mod/Mesh/App/MeshFeature.h): `Mesh::Feature`, representation mesh riêng.
- S8 — [Link.h](../../../../../src/App/Link.h): property set của Link gồm placement và liên kết; không suy full Rhino Block parity từ class này.
- S9 — [FeatureExtrusion.cpp](../../../../../src/Mod/Part/App/FeatureExtrusion.cpp): `Part::Extrusion`, dựng shape từ profile.
- S10 — [CurveGeometry.cpp](../../../Gui/CurveGeometry.cpp): nhận poles từ Rust rồi gọi `buildFromPolesMultsKnots`; đây là một slice adapter hiện hữu.

## Chi tiết bổ sung từ code decompile

Phần này tách logic C# đọc được khỏi mô tả API đi kèm và suy luận từ mã giả native.
Nhánh wrapper xác nhận cách xử lý/tham số được chuyển tiếp, nhưng không tự chứng minh thuật toán kernel, default UI hoặc kết quả runtime.
Các chi tiết này giữ nguyên phân loại FreeCAD, supported slice OM9 và trạng thái `source_inspected` / `not_run`.

| Hành vi và yêu cầu OM9 | Giới hạn cần kiểm chứng |
|---|---|
| **`DuplicateShallow`, `Duplicate` và quyền sở hữu mutable**. Logic C# đọc được: shallow duplicate giữ liên kết parent; full duplicate đi qua bridge tới phép duplicate native, trả null khi input null. Nhánh chuyển sang mutable tạo duplicate khi wrapper chưa có private pointer và cập nhật bookkeeping original/edited của parent. OM9 phải phân biệt borrowed/shared snapshot với owned mutable output trước preview/worker. | Có nhánh riêng cho polycurve segment và Brep loop; không được khái quát rằng mọi subobject mutation luôn tạo bản sao độc lập. Bridge xác nhận điểm gọi duplicate, chưa chứng minh commit document, lifetime sau dispose hoặc thread safety. |
| **`HasNurbsForm` và mapping sau conversion**. Mô tả API đi kèm phân biệt mức 1 giữ parameterization với mức 2 có thể chỉ giữ locus/domain. Bridge query trả 0 cho input null và dispatch sang geometry cho input có giá trị. `ToNurbsCurve` chuyển tolerance 0, bỏ giới hạn subdomain ở overload toàn curve và chuyển interval ở overload có subdomain; bridge yêu cầu native tạo output riêng. Có hai API đổi tham số theo hai chiều, dùng bool kết quả riêng với giá trị output. OM9 cần `parameter_map` hoặc `parameterization_preserved`, không reuse raw parameter chỉ vì locus giống nhau. | Kiểu trả về của hàm conversion do decompiler suy ra mâu thuẫn với khai báo managed và native callee; đây là giới hạn phục hồi ABI, không phải chứng cứ conversion không trả geometry. Chưa xác nhận tolerance 0 tương đương bound nào, mapping số cụ thể hay độ chính xác cho mọi loại curve. Nếu mapping trả false, giá trị output khởi tạo không được dùng như mapping hợp lệ. |
| **`IsSolid`, `SolidOrientation` và `IsManifold`**. Logic C# đọc được: `IsSolid` và `SolidOrientation` dùng cùng native selector nhưng một bên trả kiểm tra khác zero, một bên giữ giá trị orientation; `IsManifold` dùng selector riêng. Adapter không được làm mất orientation sau khi rút gọn thành bool, hoặc thay solid test bằng manifold test. | Mô tả closed/oriented manifold là mô tả API đi kèm; wrapper không trình bày thuật toán closure, self-intersection hay chứng nhận sản xuất. |
| **`Transform`, `IsDeformable` và `MakeDeformable`**. Logic C# đọc được: transform là mutation trả bool; query khả năng biến dạng và yêu cầu chuyển sang dạng deformable là hai thao tác riêng. Mô tả API đi kèm cảnh báo orientation sau transform đảo hướng. Conversion/repair orientation phải là quyết định riêng có report. | Không thấy bước tự động flip trong wrapper `Transform`; không suy native luôn tự flip hoặc luôn không flip mọi representation. Thành công của transform không chứng minh preservation của trim/topology/attributes. |

## Kế hoạch adapter tối thiểu, Rust-first

1. Rust định nghĩa type tags, owned snapshots, validation và capability registry có version; giữ command ID hiện hành.
2. C++ resolve document/subelement/placement và sao chép geometry/topology cần thiết qua ABI nhỏ; không đưa logic lựa chọn backend vào widget.
3. Rust chọn operation theo capability; unsupported trả lý do cụ thể và command vẫn hiển thị disabled.
4. C++ gọi OCCT cho BRep, openNURBS cho archive trong phạm vi đã kiểm tra; mỗi API exception được chặn và ánh xạ lỗi.
5. Validate geometry lẫn topology, tạo conversion report; Rust quyết định có thể commit theo contract hay phải từ chối.
6. GUI thread commit shape, metadata, links và History cùng transaction; đánh invalid cache theo generation.
7. Native exception là thường trực do FreeCAD/Qt/OCCT API; Python chỉ còn bootstrap/API glue hiện hữu có lý do, không thêm business logic.

Lỗi chuẩn dự kiến: `UnsupportedRepresentation`, `InvalidBasis`, `InvalidTrim`, `NonFiniteGeometry`, `StaleGeometry`, `UnsupportedTransform`, `KernelFailure`, `ConversionLoss`.
Thông báo phải chỉ object/subelement nào lỗi và phép toán nào không được hỗ trợ; không fallback âm thầm sang mesh/heal.
Không sửa ngầm độ chính xác để giữ trạng thái success.

## Phụ thuộc

- RCORE-02 cung cấp unit/tolerance context cho mọi giá trị độ dài và conversion.
- RCORE-03/05 cung cấp frame/pick world; RCORE-06 cung cấp lock, layer và instance ownership.
- RCORE-07/08 cung cấp solver và validator; RCORE-09 quy định revision/subelement/dependency map.
- RCORE-10 quy định preservation/roundtrip; RCORE-11 annotation; RCORE-12 cache display.

## Acceptance fixtures — chưa chạy tại baseline audit nguồn

| ID | Fixture và hành động | Expected invariant / bằng chứng phải lưu |
|---|---|---|
| RCORE-01.T01 | Circle rational bán kính 10 mm, chuyển analytic → NURBS → native | Bán kính/độ lệch nằm trong tolerance công bố; weights không bị thay bằng toàn 1; lưu basis và report |
| RCORE-01.T02 | Face phẳng 20 × 20 mm có lỗ tròn bán kính 3 mm | Trim hole còn; điểm giữa lỗ không thuộc face; area gần `400 − 9π` mm² theo bound kiểm tra |
| RCORE-01.T03 | Mặt trụ periodic U, chưa cap | Không được gắn loại solid chỉ từ periodic/closed U; hai biên tròn hở được nhận diện |
| RCORE-01.T04 | BRep sphere có display mesh, đổi mesh density và display mode | Type, CAD hash/revision và radius không đổi; routing vẫn CAD |
| RCORE-01.T05 | Polycurve line + rational arc, đảo hướng rồi save/reload | Thứ tự/hướng đoạn, endpoint và joint policy còn; nếu merge phải báo conversion trước commit |
| RCORE-01.T06 | Hai instances cùng definition, một instance mirror và scale không đều | Transform áp đúng một lần; không biến đổi definition của instance kia; unsupported transform phải từ chối rõ |
| RCORE-01.T07 | NURBS chứa NaN, weights sai hoặc knot vector không hợp lệ | Từ chối trước native allocation/commit; không object/undo entry thừa |
| RCORE-01.T08 | Preview solver, sửa source hoặc đóng document trước completion | Kết quả stale bị loại; không crash/UAF; preview và handles được giải phóng |
| RCORE-01.T09 | Lưu/reload FCStd của curve periodic và face có seam/lỗ | So type, topology, basis, world bounds và deviation; không dùng ảnh shaded làm bằng chứng duy nhất |
| RCORE-01.T10 | Analytic curve chuyển sang NURBS, dependency lấy điểm theo tham số gốc | Đo locus riêng với parameter mapping; không reuse raw parameter khi conversion chưa chứng minh tương ứng; lưu kết quả query và mapping |
| RCORE-01.T11 | Clone/shallow snapshot rồi sửa subcurve hoặc face, giữ source và sibling instance | Xác định owner mutation trước thao tác; source/sibling chỉ đổi khi contract yêu cầu; báo lifecycle khác nhau của standalone và subobject |

## Giới hạn và câu hỏi cần thêm evidence

Chưa có cam kết semantic parity giữa mọi OCCT wire/face và Rhino polycurve/BRep; cần bảng conversion từng type.
`TopoShape::isClosed` có nhánh theo topology type, không thay kiểm orientation, self-intersection hoặc điều kiện volume.
Chưa xác nhận toàn bộ payload userdata/annotation/extrusion qua FCStd và 3DM; các cột read/edit/preserve/export phải độc lập.
Các kết luận chưa thấy equivalent chỉ giới hạn file/nhánh source đã nêu; không kết luận FreeCAD hoặc addon bên ngoài không có khả năng đó.
Lần audit nguồn ban đầu chỉ viết tài liệu và không chạy build/native tests; evidence triển khai mới nằm trong phần cập nhật, không nâng implementation status chỉ từ spec hoặc symbol.
