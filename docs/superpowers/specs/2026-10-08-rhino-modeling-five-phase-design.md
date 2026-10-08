# Thiết kế workflow Rhino → OpenMatrix9: 5 phase hỗ trợ hình học

Ngày: 2026-10-08 (Asia/Saigon).
Trạng thái: bản thiết kế để review theo yêu cầu chia 5 phase; chưa cho phép suy ra đã implement hay đã nghiệm thu.
Người dùng đã xác nhận ưu tiên file 3DM trước, clipboard sau. Yêu cầu trước đây về full exchange giữ lịch sử/render được thu hẹp cho workflow mới; không sửa báo cáo hoặc chính sách preservation lịch sử.

## Mục tiêu

Chuyển selection từ Rhino qua 3DM, nhận geometry đúng, phân biệt CAD/Mesh trước khi bắt điểm, sửa/vẽ tiếp, lưu FCStd rồi xuất selection gồm cả hình học nhập và mới tạo.
Mốc sử dụng chính là hết Phase 3 đối với mô hình curve/NURBS/BRep thông thường. Đây không phải cam kết tái tạo mọi lệnh Rhino hay bảo toàn mọi bảng openNURBS.

## Global Constraints

- Source thực hiện: H:/FreeCAD-src/build/om9-dev; H:/FreeCAD-src/Mod/OpenMatrix9 chỉ là nguồn tích hợp có đối chiếu, không ghi đè nguyên checkout.
- openNURBS pin: eb92af3ba1806b0a34a99aba0d3bda83e3d46083; Rhino5/V5 là target baseline hiện tại.
- Phase 1 ưu tiên Import/Export Selected qua file 3DM; Ctrl+C/Ctrl+V trực tiếp để sau và không chặn nghiệm thu năm phase này.
- History Rhino/Matrix, materials, textures, lights, rendering và layouts không thuộc workflow dựng tiếp; Undo/Redo FreeCAD, chọn cạnh/mặt và Wireframe/Shaded thuộc phạm vi.
- CAD/NURBS có tessellation hiển thị vẫn là CAD; geometry kind, host representation và display-mesh state là ba thuộc tính riêng.
- Snap mặc định không enumerate mesh vertices, PointCloud points hoặc SubD display-mesh vertices; không biến CAD thành mesh để chỉnh sửa.
- Geometry hiện tại và geometry mới là nguồn export modeling; không phục hồi snapshot cũ để ghi đè sửa/xóa/copy của người dùng.
- Import/modeling là chế độ mới tách khỏi preservation; không làm yếu các guard của chế độ preservation đã có.
- Giữ ngưỡng nhẫn: bounds 0.001 mm; area/volume max(0.001, 0.0001 * abs(reference)); fixture analytical dùng ngưỡng riêng ghi trong oracle hiện hữu.
- Mỗi phase nghiệm thu trên cùng source/binary với hashes, fixture và kết quả rõ ràng; test lịch sử không được tính là lần chạy mới.
- Không tự commit/push, ghi đè thay đổi của tác vụ khác, thay SDK hoặc sửa skill trong nhiệm vụ lập kế hoạch này.

## Lựa chọn kiến trúc

1. **Khuyến nghị: thêm modeling mode.** Dùng reader/converter native đang có, tạo bản geometry làm việc độc lập và writer từ trạng thái hiện tại. Giữ preservation mode riêng.
2. Dùng geometry-only hiện tại nguyên trạng: ít đổi code nhưng chưa có báo cáo capability theo thao tác, snap budget và semantics block/reference mong muốn.
3. Tiếp tục full archive preservation làm điều kiện cho dựng tiếp: giữ nhiều dữ liệu nhưng bị chặn bởi plugin/history/render không cần thiết. Không chọn cho workflow này.

Rust giữ mode/capability/snap policy. C++/Qt đọc archive, chuyển CAD, chọn vùng snap, thực hiện typed Part adapters và transaction. Python bind object/FCStd và chuẩn bị selection. Không thêm Python Rhino converter hoặc thực thi văn bản người dùng.

## Bằng chứng baseline và giới hạn

- Bản dev có ThreeDm.py, ThreeDmArchiveState.py, Gui/ThreeDmArchive.cpp, Gui/ThreeDmMerge.cpp, CoreSnaps.cpp và CoreSnapGeometry.cpp.
- CoreSnaps::pick hiện duyệt document->getObjects() mỗi lần pick. CoreSnapGeometry lấy Edges/Vertexes qua native getSubObject; chưa có explicit mesh/cloud classifier và query budget.
- Không kết luận snap hiện đang enumerate mesh vertices: code chủ yếu đọc CAD shape. Việc phải bổ sung là guard phân loại và tránh duyệt geometry/document lớn.
- Checkout chính có CurveGeometry/CoreRebuild/SurfaceGeometry/EditGeometry cùng Rust spline/surface/edit; bản dev chưa có đủ các file đó. Phải tích hợp chọn lọc và test trên cùng binary.
- Báo cáo fast-3dm-import: 12 runtime reports/151 checks và 7 native suites mục tiêu, không phải chạy lại mọi historical suite.
- CAD wireframe: 8 reports/105 checks; BRep seam/pole edited V5: 4 target files đạt với native/host checks riêng.
- Source/binary của các báo cáo khác nhau; không cộng chúng thành chứng nhận workflow cuối.

