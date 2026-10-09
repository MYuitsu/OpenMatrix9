# 00 - Nền tảng Rhino và khả năng tái sử dụng FreeCAD

<!-- rhino-core-p0-checkpoint:start -->
## Checkpoint triển khai P0 — 2026-10-09

Sau baseline audit bên dưới, người dùng đã chọn triển khai **chỉ P0**.
[Báo cáo triển khai và kiểm thử](../../../docs/validation/2026-10-09-rhino-core-p0.md)
ghi context units, parser/CPlane, snap/repeat, spline validation, History và 3DM safeguards
đã nối vào native; phần còn thiếu vẫn được liệt kê theo từng chương.
Các câu `source_inspected`/`not_run` trong baseline là kết quả lần đối chiếu ban đầu.
Checkpoint mới không biến toàn bộ capability hay fixture trong chương thành đã pass.
CAPABILITY_LOOKUP giữ nguyên từng dòng nguồn và thêm checkpoint runtime có phạm vi.
<!-- rhino-core-p0-checkpoint:end -->

Ngày đối chiếu: **2026-10-09**. Nhóm này mô tả các dịch vụ CAD cần thiết để
thực hiện quy trình Matrix trên FreeCAD: từ nhập điểm, kiểu hình học và solver
đến History, tài liệu, xuất bản vẽ và trao đổi 3DM.

**FreeCAD đã có nhiều thành phần nền tảng**, nhưng nằm ở App/Gui và các module
Part, Surface, Draft, Mesh, Points, TechDraw, Material. Có thành phần tương ứng
không có nghĩa OpenMatrix9 đã nối vào thành phần đó hoặc hành vi giống Rhino.
Các chương dưới đây trả lời riêng ba câu hỏi: Rhino cần hành vi gì; FreeCAD
trong checkout này cung cấp gì; OpenMatrix9 còn phải viết/tích hợp/kiểm chứng gì.

Đây là **12 đặc tả nền tảng**, không phải 12 command bổ sung vào catalog.
Việc viết nhóm `00` không tự sửa ID, mẫu Rust hoặc trạng thái native của
607 feature trong 16 domain `01`–`16`; các cập nhật song song được giữ lại.
Thư mục `00` đứng trước các domain đó và cung cấp hợp đồng dùng chung cho chúng.

Bộ tài liệu hiện có **148 mục capability** đối chiếu FreeCAD/OM9 và **155 ca
nghiệm thu nền tảng cần chạy**, cùng **11 ca UI bổ sung**. Bảng tra cứu giữ từng mục riêng để giao việc và cập nhật
bằng chứng; các con số này không thể hiện tỷ lệ hoàn thành hay kết quả runtime.

## Nguồn Rhino 5 phải đọc khi triển khai core

