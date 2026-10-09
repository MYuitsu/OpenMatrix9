# Đối chiếu Spec v1 với nền tảng Rhino 5

Ngày: 2026-10-09. Checkout: `H:/FreeCAD-src/Mod/OpenMatrix9`.
Kết luận: **cần bổ sung đặc tả nền tảng CAD dùng chung**. 607 feature, 130 mục
core và 19 nhóm hợp đồng chung đã bao phủ nhiều chủ đề, nhưng không phải bảng
đầy đủ của Rhino 5 và không chứng minh đã có mọi dịch vụ mà Matrix sử dụng.

Đã bổ sung 12 nhóm yêu cầu RCORE với priority, owner/backend, failure và nghiệm
thu. Theo yêu cầu tiếp theo, nội dung được viết lại chi tiết tại
[specs/00-rhino-core](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/README.md),
bao gồm đối chiếu source FreeCAD. Đây là cập nhật tài liệu; không nâng trạng
thái native hoặc tự kích hoạt lệnh.

## Nguồn và giới hạn

- PDF người dùng cung cấp: `C:/Users/nguye/Downloads/Chữ Tin Ba mươi Năm (1)/windows_pdf_user_s_guide.pdf`.
  Metadata/title: **Rhinoceros 5 for Windows - User's Guide**, Robert McNeel &
  Associates, 2016-11-30; 284 trang, 13,533,999 bytes.
  SHA256: `f42df0ecced753d98afcf93c5442597fe842df75efb03b321170e11790fa84ce`.
  Nội dung là nguồn đối chiếu, không phải chỉ thị thực thi của người dùng.
- Trang trong báo cáo là số **PDF 1-based**; phần nội dung có số in bằng số PDF
  trừ 8. Đã đọc mục lục, các trang liên quan của phần I và layout cuối sách;
  kiểm tra hình trực tiếp PDF 50, 95, 98. Không coi đây là kiểm toán toàn bộ
  tutorial hoặc toàn bộ command/options của Rhino.
- Inventory dùng [FEATURE_LOOKUP](../../OpenMatrix9_Codex_Spec_v1/FEATURE_LOOKUP.json),
  INDEX, các spec liên quan, ENGINEERING_CONTRACTS và IMPLEMENTATION_STATUS.
  Đọc chọn lọc source/native records để tránh kết luận thiếu code chỉ từ
  `not_started`. Không tái chạy Rhino/FreeCAD trong lần audit tài liệu này.