Tham chiếu: ../../validation/2026-10-08-fast-3dm-import.md, ../../validation/2026-10-08-cad-wireframe.md, ../../validation/2026-10-08-seam-trim-complete.md, ../../validation/2026-10-08-native-reference-remap.md.

## Hợp đồng dữ liệu và API đề xuất

Các tên dưới đây là thiết kế mới, không phải API được xác nhận đang tồn tại.

- GeometryKind: CadPoint=1, CadCurve=2, CadBrep=3, Mesh=4, PointCloud=5, SubD=6, Mixed=7, Retained=8, Unknown=9.
- Representation: native-cad, native-mesh, native-points, native-subd, preview, retained.
- DisplayMeshState: unknown, absent, present. Chỉ đọc cache metadata sẵn có; không tạo tessellation để xác định.
- OperationCapabilities: select, snap, transform, curve-edit, surface-edit, boolean, export-v5; mỗi giá trị có trạng thái supported/scoped/unsupported và reason.
- classifySnapObject(const App::DocumentObject*) -> SnapObjectInfo trong Gui/SnapObjectInfo.h; loại object lấy từ runtime type/native source và resolved Link member, không dựa vào tên hoặc CountFacets>0.
- Rust modeling_exchange::snap_allowed(kind: GeometryKind, native_cad: bool, preview: bool, mode: u32) -> bool; FFI om9_modeling_snap_allowed(kind: u32, native_cad: bool, preview: bool, mode: u32) -> bool. Giữ mode End=2, Mid=4, Point=8 hiện có.
- SnapQuery chứa cursor_px, radius_px=8, max_objects=64, max_candidates_per_object=2048, max_candidates_total=8192. Đây là budget thiết kế ban đầu có thể cấu hình, chưa là số đo.
- querySnapCandidates(view, query) -> SnapQueryResult: candidates, complete, visited_objects, read_mesh_vertices, read_cloud_points, elapsed_us, reason. Không trả incomplete như một snap chắc chắn; khi vượt budget giữ pick thủ công và thông báo scoped.
- Thêm ThreeDm.import_file(..., mode='modeling') và ThreeDm.export_file(..., modeling=True) với mặc định cũ giữ nguyên. ModelingRequest schema_version=1 chứa selection roots, world transforms, current geometry và geometry dependencies, không chứa history/render closure.
- ThreeDmModeling.prepare_modeling(path, staging, scale) -> prepared manifest; ThreeDmModeling.stage_modeling_selection(objects, staging) -> request.
- Gui/ThreeDmModeling.h: prepareModelingArchive(path, staging, customUnitMm) -> QJsonObject; writeModelingArchive(request, destination) -> void. Errors đi qua native bridge như các ThreeDm API hiện có.
- Unknown/opaque geometry ảnh hưởng selection phải báo trước mutation; không tự bỏ geometry rồi báo thành công. History/render ngoài scope có thể loại khỏi modeling output với báo cáo.
- normalizeModelingCurve(const ON_Curve&, const NativeReferenceGraph&, double tolerance) -> NormalizedCurve {owned_curve, fidelity, reason}. Mặc định chỉ nhận exact conversion; phép xấp xỉ cần lựa chọn riêng, không phải mặc định.
- materializeModelingBlock(const ModelingBlockRequest&) -> ModelingBlockResult: current CAD/mesh members, world transforms, source/member map và issues; linked resource không embedded báo thiếu, không tự tải.
- writeModelingTarget(request, destination, targetVersion) -> TargetWriteReport: written classes, converted classes, rejected classes, validation bindings. V5 là baseline; target mới cho SubD chỉ mở sau proof version cụ thể.

## Phase 1 — Nhận geometry qua 3DM và tạo bản làm việc

P1.1 Chốt một baseline source/runtime và nối modeling mode cho File/menu/CMD/API.
P1.2 Import points/curves/BRep/extrusion/mesh/cloud thông thường đúng mm/cm, metadata cơ bản, transform; unsupported geometry preflight rõ.
P1.3 Selected export ghi current geometry và geometry mới cùng file V5; reread rồi atomic replace.

Gate: import selection → chọn đúng cạnh/mặt → thêm một object mới → export selected → FCStd reopen sau xóa nguồn. Không có clipboard gate.
Status kế thừa: file exchange/display đã có scoped evidence; modeling mode là planned.

## Phase 2 — Phân loại object, snap nhẹ và sửa curves

P2.1 Tách kind/representation/display cache; CAD có display mesh vẫn snap native.
P2.2 Snap broad-phase bằng screen index/picking, cập nhật index theo geometry/placement/visibility/camera; không scan cả document mỗi hover.
P2.3 Tích hợp Curve/Rebuild chọn lọc từ checkout chính vào dev, giữ command IDs/mouse/CMD.
P2.4 Typed CV/weight/knot editing cho curve được công bố; cache invalidation, Undo/Redo và export current curve.

