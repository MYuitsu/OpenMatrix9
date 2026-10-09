---
id: RCORE-08
title: Phân tích, đo chất lượng và sửa lỗi hình học
priority: P1
date: 2026-10-09
evidence_level: source_inspected
runtime_validation: not_run
---

# RCORE-08 — Phân tích và sửa lỗi hình học

## Mục tiêu và phạm vi

Cho người dùng biết checker đã kiểm gì, lỗi ở đâu và repair sẽ đổi dữ liệu nào trước khi sửa model.
Phạm vi gồm `OM9-ANALYSIS`, `OM9-UTIL-001..007/013`, `OM9-RENDER-018`, `OM9-TOOLS-018..021` và metal weight.
Spec cũ đã có Check/SelBadObjects/ShowEdges/Dir/EMap; không thêm EMap trùng `OM9-RENDER-018`.
CurvatureGraph/CurvatureAnalysis/Zebra/DraftAngleAnalysis/RebuildEdges/Audit3dmFile cần contract độc lập trước khi đăng ký executable alias.
Trạng thái **source-inspected / not-runtime-validated**; chưa chạy checker, renderer, repair hoặc Rhino fixture.
FreeCAD HEAD `21d36cfa1eb110a1d0667050ff31706298805bbd`; OM9 HEAD `52e887ab706dcd5d80e23979fb1f1b62d8d869b0`.
Checkout có thay đổi làm việc; các SHA không thay baseline/hash source ở [chỉ mục nhóm](README.md).
Nguồn Rhino User's Guide PDF 93–96 / trang in 85–88 đã được đối chiếu trong [audit](../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md).

## Hợp đồng chẩn đoán chỉ đọc

`AnalysisRequest = {document_id, sources_with_revisions, checks, tolerance_context, frame, display_options, budget}`.
`CheckResult = {check_id, status, severity, object_id, subelements, measured_values, units, thresholds, explanation}`.
`AnalysisReport = {schema_version, backend_version, context_revision, checks_run, checks_skipped, results, coverage, elapsed, cancelled}`.
`status` phải phân biệt `valid`, `invalid`, `unknown`, `not_applicable`, `not_run` và `cancelled`; không ép tất cả thành bool.
Không lỗi cấu trúc không đồng nghĩa không self-intersection, solid kín, in 3D được hoặc có wall thickness đủ.
Diagnostic không heal, xóa, đổi tolerance topology hay thay display mesh làm master geometry.
Highlight phải theo object/subelement revision; source đổi khiến report stale và phải bỏ marker cũ hoặc re-evaluate.
SelBadObjects lấy selection từ report của bộ checks chỉ định; không đổi geometry và không gọi mọi `unknown` là bad.
Budget exhausted hoặc native exception phải báo phần đã kiểm và phần chưa kiểm; không chuyển thành pass.
Với chuỗi checks theo RhinoCommon, kiểm topology trước geometry, rồi tolerances/flags; preconditions được API doc nêu nhưng wrapper không tự gọi/gate checks trước đó.
Nếu precondition fail, check phụ thuộc ghi `not_run` hoặc trạng thái blocked có nguyên nhân; giữ raw diagnostic phục vụ kỹ thuật và thông báo UI có cấu trúc riêng.

| Check | Input bắt buộc | Output và giới hạn phải ghi |
|---|---|---|
| Structural validity | BRep/mesh typed, revision | Loại lỗi, subelement, backend/check mode; không chứng nhận mọi điều kiện sản xuất |
| Naked/non-manifold edges | Edge-face usage và orientation | Danh sách biên/adjacency; seam/internal edges có policy riêng, không chỉ đếm edge uniques |
| Closedness | Shape type và topology | Closed theo wire/shell/solid được phân biệt; thêm orientation/self-intersection nếu cần volume |
| Direction/orientation | Curve/face/solid và world placement | Curve tangent, UV directions, face normal/orientation; reverse là mutation riêng |
| Self-intersection | BRep/mesh và mode kiểm | Cặp subelements/contact type, tolerance; không gọi `isValid` cơ bản là đã kiểm toàn bộ |
| Deviation/continuity | Hai sources/domains có mapping | G0/G1/G2 riêng, units, tolerance, sample/bound và uncovered regions |
| Manufacturability | Công nghệ, vật liệu, min wall/clearance và quy tắc được chọn | Report theo profile sản xuất; không suy từ validity hoặc mesh watertight |
| Area/volume/mass | Geometry type, closed/oriented status, density có dimensions | Đại lượng, phương pháp, units/error estimate; shell hở không được mặc nhiên cho metal mass đáng tin |

