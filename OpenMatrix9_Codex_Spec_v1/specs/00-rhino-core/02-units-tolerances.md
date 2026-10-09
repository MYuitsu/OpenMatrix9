---
id: RCORE-02
title: Units, tolerance và độ chính xác
priority: P0
date: 2026-10-09
evidence_level: source_inspected
runtime_validation: not_run
evidence_scope: initial_source_audit_2026-10-09
---

# RCORE-02 — Units, tolerance và độ chính xác

## Cập nhật triển khai P0 — 2026-10-09

Metadata và nhận định nguồn phía dưới ghi baseline audit ban đầu. Context Rust
có version/revision, model/page scale, absolute/relative/angular tolerance đã nối
với native `UnitSettings`, transaction và FCStd. Line/Polyline/Circle dùng input
scale đã chụp; output lưu `OM9UnitContext`, context đổi làm phiên curve cũ hết hiệu
lực. Đổi khai báo theo physical size giữ geometry; yêu cầu scale toàn model chưa
được hỗ trợ bị từ chối trước mutation. Legacy document không được gán tolerance
chung bằng suy đoán; ngưỡng solver hiện hữu vẫn thuộc từng operation.

Native units **20 checks** và cold restore **3 checks** đã pass trong phạm vi
được ghi tại [báo cáo P0](../../../docs/validation/2026-10-09-rhino-core-p0.md).
Đây là slices, không đóng trọn RCORE-02.T01–T13.

**P0 còn thiếu:** propagation tới mọi command/History/clipboard; migration và
scale geometry/placement/length parameters đồng bộ theo lựa chọn declared
numbers; áp tolerance context xuyên mọi solver; đầy đủ locale/unit aliases,
layout/annotation policy và regression tại giới hạn số học lớn/nhỏ.

## Mục tiêu và phạm vi

Một command phải biết số đang dùng đơn vị nào, dung sai nào và context phiên bản nào trước khi giải hình học.
Phạm vi gồm `OM9-COORD`, `OM9-INFO-002`, `OM9-FILE-002/004`, `OM9-EDIT`, clipboard và mọi modeling/analysis operation.
Đây là mở rộng RCORE-02 cũ, không thay default/ngưỡng của các supported slices đã có.
Baseline audit nguồn ban đầu là **source-inspected / not-runtime-validated**; các ca mm/inch, Undo/Redo và save/reload chưa chạy trong audit đó.
FreeCAD HEAD: `21d36cfa1eb110a1d0667050ff31706298805bbd`; OM9 HEAD: `52e887ab706dcd5d80e23979fb1f1b62d8d869b0`.
Checkout có thay đổi làm việc; HEAD không thay hash nội dung source trong [baseline của nhóm](README.md).