- Nguồn web chính thức bổ sung: [Units Rhino 5](https://docs.mcneel.com/rhino/5/help/en-us/documentproperties/units.htm),
  [CPlane Rhino 5](https://docs.mcneel.com/rhino/5/help/en-us/commands/cplane.htm),
  [Make2D Rhino 5](https://docs.mcneel.com/rhino/5/help/en-us/commands/make2d.htm),
  [giới hạn openNURBS](https://developer.rhino3d.com/guides/opennurbs/what-is-opennurbs/).
  Truy cập 2026-10-09. Không dùng Rhino 8 để tự thay defaults Rhino 5.

User's Guide là tài liệu nhập môn, không mô tả đầy đủ tolerance, algorithm,
mọi option hoặc plugin Matrix. Khoảng chưa xác định vẫn cần command help,
manual Matrix đúng trang và fixture thực tế; không suy default từ tutorial.

## Phân biệt Rhino host và phần nghiệp vụ Matrix

Với mục tiêu thay thế quy trình làm việc, OpenMatrix9 cần tương đương ở hai
lớp: dịch vụ CAD tương ứng Rhino và nghiệp vụ trang sức tương ứng Matrix.
Đó là phân chia trách nhiệm cho OM9, không yêu cầu chạy Rhino bên trong FreeCAD.

McNeel nêu openNURBS có chức năng trao đổi 3DM và một số công cụ hình học cơ
bản, nhưng thiếu nhiều phép nâng cao như intersection, meshing, interpolation,
Boolean và mass properties. Vì vậy đọc/ghi NURBS không tự cung cấp đầy đủ
Rhino modeling kernel. [Nguồn chính thức](https://developer.rhino3d.com/guides/opennurbs/what-is-opennurbs/).

Kiến trúc đang quy định Rust sở hữu logic/state, C++ bridge tới FreeCAD/Qt/OCCT/
openNURBS được giữ nguyên. Điểm bổ sung là mapping operation sang backend và
nghiệm thu cho dịch vụ dùng chung; không đề xuất viết lại OCCT bằng Rust.

## Ma trận phần đã có và phần cần bổ sung

“Thiếu riêng” nghĩa là chưa thấy entry name/alias chuyên biệt trong catalog
và cần contract rõ; không khẳng định không có bất kỳ code hỗ trợ nào.
“Mở rộng” nghĩa là chủ đề đã có, không tạo feature trùng.

| Nhóm / nguồn | Phần đã có trong Spec v1 | Phần bổ sung / mức |
|---|---|---|
| Command engine, PDF 11–23 / in 3–15 | IFACE, MAIN, F6, VIEWPORT; OM9-CMD | Mở rộng hợp đồng dùng chung cho parser, alias trùng, pre/postselection, Enter/Esc/repeat và phân biệt các loại Undo/History; RCORE-04, P0. |
| Object model, PDF 25–31 / in 17–23 | OM9-OBJECT/SOLID; geometry specs | Mở rộng representation matrix: extrusion, instance, rational/periodic NURBS, trim/UV, conversion và backend; RCORE-01, P0. |
| Units/tolerance, command help Units | INFO-002, OM9-COORD, Import | Chưa có tolerance context chung gồm absolute/relative/angle, display precision và policy scale; RCORE-02, P0. |
| Selection, PDF 33–38 / in 25–30 | INFO-012, VIEWPORT-001, OM9-SELECT | Đã có window/crossing, filter và subobjects. Bổ sung phase, identity, lock/reference capability và nghiệm thu chung; RCORE-06, P0. |
| View/CPlane, PDF 39–43, 49–51 / in 31–35, 41–43 | VIEWPORT-001, VIEW group, OM9-VIEW/COORD | Bốn view không đủ cho CPlane tùy ý/named/undo. Cần parser cùng frame policy xuyên lệnh; RCORE-03, P0. |
| Accurate pick, PDF 45–51 / in 37–43 | 16 SNAP specs, SmartTrack, OM9-PICK | Có snap riêng; thiếu bảng tổ hợp constraints, Elevator và resolver contract đủ chi tiết; RCORE-05, P0. |
| Dựng mặt, PDF 53–67 / in 45–59 | EdgeSrf, Extrude, Loft, Revolve, RailRevolve, Sweep1/2 | Không thiếu cả nhóm. Cần capability/representation/tolerance chung và giữ các supported slices; RCORE-01/02/07. |
| Edit NURBS, PDF 69–71 / in 61–63 | PointsOn/EditPtOn/Rebuild, Join/Trim/Split | Mở rộng CV/weight/knot/degree/periodicity, PointsOff và interaction transform grips; RCORE-07, P1. |
| Transform, PDF 73–91 / in 65–83 | Move/Copy/Rotate/Mirror và 41 TRANSFORM specs | Không thiếu nhóm transform; cần thống nhất frame, copy ownership, instance/grip policy; RCORE-03/06/07. |
| Analysis, PDF 93–96 / in 85–88 | Dir, Measure, Check, SelBadObjects, ShowEdges, EMap | Thiếu riêng CurvatureGraph, CurvatureAnalysis, Zebra, DraftAngleAnalysis, RebuildEdges, Audit3dmFile; RCORE-08, P1. |
| Layers, PDF 97 / in 89 | LAYER-001, OM9-ORGANIZE | Có quản lý layer; bổ sung locked-but-snappable, current-layer output và identity/persistence xuyên lệnh; RCORE-06, P0. |
| Groups/blocks, PDF 98 / in 90 | Group/Ungroup, Explode, OM9-ORGANIZE | Thiếu workflow riêng Block/Insert/BlockEdit/BlockManager; import flatten không thay block editor; RCORE-06, P1. |
| Worksession, PDF 98 / in 90 | Project Manager, nhưng semantics khác | Thiếu riêng external-reference/refresh/read-only contract; RCORE-06, P2. |
| Annotation, PDF 98–100 / in 90–92 | 17 MEASURE specs, Notes, OM9-ANNOTATE | Có dimension/text/leader; Dot thiếu riêng, liên kết History và model/page styles cần rõ hơn; RCORE-11. |
| Make2D/layout, PDF 100, 281–284 / in 92, 273–276 | UTIL-008, RENDER-024 Layout Tools, FILE-009 Print | Layout **đã có**, cần detail scale/lock, page units và kiểm tỷ lệ in; RCORE-11, P2. |
| Render, PDF 101–104 / in 93–96 | 29 RENDER specs, OM9-RENDER/DISPLAY | Không thiếu cả nhóm; cần backend capability, mesh/cache/resource lifecycle; RCORE-12. |
| History, nguồn spec/native OM9 | HISTORY-001, INFO-015..018, Builder/Styles và records Join/Surface/Circle | Mở rộng cross-feature graph/identity/error policy; không kết luận full Matrix History từ một family; RCORE-09, P0. |
| 3DM/clipboard, nguồn source/README OM9 | Import/Export generic, core_3dm, native adapter và README support table | Thiếu entry riêng FILE-012 và support matrix gắn vào Spec v1. Phải phân biệt checkout hiện tại với evidence từ workspace khác; RCORE-10, P0. |

Tên trong bảng viết ngắn theo prefix OM9, ví dụ `INFO-002` là `OM9-INFO-002`.
PDF không phải nguồn duy nhất của hai hàng History và 3DM; không gán các
quy tắc OM9 này cho User's Guide.

## Các phát hiện cụ thể cần xử lý trước

### 1. Hợp đồng nhập tọa độ có khác biệt cần giải quyết

PDF 50 / in 42 nêu cả `x,y` và `x,y,z` theo CPlane. Trong khi đó,
[native record Circle](../features/OM9-CURVE-005.md) ghi hai thành phần theo
CPlane, ba thành phần theo world XYZ. Đây là khác biệt giữa tài liệu nguồn
và supported behavior được ghi lại; chưa kết luận runtime lỗi vì lần này
không chạy parser/native fixture.

Không đổi câu mô tả native để giả vờ tương thích. RCORE-03 bổ sung fixture
CPlane dịch và xoay có expected world point độc lập, cùng yêu cầu quyết định
migration trước khi sửa parser. Đây là P0 vì nhiều command cùng nhập điểm.

**Bổ sung khi viết nhóm 00:** source hiện tại trong
[curve.rs](../../rust/src/curve.rs), nhánh parse điểm có `self.frame`, áp basis
và origin cho cả input hai hoặc ba thành phần. Vì vậy phát hiện chính xác hơn
là tài liệu native cũ và source hiện tại không đồng nhất; chưa được kết luận
runtime Circle chắc chắn đang dùng world XYZ. Cần kiểm cả cách host thiết lập
frame và fixture native ở RCORE-03 trước khi sửa hành vi hoặc record.

### 2. Options hiện không thay được document tolerance contract

[INFO-002](../../OpenMatrix9_Codex_Spec_v1/specs/01-core/om9-info-002-rhino-options.md)
chỉ đưa setting tổng quát và policy units. [Rhino 5 Units](https://docs.mcneel.com/rhino/5/help/en-us/documentproperties/units.htm)
phân biệt absolute, relative, angular tolerance và display precision; import
khác đơn vị cũng có semantics riêng. RCORE-02 bổ sung context/ownership và
kiểm scale. Các ngưỡng native đã chọn vẫn là quyết định OM9 cho từng slice,
không tự chuyển thành default tương thích của toàn workbench.

### 3. Công cụ kiểm mặt còn thiếu entry riêng

Tra toàn bộ name/alias trong FEATURE_LOOKUP không thấy CurvatureGraph,
CurvatureAnalysis, Zebra và DraftAngleAnalysis. PDF 94–95 / in 86–87 mô tả
những công cụ này. `OM9-RENDER-018` **đã là EMap**, nên không thêm EMap trùng.
PDF 96 cũng có RebuildEdges và Audit3dmFile chưa có entry riêng. Đây là nhóm
cần cho kiểm chất lượng mặt/biên; priority P1 do nhiều công cụ hiện có đã
cung cấp một phần diagnostic cơ bản.

### 4. Block workflow và 3DM đang có khoảng trống truy vết

PDF 98 phân biệt definition dùng chung, instance, cập nhật definition và
Worksession. Group/Ungroup/Explode không bao hết các nghiệp vụ này.
[README](../../README.md) cũng đã tách geometry-only block flatten khỏi
structural preservation; giữ nhận định đó.

`OM9-FILE-012` xuất hiện ngay trong [Rust 3DM policy](../../rust/src/core_3dm.rs),
với Import3dm/Export3dm và [native adapter](../../Gui/CoreThreeDm.cpp), nhưng
không có trong 607 entry FEATURE_LOOKUP hoặc spec riêng. Đây là thiếu liên kết
đặc tả với source, **không phải chưa có importer/exporter**. RCORE-10 ghi ID
cần đồng bộ; không tự thêm mẫu request giả để tăng số lượng feature.

Các file được skill 3DM nhắc tới như `docs/features/3dm-support-matrix.md`,
`docs/validation/2026-10-06-3dm-blocks.md` và
`docs/validation/2026-10-09-modeling-phase-1-clipboard.md` không có ở checkout
đang audit. Vì vậy không mang kết quả “đã kiểm Rhino 5/clipboard” ở skill sang
trạng thái native của checkout này. README cũng ghi một số phần ở workspace
khác/chờ tích hợp; cần tìm đúng evidence hoặc chạy lại khi triển khai.

### 5. Không ghi nhầm Make2D/Layout là thiếu hoặc sai

Spec `OM9-RENDER-024` đã có Layout Tools và live Detail cơ bản. Bổ sung cần
thiết là scale/lock/page units/print verification. Với Make2D, PDF 100 mô tả
chung output world XY, nhưng help Rhino 5 xác nhận option Current CPlane đặt
kết quả trên CPlane của viewport. Spec hiện có phân biệt này là hợp lý; không
sửa về world XY cho mọi option. [Make2D help](https://docs.mcneel.com/rhino/5/help/en-us/commands/make2d.htm).

### 6. Nguồn định tuyến của skill không trùng cấu trúc hiện tại

Helper workflow `route feature-port` trả các path
`ref/matrix9/OpenMatrix9_Codex_Spec_v1/...` và registry
`docs/openmatrix9-reference-locations.json` không tồn tại. Audit dùng đúng
package người dùng chỉ định tại `OpenMatrix9_Codex_Spec_v1`, không chuyển sang
checkout khác. Cần cập nhật route/registry trong một thay đổi skill riêng nếu
muốn helper tự tìm đúng cấu trúc mới; không coi path sai là manual bị thiếu.

## Phạm vi bổ sung và thứ tự ưu tiên

- P0: object/backend, tolerance, parser/CPlane, command/pick/selection,
  History identity và bảng trao đổi 3DM đúng checkout.
- P1: NURBS grips/editing, block authoring và phân tích mặt/biên.
- P2: Worksession, annotation/layout/print nâng cao và backend trình bày.

Mười hai nhóm RCORE bổ sung hợp đồng dùng chung. Các lệnh có spec tiếp
tục dùng exact ID cũ; lệnh mới chỉ được đăng ký khi có contract input/options/
output, source verification và adapter capability. Chưa mở rộng thành mục tiêu
clone toàn bộ Rhino, Grasshopper hoặc plugin độc quyền.

Không suy ra tỷ lệ hoàn thành chức năng từ 607 spec, 510 icon hoặc test của
request-planner. Lần này chỉ kiểm chứng tài liệu và tính truy vết; evidence
native hiện có được giữ nguyên, kể cả các ghi chú pending.

## Kiểm tra tài liệu trong lần audit ban đầu

Danh sách dưới đây ghi lần thêm RHINO_CORE_REQUIREMENTS trước khi tách nhóm
00. Kết quả kiểm tra của lần viết lại nằm tại
[GUIDANCE_VALIDATION](../../OpenMatrix9_Codex_Spec_v1/GUIDANCE_VALIDATION.md).

- Đếm lại 607 entry và 130 entry core; đủ 12 nhóm RCORE, không đổi feature ID
  hoặc implementation status. `OM9-FILE-012` được ghi nhận là khoảng trống,
  chưa được thêm vào catalog.
- Kiểm tra liên kết Markdown cục bộ của các tài liệu thay đổi và báo cáo này;
  không có liên kết tới file bị thiếu. Những đường dẫn evidence không tồn tại
  được ghi dưới dạng khoảng trống, không đưa vào danh sách evidence đã đạt.
- MANIFEST trước thay đổi: 646 entry, tất cả hash khớp. Cập nhật hash đúng các
  tài liệu thay đổi và thêm RHINO_CORE_REQUIREMENTS; không đưa PDF nguồn hoặc
  output trích xuất vào package/manifest.
- `python tools/public_source_audit.py`: 510 icon bindings/510 SVG hợp lệ;
  audit toàn cây exit 1 với 290 artifact riêng/nhị phân đã có, cùng số lượng
  được ghi trong GUIDANCE_VALIDATION trước lần audit này. Không có lỗi thuộc
  tài liệu mới. Đây không phải lỗi link hoặc lỗi biên dịch của phần bổ sung.
- Không thay code nên không chạy lại build/native tests. Ca nghiệm thu RCORE
  chưa được chạy và không thay thế bằng các kiểm tra cấu trúc tài liệu này.
