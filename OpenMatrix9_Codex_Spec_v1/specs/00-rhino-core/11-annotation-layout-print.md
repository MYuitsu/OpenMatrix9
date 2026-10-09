---
id: RCORE-11
title: Annotation, Make2D, layout/detail và in
priority: P1/P2
review_date: '2026-10-09'
evidence_level: source_inspected
runtime_validation: not_run
freecad_commit: 21d36cfa1eb110a1d0667050ff31706298805bbd
openmatrix9_commit: 52e887ab706dcd5d80e23979fb1f1b62d8d869b0
---

# RCORE-11 — Annotation, Make2D, layout/detail và in

## Phạm vi và mức bằng chứng

Chi tiết hóa [nhóm nền Rhino](README.md).
P1 gồm dimension, styles, text/leader/dot và Make2D; P2 gồm page/detail/print nâng cao.
Giữ các exact ID `OM9-MEASURE-004..017`, `OM9-UTIL-008`, `OM9-RENDER-024`, `OM9-FILE-009`.
[Layout Tools](../13-render/om9-render-024-layout-tools.md) đã có spec; không ghi là thiếu cả nhóm.
[Make2D](../01-core/om9-util-008-make-2d-drawing.md) đã phân biệt output Current CPlane.
Source đã kiểm: Draft annotation/dimension, TechDraw page/view/dimension/HLR/print và OM9 liên quan.
Không chạy ứng dụng, đo PDF hoặc in thử; các capability là source-inspected ngày 2026-10-09.
Working tree có thay đổi; không suy executable/runtime từ commit hoặc sample planner đã biên dịch.