Rhino 5 cung cấp nền CAD cho workflow Matrix; các dịch vụ core của OM9 cần
được đối chiếu với hành vi đó, rồi nối vào backend FreeCAD thích hợp.
[Bảng tra Rhino 5](../../../ref/rhino5/README.md) ánh xạ 12 chương này sang
trang PDF, Command Help và các phần cần kiểm chứng. Bản
[User's Guide gốc](../../../ref/rhino5/windows_pdf_user_s_guide.pdf) cùng
[nguồn và hash](../../../ref/rhino5/SOURCES.json) được giữ trong ref/rhino5
theo ngoại lệ tài liệu người dùng đã cho phép.

Trước khi đóng một capability, đọc trang được chỉ mục và Help của đúng lệnh
Rhino 5, chốt ngữ cảnh/defaults/tolerance, đối chiếu adapter FreeCAD và chạy
fixture liên quan. User's Guide không mô tả mọi thuật toán, không thay kết
quả runtime và không tự nâng các checkpoint/native status đang có. Nguồn
decompile và tài liệu Matrix vẫn ở kho riêng để đọc phần cần thiết.

## Chi tiết bổ sung từ code decompile

Đã bổ sung cả 12 chương bằng hành vi đọc được từ export Rhino/Matrix,
đặc biệt input/base point, selection flags, unit metadata, ownership,
Rebuild/Fit, diagnostic, History, serializer, Detail và display/resources.
[Giao diện chi tiết](UI_DETAILS.md) mô tả panel, F6, Shade, pick/window,
control số, View Manager và Layout; tám feature spec tương ứng đã có
supported slice, parameters/defaults, lifecycle và lỗi cụ thể hơn.

Phần decompile viết trực tiếp hành vi và giới hạn, không kèm bảng tham
chiếu code. [Phạm vi đối chiếu và phần còn thiếu](DECOMPILE_AUDIT.md) ghi
giới hạn export; nguyên tắc `source_inspected` / `not_run` tiếp tục áp dụng.
Fixture nền tảng và UI là yêu cầu nghiệm thu, chưa phải kết quả pass.

Lần kiểm tra lại bridge native sau export đã có thân hàm và header đi kèm.
Các chương 01/02/03/05/06/07/08 được bổ sung mapping, guards và boundary
đọc được; ghi chú thiếu thân bridge đã được sửa. Phân loại capability và
fixture runtime vẫn giữ nguyên, không nâng trạng thái implementation.

## Mục lục và câu hỏi từng chương

| ID | Chương chi tiết | Câu hỏi chính | Ưu tiên |
|---|---|---|---|
| RCORE-01 | [01 - Object model và kernel](01-object-model-kernel.md) | FreeCAD biểu diễn NURBS, trims, solid, mesh, instance ra sao; operation nào thuộc backend nào? | P0 |
| RCORE-02 | [02 - Units và tolerance](02-units-tolerances.md) | Units/precision đã có; document tolerance và chính sách chuyển đổi còn phải bổ sung gì? | P0 |
| RCORE-03 | [03 - Coordinates và CPlane](03-coordinates-cplanes.md) | Working plane của FreeCAD có thay được CPlane và cú pháp nhập điểm Rhino không? | P0 |
| RCORE-04 | [04 - Command engine](04-command-engine.md) | Command/console/Undo của FreeCAD khác session prompt/option/repeat của Rhino ở đâu? | P0 |
| RCORE-05 | [05 - Picking, snapping và constraints](05-picking-snapping.md) | Có snap và ray picking nào; kết hợp snap/Ortho/Project/Elevator thế nào? | P0 |
| RCORE-06 | [06 - Selection, layer, group và block](06-selection-organization-blocks.md) | Selection, Draft Layer, App::Link và nhóm khác Rhino block/Worksession thế nào? | P0/P1/P2 |
| RCORE-07 | [07 - NURBS editing và modeling](07-nurbs-editing-modeling.md) | Part/Surface đã cung cấp solver gì; CV/knots/seams/options nào cần editor và adapter? | P0/P1 |
| RCORE-08 | [08 - Analysis và repair](08-analysis-repair.md) | Check, curvature, zebra/reflection, draft và mesh repair có sẵn ở mức nào? | P1 |
| RCORE-09 | [09 - History và dependencies](09-history-dependencies.md) | Recompute graph của FreeCAD giúp gì; phần nào là policy Rhino/Matrix phải viết? | P0 |
| RCORE-10 | [10 - Persistence, 3DM và clipboard](10-persistence-3dm-clipboard.md) | FCStd, import/export và clipboard của FreeCAD khác trao đổi Rhino 5 thế nào? | P0 |
| RCORE-11 | [11 - Annotation, layout và print](11-annotation-layout-print.md) | Draft/TechDraw thay được bao nhiêu phần dimension, Make2D, Detail và in đúng tỷ lệ? | P1/P2 |
| RCORE-12 | [12 - Display, render và resources](12-display-render-resources.md) | Viewport/material/cache đã có; renderer, plugin và quản lý tài nguyên còn thiếu gì? | P1/P2 |

P0 là nền tảng ảnh hưởng tính đúng của nhiều lệnh; P1 là công cụ hoàn thiện
thiết kế/chẩn đoán; P2 là tài liệu, cộng tác và trình bày nâng cao. Priority là
quyết định sắp xếp công việc OM9, không phải độ quan trọng của lệnh trong Rhino.

## Cách đọc cột “FreeCAD đã có chưa”

| Nhãn | Điều đã được xác minh | Điều chưa được kết luận |
|---|---|---|
| `UI+API` | Đã đọc code cho đường thao tác giao diện và API/nền tảng liên quan. | Chưa chứng minh đã build/cài đủ module hoặc runtime giống Rhino. |
| `API` | Có primitive/API/solver trong source để adapter sử dụng. | Chưa có bằng chứng workflow giao diện tương đương hoặc command OM9 đã dùng API đó. |
| `Một phần` | Có thành phần liên quan nhưng thiếu một phần contract hoặc khác semantics. | Không được cộng như full parity; đọc phần thiếu trong từng dòng. |
| `Chưa thấy trong phạm vi rà` | Không tìm được đường tương đương trong các module và symbols ghi ở chương đó. | Không khẳng định mọi addon hoặc mọi phiên bản FreeCAD đều không có. |
| `Chưa kiểm chứng` | Chưa đủ source/fixture/bằng chứng để quyết định. | Không chuyển thành “có” hoặc “không” từ tên command/icon. |

Các mã như `RCORE-05.C03` là dòng capability, không phải mã command.
Tra mã nhanh tại [CAPABILITY_LOOKUP.json](CAPABILITY_LOOKUP.json); index này
giữ riêng trạng thái source FreeCAD và việc OM9 cần làm, không nhập vào
FEATURES.json hoặc dùng để bật command.
Trạng thái OM9 nằm ở cột riêng. “Có source”, “có record kiểm thử cũ” và “đã chạy
nghiệm thu phiên này” là ba mức khác nhau; phiên này chỉ audit source/tài liệu.

Mỗi chương giữ các phần: hành vi mục tiêu; ma trận khả năng FreeCAD/OM9;
input/output và lifecycle; điểm vào source; công việc tích hợp; dependencies;
fixture nghiệm thu và phần cần evidence. Một nhóm có thể dùng nhiều module,
và một API có thể phục vụ nhiều nhóm; không cộng số dòng để ra phần trăm parity.

## Câu trả lời nhanh: phần nào có thể dùng lại?

| Nhóm | Nền tảng đã thấy trong source FreeCAD | Phần cần OM9 xử lý hoặc chưa chứng minh tương đương |
|---|---|---|
| [01 - Object/kernel](01-object-model-kernel.md) | Part BRep/NURBS, Points, Mesh, App::Link và native shape properties. | Type/capability registry, mapping Rhino representations và conversion report. |
| [02 - Units/tolerance](02-units-tolerances.md) | Quantity có dimension, document UnitSystem, quantity widgets và shape tolerances. | Context absolute/relative/angular chung, scale policy và mapping per-operation; không thể nói host chỉ có global units. |
| [03 - CPlane](03-coordinates-cplanes.md) | Draft working plane theo view, phép chuyển frame, history và proxy. | Grammar Rhino, latch xuyên command, named-plane semantics và integration không kết thúc session OM9. |
| [04 - Command](04-command-engine.md) | Command registry/actions, Python console, transaction và selection APIs. | Prompt/options/session/checkpoint/repeat thống nhất kiểu Rhino; Python console không phải Rhino command engine. |
| [05 - Snap](05-picking-snapping.md) | Draft có nhiều snap, kể cả BSpline knots, cùng native picking. | Resolver/constraint priority dùng chung; xử lý khác biệt grid ẩn và object không selectable. |
| [06 - Organization](06-selection-organization-blocks.md) | Selection/subelements, groups, Draft layers và App::Link. | Window fully-enclosed khác center-based selection; definition/instance authoring và Worksession cần contract riêng. |
| [07 - NURBS/modeling](07-nurbs-editing-modeling.md) | API poles/weights/knots/periodicity, approximation và các solver Part/Surface. | Editor grips chung, options/seams/retrim, deviation/continuity và lifecycle tương thích. |
| [08 - Analysis](08-analysis-repair.md) | Part validity/BOP checks, curvature evaluation, Sketcher comb và Mesh analysis/repair. | Diagnostic CAD thống nhất; không suy Zebra/DraftAngleAnalysis đầy đủ từ công cụ gần tên. |
| [09 - History](09-history-dependencies.md) | Property links, dependency graph, recompute, transactions và persistence. | Record/Update/Lock/Clear, topology identity và replay theo family Rhino/Matrix. |
| [10 - Persistence/I/O](10-persistence-3dm-clipboard.md) | FCStd, object clipboard của FreeCAD và các format exchange của Part. | 3DM là adapter riêng OM9; read/edit/retain/export và hai chiều Rhino clipboard phải kiểm riêng. |
| [11 - Annotation/layout](11-annotation-layout-print.md) | Draft/TechDraw có dimension, projection, page và print primitives. | Mapping style/model/page scale, Rhino Detail workflow và annotation/History semantics. |
| [12 - Display/render](12-display-render-resources.md) | Native viewport, display/material, tessellation và environment texture mapping. | Resource lifecycle/capability thống nhất, EMap workflow, render backend và plugin parity riêng. |

Các dòng này tóm tắt **source đã có**, không phải checklist đã chạy. Đọc từng
chương để thấy symbol, điều kiện và những nhánh chưa kiểm chứng. Nếu API host
đáp ứng, công việc là adapter/workflow/verification; chỉ viết solver mới khi
đã xác định thiếu API hoặc không đáp ứng contract thực tế.

## Baseline của câu trả lời

| Thành phần | Baseline |
|---|---|
| FreeCAD source | `H:/FreeCAD-src`, Git HEAD `21d36cfa1eb110a1d0667050ff31706298805bbd`. |
| Version metadata | [version.json](../../../../../version.json) tại checkout ghi `27.1.0`, suffix `dev`; đây là metadata nguồn, không phải phiên bản binary đang chạy. |
| OpenMatrix9 | Nested repository `H:/FreeCAD-src/Mod/OpenMatrix9`, Git HEAD `52e887ab706dcd5d80e23979fb1f1b62d8d869b0`; có thay đổi/untracked trong working tree. |
| Mức kiểm chứng | Source/API và tài liệu được đọc; không chạy lại FreeCAD/Rhino, build hoặc geometry regression trong lần viết này. |
| Dấu vết file | [SOURCE_BASELINE.json](SOURCE_BASELINE.json) ghi hash file được liên kết làm evidence. HEAD không thay thế hash của working tree. |
| Rhino mục tiêu | Rhino 5 for Windows trong quy trình Matrix; không tự gộp Rhino SubD đời mới, Grasshopper, T-Splines, Clayoo hoặc renderer độc quyền vào core đã có. |

FreeCAD có các build option theo module. Kiểm tra
[InitializeFreeCADBuildOptions](../../../../../cMake/FreeCAD_Helpers/InitializeFreeCADBuildOptions.cmake)
và [CheckInterModuleDependencies](../../../../../cMake/FreeCAD_Helpers/CheckInterModuleDependencies.cmake)
khi tạo bản chạy. File source tồn tại chưa xác nhận module được cài trong
FreeCAD của người dùng hoặc tương thích ABI với OpenMatrix9Gui.

Source citations dùng đường tương đối tới checkout FreeCAD chứa module OM9.
Nếu đọc gói riêng ngoài cấu trúc đó, dùng path và SHA256 trong SOURCE_BASELINE
để tìm đúng nguồn; không xem liên kết ngoài gói bị thiếu là năng lực bị mất.

Nguồn Rhino và khoảng trống của lần đối chiếu đầu được giữ tại
[báo cáo nguồn](../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md).
Không đưa PDF, screenshot thương mại hoặc bản trích nguyên văn vào nhóm này.
Hợp đồng mới là diễn giải kỹ thuật cho OM9; mọi default chưa xác định cần
command help/fixture hoặc quyết định OM9 được ghi riêng.

## Các ranh giới cần giữ khi triển khai

1. **CAD geometry:** Part/OCCT cung cấp nhiều solver; Rust sở hữu request,
   state, validation và logic độc lập. C++ giữ native adapter cần thiết, không
   chuyển mọi thuật toán sang C++ vì source FreeCAD hiện dùng C++.
2. **Workflow:** Draft/TechDraw có UI và đối tượng riêng. Tái sử dụng khả năng
   cần adapter/capability explicit; không chạy chuỗi command rồi giả định
   đã bảo đảm Cancel, Undo, layer, CPlane và History của OM9.
3. **Representation:** Mesh hiển thị, CAD shape, block instance, snapshot
   archive và annotation là các loại dữ liệu khác nhau. Khả năng giữ nguyên
   một payload chưa đồng nghĩa sửa hoặc xuất được trạng thái mới của nó.
4. **History:** FreeCAD dependency graph là nền tảng. Rhino History, Matrix
   builder lifecycle và transcript command là các semantics phải mô tả riêng.
5. **I/O:** FCStd clipboard/document persistence không mặc nhiên tương thích
   Rhino 3DM clipboard. openNURBS không cung cấp toàn bộ modeling kernel Rhino.
6. **Evidence:** Numerical/unit tests, native host integration và Rhino
   roundtrip là ba lớp nghiệm thu. Đọc symbol hoặc có screenshot không đóng
   cả ba lớp này.

## Ánh xạ tới các nhóm nghiệp vụ hiện có

| Nhóm chức năng | Nền tảng cần xem trước |
|---|---|
| [01-core](../01-core/README.md) | RCORE-02–06, 09–12: command, view, pick, document và I/O. |
| [02-curve](../02-curve/README.md), [03-surface](../03-surface/README.md), [04-solid](../04-solid/README.md) | RCORE-01–05, 07–09: representation, input, solver, diagnostic và History. |
| [05-transform](../05-transform/README.md) | RCORE-01–07, 09: frame, instance/grip ownership và dependency. |
| [08-builders](../08-builders/README.md), [09-tools](../09-tools/README.md) | RCORE-01–10: lifecycle, geometry và output có metadata. |
| [10-gems](../10-gems/README.md), [11-settings](../11-settings/README.md), [12-cutters](../12-cutters/README.md) | RCORE-01–10: hình học/units, layer/instance, Boolean, History và archive. |
| [13-render](../13-render/README.md) | RCORE-01/06/10/11/12: display, material/texture, layout và resources. |
| [06-tsplines](../06-tsplines/README.md), [07-matrix-art](../07-matrix-art/README.md), [14-subd](../14-subd/README.md), [15-emboss](../15-emboss/README.md) | RCORE-01/07/08/10/12 cộng backend riêng của nhóm; không suy capability plugin từ Part/OCCT. |
| [16-matrix-tools](../16-matrix-tools/README.md) | Chọn RCORE theo tác động thật của từng feature; không mặc định toàn nhóm chỉ là UI. |

## Thứ tự phát triển và điều kiện đóng công việc

1. Khóa object/type capabilities, units/tolerance và parser/frame dùng chung.
   Kiểm tra những lệnh Line/Circle/Rectangle/Box đang dùng các dịch vụ đó.
2. Hoàn thiện command session, pick/selection/layer và ownership. Mỗi preview
   phải dùng cùng solver/policy với geometry cuối.
3. Tích hợp solver/NURBS editor trên API đã có; bổ sung diagnostic để có cách
   đánh giá kết quả. Tách giới hạn kernel khỏi giới hạn editor.
4. Nối History, FCStd, 3DM/clipboard theo supported type và revision. Có fixture
   cold reload và kiểm output sau khi nguồn đã được chỉnh sửa.
5. Hoàn thiện block authoring, analysis, annotation/layout/render theo nhu cầu
   nghiệp vụ sau khi phần P0 đã có regression.

Mỗi thay đổi chọn một tập capability có giới hạn, liên kết exact OM9 feature
ID và ghi API/backend, options, defaults, units, input/output ownership, lỗi,
Undo/Redo và persistence. Khi có evidence mới, cập nhật spec tương ứng cùng
[IMPLEMENTATION_STATUS](../../IMPLEMENTATION_STATUS.md) theo
[IMPLEMENTATION_RULES](../../IMPLEMENTATION_RULES.md). Không tự đổi 607 trạng
thái native chỉ vì thư mục `00` đã được viết xong.