Fixture nặng: mesh 1,000,000 vertices và cloud 1,000,000 points cùng CAD spline/BRep; pointer-events không đọc toàn mesh/cloud. Dùng poisoned accessors để kiểm tra zero-read độc lập với thời gian máy.
Performance target bổ sung: 500 query sau warmup, p95 <=16 ms trên máy/viewport/build đã ghi; đây là mục tiêu nghiệm thu, không phải performance đã đạt. Correctness zero-read là gate cứng.
Gate: snap native → sửa/nối curve nhập → dựng curve mới → save/reopen → current export. Mesh/cloud không tham gia snap mặc định.
Status: imported CAD curves và basic snap có code; classifier/index/CV editor và combined lifecycle chưa nghiệm thu.

## Phase 3 — Dựng tiếp trên surfaces/BRep/solid

P3.1 Nghiệm thu trim/holes/seam/pole/cavity và geometry validity bằng converter hiện có; chỉ sửa khi regression chứng minh lỗi.
P3.2 Tích hợp Surface/Edit controllers từ checkout chính; dùng imported curves cho Loft/Sweep, imported faces/solid cho Trim/Join/Explode/Boolean.
P3.3 Workflow nhẫn: import → curve/cutter mới → BooleanDifference → chi tiết Sweep/Loft mới → current selected export → Rhino5/FreeCAD reread.

Fillet/offset và surface CV editing ghi là capability riêng; chưa có adapter/proof thì disabled, không yêu cầu tái tạo toàn Rhino SDK để đóng workflow CAD baseline.
Gate: CAD nguyên bản trong dung sai, không dùng mesh thay CAD; source-deleted FCStd; undo thất bại không thay input; new+edited selected output đúng.
Status: converter và edited V5 sphere/cylinder có scoped evidence; workflow nhẫn dựng tiếp chưa nghiệm thu.
Đây là mốc phát hành thử phục vụ nhu cầu chính.

## Phase 4 — Blocks và geometry phụ thuộc references

P4.1 Modeling block flatten/materialize world geometry có lựa chọn; copy độc lập và nested affine chính xác.
P4.2 Giải quyết CurveOnSurface/PolyEdge/reference thành owning editable curve khi exact representation đã được chứng minh; không giữ history dependency cho modeling.
P4.3 Copy/member edit/delete/export/lifecycle phải dùng current owner; không silent snapshot restoration.

Gate: sửa bản copy không đổi bản gốc; compound/block mixed CAD+mesh snap phân loại theo từng member; reference thiếu không mutation; copy/delete không hồi sinh geometry.
Status: structural/preservation có code/proof; generic owner geometry edit và independent modeling normalization chưa hoàn tất.

## Phase 5 — Geometry còn lại, version và bản cài thống nhất

P5.1 Enumerate applicable geometry theo capability catalog: Mesh/PointCloud, SubD, PointGrid, NurbsCage/MorphControl, OffsetSurface/concrete surface owners, Hatch/TextDot và remaining concrete geometry owners.
Abstract/helper/proxy không serialize độc lập có applicability ghi rõ; clipping/layout/annotation dùng cho bản vẽ kỹ thuật là ngoài baseline dựng 3D, ghi excluded-by-scope chứ không giả vờ supported.
SubD giữ native cage/limit representation; display mesh không là NURBS edit. Không đòi exact SubD→NURBS cho mọi extraordinary region.
P5.2 Version-target adapters/gates; không ép mọi geometry mới thành V5. Native newer-target support phải có native+host+actual target proof trước khi mở lựa chọn.
P5.3 Tích hợp source chính và packaging cùng matching FreeCAD patches/import workers; full regression trên binary cuối, effective capability inventory và performance.

Gate: mỗi geometry áp dụng trong phạm vi cuối có operation/version evidence hoặc giới hạn tương thích đã chứng minh. Adapter còn thiếu trong promised scope không được tính là phase hoàn tất.
Status: cloud/hatch/dot slices đã có; SubD/cage/morph và final combined runtime còn thiếu.

## Cách nghiệm thu và báo tiến độ

Mỗi task đi theo failing regression → xác nhận nguyên nhân → thay đổi nhỏ → targeted checks → reviewer/self-review → evidence.
Mỗi phase cần requirement IDs, source hash, binary hash, fixture paths/hashes, measurements và lỗi/giới hạn; không đếm assertions thành phần trăm support.
Không sửa execution ledger full-exchange cũ vì kế hoạch này không phải measurement attempt. Đây là backlog workflow mới, không phải prepared Rhino batch.
Nếu thực hiện, cập nhật cả ledger tương ứng và summary phase; tổng future batches vẫn unknown khi ma trận chưa liệt kê hết.
Chỉ tích hợp source/binary khi phase proof đủ; không ghi đè toàn checkout, không publish trong nhiệm vụ này.

## Review thiết kế

- User intent: dựng tiếp CAD; file3DM trước; phân loại mesh/points ở Phase2; không history/render.
- Mỗi phase có deliverable, dependencies và gate; Phase3 có workflow cụ thể thay cho class-count.
- Chưa thay source runtime, policy preservation hay skill; cần review tài liệu trước execution.

