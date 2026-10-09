# Rhino 5 for Windows - nguồn đọc và kiểm chứng core

[PDF nguồn nguyên bản](windows_pdf_user_s_guide.pdf) là **Rhinoceros 5 for Windows - User's Guide**, Robert McNeel & Associates, ngày 2016-11-30: 284 trang, 13,533,999 bytes, SHA256 `f42df0ecced753d98afcf93c5442597fe842df75efb03b321170e11790fa84ce`.
Thông báo tại PDF 1 được giữ nguyên: **© Robert McNeel & Associates, 11/30/2016.**
PDF công khai vẫn giữ thông báo/quyền của nguồn, không được đổi thành giấy phép của project.
Xem [metadata và kiểm hash nguồn](SOURCES.json), [PDF chính thức](https://docs.mcneel.com/rhino/5/usersguide/en-us/windows_pdf_user_s_guide.pdf) và [Guide HTML chính thức](https://docs.mcneel.com/rhino/5/usersguide/en-us/html/webtitlepage.htm).

Index này giúp triển khai dịch vụ CAD Rhino 5 mà quy trình Matrix dựa vào.
Nó định tuyến **12 chương, 148 capability và 155 fixture nền tảng** hiện có, cùng liên kết tới 11 fixture UI;
các số là phạm vi tra cứu/nghiệm thu, không phải tỷ lệ hoàn thành.
[CORE_REFERENCE_INDEX.json](CORE_REFERENCE_INDEX.json) chứa đầy đủ `spec_path`, `guide_pdf_ranges`,
exact Help URLs, `capability_ids`, `fixture_ids`, `read_next`, `verify` và khoảng trống từng chương.
Các path trong JSON được tính từ thư mục này.

## Workflow bắt buộc trước khi sửa hành vi hoặc công bố parity

1. Chọn exact OM9 feature ID và dòng capability RCORE. Đọc chương hiện tại,
   [CAPABILITY_LOOKUP](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/CAPABILITY_LOOKUP.json)
   và [checkpoint P0](../../docs/validation/2026-10-09-rhino-core-p0.md) để biết slice đã nối và phần còn thiếu.
2. Đọc các trang Guide được định vị dưới đây, **sau đó đọc Help của đúng command/variant Rhino 5**.
   PDF 23 / in 15 hướng người dùng tới F1 trong command và Command Help Auto-Update.
   Guide là nhập môn; Units/tolerance, History, 3DM/clipboard cần help/API và fixture riêng.
3. Ghi Rhino **5 Windows SR/build**, template, units model/page, absolute/relative/angular tolerance,
   command options, nguồn default, pre/postselection, viewport/CPlane và settings/plugin context.
   Default Help, default template, setting đang lưu và option explicit là những dữ kiện khác nhau.
   Không dùng Rhino đời mới để âm thầm thay default Rhino 5.
4. Tách `source_verified` (nguồn/version/page hoặc URL), `fixture_observed` (binary/input/kết quả đo)
   và `OM9_decision` (policy/threshold/priority/migration lựa chọn cho OM9).
   Default chưa biết giữ unresolved; quyết định OM9 không được gán thành sự thật Rhino.
5. So đúng contract với FreeCAD native UI/API/backend và adapter OM9 hiện tại:
   input/type/options, ownership, output, failure, cancel, Undo/Redo và persistence.
   Có API hoặc tên command tương tự chưa chứng minh semantics tương đương.
6. Chạy fixture đại diện trên Rhino 5 và native FreeCAD/OM9 với cùng input/context;
   đo geometry/topology/units/state và lifecycle phù hợp. Để rõ unsupported, skipped, failed và not_run.
   Chỉ đóng acceptance/status cho phạm vi có evidence thực; một fixture đại diện không đóng mọi fixture của chương.

[Guide/kernel source audit](../../docs/reviews/2026-10-09-rhino-core-gap-audit.md) và
[nhóm đặc tả 00](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/README.md) là tài liệu đối chiếu đi kèm.
[Giới hạn openNURBS chính thức](https://developer.rhino3d.com/guides/opennurbs/what-is-opennurbs/)
phải đọc trước khi chọn 3DM/backend: thư viện trao đổi không cung cấp toàn bộ modeling kernel Rhino.
Guide cũng không cung cấp solver hoặc bằng chứng tương đương số học.

## Tra đúng trang và Help

Số **PDF là 1-based**; số **in** là footer thật của nội dung.
Các đoạn dưới đây đã kiểm heading/footer, với quan hệ body `in = PDF - 8`.
PDF 3-7 là mục lục in iii-vii; trang trắng không có footer body.
Không lấy offset này để gán số in cho bìa/mục lục/trang trắng.

| Chương | Guide PDF / in và nội dung | Đọc Help tiếp |
|---|---|---|
| [RCORE-01 - Kiểu đối tượng và ranh giới kernel](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md) | 25-31 / 17-23 | [ExtrudeCrv](https://docs.mcneel.com/rhino/5/help/en-us/commands/extrudecrv.htm); [Mesh](https://docs.mcneel.com/rhino/5/help/en-us/commands/mesh.htm) |
| [RCORE-02 - Units, tolerance và độ chính xác](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/02-units-tolerances.md) | Không gán Guide thành đặc tả đầy đủ của nhóm này | [Units properties](https://docs.mcneel.com/rhino/5/help/en-us/documentproperties/units.htm) |
| [RCORE-03 - Tọa độ, parser điểm và CPlane](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/03-coordinates-cplanes.md) | 39-43 / 31-35; 49-51 / 41-43 | [CPlane](https://docs.mcneel.com/rhino/5/help/en-us/commands/cplane.htm); [NamedCPlane](https://docs.mcneel.com/rhino/5/help/en-us/commands/namedcplane.htm)*; [Cursor constraints / coordinate entry](https://docs.mcneel.com/rhino/5/help/en-us/user_interface/cursor_constraints.htm) |
| [RCORE-04 - Command engine và vòng đời tương tác](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/04-command-engine.md) | 11-23 / 3-15 | [Undo / Redo](https://docs.mcneel.com/rhino/5/help/en-us/commands/undo.htm); [Aliases](https://docs.mcneel.com/rhino/5/help/en-us/options/aliases.htm); [Rhino scripting / command macros](https://docs.mcneel.com/rhino/5/help/en-us/information/rhinoscripting.htm) |
| [RCORE-05 - Pick, snapping và constraints](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/05-picking-snapping.md) | 45-51 / 37-43 | [Object snaps](https://docs.mcneel.com/rhino/5/help/en-us/user_interface/object_snaps.htm); [Ortho / OrthoAngle / SetOrtho](https://docs.mcneel.com/rhino/5/help/en-us/commands/ortho.htm); [Cursor constraints / Elevator](https://docs.mcneel.com/rhino/5/help/en-us/user_interface/cursor_constraints.htm) |
| [RCORE-06 - Selection, layer, group, block và Worksession](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/06-selection-organization-blocks.md) | 33-38 / 25-30; 97-98 / 89-90 | [Selection commands / sub-object selection](https://docs.mcneel.com/rhino/5/help/en-us/commands/selection_commands.htm); [Layer](https://docs.mcneel.com/rhino/5/help/en-us/commands/layer.htm); [Block](https://docs.mcneel.com/rhino/5/help/en-us/commands/block.htm) |
| [RCORE-07 - NURBS editing và modeling](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/07-nurbs-editing-modeling.md) | 53-67 / 45-59; 69-71 / 61-63; 73-91 / 65-83 | [PointsOn / PointsOff](https://docs.mcneel.com/rhino/5/help/en-us/commands/pointson.htm); [Rebuild](https://docs.mcneel.com/rhino/5/help/en-us/commands/rebuild.htm); [FitCrv](https://docs.mcneel.com/rhino/5/help/en-us/commands/fitcrv.htm) |
| [RCORE-08 - Analysis, diagnostic và repair](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/08-analysis-repair.md) | 93-96 / 85-88 | [CurvatureGraph](https://docs.mcneel.com/rhino/5/help/en-us/commands/curvaturegraph.htm); [CurvatureAnalysis](https://docs.mcneel.com/rhino/5/help/en-us/commands/curvatureanalysis.htm); [Zebra](https://docs.mcneel.com/rhino/5/help/en-us/commands/zebra.htm) |
| [RCORE-09 - Associative History và dependency identity](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/09-history-dependencies.md) | 98 / 90 (chỉ ghi chú dimension) | [History / HistoryUpdate / HistoryPurge](https://docs.mcneel.com/rhino/5/help/en-us/commands/history.htm); [Loft (history-enabled example)](https://docs.mcneel.com/rhino/5/help/en-us/commands/loft.htm); [Undo / Redo](https://docs.mcneel.com/rhino/5/help/en-us/commands/undo.htm) |
| [RCORE-10 - Persistence, 3DM và clipboard](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/10-persistence-3dm-clipboard.md) | Không gán Guide thành đặc tả đầy đủ của nhóm này | [Save / -Save](https://docs.mcneel.com/rhino/5/help/en-us/commands/save.htm); [Import](https://docs.mcneel.com/rhino/5/help/en-us/commands/import.htm); [Export](https://docs.mcneel.com/rhino/5/help/en-us/commands/export.htm) |
| [RCORE-11 - Annotation, Make2D, Layout/Detail và print](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/11-annotation-layout-print.md) | 98-100 / 90-92; 281-284 / 273-276 | [Dim](https://docs.mcneel.com/rhino/5/help/en-us/commands/dim.htm); [Make2D](https://docs.mcneel.com/rhino/5/help/en-us/commands/make2d.htm); [Detail](https://docs.mcneel.com/rhino/5/help/en-us/commands/detail.htm) |
| [RCORE-12 - Display, render và resource lifecycle](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/12-display-render-resources.md) | 39-43 / 31-35; 101-104 / 93-96 | [SetDisplayMode](https://docs.mcneel.com/rhino/5/help/en-us/commands/setdisplaymode.htm); [Mesh](https://docs.mcneel.com/rhino/5/help/en-us/commands/mesh.htm); [Render / render-window commands](https://docs.mcneel.com/rhino/5/help/en-us/commands/render.htm) |

Dải Guide chính: object model 25-31 / in 17-23; selection 33-38 / in 25-30;
viewport 39-43 / in 31-35; accurate modeling 45-51 / in 37-43;
dựng mặt 53-67 / in 45-59; NURBS edit 69-71 / in 61-63;
transforms 73-91 / in 65-83; analysis 93-96 / in 85-88;
organization/annotation 97-100 / in 89-92; render 101-104 / in 93-96;
layout tutorial 281-284 / in 273-276. Mục lục và phần mở đầu command ở PDF 11-23 / in 3-15.

**RCORE-02**: Units Help là nguồn của units/tolerance contract; layout example không xác định default chung.
**RCORE-09**: PDF 98 chỉ ghi chú dimension dùng History; đó không phải đặc tả History engine.
**RCORE-10**: không tìm/gán contract 3DM preservation hay binary clipboard trong các phần Guide được index.
Cả ba nhóm vẫn cần exact help/API và native fixture, không dựng coverage giả từ tên tutorial hoặc việc mở file .3dm.

## Đọc tiếp và kiểm theo chương

### RCORE-01

**Đọc tiếp:** Đọc representation trong Guide trước; chọn operation × input/output type trong chương 01, rồi đọc exact command help và API backend tương ứng.

**Kiểm:** Đối chiếu rational arc, face có lỗ, mặt trụ mở và CAD kèm mesh. Đo locus/weights, trim/area, closedness và representation trước-sau; preview/commit phải cùng contract. Chọn ca `RCORE-01.T01`, `RCORE-01.T02`, `RCORE-01.T03`, `RCORE-01.T04` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/01-object-model-kernel.md).

**Khoảng trống cần giữ:** Không suy kernel intersection/Boolean/refit hoặc bảo toàn parameterization từ guide/openNURBS, render mesh hay tên BRep.

### RCORE-02

**Đọc tiếp:** Đọc Units properties trước và chapter 02 context; lấy units/tolerance từ document/template cụ thể trong Rhino 5, rồi xem API host cho scale/tolerance của đúng operation.

**Kiểm:** So mm/inch và đổi precision; thử endpoint gap quanh τ cho đúng Join variant; đo geometry/context/Undo khi đổi units ở từng scale mode. Không áp rule Join cho mọi solver. Chọn ca `RCORE-02.T01`, `RCORE-02.T02`, `RCORE-02.T03`, `RCORE-02.T04`, `RCORE-02.T13` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/02-units-tolerances.md).

**Khoảng trống cần giữ:** Guide/layout tutorial không định nghĩa shared tolerance context. Default tài liệu help, template, setting hiện tại và ngưỡng solver OM9 phải ghi riêng.

### RCORE-03

**Đọc tiếp:** Đọc PDF 50 / in 42 và Cursor constraints trước; kiểm grammar với CPlane dịch/xoay. Chọn latch/revision theo family và đọc checkpoint mới trước khi sửa parser.

**Kiểm:** Dùng O=(10,20,30), X=(0,1,0), Y=(0,0,1), Z=(1,0,0): 1,2,3 phải đo world (13,21,32); thử w/r/wr với previous point reset, đổi view trước/sau first pick và lưu binary/source hash. Chọn ca `RCORE-03.T01`, `RCORE-03.T02`, `RCORE-03.T03`, `RCORE-03.T04`, `RCORE-03.T08` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/03-coordinates-cplanes.md).

**Khoảng trống cần giữ:** Guide xác nhận x,y và x,y,z theo CPlane; không dùng native record Circle cũ để kết luận binary hiện tại dùng world XYZ. UI CPlane còn pending theo checkpoint.

### RCORE-04

**Đọc tiếp:** Đọc prompts/options/repeat trên PDF 22 và exact feature command help; đối chiếu aliases/macros và Undo với command session OM9.

**Kiểm:** Chạy cùng Circle/Line từ menu/CMD/F6/shortcut; thử Esc theo phase, repeat sau success/error/cancel, session Undo so document Undo và document-close/late-worker cleanup. Chọn ca `RCORE-04.T01`, `RCORE-04.T02`, `RCORE-04.T04`, `RCORE-04.T05`, `RCORE-04.T08` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/04-command-engine.md).

**Khoảng trống cần giữ:** Command transcript, successful repeat candidate, point/session Undo, document Undo và associative History là các loại state riêng. Repeat whitelist P0 không đóng toàn command engine.

### RCORE-05

**Đọc tiếp:** Đọc marker/cursor và persistent/one-shot snaps trong Guide, rồi Object snaps và Cursor constraints cho đúng mode; xác định document/CPlane/base/previous point và phase.

**Kiểm:** Đo accepted point và marker trên fixture CPlane xoay; previous/base khác nhau, Ortho/Planar/Project và visible/locked/hidden. Ghi setting thật và source/subelement identity; không tự đặt precedence còn chưa có evidence. Chọn ca `RCORE-05.T01`, `RCORE-05.T02`, `RCORE-05.T11`, `RCORE-05.T12`, `RCORE-05.T14` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/05-picking-snapping.md).

**Khoảng trống cần giữ:** Lifecycle one-shot/suspend trong checkpoint không chứng minh geometric snapping, ranking hay precedence Project/Ortho/SmartTrack/Elevator/SnapToLocked.

### RCORE-06

**Đọc tiếp:** Đọc PDF 34 window/crossing, PDF 97 locked-but-snappable/current-layer và PDF 98 definition/instance/reference; đọc exact help theo authoring hoặc reference intent.

**Kiểm:** Thử object cắt biên window, locked-visible layer và layer xóa giữa preview/commit; hai instance chung definition, Explode một instance và Worksession reference dùng creation input nhưng chặn Move. Chọn ca `RCORE-06.T01`, `RCORE-06.T03`, `RCORE-06.T04`, `RCORE-06.T06`, `RCORE-06.T08`, `RCORE-06.T10` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/06-selection-organization-blocks.md).

**Khoảng trống cần giữ:** Group/App::Link/import flatten không chứng minh block editor hay Worksession. Chương 06 vẫn not_run trong CAPABILITY_LOOKUP dù có bounded identity/I/O checkpoint liên quan.

### RCORE-07

**Đọc tiếp:** Đọc dựng mặt/CV/transforms theo slice; lấy đúng exact command help kể cả options U/V, degree/count, seam, retrim và fitting tolerance; tra primitive Part/Surface trước khi chọn adapter.

**Kiểm:** Đo locus/basis sau knot/degree, cancel/Undo grip edit, trim hole sau Rebuild, seam/order của Loft và narrow spike của Fit; báo numerical bound/budget và continuity riêng. Chọn ca `RCORE-07.T01`, `RCORE-07.T03`, `RCORE-07.T05`, `RCORE-07.T07`, `RCORE-07.T08`, `RCORE-07.T10` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/07-nurbs-editing-modeling.md).

**Khoảng trống cần giữ:** Tutorial success, valid basis hay coarse sampled deviation không chứng minh shared solver, G0/G1/G2, knot/domain semantics hoặc full NURBS editor.

### RCORE-08

**Đọc tiếp:** Đọc PDF 94-96 và đúng help numeric vs visual vs mutation; chọn measurable diagnostic và repair policy trong chapter 08.

**Kiểm:** Dùng known curvature/naked seam/draft CPlane và invalid preconditions; đo before/after topology/deviation khi RebuildEdges hoặc repair, reject không mutation/Undo giả; giữ diagnostic read-only theo contract. Chọn ca `RCORE-08.T01`, `RCORE-08.T04`, `RCORE-08.T06`, `RCORE-08.T07`, `RCORE-08.T11`, `RCORE-08.T12` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/08-analysis-repair.md).

**Khoảng trống cần giữ:** Zebra/EMap/shaded không tự chứng nhận G2, closedness hoặc archive integrity; validity/BOP coverage và skipped/not_run phải giữ rõ. PDF liên kết Audit3dmFile tới audit.htm.

### RCORE-09

**Đọc tiếp:** Đọc History help và replay/API của đúng command family trước; dùng Guide 98 chỉ để định vị ghi chú dimension. Đọc P0 checkpoint và policy OM9 Record/Update/Detach riêng.

**Kiểm:** Thử hai thế hệ, topology change, Record/Update matrix, child lock/clear, Undo/delete và cold restore; ghi family/version/IDs/units/frame, trạng thái dirty/failed và replay metadata; Rhino và OM9 decision so riêng. Chọn ca `RCORE-09.T01`, `RCORE-09.T03`, `RCORE-09.T04`, `RCORE-09.T05`, `RCORE-09.T08`, `RCORE-09.T11`, `RCORE-09.T12` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/09-history-dependencies.md).

**Khoảng trống cần giữ:** Guide không đặc tả graph/schema/migration/identity. OM9 Detach/Undo hoặc dirty-state policy là quyết định OM9; không suy giống HistoryPurge hoặc Rhino queue từ FreeCAD recompute.

### RCORE-10

**Đọc tiếp:** Đọc exact Save/Import/Export/Clipboard help, file format/version và openNURBS/API; chọn type/field read/edit/retain/export support table của checkout này rồi fixture.

**Kiểm:** Import mixed rational CAD/trim/solid/mesh; sửa A, xóa B, tạo C rồi xuất/mở/edit/save thực trong Rhino 5 và reimport; kiểm units/table remap/nested transform, cold FCStd và clipboard riêng từng chiều. Chọn ca `RCORE-10.T01`, `RCORE-10.T02`, `RCORE-10.T03`, `RCORE-10.T05`, `RCORE-10.T06`, `RCORE-10.T08`, `RCORE-10.T10`, `RCORE-10.T12` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/10-persistence-3dm-clipboard.md).

**Khoảng trống cần giữ:** Guide hoặc archive version 5/parser reread không chứng minh Rhino-readable edited output, units, structural blocks, userdata/History hay hai chiều binary clipboard. FCStd khác 3DM.

### RCORE-11

**Đọc tiếp:** Đọc Guide dimension/space và layout tutorial; đọc Make2D projection option, Detail scale/lock và Print help trước khi nối Draft/TechDraw/OM9 Layout Tools.

**Kiểm:** So snapshot/History dimension, Dot pixel size; Make2D trên CPlane dịch/xoay; Detail parallel paper:model 2:1 đo line 20 mm thành 40 mm trên PDF, lock pan/zoom và perspective-scale gate. Chọn ca `RCORE-11.T01`, `RCORE-11.T03`, `RCORE-11.T04`, `RCORE-11.T06`, `RCORE-11.T07`, `RCORE-11.T11` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/11-annotation-layout-print.md).

**Khoảng trống cần giữ:** Layout/Make2D đã có spec; guide overview world XY không ghi đè help Current CPlane. Ví dụ page 11×8.5 inch và scale 1:1 không phải defaults chung.

### RCORE-12

**Đọc tiếp:** Đọc modes và scene workflow trong Guide; đọc renderer/material/environment help và backend capability trước khi tạo UI/job/cache contract.

**Kiểm:** Đổi display/mesh density mà CAD không đổi; thử revision/Undo cache, material inheritance, missing/relink texture và stale/cancel/close. Phân biệt capture/renderer; output dimensions/channels và write errors phải đo. Chọn ca `RCORE-12.T01`, `RCORE-12.T02`, `RCORE-12.T04`, `RCORE-12.T05`, `RCORE-12.T06`, `RCORE-12.T08`, `RCORE-12.T13` trong [chương](../../OpenMatrix9_Codex_Spec_v1/specs/00-rhino-core/12-display-render-resources.md).

**Khoảng trống cần giữ:** Viewport capture, EMap hoặc Rendered label không chứng minh production renderer; mesh/cache/resource lifecycle và plugin availability cần fixture/backend evidence riêng.

## Mức xác minh của index - 2026-10-09

- **PDF:** bundled pypdf 6.10.0; đã đối chiếu mục lục, heading/footer của tất cả dải trên
  và đọc các prose anchors liên quan. Poppler render và xem trực tiếp PDF 50, 95, 98, 283.
  JSON ghi riêng các trang prose đã đọc, dải heading/footer và trang đã xem hình.
  Đây không phải audit mọi tutorial, option/default, API hoặc thuật toán solver.
- **Web:** Guide HTML trả nội dung. **50 URL Help/topic riêng**, **44 trả nội dung**, **6 fetch thất bại**:
  NamedCPlane, Enter key, BlockEdit, Worksession, List, GroundPlane.
  URL thất bại vẫn là hyperlink gốc trong PDF hoặc Help chính thức; JSON ghi provenance và kết quả fetch,
  không gán `content_verified`. Dấu * ở bảng nhắc URL chưa fetch được.
  Fetch thành công xác nhận page identity/accessibility, chưa đóng full option/default contract.
  Audit3dmFile dùng [Audit Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/audit.htm),
  đúng đích hyperlink ở PDF 96.
- **Runtime:** không chạy Rhino/FreeCAD trong lần lập index này.
  Các baseline `source_inspected/not_run` và **checkpoint P0 mới có bounded native validation**
  được giữ riêng. Runtime status snapshot trong JSON lấy từ CAPABILITY_LOOKUP tại thời điểm lập index;
  đọc report hiện tại để biết assertion/slice thật. RCORE-06/08/11/12 vẫn `not_run` theo lookup;
  các chương có `partial_native_validation` cũng chưa pass toàn bộ fixture.

Khi triển khai, cập nhật evidence ở spec/status/report chính theo
[IMPLEMENTATION_RULES](../../OpenMatrix9_Codex_Spec_v1/IMPLEMENTATION_RULES.md).
Index này chỉ hỗ trợ tìm đúng nguồn và phép kiểm; không thay capability/fixture hay nâng trạng thái native.
Không đưa code decompile, Matrix PDF hoặc bản trích dài/screenshot nguồn vào index.