Rhino 5 phân biệt model/layout units, absolute/relative/angle tolerance và precision hiển thị.
Đổi units có lựa chọn scale geometry; tolerance tương đối chỉ áp dụng ở những command có dùng nó.
Import có units không tự thay units/tolerance của document đích. [Rhino 5 Units](https://docs.mcneel.com/rhino/5/help/en-us/documentproperties/units.htm) (truy cập 2026-10-09).
Phần schema, migration và xử lý lỗi sau đây là quyết định OM9 được đề xuất, cần nghiệm thu riêng.
Không lấy default tutorial hoặc giá trị ví dụ để thay tolerance của toàn bộ workbench.

## Các đại lượng phải tách

| Trường | Ý nghĩa và đơn vị | Không được dùng thay |
|---|---|---|
| Model unit | Đơn vị khai báo/nhập/trao đổi của không gian model; kèm hệ số sang mm | Internal storage hoặc page unit |
| Page unit | Đơn vị vật lý trang/layout, liên hệ scale qua RCORE-11 | Đơn vị tính Boolean/Join |
| Host storage unit | Đơn vị cơ sở của giá trị native; FreeCAD dùng mm cho length và degree cho angle trong Base units | Đơn vị tùy chọn trên màn hình |
| Display precision | Số chữ số/cách biểu diễn, chỉ ảnh hưởng format | Ngưỡng hình học, topology hoặc snap |
| Absolute tolerance | Sai số độ dài cho operation đã nhận contract này; canonical mm | Mọi OCCT `Precision::*` một cách tự động |
| Relative tolerance | Tỷ lệ không thứ nguyên; phải có reference length và formula theo operation | Công thức chung áp cho tất cả solver |
| Angular tolerance | Ngưỡng góc; schema chọn radian nội bộ Rust, adapter đổi rõ sang degree/radian API yêu cầu | Tessellation angular deflection |
| Kernel topology tolerance | Tolerance gắn trên vertex/edge/face native; đọc/ghi có report | Document absolute tolerance hoặc quyền tự heal |
| Numerical epsilon | Sai số số học phụ thuộc thuật toán/scale/world coordinates | Yêu cầu thiết kế của người dùng |
| Minimum feature size | Miền hỗ trợ cho fillet/offset/chi tiết nhỏ của operation | Giá trị round lên trong UI |
| Pick radius | Khoảng cách màn hình theo pixel/DPI và view | Ngưỡng Join/intersection trong model units |
| Tessellation deflection | Sai lệch render mesh, linear/angular theo pipeline hiển thị | Chất lượng BRep gốc hoặc deviation sau repair |

Giá trị độ dài không gắn unit chỉ hợp lệ sau khi được resolve bằng input policy của command.
Độ dài, diện tích, thể tích phải chuyển theo lũy thừa 1/2/3; curvature theo `1/L`, Gaussian curvature theo `1/L²`.
Angle không scale theo model length; density phải giữ dimensions và đồng bộ với mass/volume.
Knot values/UV domains không mặc nhiên là millimeter và không được scale cùng poles chỉ vì đều là số thực.

## Context versioned và dữ liệu command

`UnitContext = {schema_version, document_id, revision, model_unit, custom_mm_per_unit?, page_unit}`.
`ToleranceContext = {revision, absolute_mm, relative_ratio, angular_rad, source, operation_overrides}`.
`DisplayFormat = {length_style, decimals, denominator?, locale}` có revision riêng hoặc được ghi rõ không tác động solver.
`OperationTolerance = {context_revision, inherited_fields, overrides, effective_values, application_rule_id}`.
`KernelToleranceReport = {min, max, distribution?, affected_subelements, units_mm}` chỉ là kết quả đọc native.
`UnitResolution = {source_units, target_units, scale_factor, mode, evidence, unresolved_reason?}`.
`UnitChangePlan = {old_context, new_context, numeric_scale, affected_objects, affected_parameters, metadata_changes}`.
Mọi floating point phải hữu hạn; scale dương; tolerance dương trong miền operation hỗ trợ; angle có miền riêng.
Không cho custom unit có hệ số bằng 0, âm, NaN hoặc vô hạn.
Điều kiện hữu hạn/dương là validation OM9; wrapper setter RhinoDoc chỉ chuyển giá trị vào native, không chứng minh cùng rejection policy.
Nếu adapter tiếp nhận sentinel kiểu Rhino `Curve.Fit`, phải resolve sentinel thành effective tolerance dương trước tạo request solver OM9; giữ cả giá trị gốc và nguồn context trong report.
Giá trị tỷ lệ được lưu như ratio, UI percent phải chuyển explicit; `1%` không được lưu thành `1.0` nếu schema hiểu ratio.

Feature/History lưu context revision và override thực sự dùng, không chỉ một số `tolerance` mất đơn vị.
Command bắt đầu bằng snapshot context; nếu context đổi giữa preview và commit phải revalidate/recompute hoặc từ chối stale.
Preview và commit cùng một effective context; không parse lại textbox bằng unit scheme mới ngay lúc commit.
Override phải thể hiện trong prompt/report và persistence; không âm thầm thay preference toàn cục của FreeCAD.
Các giá trị native đang hard-code cần kiểm từng operation trước migration; không mass-replace bằng một default mới.

## Đổi units và import

OM9 đề xuất hai mode rõ tên: `preserve_physical_size` và `preserve_declared_numbers`.
Trong host mm, đổi từ mm sang inch mà giữ kích thước vật lý giữ nguyên native geometry; chỉ số hiển thị/khai báo đổi.
Giữ số khai báo khiến native length scale theo `new_mm_per_unit / old_mm_per_unit`; phải preview phạm vi ảnh hưởng.
Đây là mapping OM9 cho host có storage cố định, không phải mô tả nội bộ lưu trữ Rhino.
Ví dụ đoạn 25.4 mm đổi khai báo sang inch: mode vật lý cho 1 inch; mode giữ số cho 25.4 inch, dài native 645.16 mm.
Plan phải xử lý poles, primitive dimensions, placements translation, layer-dependent metadata, annotation và history parameters.
Không scale unitless ratio, direction vector đã chuẩn hóa, angle hoặc camera orientation như length.
Nếu object/plugin parameter chưa có dimension metadata, plan phải báo unsupported thay vì scale bằng heuristic tên field.
Undo/Redo khôi phục context và mọi mutation liên quan trong cùng transaction.

New/template phải chỉ ra nguồn model units/tolerance: template, document, explicit user setting hoặc migration record.
Import khác units resolve source → canonical mm → document presentation, không nhân scale hai lần qua nested instance.
File unitless/custom phải có explicit mapping hoặc báo unresolved trước commit; không đoán inch từ bbox.
Clipboard mang units, current geometry và context provenance; paste không tái dùng raw số của source với units đích khác.
RCORE-10 quyết định preservation/export, nhưng giá trị effective scale phải do context này cung cấp.
Kernel thất bại không cho phép nới tolerance, heal, đóng lỗ hoặc đổi sang mesh rồi báo success.
Repair/tolerance mutation là operation explicit thuộc RCORE-08, có Undo và báo thay đổi geometry/topology.

## Ma trận capability theo checkout

| ID | Capability cần có | FreeCAD | Bằng chứng và giới hạn | OM9 riêng cần làm/đối soát |
|---|---|---|---|---|
| RCORE-02.C01 | Đơn vị có dimension và canonical values | API | `Base::Unit`, `Quantity`; mm/deg được khai báo ở S1/S2 | Rust typed quantities và conversion biên ABI |
| RCORE-02.C02 | Unit system trên document | API | `Document::UnitSystem`, khởi tạo từ preference `UserSchema`; S3 | Không gọi host là chỉ có global units; bổ sung context revision/schema |
| RCORE-02.C03 | Quantity input và quantity property | UI+API | `QuantitySpinBox`, `PropertyLength`, `PropertyAngle`; S4/S5 | Parser command/CPlane dùng cùng units policy, không chỉ widget |
| RCORE-02.C04 | Định dạng precision/fraction | API | `UnitsApi::setDecimals/getDecimals`, Quantity format; S2/S6 | Tách revision format khỏi geometric tolerance và bảo đảm không mutate shape |
| RCORE-02.C05 | Tolerance topology theo shape/subshape | API | `getTolerance`, `overTolerance`, `inTolerance`, `fixTolerance`; S7 | Report read-only riêng; mutation cần explicit repair contract |
| RCORE-02.C06 | GUI thay tolerance native | UI+API | `BOPTools.ToleranceFeatures`, `FeatureToleranceSet`; S8 | Không coi đây là Rhino document tolerance context |
| RCORE-02.C07 | Tham số tolerance cho sewing/offset/fix | API | `TopoShape::sewShape`, `makeOffsetShape`, `fix(precision,min,max)`; S9 | Mapping per operation, không áp relative formula chung |
| RCORE-02.C08 | Context absolute/relative/angular chung toàn OM9 | Chưa thấy trong phạm vi rà | Đã đọc Base units, Document, Part tolerance và OM9 surface/3DM paths; chưa thấy schema thống nhất | Rust context persisted, migration từng supported slice |
| RCORE-02.C09 | Đổi units kèm scale document có transaction | Chưa kiểm chứng | Property UnitSystem không chứng minh compound scale policy và Undo/History | Implement plan/apply và fixture trên objects có placements/History |
| RCORE-02.C10 | Refit có tolerance và numerical budget | Một phần | `GeomBSplineCurve::approximate` có tolerance và degree/segment limits; S12; chưa chứng minh bound toàn workflow | S10: Rust candidate và native `certifySurfaceRefit` có bound/budget; giữ semantics slice riêng |
| RCORE-02.C11 | 3DM scale/override | Chưa kiểm chứng | File này không audit importer 3DM độc lập của host FreeCAD | S11: `core_3dm::scale(unit_mm, override_mm)` có ở OM9; đối soát unitless/custom theo RCORE-10 |
| RCORE-02.C12 | Page units độc lập model units | Chưa kiểm chứng | File này không audit toàn bộ TechDraw/layout | Contract RCORE-11 phải có fixture tỷ lệ in, không lấy UnitSystem làm đủ |

## Điểm vào mã nguồn đã kiểm tra

- S1 — [Unit.h](../../../../../src/Base/Unit.h): `unitSymbols`, exponent dimensions, `Unit::Length/Angle`.
- S2 — [Quantity.h](../../../../../src/Base/Quantity.h), [Quantity.cpp](../../../../../src/Base/Quantity.cpp): quantity/format, `Degree` là internal standard angle, `Radian = 180/π` degree.
- S3 — [Document.h](../../../../../src/App/Document.h), [Document.cpp](../../../../../src/App/Document.cpp): `UnitSystem`, `ADD_PROPERTY_TYPE`, enum descriptions và default từ user preference.
- S4 — [QuantitySpinBox.cpp](../../../../../src/Gui/QuantitySpinBox.cpp): `commitQuantity`, parsing qua `App::QuantityInputGrammar`; UI input không là bằng chứng parser Rhino.
- S5 — [PropertyUnits.cpp](../../../../../src/App/PropertyUnits.cpp): các property có dimension và constraints.
- S6 — [UnitsApi.cpp](../../../../../src/Base/UnitsApi.cpp): `getBasicLengthUnit`, `setDecimals`, `getDecimals`, schema translation.
- S7 — [TopoShapePyImp.cpp](../../../../../src/Mod/Part/App/TopoShapePyImp.cpp): shape tolerance analysis và fixing API.
- S8 — [ToleranceFeatures.py](../../../../../src/Mod/Part/BOPTools/ToleranceFeatures.py): `cmdCreateToleranceSetFeature`, `FeatureToleranceSet`, native tolerance mutation.
- S9 — [TopoShape.h](../../../../../src/Mod/Part/App/TopoShape.h), [TopoShape.cpp](../../../../../src/Mod/Part/App/TopoShape.cpp): sewing/offset/fix có các tolerance riêng.
- S10 — [surface_refit.rs](../../../rust/src/surface_refit.rs), [SurfaceRefit.cpp](../../../Gui/SurfaceRefit.cpp), [SurfaceController.cpp](../../../Gui/SurfaceController.cpp): tolerance mm, candidate budget, chứng nhận native có giới hạn subdivision.
- S11 — [core_3dm.rs](../../../rust/src/core_3dm.rs): `scale`; đây là điểm vào policy, không bằng chứng roundtrip hoàn chỉnh.
- S12 — [Geometry.h](../../../../../src/Mod/Part/App/Geometry.h), [BSplineCurvePyImp.cpp](../../../../../src/Mod/Part/App/BSplineCurvePyImp.cpp): approximation với tolerance và giới hạn degree/segments của FreeCAD/OCCT.

## Chi tiết bổ sung từ code decompile

Phần này tách logic C# đọc được khỏi mô tả API đi kèm và suy luận từ mã giả native.
Nhánh wrapper xác nhận cách xử lý/tham số được chuyển tiếp, nhưng không tự chứng minh thuật toán kernel, default UI hoặc kết quả runtime.
Các chi tiết này giữ nguyên phân loại FreeCAD, supported slice OM9 và trạng thái `source_inspected` / `not_run`.

| Hành vi và yêu cầu OM9 | Giới hạn cần kiểm chứng |
|---|---|
| **Absolute, relative và angular tolerance trong `RhinoDoc`**. Absolute, angular và relative dùng selector riêng; property degrees đổi qua radians trong cả getter/setter. Bridge lấy document theo ID, đọc bộ units/tolerances của model hoặc page; setter sao chép bộ hiện tại, đổi đúng trường rồi gọi setter native với scale=false. OM9 giữ một angular effective value với conversion biên rõ, đồng thời phân biệt cập nhật tolerance với scale geometry. | Không thấy kiểm finite/range trong wrapper/bridge; các body này không thiết lập default document. Chưa chứng minh validation sâu hơn, công thức relative tolerance, command coverage hoặc behavior của document không còn hợp lệ. |
| **Unit setter so với `AdjustModelUnitSystem` / `AdjustPageUnitSystem`**. Bridge của cả hai đường sao chép units/tolerances hiện tại rồi đổi unit enum. Property setter luôn truyền scale=false; adjust chuyển tiếp lựa chọn scale của caller. Native model/page setters dùng unit scale factor và chỉ vào nhánh scale khi flag bật, factor khác 0 và khác 1. Nhánh model có transform objects, model basepoint và named views; page dùng luồng transform objects riêng. | Đây là xác nhận nhánh điều khiển, chưa chứng nhận mọi object/plugin, annotation/history, tolerance propagation hoặc Undo/rollback. Iterator/filter và callbacks còn cần nghiệm thu; không gọi đổi units là transaction atomic chỉ từ body này. Hai mode host mm ở trên vẫn là thiết kế OM9. |
| **Sentinel và kink threshold trong `Curve.Fit`**. Bridge chuyển degree và hai tolerances đúng thứ tự vào native, yêu cầu output curve mới. Nhánh native đã đọc dùng fallback khi fit tolerance không dương hoặc angle âm và kẹp degree trong 2–11; angle zero không vào nhánh fallback angular. Mô tả API đi kèm xác định fallback document absolute/angular và semantics xử lý kink. `Surface.Fit` cũng có nhánh fallback cho tolerance không dương, nhưng không nhận angle threshold của curve. | Danh tính các trường state cấp fallback chưa được xác nhận chỉ bằng body; còn phụ thuộc context Rhino. Không suy fallback headless/worker, default số cụ thể hoặc error bound từ việc call thành công. Rule kink của curve không áp sang surface, Boolean hay Join. |

## Kế hoạch adapter tối thiểu, Rust-first

1. Rust tạo context/typed conversion và migration record, kiểm dimensions/finite/range trước gọi native.
2. Native đọc document UnitSystem và parameter/property units; Rust resolve precedence template/document/command override.
3. Một resolver trả effective tolerance theo operation ID/version; registry ghi trường nào không được solver dùng.
4. C++ nhận canonical numeric values rồi đổi đơn vị góc đúng API OCCT; ghi report actual values và native error.
5. Unit-change planner chạy trên snapshot, báo object/parameter không thể scale; commit GUI thread một transaction.
6. Nạp/lưu context qua property hoặc metadata versioned, không phụ thuộc preference hiện tại ở máy đọc file.
7. Giữ Python UI/tool hiện hữu như nền tích hợp; không mở rộng business logic mới bằng Python chỉ vì có ToleranceFeatures.

Lỗi dự kiến: `UnknownUnits`, `InvalidUnitScale`, `DimensionMismatch`, `InvalidTolerance`, `ToleranceBelowResolution`, `UnsupportedScaleTarget`, `StaleContext`, `KernelFailure`.
Nếu thông báo cần số đo, hiển thị cả đơn vị người dùng và canonical value trong diagnostic đủ để tái lập.
Tolerance không được sửa sau khi solver chạy chỉ để đổi kết quả từ fail sang pass.

## Phụ thuộc

RCORE-01 quy định representation và dimension metadata; RCORE-03/04 quy định parser/session.
RCORE-05 tách pick pixel radius; RCORE-07/08 công bố application rule cho solver/deviation/repair.
RCORE-09 lưu context với History; RCORE-10 resolve import/clipboard; RCORE-11 giữ page scale riêng.

## Acceptance fixtures — chưa chạy tại baseline audit nguồn

| ID | Fixture và thao tác | Expected invariant / evidence |
|---|---|---|
| RCORE-02.T01 | Cùng circle khai báo radius 25.4 mm và 1 inch | World radius bằng nhau trong tolerance fixture; area/volume chuyển đúng dimensions |
| RCORE-02.T02 | Đổi precision từ 2 sang 8, chạy cùng Join/Boolean inputs | Shape/tolerance effective và quyết định accept/reject giống nhau; chỉ transcript số hiển thị đổi |
| RCORE-02.T03 | Hai endpoint gap 0.5τ và 1.5τ, chọn operation Join với rule cụ thể | Report ghi effective τ và decision đúng rule của Join; không dùng test này để gán rule cho mọi solver |
| RCORE-02.T04 | Đoạn 25.4 mm đổi khai báo mm → inch trong hai mode | Mode vật lý native dài 25.4 mm; mode giữ số native dài 645.16 mm; Undo/Redo trả cả geometry/context |
| RCORE-02.T05 | Import custom unit 2.5 mm/unit và nested instance translation 4 units | Translation vật lý 10 mm, không bị scale hai lần; userdata unitless không tự đổi |
| RCORE-02.T06 | Import unitless không có override, và custom scale NaN/0 | Không commit; báo unresolved/invalid scale có thể xử lý; không đoán theo bbox |
| RCORE-02.T07 | Preview đang chạy, đổi context tolerance hoặc model units | Kết quả cũ bị invalidate/recompute hoặc stale reject; preview và commit dùng một revision |
| RCORE-02.T08 | Cold FCStd reload trên máy có preference unit khác | Context document, override, physical size và History giữ nguyên; format tuân policy rõ |
| RCORE-02.T09 | Refit ở tọa độ world rất lớn với tolerance dưới numerical bound | Lỗi resolution/budget có report; không tự tăng tolerance, không partial mutation |
| RCORE-02.T10 | Paste từ inch sang mm, source có CV/placement và dimensions | Geometry hiện tại giữ kích thước vật lý; ratio/angle/UV không bị scale như length |
| RCORE-02.T11 | Đặt angular tolerance theo degrees rồi đọc radians và đổi ngược | Một giá trị vật lý nhất quán, sai số theo numeric bound; absolute/relative không đổi; thử thêm NaN/âm với policy OM9 riêng |
| RCORE-02.T12 | Curve Fit với tolerance âm, zero, dương và document context khác nhau | Log phân biệt sentinel/fallback với giá trị explicit; zero angle giữ ý nghĩa riêng; không dùng fallback ngầm từ document khác khi worker chạy |
| RCORE-02.T13 | Rhino reference: property unit setter so với adjust scale true/false, model/page độc lập | Đo geometry, tolerance, annotation và Undo trước-sau để xác nhận coverage native; chưa coi bất kỳ nhánh nào tương đương kế hoạch OM9 chỉ từ chữ ký API |

## Giới hạn và câu hỏi cần thêm evidence

Đã thấy document UnitSystem ở checkout hiện tại nhưng chưa nghiệm thu UI đổi scheme, persistence và propagation tới mọi module.
Tại baseline chưa tìm thấy context Rhino-style thống nhất trong phạm vi source đã rà; context triển khai sau baseline được ghi ở phần cập nhật, còn tolerance riêng cần migration từng module.
Cần inventory ngưỡng thực sự được dùng trong từng supported slice và quyết định migration trước khi nối context chung.
Relative tolerance cần đặc tả theo lệnh; chưa có cơ sở để áp `min(absolute, relative × bbox)` cho mọi operation.
Chưa xác định default OM9 toàn document từ nguồn Matrix; các con số trong fixture là lựa chọn thử nghiệm, không default tương thích.
Audit nguồn ban đầu chỉ viết tài liệu và không chạy native/build/tests; phần cập nhật ghi evidence mới theo slice, mọi capability/fixture khác vẫn cần nghiệm thu riêng.