Rhino 5 User's Guide PDF 98–100 / trang in 90–92, được ghi trong
[audit nguồn](../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md), tách dimension khỏi constraint:
dimension chỉ cập nhật theo geometry khi có History, và sửa số dimension không điều khiển geometry.
Guide mô tả Dot giữ kích thước màn hình; text model/page có hệ kích thước khác.
[Make2D help](https://docs.mcneel.com/rhino/5/help/en-us/commands/make2d.htm)
quy định Current CPlane đặt kết quả trên CPlane của viewport, còn 4-View dùng world orthographic.
[Detail help](https://docs.mcneel.com/rhino/5/help/en-us/commands/detail.htm)
phân biệt scale layout:model và khóa Detail ngăn pan/zoom.
Các mô tả Rhino này không tự chứng minh adapter FreeCAD có cùng hành vi.

## Hợp đồng annotation và dimension

Input gồm reference object/subelement hoặc tọa độ snapshot, placement/frame và dimension kind.
Dimension kind tối thiểu: horizontal, vertical, aligned, rotated, radius, diameter và angle.
Mỗi kind công bố cardinality, loại curve/edge được đo, degeneracy và true/projected measurement.
Output gồm object annotation, measured value, displayed text và style identity/version.
Reference và số đo thực phải giữ riêng với text override; override không sửa geometry.
Snapshot dimension giữ vị trí/giá trị theo policy; associative dimension lưu dependency rõ.
Chỉ bật History cho family có resolver và fixture; mất reference phải stale/unresolved rõ.
Annotation không tự thêm Sketcher constraint hoặc đổi tham số model khi người dùng sửa text.

| Nhóm dữ liệu | Fields và điều kiện |
|---|---|
| Geometry/reference | Identity, subelement, anchor points, measurement plane và revision. |
| Measurement | True/projected, length/angle type, source units và displayed unit conversion. |
| Style | Font, text/arrow size, line/extension offsets, alignment, precision, prefix/suffix. |
| Space | Model/page/screen; conversion có page/detail scale, không dựa vào zoom hiện hành. |
| Text | Unicode, line breaks, override marker và fallback font đã resolve. |
| Persistence | Stable style ID, local overrides, association policy, resource references và schema version. |

Style đổi phải cập nhật đối tượng đang dùng style theo policy; local overrides không bị mất ngầm.
Font thiếu phải báo fallback đã chọn; đo text/print dùng cùng font hoặc ghi rõ khác biệt.
Precision chỉ đổi cách hiển thị; không đổi phép đo, topology hoặc document tolerance.
Leader là text/arrow/anchor có contract riêng; không dùng đường CAD tùy ý để giả mọi leader.
Dot luôn hướng nhìn và giữ cỡ màn hình theo Rhino target; cần contract pick/export riêng.
Annotation page không bị scale hai lần khi nằm trong detail đã có model:paper scale.

## Hợp đồng Make2D

Input là selection supported geometry, frozen camera/CPlane, option set và tolerance context.
Phân biệt Current View, Current CPlane, 4-View USA và 4-View Europe bằng enum có nghĩa.
Current View lấy camera đã chốt; Current CPlane dùng plan view và trả geometry đúng frame đó.
4-View dùng world axes; USA/Europe khác bố trí projection, không dùng CPlane tùy chỉnh ngầm.
Output là curves có classification visible/hidden/tangent/silhouette và layer mapping rõ.
Maintain Source Layers không được suy thành giữ nguyên mọi source UUID/layer structure.
Viewport rectangle chỉ áp dụng khi projection/options cho phép; clipping lines có capability riêng.
Phân biệt HLR chính xác và polygon HLR; backend xấp xỉ cần khai báo tolerance/giới hạn.
Mesh unsupported ở Rhino 5 Make2D target phải bị từ chối hoặc là extension OM9 ghi riêng.
Hai vật thể giao nhau không tự suy thành đường giao nếu solver contract không tính intersection.
Degenerate projection, near-overlap và classification ambiguity phải được báo; không xóa lặng curves.
Tất cả view được stage trước một commit; Cancel không để nửa bộ drawing hoặc layers rỗng.
Đổi shaded/wireframe của input không được đổi projected geometry.

## Hợp đồng layout/detail/print

Page model lưu name, width/height, page units, orientation, margins và template identity.
Page content phân loại live detail, drawing view, image, frame, background và annotation.
Live detail lưu camera, projection type, clipping, viewport bounds, scale và lock state.
Ảnh chụp render là image có pixel resolution; không được gọi là live detail có scale hình học.
Với Detail parallel, scale là tỷ số độ dài paper/model sau unit conversion; positive finite.
Detail perspective không có numeric scale này: wrapper trả ratio 0 và từ chối SetScale.
UI phải phân biệt projection trước khi cho nhập scale; không lấy scale kỹ thuật từ độ zoom.
Lock detail chặn pan/zoom theo target; khóa vị trí page item là thao tác độc lập.
Ẩn object/layer trong detail phải có phạm vi detail riêng, không sửa visibility toàn model ngầm.
Print width/linetype/page units tách khỏi screen line width và tessellation density.
Preview in và PDF phải dùng cùng page transform; Fit-to-page là option explicit, không mặc định ngầm.
Tạo/sửa/xóa page/detail/style là document transaction; view navigation có lifecycle riêng.
Undo/Redo và FCStd reload giữ references, scale, lock, units và image placement.
Lỗi font, image thiếu, paper size/printer không khả dụng phải báo trước hoặc cùng output diagnostic.
Xuất PDF/file lỗi phải giữ file đích tốt; hủy print/export không đổi geometry model.

## Chi tiết bổ sung từ code decompile

Các chi tiết dưới đây được đọc từ code decompile; chưa chạy binary, UI hoặc fixture runtime.
Phân biệt logic managed, wrapper gọi native và phần triển khai native khi xác định phạm vi đã hiểu.
Các phát hiện này không nâng trạng thái triển khai hoặc đóng nghiệm thu FreeCAD/OM9.

| Hành vi và hệ quả hợp đồng | Giới hạn |
|---|---|
| DetailView tách IsProjectionLocked khỏi loại projection. PageToModelRatio trả 0 nếu không parallel; SetScale trả false trước khi gọi native nếu không parallel và nhận cả model length/units lẫn page length/units. Numeric scale chuẩn kỹ thuật chỉ áp dụng Detail parallel, không suy từ zoom perspective. | Điều kiện parallel nằm trong managed body; kiểm giá trị độ dài/units và thực thi projection lock ở native chưa được chạy. |
| DetailViewObject giữ Viewport riêng; CommitViewportChanges chỉ báo thành công khi native trả serial mới, rồi cập nhật serial/cache. RhinoPageView.SetPageAsActive tắt trạng thái active của các Detail; SetActiveDetail(Guid) chọn đúng ID. PageWidth/Height dùng PageUnitSystem của document. Chuyển context page/detail, commit camera và kích thước giấy là ba thao tác khác nhau. | Không đồng nhất runtime serial với UUID bền vững; chưa kiểm Undo/Redo, pan/zoom lock hoặc save/reload native. |
| Matrix LayoutToolsForm khởi tạo checkbox Detail=true. Handler thumbnail chỉ làm việc trong page view: chọn Detail thì thay/copy viewport projection và commit; bỏ Detail thì tạo hoặc thay ảnh capture 1600×1200. Nhánh tạo live Detail dùng rectangle ban đầu 100×80 theo hệ tọa độ page rồi copy projection nguồn. Nhánh ảnh nhúng bitmap table và material lên PageSpace surface, nên metadata camera của ảnh không biến ảnh thành live Detail. | Đây là default của form và giá trị của handler cụ thể, không phải paper size, scale, resolution mọi lệnh hay default toàn Rhino. Không đọc/copy bitmap resources; chưa chạy UI/renderer. |

Các yêu cầu về schema state/identity, worker, transaction và xử lý lỗi của OM9 vẫn là quyết định
thiết kế, trừ hành vi gốc được nêu rõ ở trên. Không suy defaults toàn ứng dụng từ một caller.

## FreeCAD đã có đến đâu; OM9 cần bổ sung gì

Các nhãn chỉ nguồn đã rà; không suy trạng thái mọi add-on, driver in hoặc plugin bên ngoài.

| Capability | Yêu cầu | FreeCAD | Bằng chứng/phạm vi | OM9 riêng |
|---|---|---|---|---|
| RCORE-11.C01 | Model dimensions | UI+API | [make_dimension.py](../../../../../src/Mod/Draft/draftmake/make_dimension.py) có linear/radial/angular; [gui_dimensions.py](../../../../../src/Mod/Draft/draftguitools/gui_dimensions.py) có GUI. | Cần mapping exact kind/selection/options của MEASURE specs. |
| RCORE-11.C02 | Associative/projected dimensions | UI+API | [DrawViewDimension.cpp](../../../../../src/Mod/TechDraw/App/DrawViewDimension.cpp): `References2D/3D`, `MeasureType`, `FormatSpec`; [CommandCreateDims.cpp](../../../../../src/Mod/TechDraw/Gui/CommandCreateDims.cpp): dimension commands. | Phải tách snapshot và History, không mặc định mọi dimension associative. |
| RCORE-11.C03 | Annotation styles | UI+API | [gui_annotationstyleeditor.py](../../../../../src/Mod/Draft/draftguitools/gui_annotationstyleeditor.py): style metadata/editor. | Cần schema units/font/scale và style IDs chung với page dimension. |
| RCORE-11.C04 | Text và leader | API | [make_text.py](../../../../../src/Mod/Draft/draftmake/make_text.py), [make_label.py](../../../../../src/Mod/Draft/draftmake/make_label.py) có text/leader data. | Adapter phải giữ font fallback, text override và history policy. |
| RCORE-11.C05 | Dot cỡ màn hình đúng Rhino | Chưa kiểm chứng | Draft có text/screen option; chưa đối chiếu đủ Dot size/pick/color/persistence. | Cần contract riêng, không bật từ tên Text/Label. |
| RCORE-11.C06 | HLR/projection geometry | API | [GeometryObject.cpp](../../../../../src/Mod/TechDraw/App/GeometryObject.cpp): `projectShape`, polygon HLR; [ProjectionAlgos.cpp](../../../../../src/Mod/TechDraw/App/ProjectionAlgos.cpp). | Dùng được làm solver candidate; cần frame/layer/option wrapper Make2D. |
| RCORE-11.C07 | Trang, template và scale | UI+API | [DrawPage.cpp](../../../../../src/Mod/TechDraw/App/DrawPage.cpp): `Template`, `Scale`, `KeepUpdated`; [DrawView.cpp](../../../../../src/Mod/TechDraw/App/DrawView.cpp): ScaleType; [Command.cpp](../../../../../src/Mod/TechDraw/Gui/Command.cpp): PageDefault/PageTemplate. | Layout Tools đã có spec, adapter runtime riêng chưa được xác minh. |
| RCORE-11.C08 | Camera/projection/hidden edges trên drawing view | API | [DrawViewPart.cpp](../../../../../src/Mod/TechDraw/App/DrawViewPart.cpp): Direction, XDirection, Perspective, Smooth/Seam/HardHidden. | Không đồng nhất drawing projection với live Rhino Detail. |
| RCORE-11.C09 | Khóa pan/zoom live Detail | Một phần | `DrawView::LockPosition` khóa page position; nguồn này không chứng minh khóa camera tương đương Rhino Detail. | Cần detail interaction state/scale lock riêng và nghiệm thu. |
| RCORE-11.C10 | PDF/print theo paper size | UI+API | [PagePrinter.cpp](../../../../../src/Mod/TechDraw/Gui/PagePrinter.cpp): `makePageLayout`, `printAll`, `printAllPdf`; [MDIViewPage.cpp](../../../../../src/Mod/TechDraw/Gui/MDIViewPage.cpp): print UI. | Cần kiểm 20 mm tại scale 2:1 cho output 40 mm và driver thực. |
| RCORE-11.C11 | Layout image/reference resources | Một phần | Host image objects và drawing page có nền riêng. | [CorePictureFrame.cpp](../../../Gui/CorePictureFrame.cpp) tạo ImagePlane; chưa chứng minh full layout resources. |
| RCORE-11.C12 | Roundtrip Rhino annotations/layout | Chưa kiểm chứng | Native FCStd/TechDraw serialization không là 3DM annotation writer. | 3DM retained records không chứng minh editable/exported layout. |

## Đọc source và giới hạn kết luận

`References2D/3D` cho phép tham chiếu geometry; cần kiểm topology change bằng fixture riêng.
`DrawViewPart` có Perspective/Direction không có nghĩa mọi Make2D option đã được route đúng.
`GeometryObject::projectShape` dùng OCCT HLR, còn polygon path là một representation khác.
`DrawView::LockPosition` phải được phân biệt với Detail Lock trong mô tả UX và serialization.
`PagePrinter::getPaperAttributes` lấy template width/height; code dùng page size millimeter.
Sự có mặt QPdfWriter không thay phép đo tọa độ/vector của PDF output.
[Layout Tools spec](../13-render/om9-render-024-layout-tools.md) có live Detail và VRay static trong hợp đồng.
Đây là mục tiêu feature; chưa có bằng chứng adapter V-Ray hoặc live Detail chạy ở checkout này.

## Kế hoạch adapter Rust trước

Rust sở hữu annotation/style/page/detail schema, unit conversion, reference validation và command state.
Rust lập kế hoạch HLR projection frames, layer classification, page layout và print transform.
C++ bridge gọi OCCT/TechDraw cho HLR/dimension measurement và App transaction khi cần native API.
Qt bridge dựng widget/view/printing; Rust không giữ QGraphicsItem/DocumentObject raw pointer.
Backend Draft Python hiện có chỉ dùng khi API đó bắt buộc; không đặt business policy mới trong Python.
Mọi native exception chuyển sang typed error; stage curves/page items trước commit GUI thread.
Worker nhận source snapshots; page/model revision đổi thì không được publish drawing cũ như mới.
Style/template import phải validate schema và missing resource trước tạo object hàng loạt.
Phụ thuộc RCORE-02 units, RCORE-03 frames, RCORE-06 layers/selection, RCORE-07 projection geometry.
Liên kết History dùng [RCORE-09](09-history-dependencies.md); file/render dùng RCORE-10 và RCORE-12.

## Fixtures nghiệm thu bắt buộc — chưa chạy

| Fixture | Thao tác | Expected invariant |
|---|---|---|
| RCORE-11.T01 — dimension policy | Đo line 20 mm thành snapshot và associative, đổi line thành 25 mm. | Snapshot giữ policy cũ; associative thành 25 mm; text override không sửa model. |
| RCORE-11.T02 — styles/units | Hai style khác precision và font; đổi display mm/inch. | Giá trị vật lý không đổi; arrow/text size theo đúng model/page space; fallback báo rõ. |
| RCORE-11.T03 — Dot zoom | Tạo Dot và model Text cạnh nhau; zoom 10×, xoay camera. | Dot giữ pixel size/hướng nhìn; Text tuân theo space; pick identity không lẫn nhau. |
| RCORE-11.T04 — Current CPlane | CPlane dịch (10,20,30), xoay trục; Make2D rectangle. | Curves nằm trên CPlane đã chốt; không ép về world XY; geometry độc lập display mode. |
| RCORE-11.T05 — HLR options | Cube/cylinder có hidden/tangent/seam, đổi USA/Europe. | Các class/layers đúng option; bố trí view đổi đúng, model geometry không đổi. |
| RCORE-11.T06 — print scale | Line 20 mm trên Detail parallel scale paper:model 2:1; xuất PDF. | Đo vector trên trang được 40 mm trong tolerance print đã định; zoom không ảnh hưởng. |
| RCORE-11.T07 — detail lock | Lock camera, thử pan/zoom; riêng thao tác move page item. | Camera/scale giữ nguyên; page position theo policy riêng; reload giữ lock. |
| RCORE-11.T08 — live vs image | Sửa model sau khi tạo live view và render image trong page. | Live view đổi theo policy; image giữ snapshot rõ, không tự đổi label thành live. |
| RCORE-11.T09 — lifecycle | Cancel Make2D nhiều views, xóa parent dimension, Undo/Redo, cold FCStd. | Không partial curves/layers; reference stale rõ; Undo/Redo/reload giữ page/style/identity. |
| RCORE-11.T10 — output errors | Font/image thiếu, disk-full hoặc printer mất kết nối. | Error có resource/output đích; không file hỏng thay file tốt hoặc model mutation ngoài yêu cầu. |
| RCORE-11.T11 — perspective scale | Thử nhập numeric scale cho Detail perspective rồi chuyển parallel. | Perspective không nhận tỷ lệ kỹ thuật giả; parallel dùng cả model/page units, scale hợp lệ và commit camera riêng. |
| RCORE-11.T12 — Layout Detail toggle | Chọn thumbnail với Detail bật/tắt, thêm mới và thay page item đang chọn. | Live Detail giữ liên hệ view; ảnh là snapshot/material nhúng; default/size caller không bị quảng bá thành paper/scale mặc định chung. |

## Unknowns và điều kiện đóng

Chưa kiểm mapping font metrics giữa viewport/Qt/PDF và printer driver thực tế.
Chưa chứng minh Rhino Detail interaction bằng TechDraw drawing views hoặc OM9 workspace views.
Chưa xác định mọi annotation kind và clipping classification đã có native adapter OM9.
3DM layout/dimension style preservation phải có matrix riêng, không suy từ FCStd reload.
Đóng capability theo fixture geometry/state/print đã chạy; sample Rust request validation không thay runtime.