## Visual analysis, curvature và draft

Zebra dùng stripe map để hỗ trợ quan sát continuity; mật độ analysis mesh ảnh hưởng chi tiết nhìn thấy.
OM9 phải công bố hướng/kích thước/màu stripe, camera dependence, mesh settings và nút tắt/restore display.
Zebra/EMap không thay phép đo số G0/G1/G2. [Rhino 5 Zebra](https://docs.mcneel.com/rhino/5/help/en-us/commands/zebra.htm) (truy cập 2026-10-09).

CurvatureGraph cần chọn curve, miền tham số, sampling rule, comb scale và hướng curvature vector.
Zero curvature, cusp, undefined tangent hoặc discontinuity phải được thể hiện khác với giá trị số bình thường.
CurvatureAnalysis cần chọn đại lượng, signed/absolute convention, manual/auto range và đơn vị legend.
Principal/mean curvature có đơn vị `1/L`; Gaussian có `1/L²`; radius có `L` và xử lý infinity khi curvature bằng 0.
Phải phân biệt giá trị evaluate từ NURBS với ước lượng trên mesh vertex; mesh refinement không làm biến đổi BRep.
Trên face đã trim, chỉ tô miền trim; không vẽ surface nền qua lỗ rồi gọi đó là phân tích face.
Rhino có các style Gaussian/Mean và radius; mapping màu/range là một phần workflow, không chỉ một hàm curvature. [Rhino 5 CurvatureAnalysis](https://docs.mcneel.com/rhino/5/help/en-us/commands/curvatureanalysis.htm).

DraftAngleAnalysis cần lưu pull direction world lấy từ CPlane Z ở viewport khi bắt đầu theo target Rhino 5.
Mặt vuông góc CPlane có draft 0°, mặt song song CPlane có độ lớn draft 90°; sign dựa normal convention được công bố.
Đổi CPlane giữa phiên không âm thầm thay direction snapshot; muốn đổi phải explicit và chạy lại.
PartDesign Draft là phép dựng hình thay mặt, không chứng minh có DraftAngleAnalysis chỉ đọc. [Rhino 5 DraftAngleAnalysis](https://docs.mcneel.com/rhino/5/help/en-us/commands/draftangleanalysis.htm).

`VisualAnalysisState = {mode, source_revisions, pull_direction?, scalar_kind?, range, palette, mesh_generation, overlay_owner}`.
Tắt tool/cancel/đóng document phải bỏ overlay và khôi phục display theo RCORE-12, không reset material gốc.
Cache theo geometry revision, transform/frame và mesh settings; không reuse curvature map của shape trước sửa.

## Repair là operation có kiểm soát

`RepairPlan = {source_revision, operations, tolerances, allowed_geometry_changes, deviation_limit, topology_expectations, keep_original}`.
`RepairResult = {before_report, after_report, changed_subelements, deviation_report, topology_map, warnings, commit_status}`.
Quy trình: snapshot/clone → chẩn đoán → chọn repair explicit → preview → kiểm lại → commit atomic hoặc discard.
Một repair có thể sửa topology mà giữ locus, hoặc đổi curve/surface/mesh; phải nói rõ nhánh nào trước khi commit.
Không gọi `fix()` trực tiếp trên live shared shape mà chưa hiểu copy/lifetime và biến thể API đang dùng.
FreeCAD có cả `fix()` và `fix(precision,min,max)` với đường thực thi khác nhau; không giả định cả hai có cùng copy protection.
Đo deviation trước/sau theo RCORE-07; sample-only report phải ghi hạn chế, không gọi global maximum đã chứng minh.
Sau repair phải kiểm lại validity, orientation, closedness và các checks liên quan; một check được cải thiện không che lỗi mới.
Nếu vượt allowed deviation, mất lỗ, đổi số components ngoài contract hoặc native fail: giữ nguyên source và không partial commit.

RebuildEdges cần sửa tính nhất quán edge 3D với surface/trim, giữ output có tolerance và report.
Plan phải nêu phạm vi shared edges và có cập nhật vertex hay không: `BrepFace.RebuildEdges` có hai option riêng cho các quyết định này.
Không ghi default cho chúng từ chữ ký; thay mặt nền cũng không tự chứng minh edge đã được rebuild hoặc surface cũ đã được dọn.
Không được xem sewing, refine shape, bỏ splitter hay tăng topology tolerance là equivalent tự động.
Rhino mô tả lệnh khôi phục edge 3D đã bị kéo lệch khỏi surface và có tolerance override. [Rhino 5 RebuildEdges](https://docs.mcneel.com/rhino/5/help/en-us/commands/rebuildedges.htm).
Audit3dmFile là kiểm archive/file-level; report record/table/reference corruption tách khỏi checker BRep sau import.
Khả năng đọc file thành công không chứng minh archive đã audit đầy đủ; không viết lại nguồn file trong diagnostic chỉ đọc.
Mesh repair có contract riêng cho orientation, duplicate vertices/faces, degeneracy, non-manifold, self-intersection và holes.
Weld/fill/delete facet có thể đổi topology và volume; metadata/material/UV mất phải được báo nếu nằm trong supported scope.

## Ma trận capability theo checkout

| ID | Capability cần có | FreeCAD | Bằng chứng và giới hạn | OM9 riêng cần làm/đối soát |
|---|---|---|---|---|
| RCORE-08.C01 | Check BRep structural validity | UI+API | `Part_CheckGeometry`, `TaskCheckGeometry`, `TopoShape::isValid/analyze`; S1/S2 | Chuẩn hóa report/selection/highlight, checks coverage |
| RCORE-08.C02 | BOP/self-intersection diagnostic | UI+API | BOP analyzer có mode; GUI chỉ chạy sau BRep pass và `RunBOPCheck`; S1/S2 | Ghi mode thực dùng; không nói mọi Check đã chạy BOP |
| RCORE-08.C03 | Naked/non-manifold topology | Một phần | BRepCheck lỗi free-edge/multi-connexity và ancestor access; S1/S3 | ShowEdges filtering, seam/edge-use classification và marker lifecycle |
| RCORE-08.C04 | Curve/face direction và normals | API | `normalAt`, curve tangent/curvature và face orientation; S3/S4 | Dir overlay, frame và reverse transaction rõ |
| RCORE-08.C05 | Curve curvature graph | Một phần | Sketcher có B-spline curvature comb UI; curve API có curvature; S4/S5 | Mở rộng tới generic 3D curves; range/scale/sample contract |
| RCORE-08.C06 | NURBS surface curvature values | API | `GeometrySurfacePy::curvature` Max/Min/Mean/Gauss, directions; S4 | NURBS-trimmed map/legend/range và invalid-point handling |
| RCORE-08.C07 | Mesh curvature visualization | UI+API | Mesh curvature feature và view provider với Mean/Gaussian/Min/Max/Absolute modes; S6 | Báo mesh estimate, không gắn nhãn exact NURBS measurement |
| RCORE-08.C08 | EMap/environment reflection foundation | Một phần | `Std_TextureMapping` và `SoTextureCoordinateEnvironment`; S7 có UI execution | `OM9-RENDER-018` cần selection/lifecycle/options riêng, không thêm feature trùng |
| RCORE-08.C09 | Zebra stripe analysis chuyên biệt | Chưa thấy trong phạm vi rà | Tìm Zebra/reflection-line trong src Gui/Mod và shader/UI files; S7 chỉ là environment mapping foundation | Stripe generation, density, overlay và G0/G1/G2 numeric companion |
| RCORE-08.C10 | Draft-angle analysis chỉ đọc | Một phần | Normal evaluation S4 có; PartDesign Draft S8 là modeling, chưa thấy dedicated analysis trong phạm vi rà | Rust pull-direction/range/sign policy và display adapter |
| RCORE-08.C11 | BRep fixing/sewing | API | `TopoShape::fix`, tolerance overload, `sewShape`; S2 | Clone before repair, deviation/atomic commit; không tự heal trong Check |
| RCORE-08.C12 | RebuildEdges semantics Rhino | Một phần | ShapeFix và pcurve access S2/S3 là primitives; chưa chứng minh cùng repair behavior | Edge 3D–UV consistency operation và fixtures riêng |
| RCORE-08.C13 | Mesh Evaluate and Repair | UI+API | `Mesh_Evaluation`, dialog check/repair modes, `MeshPy` solid/self-intersection; S9 | Chọn subset supported, preserve input/metadata và report mutations |
| RCORE-08.C14 | Area/volume tích phân trên BRep | API | `BRepGProp::SurfaceProperties/VolumeProperties`; S10 | Precondition solid/orientation, density/units và sai số cho metal weight |
| RCORE-08.C15 | Audit3dmFile archive-level | Chưa kiểm chứng | File này không audit thư viện archive backend hoặc host importer đầy đủ | RCORE-10 phải thêm read-only archive audit contract/evidence |
| RCORE-08.C16 | Chứng nhận sản xuất/độ dày mọi loại model | Chưa kiểm chứng | Validity/curvature/mesh checks không đủ chứng minh profile sản xuất | Profile theo công nghệ, thresholds và fixtures nghiệp vụ riêng |

## Điểm vào source đã kiểm tra

- S1 — [Part Gui Command.cpp](../../../../../src/Mod/Part/Gui/Command.cpp), [TaskCheckGeometry.cpp](../../../../../src/Mod/Part/Gui/TaskCheckGeometry.cpp): `Part_CheckGeometry`, BRep check trước BOP, `RunBOPCheck`, SelfInter/SmallEdge/Continuity/Tangent/CurveOnSurface modes.
- S2 — [TopoShape.cpp](../../../../../src/Mod/Part/App/TopoShape.cpp), [TopoShapePyImp.cpp](../../../../../src/Mod/Part/App/TopoShapePyImp.cpp): `isValid`, `analyze`, Python `check(runBopCheck=False)`, sewing/fix; overload repair khác đường thực thi.
- S3 — [TopoShapeFacePyImp.cpp](../../../../../src/Mod/Part/App/TopoShapeFacePyImp.cpp), [TopoShapeEdgePyImp.cpp](../../../../../src/Mod/Part/App/TopoShapeEdgePyImp.cpp): `isPartOfDomain`, `normalAt`, `curvatureAt`, pcurve/seam; `ancestorsOfType` ở S2.
- S4 — [GeometryCurvePyImp.cpp](../../../../../src/Mod/Part/App/GeometryCurvePyImp.cpp), [GeometrySurfacePyImp.cpp](../../../../../src/Mod/Part/App/GeometrySurfacePyImp.cpp): curvature evaluation, principal directions và lỗi type không hợp lệ.
- S5 — [CommandSketcherOverlay.cpp](../../../../../src/Mod/Sketcher/Gui/CommandSketcherOverlay.cpp), [EditModeInformationOverlayCoinConverter.cpp](../../../../../src/Mod/Sketcher/Gui/EditModeInformationOverlayCoinConverter.cpp): B-spline curvature comb command và geometry của overlay.
- S6 — [FeatureMeshCurvature.cpp](../../../../../src/Mod/Mesh/App/FeatureMeshCurvature.cpp), [ViewProviderCurvature.cpp](../../../../../src/Mod/Mesh/Gui/ViewProviderCurvature.cpp): curvature property và display modes/info callback.
- S7 — [CommandView.cpp](../../../../../src/Gui/CommandView.cpp), [TextureMapping.cpp](../../../../../src/Gui/TextureMapping.cpp): `StdCmdTextureMapping::activated`, `TaskTextureMapping`, `onCheckEnvToggled`, Coin environment-coordinate node và cleanup.
- S8 — [FeatureDraft.cpp](../../../../../src/Mod/PartDesign/App/FeatureDraft.cpp): native `BRepOffsetAPI_DraftAngle`; đây là thay shape, không là bằng chứng false-color analysis.
- S9 — [Mesh Gui Command.cpp](../../../../../src/Mod/Mesh/Gui/Command.cpp), [DlgEvaluateMeshImp.cpp](../../../../../src/Mod/Mesh/Gui/DlgEvaluateMeshImp.cpp), [MeshPyImp.cpp](../../../../../src/Mod/Mesh/App/MeshPyImp.cpp): `Mesh_Evaluation`, check/analyze/repair callbacks, `isSolid/hasSelfIntersections/fixSelfIntersections`.
- S10 — [TopoShapePyImp.cpp](../../../../../src/Mod/Part/App/TopoShapePyImp.cpp): `getArea/getVolume` gọi BRepGProp; giá trị trả về không tự chứng nhận preconditions của metal weight.

## Chi tiết bổ sung từ code decompile

Phần này tách logic C# đọc được khỏi mô tả API đi kèm và suy luận từ mã giả native.
Nhánh wrapper xác nhận cách xử lý/tham số được chuyển tiếp, nhưng không tự chứng minh thuật toán kernel, default UI hoặc kết quả runtime.
Các chi tiết này giữ nguyên phân loại FreeCAD, supported slice OM9 và trạng thái `source_inspected` / `not_run`.

| Hành vi và yêu cầu OM9 | Giới hạn cần kiểm chứng |
|---|---|
| **`IsValidTopology`, `IsValidGeometry`, `IsValidTolerancesAndFlags`**. C# chuyển selector 0/1/2 vào cùng native bridge. Thân bridge mới xác nhận từng selector gọi riêng check topology, geometry hoặc tolerances/flags; null Brep hoặc selector khác trả false. Log được truyền khi caller cung cấp output string và được chép ra sau check. Bridge không tự chạy/gate hai check còn lại. Preconditions topology→geometry→tolerances/flags vẫn do caller orchestration giữ, với coverage từng check. | Có thân bridge không chứng minh toàn bộ thuật toán check, self-intersection hoặc mức sâu của kernel. Không kiểm runtime log/coverage trong lần đọc này. Log tiếng Anh không phải taxonomy UI ổn định, không parse chuỗi làm identity duy nhất. |
| **`BrepFace.ChangeSurface` và `RebuildEdges`**. C# lấy mutable Brep owner cho hai native calls riêng. Bridge xác nhận kiểm owner khác null và face index trong phạm vi; invalid face trả false. RebuildEdges chuyển tolerance cùng hai flags shared-edge/vertices riêng, chuẩn hóa byte flags thành bool trước gọi kernel. ChangeSurface kiểm face rồi chuyển surface index tới thao tác của face; không tự gọi RebuildEdges sau đó. API mô tả surface cũ cần cleanup riêng và edge consistency cần xử lý riêng. | Bridge không gán default flags hoặc chứng nhận tolerance/deviation, topology/neighbor preservation, transaction hay Undo. Giá trị tolerance được chuyển tiếp không có validation hữu hạn/dương tại bridge; validation sâu của kernel chưa được nghiệm thu. Cần preview clone và đo trước-sau. |
| **`DuplicateEdgeCurves` và `DuplicateNakedEdgeCurves`**. Logic C# đọc được: overload edge có `nakedOnly` và bật cả outer/inner; naked-edge overload cố định naked và chuyển hai flags outer/inner explicit. Diagnostic phải ghi filter đã chọn, giữ liên kết edge/subelement gốc khi tạo display curves. | Curve output không phải đầy đủ incidence map: chưa chứng minh phân biệt seam/non-manifold hay cách xử lý edge dùng nhiều lần chỉ từ output này. Không dùng số curve được duplicate để suy solid. |
| **Solid, orientation và manifold sau transform/repair**. Logic C# đọc được: solid bool bỏ chi tiết orientation của native result; orientation và manifold còn query riêng. Mô tả API đi kèm lưu ý geometry sau transform đảo hướng có thể cần flip. Report sau Mirror/repair phải giữ solid/orientation/manifold riêng và không chỉ kiểm transformed-success bool. | Wrapper transform không expose phép xác nhận closure hay tự kiểm validity. Không có chứng cứ `IsSolid` đồng nghĩa outward, positive physical volume hoặc self-intersection-free. |

## Kế hoạch adapter tối thiểu, Rust-first

1. Rust sở hữu check registry, report schema, severity/coverage, range/units và state của analysis/repair session.
2. Native đọc exact BRep/mesh snapshot, resolve subelements và thực thi checks OCCT/Mesh có modes explicit.
3. Native trả numeric samples/topology observations; Rust xử lý grouping, filters, palette/range và stable diagnostic IDs.
4. Qt/Coin adapter chỉ render markers/maps và nhận input; snapshot world frame/revision do Rust session quản lý.
5. Repair chạy trên clone độc lập; worker chỉ dùng geometry copy được phép, không đọc/mutate document tùy ý.
6. Rust xét before/after report và deviation policy; GUI commit shape/metadata/History trong một transaction.
7. Audit3DM dùng archive adapter của RCORE-10; không triển khai bằng cách import rồi gọi BRep check và đặt tên mới.

Lỗi dự kiến: `UnsupportedCheck`, `UndefinedCurvature`, `InvalidPullDirection`, `StaleAnalysis`, `BudgetExceeded`, `DeviationExceeded`, `NewDefectAfterRepair`, `ArchiveAuditUnavailable`, `KernelFailure`.
Unknown/undefined không được tô màu “an toàn”; báo vùng không có số đo và nguyên nhân ở legend/report.
Không mở rộng Python business logic mới chỉ vì GUI checker hiện gọi script; C++ chỉ giữ API native/renderer bắt buộc.

## Phụ thuộc

RCORE-01 cung cấp type/topology/ownership; RCORE-02 cung cấp units và tolerance thực dùng.
RCORE-03 cung cấp CPlane/pull-direction snapshot; RCORE-04/06 cung cấp cancellation/selection/lock.
RCORE-07 cung cấp deviation/continuity measure; RCORE-09 giữ revision/mapping sau repair.
RCORE-10 phục vụ archive audit; RCORE-12 quản lý analysis mesh/overlay cache và restore display.

## Acceptance fixtures — tất cả chưa chạy

| ID | Fixture và thao tác | Expected invariant / evidence |
|---|---|---|
| RCORE-08.T01 | Plane 20 × 20 mm có lỗ, chạy curvature map | Curvature bằng 0 trong numerical bound; lỗ không tô; infinity radius/undefined được thể hiện đúng |
| RCORE-08.T02 | Sphere R=10 mm với orientation đã ghi | Độ lớn principal/mean curvature `0.1 mm⁻¹`, Gaussian `0.01 mm⁻²`; sign theo convention; singular parameter points báo riêng |
| RCORE-08.T03 | Hai surface pairs G0-only/G1, bật Zebra rồi numerical check | Overlay hỗ trợ nhìn; report G0/G1/G2 lấy từ measurement và tolerance, không từ ảnh |
| RCORE-08.T04 | Shell hộp thiếu một face và fixture edge dùng bởi ba faces | Report naked/non-manifold tách biệt; không báo solid manufacturing-ready; seam không bị đếm sai |
| RCORE-08.T05 | BRep có lỗi self-intersection chỉ xuất hiện trong BOP checks | Báo `not_run` khi mode tắt; mode bật ghi lỗi/coverage, không sửa shape |
| RCORE-08.T06 | Cylinder đứng rồi CPlane nghiêng 30°, chạy draft | Pull direction lấy đúng CPlane Z snapshot; đổi camera không đổi số; đổi CPlane sau start không âm thầm đổi report |
| RCORE-08.T07 | Repair edge gap với deviation limit nhỏ hơn thay đổi cần thiết | Reject không partial mutation; report before/after và required deviation; Undo stack không có repair giả |
| RCORE-08.T08 | Mesh duplicate faces, flipped normal và self-intersection | Chỉ repairs đã chọn được áp; report topology/volume trước-sau; original giữ khi fail/cancel |
| RCORE-08.T09 | Solid cube cạnh 10 mm, density fixture 0.01 g/mm³ | Volume 1000 mm³ và mass 10 g theo bound; shell hở/đảo orientation phải được xử lý explicit |
| RCORE-08.T10 | Bật analysis rồi sửa source, cancel worker hoặc đóng document | Report/overlay cũ invalidated, không stale highlight/UAF; read-only diagnostics không tạo geometry/undo entry |
| RCORE-08.T11 | BRep có topology invalid, geometry invalid và tolerances/flags invalid được chuẩn bị riêng | Caller chỉ chạy check khi preconditions đạt; skipped/not_run giữ nguyên trong coverage, log kỹ thuật đi cùng user diagnostic; không gọi ba bool độc lập là một chứng nhận |
| RCORE-08.T12 | Hai faces shared edge; thay mặt nền rồi RebuildEdges với bốn tổ hợp shared/vertices flags | Đo edge 3D, vertices, neighbor face consistency, topology, surface count và deviation trước-sau; verify skip/modify theo API contract, không suy từ success flag |
| RCORE-08.T13 | Face có lỗ, seam và fixture non-manifold; đổi outer/inner/naked filter | Curves hiển thị đúng filter nhưng topology report dùng incidence riêng; giữ source identity và không suy closedness từ số curves |

## Giới hạn và câu hỏi cần thêm evidence

Có environment mapping nền trong FreeCAD; không được nói host hoàn toàn thiếu reflection vì không tìm thấy chuỗi “Zebra”.
Tìm exact Zebra/reflection-line/DraftAngleAnalysis/RebuildEdges/Audit3dmFile trong src Gui/Mod, gồm C++/Python/shader/UI, chưa thấy workflow chuyên biệt tương ứng.
Kết luận trên giới hạn checkout và phạm vi rà; không phủ định khả năng trong addon, thư viện phụ thuộc hoặc code mang tên khác chưa đọc.
Sketcher comb, Mesh curvature và Part NURBS evaluation là ba phạm vi khác nhau, cần giữ nhãn representation trong UI/report.
Chưa xác nhận all-platform renderer behavior, signed curvature conventions của mọi API, numerical bounds và cancellation của native analyzers.
Không chạy build/native tests vì chỉ viết tài liệu; các fixture là backlog nghiệm thu, không đánh dấu pass từ source presence.
