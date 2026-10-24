# Kế hoạch cập nhật milestone: tích hợp SDK trong từng owner

Ngày cập nhật: **2026-10-21**. **Milestone là đơn vị lập kế hoạch và thực thi; SDK là một phần công việc của từng milestone/L2/L3.** File này tập hợp các thay đổi đã ghi trực tiếp vào `sdk_integration_plan` của 11 L1 và 102 L2 canonical. Không có lộ trình SDK độc lập để làm sau milestone.

## Nguyên tắc thực hiện trong milestone

1. **Tính năng đã có, thiếu SDK:** bổ sung việc DTO/API/host adapter/plugin proof ngay vào owner hiện tại, tái sử dụng Rust service; giữ nguyên bằng chứng core đã có.
2. **Tính năng chưa có:** thực hiện service dùng chung, app adapter và SDK adapter trong cùng lần triển khai L3. Phần SDK thuộc điều kiện hoàn thành của tính năng có thể mở cho plugin.
3. **Chưa rõ mức triển khai:** audit code/evidence tại owner trước; sau đó ghi việc bổ sung SDK hoặc implementation kèm SDK. Không tự đánh dấu mọi L3 là chưa làm.
4. **Nội bộ host/quản trị:** ghi lý do không xuất API; nếu cần mở cho plugin thì cung cấp service/DTO có giới hạn. Không cấp document/GPU/native pointer cho plugin.
5. **Dependency và nghiệm thu:** giữ nguyên canonical IDs, thứ tự/dependency và automated/manual contracts. Không dùng việc thêm checklist để nâng runtime status, ký manual review hoặc đóng M09.
6. **Đa nền tảng:** cùng source, build binary theo target, đóng chung `.om9p` nếu có binary thật. Windows và macOS có evidence riêng; không coi egui/wgpu hoặc target manifest là bằng chứng Mac đã chạy.

Ở lượt lập kế hoạch (2026-10-21) chưa triển khai thêm Rust/native API và chưa thay business rules trong skill. Lượt thực hiện đầu tiên theo mục “Điểm bắt đầu” được ghi ở **Nhật ký thực hiện** và tại đúng owner; mọi phần còn lại vẫn là kế hoạch, không phải bằng chứng runtime.



## Cách đọc trạng thái

Theo dõi **năng lực nội bộ** và **khả năng gọi từ plugin** thành hai trục riêng:

| Nhãn nội bộ | Ý nghĩa và giới hạn |
|---|---|
| `EVIDENCE-SCOPED` | Có code và bằng chứng standalone trong phạm vi được nêu; không mặc nhiên đóng toàn bộ L2/L3 hoặc manual gate. |
| `SOURCE-PRESENT` | Có implementation nội bộ được nhận diện; phải kiểm thử đúng phạm vi trước khi xuất SDK. |
| `RUNTIME-CLAIM` | JSON owner có `runtime_implementation`; cần đọc evidence và kiểm tra revision, target, phạm vi. Đây là đầu mối, không phải xác nhận mới. |
| `AUDIT-REQUIRED` | Chưa đủ đối chiếu trong đợt này. Có thể đã có một phần code; không gọi là hoàn toàn chưa làm. |
| `NOT-IMPLEMENTED-VERIFIED` | Chỉ dùng sau khi tìm đúng owner, command, backend, test và xác nhận thiếu; phải ghi căn cứ. Chưa áp nhãn này hàng loạt. |

| Nhãn SDK | Ý nghĩa và cách xử lý |
|---|---|
| `SDK-SUBSET` | Đã gọi thật qua SDK v1 nhưng mới một phần của năng lực owner. Liệt kê chính xác API có sẵn. |
| `EXPORT-BACKLOG` | Đã nhận diện dịch vụ nội bộ; thiếu public DTO/host dispatch/capability hoặc kiểm chứng plugin. |
| `CO-IMPLEMENT` | Khi triển khai phần còn thiếu, bắt buộc kèm adapter SDK cho phần có thể dùng lại. |
| `AUDIT-THEN-ROUTE` | Kiểm tra code trước, sau đó chuyển sang `EXPORT-BACKLOG`, `CO-IMPLEMENT` hoặc `HOST-INTERNAL` với lý do. |
| `HOST-INTERNAL` | Giữ ownership trong host; xuất service façade nếu plugin có nhu cầu, không xuất mutable document/GPU/native pointer. |
| `GATE-ONLY` | Quản trị, licensing hoặc nghiệm thu; tạo tiêu chí/evidence, không tạo hàm geometry. |

Trong bảng L3 của từng milestone, `runtime` chỉ phản ánh có metadata tại thời điểm chốt. `subset` chỉ dùng cho những lát cắt đã biết; các L3 khác không bị tự động nâng thành đã tích hợp. Một owner có thể chứa cả backlog và phần phải làm mới.

## Nguồn đối chiếu và đường dẫn làm việc

- [SDK hiện có](../crates/om9-sdk/src/lib.rs), [host adapter](../crates/om9-plugin-host/src/lib.rs), [tích hợp Controller](../crates/om9-app/src/session/plugins.rs).
- [Danh mục code có thể tái sử dụng](../sdk/API-INVENTORY.md), [hướng dẫn SDK](../sdk/README.md), [thiết kế boundary](../implementation/design/M01/PLUGIN-SDK.md).
- [Bằng chứng SDK Windows](../docs/validation/2026-10-21-plugin-sdk.md), [dữ liệu kiểm chứng SDK](../docs/validation/2026-10-21-plugin-sdk.json).
- [Live ledger](../docs/openmatrix9-progress.json), [M01 foundation](<../OM9_Milestones/M01 - CAD Shell, Viewport/L1.json>), canonical owner trong các bảng bên dưới.
- [Rhino → OM9 skill](../skills/openmatrix9-plugin-port/SKILL.md). Skill chỉ ánh xạ API đã có; API đang đề xuất phải báo thiếu và trỏ về backlog này.

Snapshot catalog có **11 L1, 102 L2, 791 L3 JSON vật lý**. Đây là số lượng đặc tả, không phải số năng lực đã hoàn thành. M09 có hai release gate, không có L2/L3 mới; 30 L4 của M09 là kế hoạch kiểm định riêng. L3 STL `OM9-L3-01-12-008` nằm trong thư mục M01 nhưng `target_milestone=M07`; thực thi theo đích và dependency, không chỉ theo thư mục.

## Baseline SDK dùng để cập nhật các milestone

| Public API hiện có | Phạm vi đã xuất | Khoảng trống phải theo dõi |
|---|---|---|
| `capabilities()` | Hỏi capability host hiện tại. | Chuẩn hóa version/schema/dependency theo từng capability khi mở rộng. |
| `document_info()` | ID/name/revision/current layer, ký hiệu đơn vị hiển thị. | Tolerance, snapshot context, vòng đời document. Geometry vẫn tính bằng mm. |
| `document_units()`, `parse_length()`, `parse_angle()`, `parse_coordinates()` | Đơn vị hiển thị + mm/unit; tolerance do tài liệu nêu (`None` nếu không nêu); parse số có đơn vị ra mm/độ; tuple toạ độ pre-CPlane. Thêm 2026-10-21 tại owner M01 `01-08`. | Tolerance setter, CPlane local→world, typed builder parameter values/range/serialization. |
| `objects()` | Metadata đối tượng và placement sao chép. | `query_objects` đã bù paging/filter/revision; còn bbox và blocks/instances. |
| `query_objects()`, `object_attributes()`, `set_object_attributes()`, `rename_object()` | Query filter/paging kèm generation và geometry/attributes revision; attributes read/write và rename staged qua transaction/permission của owner; một command một loại source write mỗi object. Thêm 2026-10-21 tại owner M01 `01-03`. | Groups/blocks, change notification, snapshot/restore, import provenance. |
| `add_ellipse()`, `add_rectangle()`, `add_polygon()`, `add_circle_on_frame()` | Dựng ellipse hữu tỷ bậc 2, rectangle 4 đoạn khép kín và polygon khép kín (3+ điểm, không cạnh suy biến) qua builder của owner; frame sai/radius-size ≤0/điểm trùng bị từ chối `curve`; một transaction/Undo. Thêm 2026-10-21 tại owner M03 `02-04`. | Spiral/helix, edit/offset/fillet/blend/rebuild, periodic/seam NURBS. |
| `set_object_placement()`, `add_mesh()` | Ghi matrix placement affine của một object qua `Mutation::Placement` (validate hữu hạn/affine/khả nghịch) và tạo polygon mesh từ vertices/faces qua `Mesh::validate`; one-source-write/object; object tạo trong cùng command giữ placement qua insert. Thêm 2026-10-21 tại owner M01 `01-03`. | Nested/instance transform, gumball drag, mesh normals/colors/UV, edit/boolean/repair, change feed. || `last_change()` (extended) | Feed ghi thêm host Undo/Redo (`ChangeBatch::between` trước/sau) với cause `undo`/`redo`; revision thật. Thêm 2026-10-21 tại owner M01 `01-03`. | Các host edit path khác, multi-batch history. || `object_info()`, `ObjectInfo` | Đọc một record object (id/label/kind/placement/flags/revisions) không cần page; dùng chung helper với query. Thêm 2026-10-21 tại owner M01 `01-03`. | Instance composition, gumball drag. || `object_placement()`, `ObjectPlacement` | Đọc matrix placement local (kể cả giá trị staged trong command); không compose parent/instance. Thêm 2026-10-21 tại owner M01 `01-03`. | Instance/parent composition, gumball drag. || `clear_current_layer()` | Xoá pointer current layer (layer record giữ nguyên); no-op nếu đã trống, Undo khôi phục. Thêm 2026-10-21 tại owner M01 `01-03`. | Layer delete/reorder. || `block_definitions()`, `BlockDefinitionInfo` | Đọc definition: id/label/member count/revision; read-only. Thêm 2026-10-21 tại owner M01 `01-03`. | Instance insertion, definition editing, instance graph. || `set_document_tolerance()` | Ghi tolerance policy (absolute/relative/angular) qua `Mutation::Tolerance`; invalid → `tolerance`; một Undo khôi phục. Thêm 2026-10-21 tại owner M01 `01-08` (host-tested). | Per-object tolerance. || `is_object_pickable()`, `IsObjectPickable` | Trả `effective_visible && effective_selectable` từ `object_attributes`; `locked` chỉ chặn sửa, không chặn pick. Thêm 2026-10-22 tại owner M01 `01-03`. | Pick geometry (ray/subobject). || `object_provenance()`, `ObjectProvenance` | Đọc proof nhập nguồn đầy đủ: source name/sha256 hex/version/mode/kind/uuid/index/family/disposition; chain thiếu → `null`; read-only. Thêm 2026-10-22 tại owner M01 `01-03`. | Ghi provenance (import flow của host). || `layer_info()`, `LayerInfo` | Đọc một layer record (name, effective visible/selectable/locked, current, color) qua cùng helper với list. Thêm 2026-10-21 tại owner M01 `01-03` (host + native). | Layer delete/reorder. || `delete_group()`, `GroupDelete` | Xoá group record (member giữ nguyên) qua `Mutation::GroupDelete` tại base revision; một Undo khôi phục; `groups()` lọc deletion staged. Thêm 2026-10-21 tại owner M01 `01-03`. | Group edit, member add/remove. || `create_group()`, `GroupCreate` | Tạo một group từ member ids tồn tại (`Mutation::GroupCreate` tại base revision); một transaction/Undo; staged group hiện trong `groups()`. Thêm 2026-10-21 tại owner M01 `01-03`. | Group edit/dissolve, member add/remove. || `groups()`, `GroupInfo` | Đọc group record: id/name/member ids (thứ tự document)/revision; read-only. Thêm 2026-10-21 tại owner M01 `01-03`. | Group create/edit/dissolve, block instance graph. || `measure_mesh_area()`, `MeasuredArea` | Tổng diện tích tam giác fan của mesh (object-local mm²); non-mesh `unsupported`. Thêm 2026-10-21 tại owner M03 `03-20`. | Volume, NURBS/surface area. || `layer_order()` | Thứ tự layer bottom→top (ids), gồm layer tạo trong cùng command; read-only. Thêm 2026-10-21 tại owner M01 `01-03`. | Layer reorder, group records, block instance graph. || `report_measurement()`, `MeasureLabel` | Một label đo đạc dạng Progress line (≤512 byte, ≤32/command) qua event log; text rỗng/quá dài là `message`; command fail thì report bị bỏ. Thêm 2026-10-21 tại owner M03 `03-20`. | Structured measurement record/annotation, UI acceptance. || `object_summary()`, `ObjectSummary` | Đếm object theo geometry kind trên state staged (base + add - delete); kind không có thì vắng mặt. Thêm 2026-10-21 tại owner M03 `03-20`. | Jewelry semantic class/weight summary, grouped counts. || `measure_curve_radius()`, `MeasuredDistance` | Bán kính native circle/arc từ record (object-local mm); line/NURBS/polycurve `unsupported`. Thêm 2026-10-21 tại owner M03 `03-20`. | Ellipse radius, area/volume, statistics. || `measure_curve_length()`, `MeasuredDistance` | Chiều dài curve sampled qua 33 mẫu (tổng chord, object-local mm); non-curve `unsupported`; là xấp xỉ, không exact. Thêm 2026-10-21 tại owner M03 `03-20`. | Exact/adaptive length, radius/area/volume, statistics. || `object_bounds()`, `ObjectBounds` | Box/size/center object-local từ point, 33 mẫu curve hoặc mesh vertices; kind khác `unsupported`; không áp placement. Thêm 2026-10-21 tại owner M03 `03-20`. | Exact analytic bounds, B-Rep/surface/cloud, group bounds/count summary. || `curve_segments()`, `evaluate_curve_segment()`, `CurveSegment` | Liệt kê segment của polycurve (hoặc 1 segment của native curve) và evaluate theo index với cùng rule local-mm/D0-D3/1024 mẫu; nested polycurve hoặc index ngoài range bị từ chối. Thêm 2026-10-21 tại owner M03 `02-04`. | Segment explode/split edit, nested flatten, arc-direction edit. || `last_change()`, `ProjectChange`, `ChangeEntry` | Batch thay đổi của **plugin command trước** (from/to revision thật, cause edit, entity/layer entries) do Controller ghi tại ranh giới plugin; host command ngoài ranh giới không nằm trong feed (dùng `document_info().revision`). Thêm 2026-10-21 tại owner M01 `01-03`. | Host-command feed, multi-batch history, group/feature producer entries. || `object_extensions()`, `set_object_extension()`, `clear_object_extension()`, `ObjectExtension` | Đọc/ghi một slot extension (payload JSON-object nguyên văn, version >0, slot ≤64 byte) qua `Mutation::Extension`; one-source-write-per-object nên xung đột với placement/attributes/label/layer; object tạo trong cùng command phải tag ở command sau. Thêm 2026-10-21 tại owner M01 `01-03`. | Extension schema interpretation, same-command tagging, change notification feed. |
| `project_state()`, `save_project()`, `save_project_as()`, `ProjectInfo` | Đọc path/dirty của host và yêu cầu atomic save **sau khi command commit** qua `begin_file_operation(FileKind::Save)`, hoặc mở Save As dialog của host để user chọn đích; untitled bị từ chối trước commit, save fail sau commit là warning và document vẫn dirty; save và save-as không kết hợp; plugin không thể open/create/replace active document. Thêm 2026-10-21 tại owner M01 `01-12`. | Interactive path-dialog UI acceptance, write-failure recovery qua plugin path, load validation/reload. |
| `measure_distance()`, `measure_angle()`, `MeasuredDistance`, `MeasuredAngle` | Khoảng cách 2 điểm (mm, bound 1e9) và góc không hướng giữa 2 đường (độ, 0..180) qua `measure::distance_mm`/`angle_degrees`; đo không stage mutation. Thêm 2026-10-21 tại owner M03 `03-20`. | Đo object-to-object, length/radius/area/volume, bounding size/center, label/feedback. |
| `add_arc()`, `add_interpolated_curve()`, `add_nurbs_curve()`, `KnotMode` | Dựng arc qua 3 điểm (angle domain tăng dần), NURBS nội suy (degree lẻ 1..=11, chord/uniform) và NURBS control-point (1 weight/pole, poles+degree+1 knots, clamped domain) qua builder/validate của owner; stage một object, permission/budget/single-Undo theo document. Thêm 2026-10-21 tại owner M03 `02-04`. | Periodic/seam NURBS, ellipse/polygon, edit/offset/fillet/blend/rebuild/intersections, polycurve segment, surface/BRep. |
| `curve_definition()`, `curve_semantics()`, `CurveDefinition`, `CurveSemantics` | NURBS poles/weights/knots/degree/domain/periodic/reversed/seam, arc angle domain, polycurve segment count (bounded); closed/planar/endpoints dưới tolerance tường minh hoặc document, từ chối khi thiếu tolerance hoặc curve degenerate. Thêm 2026-10-21 tại owner M01 `02-01`. | Control write, curve construction/edit/offset/fillet/blend/rebuild/intersections, polycurve segment, surface/trim/blocks. |
| `curve_info()`, `evaluate_curve()`, `CurveInfo`, `CurveSample` | Curve line/circle/NURBS: kind, domain, degree/poles/periodic/reversed; evaluation tại tham số native với D1-D3, toạ độ object-local mm, ≤1024 mẫu; polycurve cần segment parameter nên bị từ chối. Thêm 2026-10-21 tại owner M01 `02-01`. | NURBS weights/knots write, polycurve segment, curve edit/offset/fillet/blend/rebuild/intersections, surface/trim/blocks. |
| `set_selection()`, `add_to_selection()`, `selected_objects()` | Replace/extend selection qua Controller: id canonical, tồn tại, effective selectable ở host; scene root + pick geometry kiểm tra trước commit; selection không phải document state, không transaction/Undo; command lỗi giữ selection cũ. Thêm 2026-10-21 tại owner M02 `01-07`. | Typed subobject/occurrence, pick query, window/crossing, drag/highlight. |
| `progress()`, `warning()`, `MessageLevel`, `PluginMessage` | Báo progress/warning một dòng vào EventLog của app (Started/Progress/Warn/Finished), cùng surface CMD và observer dùng; 32 message × 512 byte mỗi command; không tạo transaction/Undo; command lỗi không publish gì kể cả report. Thêm 2026-10-21 tại owner M01 `01-02`. | Interactive prompt/input sessions, option tokens, preview overlay, dynamic menu, observer callback; báo cáo trên nhánh lỗi. |
| `validate_parameter()`, `apply_parameters()`, `Dimension`/`ParameterDomain`/`ParameterSpec`/`ParameterValue` | Typed builder/command parameter contract: wire tag = đơn vị, bounds canonical, step/count/choice/text, typed default, required/unknown; dùng `OptionDomain::check` + `CommandSchema::apply`; một call ≤128 khai báo/giá trị. Thêm 2026-10-21 tại owner M01 `01-08`. | Builder parameter editor/UI parity (M04/M01-02); tolerance editing; CPlane frame. |
| `layer_create()`, `layer_update()`, `set_current_layer()`, `assign_objects_to_layer()` | Layer plan qua `LayerPlan::capture`: tạo/đổi tên/màu/cờ, chọn current, gán object; staged record đọc bằng `layer_permissions_with`; output vào layer tạo trong cùng request được chấp nhận; giữ rule một layer write/layer/command và rule chống đọc permission cũ. Thêm 2026-10-21 tại owner M01 `01-03`. | Layer delete/reparent/preset-slot; `Mutation::Publish`/cross-document Move vẫn dùng base layer; groups/blocks. |
| `selected_objects()` | Đọc ID đối tượng gốc đang chọn. | Selection write, query filters, subobjects/occurrences, interactive picking. |
| `layers()` | Đọc layer. | Create/rename/reparent, current layer, visibility/lock/selectability, attributes. |
| `object_geometry(id)` | Point/Line/Polyline/full Circle trong tọa độ object-local mm; placement riêng. | Arc/NURBS/Surface/Mesh/BRep/Block và revision-scoped handles. |
| `add_object`, `add_point`, `add_line`, `add_polyline`, `add_circle` | Tạo các DTO hình học đơn giản được hỗ trợ, host nhận publish. | Các loại hình khác; replace/edit geometry và provenance. |
| `delete_object` | Xóa qua transaction của host. | Cascade/dependency policy theo History; không mặc định xóa con. |
| `transform_object` | Ma trận row-major, world delta nhân trước placement. | Copy/instance/orient/pattern, nonuniform và mirror policy theo loại hình. |
| `Plugin`, `export_plugin!`, ABI v1 | Đăng ký plugin/command và thực thi qua C entry, JSON có giới hạn. | Dynamic menu/panel/input sessions/events/async/persistent plugin data. |

Host đã có loader, kiểm tra gói/target/hash, command staging, transaction, `PluginLoad`/`PluginRun` và tool pack/inspect/sdk-bundle. Những phần đó không đồng nghĩa public plugin đã có quyền gọi mọi phương thức nội bộ của core.

Bằng chứng kế thừa: 509 test core/scene/SDK/host trong 73 suite; 4 Controller test; 3 DLL test. Các tập có chồng lặp, không cộng thành tỷ lệ hoàn thành. Plugin build từ SDK ZIP ngoài checkout đã chạy trong OM9 Windows, tạo ba đối tượng; lưu/mở lạnh và lỗi/cancel được kiểm tra theo báo cáo. macOS chưa có evidence. Broad app còn hai lỗi dock reset trong báo cáo trước; không ghi workspace toàn bộ xanh.

## Các milestone và checklist SDK đã hợp nhất

Mỗi chương dưới đây là phần SDK bổ sung của chính milestone đó: owner, baseline, việc cần làm, chi tiết kỹ thuật và sổ L3. `sdk_integration_plan` chỉ thêm requirements; không thay nội dung nghiệp vụ hoặc lịch sử runtime của owner.

## M00 — Feasibility, QymCAD source, OCCT, legal and environment

Nguồn: [L1 canonical](<../OM9_Milestones/M00 - Feasibility, Legal/L1.json>). 7 L2 / 19 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: không có.

Ghi điều kiện license, source, toolchain, native portability và CI cần cho SDK ngay trong các owner khả thi hiện có. Không phát sinh geometry API từ nhiệm vụ quản trị.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-M00-L2-01](<../OM9_Milestones/M00 - Feasibility, Legal/L2-01 - QymCAD/L2.json>) — FEASIBILITY_GATE_ONLY_NO_IMPLEMENTATION_OWNERSHIP | AUDIT-REQUIRED governance | GATE-ONLY | Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị. |

| [OM9-M00-L2-02](<../OM9_Milestones/M00 - Feasibility, Legal/L2-02 - OCCT/L2.json>) — FEASIBILITY_GATE_ONLY_NO_IMPLEMENTATION_OWNERSHIP | AUDIT-REQUIRED governance | GATE-ONLY | Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị. |

| [OM9-M00-L2-03](<../OM9_Milestones/M00 - Feasibility, Legal/L2-03 - UI/L2.json>) — FEASIBILITY_GATE_ONLY_NO_IMPLEMENTATION_OWNERSHIP | AUDIT-REQUIRED governance | GATE-ONLY | Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị. |

| [OM9-M00-L2-04](<../OM9_Milestones/M00 - Feasibility, Legal/L2-04 - 3DM/L2.json>) — FEASIBILITY_GATE_ONLY_NO_IMPLEMENTATION_OWNERSHIP | AUDIT-REQUIRED governance | GATE-ONLY | Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị. |

| [OM9-M00-L2-05](<../OM9_Milestones/M00 - Feasibility, Legal/L2-05 - Associative/L2.json>) — FEASIBILITY_GATE_ONLY_NO_IMPLEMENTATION_OWNERSHIP | AUDIT-REQUIRED governance | GATE-ONLY | Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị. |

| [OM9-M00-L2-06](<../OM9_Milestones/M00 - Feasibility, Legal/L2-06 - License/L2.json>) — FEASIBILITY_GATE_ONLY_NO_IMPLEMENTATION_OWNERSHIP | AUDIT-REQUIRED governance | GATE-ONLY | Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị. |

| [OM9-M00-L2-07](<../OM9_Milestones/M00 - Feasibility, Legal/L2-07 - Reproducible/L2.json>) — FEASIBILITY_GATE_ONLY_NO_IMPLEMENTATION_OWNERSHIP | AUDIT-REQUIRED governance | GATE-ONLY | Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị. |

### Checklist tích hợp khi thực hiện milestone

#### Checklist: Khóa đối chiếu và hợp đồng mở rộng

- [ ] Chọn một L3 cụ thể có nhu cầu plugin thực tế; đọc owner, source, evidence và phân loại hai trục.
- [ ] Lập danh sách capability đã có từ SDK source; descriptor phải phản ánh availability, không advertise trước khi host dispatch hoạt động.
- [ ] Chốt schema/version negotiation, unknown-field/unknown-method policy; thêm mới tương thích chỉ khi semantics thật sự tương thích. Nếu đổi ABI không tương thích, tăng version và báo lỗi rõ ràng.
- [ ] Thiết kế lỗi có mã/context, request/result size, allocation budget, pagination và cancellation; API nhỏ không được tạo đường vòng qua giới hạn transaction hiện có.
- [ ] Chốt ID document/revision/object/geometry/feature/topology/occurrence; phân biệt stable entity ID với token chỉ hợp lệ trong snapshot.
- [ ] Tạo một ví dụ plugin ngoài workspace cho capability đầu tiên. Không thêm dependency từ SDK vào core/app/kernel.

**Đầu ra:** capability/schema proposal, mapping exact L3, mẫu request/response và test oracle. **Đóng gói:** chưa gọi capability “merged” nếu mới có types.

#### Checklist: macOS, distribution và compatibility, bắt đầu sớm

- [ ] Audit `om9-kernel/build.rs`, `om9-3dm/build.rs`, native memory/process APIs, `om9-native-owner`, file dialogs và packaging. Các đường `.lib`/Windows API hiện có là blocker thực sự, dù UI dùng egui/wgpu.
- [ ] Port build/link/native/process containment theo target; Rust giữ ownership, adapter C++ chỉ cho API kernel cần thiết. Không bỏ containment để tạo build Mac bề ngoài.
- [ ] Build SDK example từng target: Windows x86_64; macOS arm64 và x86_64 chỉ khi từng target có toolchain/dependency và nhu cầu phát hành. Universal binary là bước đóng gói riêng, không giả định tự có.
- [ ] Pack cùng source/version thành nhiều target trong một `.om9p`; inspect manifest/hash/architecture, báo target thiếu rõ ràng. Không đổi extension DLL thành dylib.
- [ ] CI target matrix: SDK source compile, plugin ABI, host native runtime và desktop UI smoke tách riêng. Runtime Mac cần runner/hardware Mac; Windows check không thay thế.
- [ ] Test package install/load/execute/unload, wrong target/ABI, missing dylib, path với Unicode/space, save/reopen và distribution dependencies. Ghi OS/arch/compiler/native versions.

**Đóng:** capability nào được quảng cáo Mac có actual Mac plugin/host runtime evidence. Chưa đạt thì manifest/docs ghi unsupported/unverified cụ thể. Không chờ tất cả domain M05–M08 mới xử lý portability nền tảng.

### Theo dõi L3 trong milestone

#### OM9-M00-L2-01 — 3 L3

Owner: [QymCAD GitHub & source reuse](<../OM9_Milestones/M00 - Feasibility, Legal/L2-01 - QymCAD/L2.json>). Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-M00-L3-01-01](<../OM9_Milestones/M00 - Feasibility, Legal/L2-01 - QymCAD/L3-01 - Repository provenance & pinned/l3.json>) — Repository provenance & pinned checkout | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-01-02](<../OM9_Milestones/M00 - Feasibility, Legal/L2-01 - QymCAD/L3-02 - Workspace map/l3.json>) — Workspace map and dependency audit | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-01-03](<../OM9_Milestones/M00 - Feasibility, Legal/L2-01 - QymCAD/L3-03 - REUSE ADAPT ADD architecture/l3.json>) — REUSE ADAPT ADD architecture decision | AUDIT-REQUIRED | GATE-ONLY |

#### OM9-M00-L2-02 — 3 L3

Owner: [OCCT bridge & numerical geometry](<../OM9_Milestones/M00 - Feasibility, Legal/L2-02 - OCCT/L2.json>). Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-M00-L3-02-01](<../OM9_Milestones/M00 - Feasibility, Legal/L2-02 - OCCT/L3-01 - Toolchain and C ABI bridge proof/l3.json>) — Toolchain and C ABI bridge proof | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-02-02](<../OM9_Milestones/M00 - Feasibility, Legal/L2-02 - OCCT/L3-02 - Lifetime, threading and error safety/l3.json>) — Lifetime, threading and error safety | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-02-03](<../OM9_Milestones/M00 - Feasibility, Legal/L2-02 - OCCT/L3-03 - Geometry numeric invariants/l3.json>) — Geometry numeric invariants | AUDIT-REQUIRED | GATE-ONLY |

#### OM9-M00-L2-03 — 3 L3

Owner: [UI/Viewport/GPU technology](<../OM9_Milestones/M00 - Feasibility, Legal/L2-03 - UI/L2.json>). Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-M00-L3-03-01](<../OM9_Milestones/M00 - Feasibility, Legal/L2-03 - UI/L3-01 - Source UI architecture and constraints/l3.json>) — Source UI architecture and constraints | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-03-02](<../OM9_Milestones/M00 - Feasibility, Legal/L2-03 - UI/L3-02 - GPU renderer stability and performance/l3.json>) — GPU renderer stability and performance | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-03-03](<../OM9_Milestones/M00 - Feasibility, Legal/L2-03 - UI/L3-03 - Picking + transactional selection/l3.json>) — Picking + transactional selection | AUDIT-REQUIRED | GATE-ONLY |

#### OM9-M00-L2-04 — 3 L3

Owner: [3DM / openNURBS interoperability](<../OM9_Milestones/M00 - Feasibility, Legal/L2-04 - 3DM/L2.json>). Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-M00-L3-04-01](<../OM9_Milestones/M00 - Feasibility, Legal/L2-04 - 3DM/L3-01 - 3DM capability gap and dependency/l3.json>) — 3DM capability gap and dependency candidate | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-04-02](<../OM9_Milestones/M00 - Feasibility, Legal/L2-04 - 3DM/L3-02 - NURBS - trimmed BRep - units proof/l3.json>) — NURBS / trimmed BRep / units proof | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-04-03](<../OM9_Milestones/M00 - Feasibility, Legal/L2-04 - 3DM/L3-03 - Import security and transparent/l3.json>) — Import security and transparent limitations | AUDIT-REQUIRED | GATE-ONLY |

#### OM9-M00-L2-05 — 2 L3

Owner: [Associative history & stable topology](<../OM9_Milestones/M00 - Feasibility, Legal/L2-05 - Associative/L2.json>). Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-M00-L3-05-01](<../OM9_Milestones/M00 - Feasibility, Legal/L2-05 - Associative/L3-01 - Naming and reference architecture/l3.json>) — Naming and reference architecture audit | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-05-02](<../OM9_Milestones/M00 - Feasibility, Legal/L2-05 - Associative/L3-02 - Regeneration and persistence/l3.json>) — Regeneration and persistence stress proof | AUDIT-REQUIRED | GATE-ONLY |

#### OM9-M00-L2-06 — 2 L3

Owner: [License / redistribution compliance](<../OM9_Milestones/M00 - Feasibility, Legal/L2-06 - License/L2.json>). Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-M00-L3-06-01](<../OM9_Milestones/M00 - Feasibility, Legal/L2-06 - License/L3-01 - Software license and reuse/l3.json>) — Software license and reuse strategy | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-06-02](<../OM9_Milestones/M00 - Feasibility, Legal/L2-06 - License/L3-02 - Assets, trademarks and release/l3.json>) — Assets, trademarks and release notices | AUDIT-REQUIRED | GATE-ONLY |

#### OM9-M00-L2-07 — 3 L3

Owner: [Reproducible build, CI and release baseline](<../OM9_Milestones/M00 - Feasibility, Legal/L2-07 - Reproducible/L2.json>). Đọc source/license/build feasibility evidence đúng owner; ghi điều kiện SDK và target. Không sinh geometry API từ nhiệm vụ quản trị.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-M00-L3-07-01](<../OM9_Milestones/M00 - Feasibility, Legal/L2-07 - Reproducible/L3-01 - Windows 10 - 11 x64/l3.json>) — Windows 10/11 x64 baseline | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-07-02](<../OM9_Milestones/M00 - Feasibility, Legal/L2-07 - Reproducible/L3-02 - Linux optional & packaged OCCT/l3.json>) — Linux optional & packaged OCCT | AUDIT-REQUIRED | GATE-ONLY |
| [OM9-M00-L3-07-03](<../OM9_Milestones/M00 - Feasibility, Legal/L2-07 - Reproducible/L3-03 - Continuous verification and gate/l3.json>) — Continuous verification and gate handoff | AUDIT-REQUIRED | GATE-ONLY |

## M01 — CAD shell, OCCT bridge, document, undo, GPU and viewport foundation

Nguồn: [L1 canonical](<../OM9_Milestones/M01 - CAD Shell, Viewport/L1.json>). 13 L2 / 117 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M00.

Bổ sung SDK vào nền tảng document/units/command/permissions/geometry ownership, codec và UI façade. Với code đã có, đây là checklist bổ sung còn mở tại chính owner M01; không triển khai lại thuật toán đã được chứng minh.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-L2-01-01](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L2.json>) — Khởi động/đóng ứng dụng, cửa sổ chính, docking cơ bản để bố trí giao diện Matrix. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-01-02](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L2.json>) — Registry lệnh, state-machine nhận điểm/đối tượng, transaction, Undo/Redo/Cancel thống nhất. | EVIDENCE-SCOPED một phần; SDK-SUBSET 2/11 L3 Windows (2026-10-21) | SDK-SUBSET (`006/010`) + EXPORT-BACKLOG | Transaction/command host đã nhận plugin; feedback progress/warning vào EventLog; còn registry/context/sessions, prompt và option theo hợp đồng API. |

| [OM9-L2-01-03](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L2.json>) — Document, EntityID, FeatureID, layers, blocks/instances, metadata đá và tham chiếu đối tượng. | SOURCE-PRESENT; SDK-SUBSET 6/13 L3 trên Windows (2026-10-21) | SDK-SUBSET (`002/003/004/005/009/011`) + EXPORT-BACKLOG | Đã xuất query paged/filter, attributes read/write, rename, layer service và extension slot read/write (2026-10-21); output vào layer tạo trong cùng command được transaction owner chấp nhận; groups + block-definition read đã xuất; group/instance write, change feed, snapshot vẫn backlog. |

| [OM9-L2-01-05](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L2.json>) — Adapter OCCT, BRep, NURBS evaluation, tolerances, memory ownership/FFI safety. | SOURCE-PRESENT selected native | EXPORT-BACKLOG + audit ops | Bọc selected BRep service, audit linked operations; từ chối fixture probes; Mac theo portability và packaging theo target. |

| [OM9-L2-01-06](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L2.json>) — GPU viewport wireframe/shaded, gem display, camera draw, overlays và mesh tessellation. | EVIDENCE-SCOPED display; gate mở | HOST-INTERNAL + façade command/input/UI | Giữ GPU/cache nội bộ; chỉ xuất preview/overlay DTO có budget và cleanup. |

| [OM9-L2-01-08](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-08 - Be Units Tolerances Basic Parameters/L2.json>) — Millimeter, degrees, tolerances, numeric input và tham số builder đơn giản. | SOURCE-PRESENT; SDK-SUBSET 7/7 L3 trên Windows (2026-10-21) | SDK-SUBSET (`001/002/003/004/005/006/007`) | Đã xuất units/tolerance/parse và typed builder parameters (`validate_parameter`/`apply_parameters` dùng `OptionDomain`/`CommandSchema` của owner); tolerance chỉ đọc, display/page unit mutation và CPlane frame chưa mở; UI parity thuộc M04/M01-02. |

| [OM9-L2-01-12](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L2.json>) — Save/load native document đầy đủ geometry/history/layer/gem instance; xuất STL watertight và báo lỗi hình học. | SOURCE-PRESENT codec/STL; SDK-SUBSET 1/10 L3 Windows (2026-10-21) | EXPORT-BACKLOG + audit lifecycle | Host file service: `project_state` (path/dirty) + `save_project` (atomic save sau commit) + `save_project_as` (host dialog) đã xuất; open/create/replace vẫn host-only; STL L3-008 thực thi ở M07, không nhầm folder với target. |

| [OM9-L2-02-01](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L2.json>) — Định nghĩa Rhino-like Curve/Surface/TrimmedSurface/Polysurface/solid/block và seam, UV/normal. | EVIDENCE-SCOPED semantics; SDK-SUBSET 3/10 L3 Windows (2026-10-21) | SDK-SUBSET (`002/003` curve identity/definition/evaluation/semantics) + EXPORT-BACKLOG | Simple shape DTO đã có; curve identity/domain/degree/poles/weights/knots/seam + closed/planar + evaluation D0-D3 đã xuất (read-only); còn control write, polycurve segment, surface/trim/blocks. |

| [OM9-L2-03-01](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-01 - Febe Jewelry Viewport Layout And Cplanes/L2.json>) — Viewport Looking Down/Perspective/Side/Through Finger, grid trang sức, active view | SOURCE-PRESENT view/CPlane | EXPORT-BACKLOG + HOST-INTERNAL UI | Xuất view context/CPlane owned snapshot; giữ docking/layout owner trong host. |

| [OM9-L2-03-07](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L2.json>) — Main Menu, Icon History, panel arrangement Matrix | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Plugin command/menu contribution; giữ menu names/disabled behavior, không export docking internals. |

| [OM9-L2-03-10](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-10 - Febe Matrix Commandline Feedback Adapter/L2.json>) — Command Line kiểu Rhino/Matrix, prompt, options/history | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Prompt/number/options/message service dùng chung command session, không parser/dispatcher thứ hai. |

| [OM9-L2-03-14](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L2.json>) — Layers palette, Object Info, SuperSelect cần modeling | SOURCE-PRESENT layers/permissions | EXPORT-BACKLOG | Nối layer/attributes/selection services; không tái tạo layer business rules trong plugin host. |

| [OM9-L2-03-15](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L2.json>) — File menu, native Open/Save, Import .3dm action, Job Bags/Project Manager | SOURCE-PRESENT file lifecycle | EXPORT-BACKLOG + audit Job Bags | Project save/open dùng host lifecycle; Job Bags phải audit riêng, không suy từ codec. |

### Checklist tích hợp khi thực hiện milestone

#### Checklist: Document, attributes, layer, units và transaction context

- [ ] Bổ sung DTO đơn vị/tolerance từ một nguồn chính; geometry mm, góc/domain có nhãn; tọa độ local/world và placement không bị nhân hai lần.
- [ ] Query objects có paging/filter/bbox, flags và revision; giữ API hiện có tương thích hoặc công bố migration.
- [ ] Layer CRUD/current layer và object attributes đi qua cùng validation/transaction/layer permissions với app. Không dùng setter core để né scene admission.
- [ ] Mỗi mutation được stage, kiểm tra budget/permission trước publish; thất bại bất kỳ bước nào không làm đổi model/scene/selection ngoài policy đã chốt, file và old Redo.
- [ ] Chuẩn hóa việc kết thúc command và snapshot invalidation. Không cấp `undo()` tùy ý cho plugin đang ở trong một command transaction.

**Owner:** M01 `01-02`, `01-03`, `01-08`, `03-14`. **Đóng:** plugin tạo layer, đặt current layer, thêm hình, gán attributes; UI và SDK có cùng kết quả; Undo/Redo, save/reopen, lock/hide/selectability và failure cases pass.

#### Checklist: Command sessions, input, preview và UI contributions

- [ ] Registry plugin command nối chung dispatcher: ID namespace, descriptor, enable/disable/context, arguments và errors. Dynamic menu registration vẫn là việc phải làm.
- [ ] Host-managed prompt `get_point`/`get_objects`/number/options dạng state-machine hoặc continuation đã thiết kế; không giữ mutable document pointer xuyên frame.
- [ ] Menu/CMD/toolbar/F6 gọi chung implementation. Preview là dữ liệu tạm, có lifetime/session/budget; cancel/focus/document change/unload dọn sạch.
- [ ] Panels dùng public declarative UI hoặc bridge có version được chốt; không đóng băng egui Rust ABI giữa DLL và app.
- [ ] Event callbacks, subscriptions và async jobs phải có reentrancy/thread/cancellation/unload policy trước khi mở. Chặn recursive dispatch không kiểm soát.
- [ ] Không chuyển native third-party plugin thành sandbox bằng lời mô tả; loader hiện tại chạy trusted native code.

**Owner:** M01 `01-01`, `01-02`, `01-06`, `03-07`, `03-10`; M02 `03-09`; M04 `03-11`. **Đóng:** plugin interactive mẫu, menu + CMD parity, preview→commit một transaction, error/cancel không để lại geometry hoặc resources; manual review vẫn độc lập.

Native bridge, units/semantics và codec vẫn thuộc owner M01. Chi tiết operator BRep/3DM được trình bày ở M03, STL đích M07; đây là liên kết dependency, không chuyển ownership khỏi M01. Portability được audit từ M00 và xác nhận runtime tại owner native tương ứng.

### Theo dõi L3 trong milestone

#### OM9-L2-01-01 — 8 L3

Owner: [Febe Application Shell Docking](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-01-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L3-001 - Application Startup And Main Window/l3.json>) — Application Startup And Main Window | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-01-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L3-002 - Workbench Docking Regions/l3.json>) — Workbench Docking Regions | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-01-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L3-003 - Reorder Collapse And Float Panels/l3.json>) — Reorder Collapse And Float Panels | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-01-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L3-004 - Docking Layout Default Restore/l3.json>) — Docking Layout Default Restore | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-01-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L3-005 - Docking Layout Persistence/l3.json>) — Docking Layout Persistence | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-01-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L3-006 - Active Document Window State/l3.json>) — Active Document Window State | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-01-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L3-007 - Global Status And Error Surface/l3.json>) — Global Status And Error Surface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-01-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-01 - Febe Application Shell Docking/L3-008 - Builder Panel Docking Slot/l3.json>) — Builder Panel Docking Slot | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-01-02 — 11 L3

Owner: [Be Command Dispatch Undo Redo](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L2.json>). Transaction/command host đã nhận plugin; bổ sung registry, context và sessions theo hợp đồng API và audit tại owner/document, units và attributes/command/input/UI.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-02-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-001 - Command IDs And Alias Registry/l3.json>) — Command IDs And Alias Registry | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-02-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-002 - Typed Command Descriptor/l3.json>) — Typed Command Descriptor | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-02-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-003 - Command Context And Active Document/l3.json>) — Command Context And Active Document | EVIDENCE-SCOPED subset Windows | SDK-SUBSET: document context đọc |
| [OM9-L3-01-02-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-004 - Interactive Command State Machine/l3.json>) — Interactive Command State Machine | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-02-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-005 - Command Option Parser/l3.json>) — Command Option Parser | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-02-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-006 - Atomic Command Transactions/l3.json>) — Atomic Command Transactions | EVIDENCE-SCOPED subset Windows; SDK subset Windows (2026-10-21) | SDK-SUBSET: host atomic mutation + insert admission cho layer đề xuất trong cùng request |
| [OM9-L3-01-02-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-007 - Undo Redo Journal/l3.json>) — Undo Redo Journal | EVIDENCE-SCOPED subset Windows | SDK-SUBSET: host journal, không public undo API |
| [OM9-L3-01-02-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-008 - Command Cancel Rollback/l3.json>) — Command Cancel Rollback | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-02-009](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-009 - Command Repeat And Dispatch/l3.json>) — Command Repeat And Dispatch | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-02-010](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-010 - Command Event And Observer API/l3.json>) — Command Event And Observer API | EVIDENCE-SCOPED subset Windows; SDK subset Windows (2026-10-21) | SDK-SUBSET: plugin `progress`/`warning` vào EventLog; observer callback vẫn host-internal |
| [OM9-L3-01-02-011](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-02 - Be Command Dispatch Undo Redo/L3-011 - Command Availability Preconditions/l3.json>) — Command Availability Preconditions | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-01-03 — 13 L3

Owner: [Be Document Scene Entities](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L2.json>). Read objects/layers đã có; xuất setters/query/attributes với permissions, không cấp mutable Document.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-03-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-001 - Document Entity IDs/l3.json>) — Document Entity IDs | EVIDENCE-SCOPED subset Windows | SDK-SUBSET: opaque object IDs |
| [OM9-L3-01-03-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-002 - Typed Scene Entity Records/l3.json>) — Typed Scene Entity Records | EVIDENCE-SCOPED subset Windows; SDK subset Windows (2026-10-21) | SDK-SUBSET: typed records + generation/revisions + `query_objects` paged/filter |
| [OM9-L3-01-03-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-003 - Geometry Handle And Attributes/l3.json>) — Geometry Handle And Attributes | EVIDENCE-SCOPED subset Windows; SDK subset Windows (2026-10-21) | SDK-SUBSET: attributes read/write + rename; geometry vẫn read-only |
| [OM9-L3-01-03-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-004 - Layer Domain Records/l3.json>) — Layer Domain Records | EVIDENCE-SCOPED subset Windows; SDK subset Windows (2026-10-21) | SDK-SUBSET: `layers` staged read-back + create/update, màu và cờ hiệu lực |
| [OM9-L3-01-03-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-005 - Entity To Layer Membership/l3.json>) — Entity To Layer Membership | EVIDENCE-SCOPED subset Windows; SDK subset Windows (2026-10-21) | SDK-SUBSET: `assign_objects_to_layer` + current layer đọc/đặt |
| [OM9-L3-01-03-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-006 - Layer Selection Visibility Rules/l3.json>) — Layer Selection Visibility Rules | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-03-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-007 - Group Membership Model/l3.json>) — Group Membership Model | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-03-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-008 - Block Definition And Instance Graph/l3.json>) — Block Definition And Instance Graph | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-03-009](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-009 - Jewelry Semantic Extension Record/l3.json>) — Jewelry Semantic Extension Record | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: slot read + set/clear qua `Mutation::Extension`; schema interpretation mở |
| [OM9-L3-01-03-010](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-010 - Selection State Document Contract/l3.json>) — Selection State Document Contract | EVIDENCE-SCOPED subset Windows | SDK-SUBSET: selected root IDs read |
| [OM9-L3-01-03-011](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-011 - Document Change Notification/l3.json>) — Document Change Notification | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-03-012](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-012 - Document Snapshot And Restore/l3.json>) — Document Snapshot And Restore | AUDIT-REQUIRED | HOST-INTERNAL ownership; façade/DTO nếu cần, không public mutable state |
| [OM9-L3-01-03-013](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-03 - Be Document Scene Entities/L3-013 - Import Source Provenance Metadata/l3.json>) — Import Source Provenance Metadata | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-01-05 — 11 L3

Owner: [Be Occt Brep Kernel Bridge](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L2.json>). Bọc selected BRep service, audit linked operations; từ chối fixture probes; Mac theo portability và packaging theo target.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-05-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-001 - OCCT FFI Safe Handle Lifecycle/l3.json>) — OCCT FFI Safe Handle Lifecycle | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-002 - B-Rep Shape Type And Validation/l3.json>) — B-Rep Shape Type And Validation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-003 - NURBS Curve Evaluation Adapter/l3.json>) — NURBS Curve Evaluation Adapter | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-004 - NURBS Surface Evaluation Adapter/l3.json>) — NURBS Surface Evaluation Adapter | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-005 - Face Wire Edge Topology Access/l3.json>) — Face Wire Edge Topology Access | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-006 - OCCT Transform Matrix Service/l3.json>) — OCCT Transform Matrix Service | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-007 - Geom Intersection Query Adapter/l3.json>) — Geom Intersection Query Adapter | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-008 - Shape Tessellation Interface/l3.json>) — Shape Tessellation Interface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-009](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-009 - Geometry Operation Error Contract/l3.json>) — Geometry Operation Error Contract | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-010](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-010 - Kernel Thread And Transaction Policy/l3.json>) — Kernel Thread And Transaction Policy | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-05-011](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-05 - Be Occt Brep Kernel Bridge/L3-011 - Reference Shape Fixture Evaluation/l3.json>) — Reference Shape Fixture Evaluation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-01-06 — 9 L3

Owner: [Febe Viewport Gpu Draw Backend](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L2.json>). Giữ GPU/cache nội bộ; chỉ xuất preview/overlay DTO có budget và cleanup.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-06-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-001 - GPU Device And Render Loop/l3.json>) — GPU Device And Render Loop | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-06-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-002 - Shared Scene Graphics Resources/l3.json>) — Shared Scene Graphics Resources | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-06-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-003 - NURBS B-Rep Display Tessellation Cache/l3.json>) — NURBS B-Rep Display Tessellation Cache | RUNTIME-CLAIM | HOST-INTERNAL ownership; façade/DTO nếu cần, không public mutable state |
| [OM9-L3-01-06-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-004 - Wireframe And Isocurve Pass/l3.json>) — Wireframe And Isocurve Pass | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-06-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-005 - Shaded Face And Solid Pass/l3.json>) — Shaded Face And Solid Pass | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-06-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-006 - Depth Clipping And Grid Overlays/l3.json>) — Depth Clipping And Grid Overlays | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-06-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-007 - Selection Highlight Render Pass/l3.json>) — Selection Highlight Render Pass | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-06-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-008 - Viewport Invalidation And Redraw/l3.json>) — Viewport Invalidation And Redraw | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-06-009](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-06 - Febe Viewport Gpu Draw Backend/L3-009 - GPU Device Loss Graceful Recovery/l3.json>) — GPU Device Loss Graceful Recovery | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-01-08 — 7 L3

Owner: [Be Units Tolerances Basic Parameters](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-08 - Be Units Tolerances Basic Parameters/L2.json>). Xuất units/tolerance/coordinate types và validation; geometry mm, display unit tách riêng.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-08-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-08 - Be Units Tolerances Basic Parameters/L3-001 - Millimeter World Unit Model/l3.json>) — Millimeter World Unit Model | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `document_units` (display unit, mm per unit) + `parse_length` mm |
| [OM9-L3-01-08-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-08 - Be Units Tolerances Basic Parameters/L3-002 - Angle Conversion And Degrees/l3.json>) — Angle Conversion And Degrees | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `parse_angle` degrees/radians |
| [OM9-L3-01-08-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-08 - Be Units Tolerances Basic Parameters/L3-003 - Document Absolute Relative Tolerances/l3.json>) — Document Absolute Relative Tolerances | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: tolerance đọc qua `document_units`, `None` khi tài liệu không nêu; chưa có setter |
| [OM9-L3-01-08-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-08 - Be Units Tolerances Basic Parameters/L3-004 - Typed Numeric Parameter Values/l3.json>) — Typed Numeric Parameter Values | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `ParameterValue` kind-tagged + `apply_parameters` defaults |
| [OM9-L3-01-08-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-08 - Be Units Tolerances Basic Parameters/L3-005 - Builder Parameter Range Validation/l3.json>) — Builder Parameter Range Validation | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: min/max/step/count/choice/text qua `OptionDomain::check` |
| [OM9-L3-01-08-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-08 - Be Units Tolerances Basic Parameters/L3-006 - Unit Safe Parameter Serialization/l3.json>) — Unit Safe Parameter Serialization | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: wire form tag = đơn vị; bare number bị từ chối |
| [OM9-L3-01-08-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-08 - Be Units Tolerances Basic Parameters/L3-007 - Coordinate Numeric Input Primitives/l3.json>) — Coordinate Numeric Input Primitives | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `parse_coordinates` grammar prompt, mm pre-CPlane; frame vẫn host-owned |

#### OM9-L2-01-12 — 10 L3

Owner: [Be Native Project Codec Stl Export](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L2.json>). Expose host file service; STL L3-008 thực thi ở M07, không nhầm folder với target.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-12-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-001 - Native Project Container Format/l3.json>) — Native Project Container Format | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-12-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-002 - Scene And Layer State Encoding/l3.json>) — Scene And Layer State Encoding | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-12-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-003 - B-Rep Mesh And Instance Encoding/l3.json>) — B-Rep Mesh And Instance Encoding | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-12-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-004 - Builder History Stub Persistence/l3.json>) — Builder History Stub Persistence | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-12-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-005 - Atomic Save And Write Failure Recovery/l3.json>) — Atomic Save And Write Failure Recovery | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `save_project` (atomic save sau commit) + `save_project_as` (host Save As dialog); write-failure qua plugin path mở |
| [OM9-L3-01-12-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-006 - Project Load Validation And Compatibility/l3.json>) — Project Load Validation And Compatibility | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-12-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-007 - Document Reload And Restoration/l3.json>) — Document Reload And Restoration | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-12-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-008 - STL Tessellation Export Codec/l3.json>) — STL Tessellation Export Codec; **đích M07** | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-12-009](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-009 - Export Mesh Units And Winding/l3.json>) — Export Mesh Units And Winding | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-12-010](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-01-12 - Be Native Project Codec Stl Export/L3-010 - Project Roundtrip Regression Fixtures/l3.json>) — Project Roundtrip Regression Fixtures | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-02-01 — 10 L3

Owner: [Be Rhino Nurbs Object Semantics](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L2.json>). Simple shape DTO đã có; đưa NURBS/surface/trim/blocks vào public contract theo từng loại.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-02-01-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-001 - Point And Polyline Semantic Types/l3.json>) — Point And Polyline Semantic Types | EVIDENCE-SCOPED subset Windows; runtime metadata | SDK-SUBSET: Point/Line/Polyline/full Circle |
| [OM9-L3-02-01-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-002 - NURBS Curve Control Points Knots Weights/l3.json>) — NURBS Curve Control Points Knots Weights | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `curve_info`/`curve_definition` (domain/degree/poles/weights/knots/seam/arc) + `evaluate_curve` D0-D3; write mở |
| [OM9-L3-02-01-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-003 - Open Closed And Planar Curve Rules/l3.json>) — Open Closed And Planar Curve Rules | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `curve_semantics` closed/planar/endpoints dưới tolerance tường minh hoặc document |
| [OM9-L3-02-01-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-004 - Surface UV Domain And Seam Rules/l3.json>) — Surface UV Domain And Seam Rules | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-01-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-005 - Trimmed Vs Underlying Surface Model/l3.json>) — Trimmed Vs Underlying Surface Model | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-01-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-006 - Surface Edge And Isocurve Semantics/l3.json>) — Surface Edge And Isocurve Semantics | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-01-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-007 - Polysurface And Closed Solid Semantics/l3.json>) — Polysurface And Closed Solid Semantics | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-01-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-008 - Extrusion Vs Full B-Rep Representations/l3.json>) — Extrusion Vs Full B-Rep Representations | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-01-009](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-009 - Mesh Vs NURBS Object Type Rules/l3.json>) — Mesh Vs NURBS Object Type Rules | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-01-010](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-02-01 - Be Rhino Nurbs Object Semantics/L3-010 - Instance And Block Semantic Type/l3.json>) — Instance And Block Semantic Type | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-01 — 7 L3

Owner: [Febe Jewelry Viewport Layout And Cplanes](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-01 - Febe Jewelry Viewport Layout And Cplanes/L2.json>). Xuất view context/CPlane owned snapshot; giữ docking/layout owner trong host.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-01-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-01 - Febe Jewelry Viewport Layout And Cplanes/L3-001 - Matrix Four Viewport Grid Layout/l3.json>) — Matrix Four Viewport Grid Layout | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-01-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-01 - Febe Jewelry Viewport Layout And Cplanes/L3-002 - Jewelry Viewport Camera Presets/l3.json>) — Jewelry Viewport Camera Presets | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-01-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-01 - Febe Jewelry Viewport Layout And Cplanes/L3-003 - Active Viewport Indicator/l3.json>) — Active Viewport Indicator | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-01-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-01 - Febe Jewelry Viewport Layout And Cplanes/L3-004 - F4 World Origin And Ring Center Display/l3.json>) — F4 World Origin And Ring Center Display | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-01-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-01 - Febe Jewelry Viewport Layout And Cplanes/L3-005 - Matrix Viewport Grid And Axes/l3.json>) — Matrix Viewport Grid And Axes | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-01-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-01 - Febe Jewelry Viewport Layout And Cplanes/L3-006 - Maximize And Restore Viewport Layout/l3.json>) — Maximize And Restore Viewport Layout | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-01-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-01 - Febe Jewelry Viewport Layout And Cplanes/L3-007 - Viewport Tab Switch When Maximized/l3.json>) — Viewport Tab Switch When Maximized | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-07 — 8 L3

Owner: [Febe Matrix Main Menu Iconhistory And Docking](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L2.json>). Plugin command/menu contribution; giữ menu names/disabled behavior, không export docking internals.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-07-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L3-001 - Matrix Left Sidebar Stack And Order/l3.json>) — Matrix Left Sidebar Stack And Order | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-07-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L3-002 - Matrix Main Menu Category Tabs/l3.json>) — Matrix Main Menu Category Tabs | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-07-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L3-003 - Command Icon Tooltip And Rightclick Slot/l3.json>) — Command Icon Tooltip And Rightclick Slot | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-07-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L3-004 - Menu Scrolling And Icon Grid/l3.json>) — Menu Scrolling And Icon Grid | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-07-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L3-005 - Recent Command Icon History/l3.json>) — Recent Command Icon History | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-07-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L3-006 - Main Menu Collapse Reorder And Float/l3.json>) — Main Menu Collapse Reorder And Float | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-07-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L3-007 - Default Menu Layout Reset/l3.json>) — Default Menu Layout Reset | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-07-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-07 - Febe Matrix Main Menu Iconhistory And Docking/L3-008 - Disabled Menu Action Visibility/l3.json>) — Disabled Menu Action Visibility | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-10 — 6 L3

Owner: [Febe Matrix Commandline Feedback Adapter](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-10 - Febe Matrix Commandline Feedback Adapter/L2.json>). Prompt/number/options/message service dùng chung command session, không parser/dispatcher thứ hai.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-10-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-10 - Febe Matrix Commandline Feedback Adapter/L3-001 - Commandline Entry And Submit/l3.json>) — Commandline Entry And Submit | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-10-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-10 - Febe Matrix Commandline Feedback Adapter/L3-002 - Interactive Command Prompts/l3.json>) — Interactive Command Prompts | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-10-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-10 - Febe Matrix Commandline Feedback Adapter/L3-003 - Command Options Clickable Tokens/l3.json>) — Command Options Clickable Tokens | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-10-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-10 - Febe Matrix Commandline Feedback Adapter/L3-004 - Numeric And Coordinate Feedback/l3.json>) — Numeric And Coordinate Feedback | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-10-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-10 - Febe Matrix Commandline Feedback Adapter/L3-005 - Command History Readable Panel/l3.json>) — Command History Readable Panel | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-10-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-10 - Febe Matrix Commandline Feedback Adapter/L3-006 - Commandline Escape Repeat Shortcuts/l3.json>) — Commandline Escape Repeat Shortcuts | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-14 — 8 L3

Owner: [Febe Matrix Layers Objectinfo Superselect](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L2.json>). Nối layer/attributes/selection services; không tái tạo layer business rules trong plugin host.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-14-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L3-001 - Matrix Layer Category Palette/l3.json>) — Matrix Layer Category Palette | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-14-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L3-002 - Current Layer And Assignment UI/l3.json>) — Current Layer And Assignment UI | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-14-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L3-003 - Layer Visible And Locked Controls/l3.json>) — Layer Visible And Locked Controls | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-14-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L3-004 - Select All Objects By Layer/l3.json>) — Select All Objects By Layer | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-14-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L3-005 - Matrix Object Info Overlay/l3.json>) — Matrix Object Info Overlay | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-14-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L3-006 - Superselect By Jewelry Object Type/l3.json>) — Superselect By Jewelry Object Type | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-14-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L3-007 - Object Property Editor Adapter/l3.json>) — Object Property Editor Adapter | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-14-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-14 - Febe Matrix Layers Objectinfo Superselect/L3-008 - Layer State Persistence Visual Check/l3.json>) — Layer State Persistence Visual Check | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-15 — 9 L3

Owner: [Febe Matrix File Project And Jobbags](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L2.json>). Project save/open dùng host lifecycle; Job Bags phải audit riêng, không suy từ codec.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-15-001](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L3-001 - Matrix File New Open Save Actions/l3.json>) — Matrix File New Open Save Actions | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-15-002](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L3-002 - Matrix File Import Export Entrypoints/l3.json>) — Matrix File Import Export Entrypoints | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-15-003](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L3-003 - Project Manager Project Row List/l3.json>) — Project Manager Project Row List | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-15-004](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L3-004 - Jobbag Slot Grid And Thumbnail/l3.json>) — Jobbag Slot Grid And Thumbnail | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-15-005](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L3-005 - Jobbag Capture Current Selection/l3.json>) — Jobbag Capture Current Selection | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-15-006](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L3-006 - Jobbag Restore And Insert/l3.json>) — Jobbag Restore And Insert | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-15-007](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L3-007 - Master Jobbag Document Snapshot/l3.json>) — Master Jobbag Document Snapshot | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-15-008](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L3-008 - Project Notes And Basic Metadata/l3.json>) — Project Notes And Basic Metadata | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-15-009](<../OM9_Milestones/M01 - CAD Shell, Viewport/L2-03-15 - Febe Matrix File Project And Jobbags/L3-009 - Project Manager Dock Interaction/l3.json>) — Project Manager Dock Interaction | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

## M02 — Viewport picking, snapping, navigation, gumball and Matrix interaction

Nguồn: [L1 canonical](<../OM9_Milestones/M02 - Viewport, Gumball/L1.json>). 9 L2 / 59 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M00, M01.

Selection, picking, CPlane, snaps, view context và interaction services phải có public adapter trong cùng owner triển khai. Review UI và performance vẫn theo contract hiện có.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-L2-01-07](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L2.json>) — Picking object/face/edge/vertex, selection set, gizmo hooks và hover hit-tests. | EVIDENCE-SCOPED; review mở; SDK-SUBSET 1/8 L3 Windows (2026-10-21) | SDK-SUBSET read roots + set/add/remove/toggle (`002`) + EXPORT-BACKLOG | Replace/extend/remove/toggle selection qua Controller validation (scene root + pick geometry) đã xuất; còn typed subobject/occurrence, pick query, window/crossing và drag. |

| [OM9-L2-02-02](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-02 - Febe Cplanes And Coordinate Input/L2.json>) — CPlane/World coordinates, gốc F4, nhập điểm chính xác, dựng hình theo view. | SOURCE-PRESENT coordinate layer | EXPORT-BACKLOG + audit UX | Export resolve/local-world/CPlane; interactive prompt cùng command/input/UI, không nhân đôi parser. |

| [OM9-L2-02-03](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L2.json>) — End/Mid/Cen/Near/Point/Int/Tan/Perp/Quad và SmartTrack cơ bản. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit osnap/smarttrack queries; xuất service theo snap context và tolerance, không state UI pointer. |

| [OM9-L2-03-02](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-02 - Febe Viewport Navigation Pan Zoom Orbit/L2.json>) — Pan/Zoom/Orbit, double-click maximize, view resets | EVIDENCE-SCOPED; performance mở | EXPORT-BACKLOG + HOST-INTERNAL scheduler | Xuất camera/view service theo nhu cầu; performance/manual nghiệm thu riêng. |

| [OM9-L2-03-03](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-03 - Febe Viewport Object Selection And Gumball/L2.json>) — Chọn đối tượng theo khung và Gumball trực quan | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Gumball result/handles service; preview/cancel/commit thống nhất với SDK transform. |

| [OM9-L2-03-04](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-04 - Febe Viewport Snaps Ortho Smarttrack/L2.json>) — Snaps palette, Ortho/Project/Planar UI | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit snaps/ortho/smarttrack; export constrain/query service, host giữ interaction state. |

| [OM9-L2-03-05](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-05 - Febe Viewport Display Shading And Gems/L2.json>) — Wireframe/Shaded/Gem View, display presets khi modeling | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Display/material DTO và mode service; renderer/gem GPU resources vẫn host-owned. |

| [OM9-L2-03-06](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-06 - Febe Viewport Pictureframe And Calibration/L2.json>) — Picture Frame và scale tham chiếu ảnh | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Pictureframe asset/calibration model service; input session dùng command/input/UI; persistence từ project và exchange. |

| [OM9-L2-03-09](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L2.json>) — F6 menu theo selection type, auto-prefill builder | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | F6 adapter gọi command registry; không tạo engine F6 riêng hoặc đóng băng widget ABI. |

### Checklist tích hợp khi thực hiện milestone

#### Checklist: Selection, object references, CPlane và viewport context

- [ ] Đưa selection query/write qua Controller scene validation hiện có; hỗ trợ add/remove/replace, root/subobject/occurrence riêng.
- [ ] Subobject token chứa source/revision và occurrence path; rebuild/import/delete/context change làm stale token bị từ chối, không remap theo index đoán.
- [ ] Export CPlane/view snapshot, projection và pick ray dưới dạng owned values. Query screen phải có viewport/viewport revision, DPI và clipping contract.
- [ ] Window/crossing/disambiguation/selection filters tái dùng dịch vụ M02; SDK không tạo một bộ picking riêng.
- [ ] Giữ prompt chọn đối tượng/điểm cho command/input/UI; không mô phỏng click chuột để thay một API hình học.

**Owner:** M02 `01-07`, `02-02`, `02-03`, `03-02`, `03-03`, `03-04`; foundation M01 `03-01`. **Đóng:** locked/hidden/unselectable, nested/mirrored occurrence, stale BRep topology, perspective/orthographic có parity. Evidence selection không đóng gate navigation performance.

### Theo dõi L3 trong milestone

#### OM9-L2-01-07 — 8 L3

Owner: [Febe Picking Selection Generic Handles](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L2.json>). Selection/query/picking write và typed subobject/occurrence qua Controller validation.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-07-001](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L3-001 - Screen Ray And Hit Test Query/l3.json>) — Screen Ray And Hit Test Query | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-07-002](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L3-002 - Object Selection Set Core/l3.json>) — Object Selection Set Core | EVIDENCE-SCOPED subset Windows; SDK subset Windows (2026-10-21) | SDK-SUBSET: selected root IDs read + replace/add/remove/toggle by persistent IDs |
| [OM9-L3-01-07-003](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L3-003 - Subobject Face Edge Vertex Picking/l3.json>) — Subobject Face Edge Vertex Picking | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-07-004](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L3-004 - Selection Window And Crossing/l3.json>) — Selection Window And Crossing | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-07-005](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L3-005 - Object Selection Disambiguation/l3.json>) — Object Selection Disambiguation | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-07-006](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L3-006 - Selection Highlighting And Hover/l3.json>) — Selection Highlighting And Hover | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-07-007](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L3-007 - Generic Translation Rotation Scale Handles/l3.json>) — Generic Translation Rotation Scale Handles | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-07-008](<../OM9_Milestones/M02 - Viewport, Gumball/L2-01-07 - Febe Picking Selection Generic Handles/L3-008 - Drag Transaction And Commit Cancel/l3.json>) — Drag Transaction And Commit Cancel | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-02-02 — 7 L3

Owner: [Febe Cplanes And Coordinate Input](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-02 - Febe Cplanes And Coordinate Input/L2.json>). Export resolve/local-world/CPlane; interactive prompt cùng command/input/UI, không nhân đôi parser.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-02-02-001](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-02 - Febe Cplanes And Coordinate Input/L3-001 - World And Construction Plane Coordinate Context/l3.json>) — World And Construction Plane Coordinate Context | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-02-002](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-02 - Febe Cplanes And Coordinate Input/L3-002 - CPlane Preset Worldtop Front Right/l3.json>) — CPlane Preset Worldtop Front Right | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-02-003](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-02 - Febe Cplanes And Coordinate Input/L3-003 - CPlane Orientation And Origin Change/l3.json>) — CPlane Orientation And Origin Change | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-02-004](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-02 - Febe Cplanes And Coordinate Input/L3-004 - Coordinate Text Point Entry/l3.json>) — Coordinate Text Point Entry | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-02-005](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-02 - Febe Cplanes And Coordinate Input/L3-005 - Matrix F4 Origin Shortcut/l3.json>) — Matrix F4 Origin Shortcut | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-02-006](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-02 - Febe Cplanes And Coordinate Input/L3-006 - Ortho Planar Project Cursor Constrain/l3.json>) — Ortho Planar Project Cursor Constrain | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-02-007](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-02 - Febe Cplanes And Coordinate Input/L3-007 - Cross Viewport Coordinate Consistency/l3.json>) — Cross Viewport Coordinate Consistency | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-02-03 — 7 L3

Owner: [Febe Osnaps And Basic Smarttrack](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L2.json>). Audit osnap/smarttrack queries; xuất service theo snap context và tolerance, không state UI pointer.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-02-03-001](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L3-001 - Osnap End Mid Center Quadrant/l3.json>) — Osnap End Mid Center Quadrant | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-03-002](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L3-002 - Osnap Near Point Intersection/l3.json>) — Osnap Near Point Intersection | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-03-003](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L3-003 - Osnap Tangent Perpendicular/l3.json>) — Osnap Tangent Perpendicular | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-03-004](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L3-004 - Osnap Surface And Polysurface/l3.json>) — Osnap Surface And Polysurface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-03-005](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L3-005 - Osnap Priority And One Shot Overrides/l3.json>) — Osnap Priority And One Shot Overrides | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-03-006](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L3-006 - Smarttrack Inferred Alignment Lines/l3.json>) — Smarttrack Inferred Alignment Lines | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-03-007](<../OM9_Milestones/M02 - Viewport, Gumball/L2-02-03 - Febe Osnaps And Basic Smarttrack/L3-007 - Osnap Debug Marker And Tooltip/l3.json>) — Osnap Debug Marker And Tooltip | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-02 — 7 L3

Owner: [Febe Viewport Navigation Pan Zoom Orbit](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-02 - Febe Viewport Navigation Pan Zoom Orbit/L2.json>). Xuất camera/view service theo nhu cầu; performance/manual nghiệm thu riêng.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-02-001](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-02 - Febe Viewport Navigation Pan Zoom Orbit/L3-001 - Perspective Right Drag Orbit/l3.json>) — Perspective Right Drag Orbit | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-02-002](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-02 - Febe Viewport Navigation Pan Zoom Orbit/L3-002 - Orthographic Right Drag Pan/l3.json>) — Orthographic Right Drag Pan | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-02-003](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-02 - Febe Viewport Navigation Pan Zoom Orbit/L3-003 - Wheel Zoom To Cursor/l3.json>) — Wheel Zoom To Cursor | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-02-004](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-02 - Febe Viewport Navigation Pan Zoom Orbit/L3-004 - Perspective Shift Right Drag Pan/l3.json>) — Perspective Shift Right Drag Pan | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-02-005](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-02 - Febe Viewport Navigation Pan Zoom Orbit/L3-005 - Zoom Extents And Selected/l3.json>) — Zoom Extents And Selected | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-02-006](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-02 - Febe Viewport Navigation Pan Zoom Orbit/L3-006 - Viewport Center Reset And Home/l3.json>) — Viewport Center Reset And Home | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-02-007](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-02 - Febe Viewport Navigation Pan Zoom Orbit/L3-007 - Viewport Maximize By Title/l3.json>) — Viewport Maximize By Title | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-03 — 6 L3

Owner: [Febe Viewport Object Selection And Gumball](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-03 - Febe Viewport Object Selection And Gumball/L2.json>). Gumball result/handles service; preview/cancel/commit thống nhất với SDK transform.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-03-001](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-03 - Febe Viewport Object Selection And Gumball/L3-001 - Matrix Click Shift Ctrl Select/l3.json>) — Matrix Click Shift Ctrl Select | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-03-002](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-03 - Febe Viewport Object Selection And Gumball/L3-002 - Matrix Left Right Drag Selection/l3.json>) — Matrix Left Right Drag Selection | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-03-003](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-03 - Febe Viewport Object Selection And Gumball/L3-003 - Matrix Ambiguous Selection Popup/l3.json>) — Matrix Ambiguous Selection Popup | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-03-004](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-03 - Febe Viewport Object Selection And Gumball/L3-004 - Matrix Gumball Axis Drag/l3.json>) — Matrix Gumball Axis Drag | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-03-005](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-03 - Febe Viewport Object Selection And Gumball/L3-005 - Matrix Gumball Alignment And Relocate/l3.json>) — Matrix Gumball Alignment And Relocate | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-03-006](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-03 - Febe Viewport Object Selection And Gumball/L3-006 - Matrix Gumball Numeric Edit/l3.json>) — Matrix Gumball Numeric Edit | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-04 — 5 L3

Owner: [Febe Viewport Snaps Ortho Smarttrack](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-04 - Febe Viewport Snaps Ortho Smarttrack/L2.json>). Audit snaps/ortho/smarttrack; export constrain/query service, host giữ interaction state.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-04-001](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-04 - Febe Viewport Snaps Ortho Smarttrack/L3-001 - Matrix Snap Toggle Panel/l3.json>) — Matrix Snap Toggle Panel | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-04-002](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-04 - Febe Viewport Snaps Ortho Smarttrack/L3-002 - Matrix Ortho Planar Project Toggles/l3.json>) — Matrix Ortho Planar Project Toggles | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-04-003](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-04 - Febe Viewport Snaps Ortho Smarttrack/L3-003 - Matrix Snap Activation And Indicator/l3.json>) — Matrix Snap Activation And Indicator | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-04-004](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-04 - Febe Viewport Snaps Ortho Smarttrack/L3-004 - Matrix Smarttrack Toggle And Feedback/l3.json>) — Matrix Smarttrack Toggle And Feedback | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-04-005](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-04 - Febe Viewport Snaps Ortho Smarttrack/L3-005 - Matrix Snap Shortcut Modes/l3.json>) — Matrix Snap Shortcut Modes | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-05 — 6 L3

Owner: [Febe Viewport Display Shading And Gems](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-05 - Febe Viewport Display Shading And Gems/L2.json>). Display/material DTO và mode service; renderer/gem GPU resources vẫn host-owned.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-05-001](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-05 - Febe Viewport Display Shading And Gems/L3-001 - Wireframe Shaded Mode Switcher/l3.json>) — Wireframe Shaded Mode Switcher | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-05-002](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-05 - Febe Viewport Display Shading And Gems/L3-002 - Gem View Material Color Cues/l3.json>) — Gem View Material Color Cues | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-05-003](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-05 - Febe Viewport Display Shading And Gems/L3-003 - Curve And Cutter Visibility Presets/l3.json>) — Curve And Cutter Visibility Presets | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-05-004](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-05 - Febe Viewport Display Shading And Gems/L3-004 - Surface Shading And Backface Cues/l3.json>) — Surface Shading And Backface Cues | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-05-005](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-05 - Febe Viewport Display Shading And Gems/L3-005 - Selection Highlight Overrides/l3.json>) — Selection Highlight Overrides | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-05-006](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-05 - Febe Viewport Display Shading And Gems/L3-006 - Display Settings Preview Presets/l3.json>) — Display Settings Preview Presets | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-06 — 5 L3

Owner: [Febe Viewport Pictureframe And Calibration](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-06 - Febe Viewport Pictureframe And Calibration/L2.json>). Pictureframe asset/calibration model service; input session dùng command/input/UI; persistence từ project và exchange.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-06-001](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-06 - Febe Viewport Pictureframe And Calibration/L3-001 - Pictureframe Image Import/l3.json>) — Pictureframe Image Import | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-06-002](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-06 - Febe Viewport Pictureframe And Calibration/L3-002 - Pictureframe Position And Scale/l3.json>) — Pictureframe Position And Scale | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-06-003](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-06 - Febe Viewport Pictureframe And Calibration/L3-003 - Reference Image Transparency And Lock/l3.json>) — Reference Image Transparency And Lock | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-06-004](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-06 - Febe Viewport Pictureframe And Calibration/L3-004 - Known Distance Image Calibration/l3.json>) — Known Distance Image Calibration | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-06-005](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-06 - Febe Viewport Pictureframe And Calibration/L3-005 - Viewport Reference Asset Persistence/l3.json>) — Viewport Reference Asset Persistence | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-09 — 8 L3

Owner: [Febe Matrix F6 Context Actions](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L2.json>). F6 adapter gọi command registry; không tạo engine F6 riêng hoặc đóng băng widget ABI.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-09-001](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L3-001 - F6 Selected Object Class Dispatch/l3.json>) — F6 Selected Object Class Dispatch | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-09-002](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L3-002 - F6 Default General Mode/l3.json>) — F6 Default General Mode | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-09-003](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L3-003 - F6 Curve And Surface Modes/l3.json>) — F6 Curve And Surface Modes | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-09-004](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L3-004 - F6 Gem And Report Modes/l3.json>) — F6 Gem And Report Modes | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-09-005](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L3-005 - F6 Custom User Mode/l3.json>) — F6 Custom User Mode | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-09-006](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L3-006 - F6 Builder Input Prefill/l3.json>) — F6 Builder Input Prefill | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-09-007](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L3-007 - F6 Floating Pin And Position/l3.json>) — F6 Floating Pin And Position | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-09-008](<../OM9_Milestones/M02 - Viewport, Gumball/L2-03-09 - Febe Matrix F6 Context Actions/L3-008 - F6 Disabled Unimplemented Actions/l3.json>) — F6 Disabled Unimplemented Actions | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

## M03 — Curve, Surface, Solid và trao đổi 3DM

Nguồn: [L1 canonical](<../OM9_Milestones/M03 - Geometry 3DM/L1.json>). 14 L2 / 243 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M01.

Mỗi operator Curve/Surface/Solid/Transform/Measure và 3DM phải nối public SDK trong cùng lát cắt của owner. Tận dụng service nội bộ trước; operator thiếu được làm cùng adapter và plugin proof.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-L2-01-09](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L2.json>) — Primitives, pipe, extrude, revolve, loft/sweep BRep và curve/surface adapters phục vụ trang sức. | SOURCE-PRESENT từng operator | EXPORT-BACKLOG + AUDIT-THEN-ROUTE | Đối chiếu curve/surface/native primitives; export operator đã đạt, làm mới operator thiếu kèm SDK. |

| [OM9-L2-01-10](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L2.json>) — Union/Difference/Intersection, trim/split support, fillet/chamfer, shell/offset cần cho nhẫn và cutter. | AUDIT-REQUIRED general ops | AUDIT-THEN-ROUTE | Chứng minh linked boolean/fillet/shell trên input tùy biến trước; fixture measurement chưa đủ. |

| [OM9-L2-01-11](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L2.json>) — Move/Rotate/Scale/Mirror/Array transformations + transform matrices làm primitive dịch vụ dùng chung. | SOURCE-PRESENT matrix/placement | SDK-SUBSET + EXPORT-BACKLOG | Matrix delta đã xuất; mirror/array/frames/orient/copy phải audit rồi xuất từng capability. |

| [OM9-L2-01-13](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L2.json>) — Đọc file Rhino .3dm (NURBS/trimmed BRep, layers, units, blocks, meshes), chuyển sang scene OCCT, giao dịch atomic; lệnh Import được gắn qua File Menu Matrix. | EVIDENCE-SCOPED 3DM; review mở | EXPORT-BACKLOG | Export native exchange service hiện có; giữ strict admission, subtype và trim limitations. |

| [OM9-L2-02-04](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L2.json>) — Line/Polyline/InterpCurve/Circle/Arc/Offset/Fillet/Blend/Rebuild/Match và point edit, G0/G1/G2 theo scope Curve cần cho Matrix. | SOURCE-PRESENT operators; scope khác nhau; SDK-SUBSET 2/12 L3 Windows (2026-10-21) | SDK-SUBSET (`002/003/005` circle/arc/ellipse/rectangle/polygon + interpolated/control-point NURBS + segment inspect) + EXPORT-BACKLOG/CO-IMPLEMENT | Line/polyline/full circle/arc/ellipse/rectangle/polygon/interpolated + control-point NURBS đã xuất; spiral/helix, edit, offset, fillet/blend/rebuild, periodic/seam vẫn backlog. |

| [OM9-L2-02-05](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L2.json>) — Sweep1/2, Loft, Network, Patch, RailRevolve, Trim/Split/Untrim/MatchSurface/Offset cùng UV surface. | SOURCE-PRESENT selected surfaces | EXPORT-BACKLOG + AUDIT-THEN-ROUTE | Bọc evaluate/loft/sweep selected path; trim/network/patch và UI workflow audit riêng. |

| [OM9-L2-02-06](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-06 - 06-febe-rhino/L2.json>) — Command sessions cho Move/Scale/Rotate/Array/Orient với pivot/ref/copy/preview kiểu Rhino. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Transform/orient sessions gọi shared geometry service; options/pivot/copy/preview qua SDK contract. |

| [OM9-L2-03-08](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-08 - 07-febe-matrix-top11/L2.json>) — 11 lệnh thường dùng theo toolbar Matrix | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Top11/shortcuts là command adapter; công bố callable command khi shared implementation pass. |

| [OM9-L2-03-16](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L2.json>) — Curve menu và lệnh chỉnh curve của Matrix8 Book1 | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Từng Curve menu L3 gọi shared Curve và đo đạc service; API và menu/CMD parity khi triển khai. |

| [OM9-L2-03-17](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L2.json>) — Surface menu, Sweep/Loft/Trim settings theo Matrix | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Từng Surface menu L3 gọi shared Surface service; không nhân đôi sweep/loft/trim. |

| [OM9-L2-03-18](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L2.json>) — Solid menu phù hợp trang sức: primitives, bool, extrusion, shell, text geometry | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Từng Solid menu L3 nối selected native BRep/Solid ops và SDK; không quảng cáo cả nhóm từ box/sphere. |

| [OM9-L2-03-19](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-19 - 11-febe-matrix/L2.json>) — Check/ShowEdges/validity, mesh inspection, model health | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Geometry/mesh health report DTO; ShowEdges visualization là command/input/UI, backend validity theo native BRep/Solid. |

| [OM9-L2-03-20](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-20 - 12-febe-matrix/L2.json>) — Distance/Angle/Length/Radius, kích thước và thống kê mẫu | SOURCE-PRESENT distance/angle/bounds/length/radius/summary/label; SDK-SUBSET 5/5 L3 Windows (2026-10-21) | EXPORT-BACKLOG + AUDIT-THEN-ROUTE | distance/angle/bounds/length/radius/summary/label/mesh-area đã xuất; volume vẫn backlog. |

| [OM9-L2-04-01](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L2.json>) — Matrix Transform menu binds to Rhino/OCCT transforms | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Toolbar Transform là adapter selection và context/command/input/UI; tái sử dụng exact operator và transaction. |

### Checklist tích hợp khi thực hiện milestone

#### Checklist: Curve model, evaluation, construction/edit và measure

- [ ] Export analytic Arc/Ellipse và rational NURBS DTO: degree, poles/weights, knots/multiplicities, domain, closed/periodic/seam policy. Giữ Circle hiện có không âm thầm đổi semantics.
- [ ] Read/create/replace theo capability; validators tái dùng core. `object_geometry` phải báo unsupported có cấu trúc cho loại chưa xuất.
- [ ] Bọc evaluate/derivatives/endpoints/bounds/planarity/length từng phần đã chứng minh. Không gộp “curve evaluate” thành mọi subtype nếu có giới hạn.
- [ ] Bọc interpolate/arc/ellipse/polyline/rectangle/rebuild/knot insert/degree elevate/fit; mỗi operator có oracle trước khi quảng cáo.
- [ ] Offset/blend/match/trim/split/project/intersection: kiểm tra backend thật theo từng L3, ghi nhánh đã có cần export và nhánh chưa có phải đồng triển khai.
- [ ] Measure distance/angle trước; curve length/radius/area sau khi có backend và semantics đúng. Kiểm tra object-local và world placement.

**Owner:** M01 `02-01`; M03 `02-04`, `03-16`, `03-20`, chia sẻ `01-09`. L3 ưu tiên: `OM9-L3-02-04-001`–`012`; subset SDK hiện có chỉ simple shapes thuộc `001` và full circle thuộc `002`.

**Đóng từng operator:** plugin ngoài repo gọi được; numerical/property tests + degenerate/invalid inputs; menu/CMD/plugin dùng chung service; Undo/Cancel/Redo và round-trip geometry. Không cần đợi toàn bộ 58 L3 menu Curve mới đóng một capability có scope rõ.

#### Checklist: Surface, trims và surface-derived operations

- [ ] DTO Surface/TrimmedSurface tách underlying surface, UV domain, seam/periodicity và trims; edge/isocurve là các khái niệm khác nhau.
- [ ] Export evaluate/derivatives/normal/isocurve trước; kiểm chứng weighted rational, UV boundary và orientation.
- [ ] Plane/cylinder/loft/sweep1/sweep2/path-frame theo từng backend đã nhận diện; preview và final dùng cùng parameters/tolerance.
- [ ] Network/Patch/revolve/rail-revolve/extrude/trim/split/untrim/shrink/match/offset phải audit từng operator; không suy từ `surface_kernel.rs` có sweep rằng cả nhóm đã xong.
- [ ] Query surface/face từ BRep phải đi qua native BRep/Solid revision token; không biến một face có lỗ thành surface không trim mà không báo semantics.

**Owner:** M03 `02-05`, `03-17`, `01-09`; foundation M01 `02-01`. L3 `OM9-L3-02-05-001`–`010` được điền scope khi triển khai. **Đóng:** surface oracle, trim/hole/seam, normal, failure atomic và 3DM/project round-trip đúng capability.

#### Checklist: Native BRep, solids, validity và operations

- [ ] BRep public handle/token do host sở hữu; copy dữ liệu theo boundary, giới hạn dung lượng và invalidation rõ ràng; không cho plugin nhận con trỏ shape.
- [ ] Export box/sphere/read/volume selected path trước nếu dependency/runtime đã có; đo bounds/volume/validity và persistence trên nhiều tham số, không dùng một fixture cứng làm API.
- [ ] Boolean/fillet/chamfer/shell/offset/sections/intersections: chứng minh linked backend và worker isolation riêng, khai báo capability đúng target.
- [ ] Tolerance propagation, topology refs, cancellation/timeout và native failure phải rollback toàn bộ. Native allocation được giải phóng đúng phía.
- [ ] Ghi ngoại lệ C++: native constraint, owner, ABI/error/lifetime/threading, crash behavior và test trên từng OS. Rust caller không làm native code tự động memory-safe.

**Owner:** M01 `01-05`; M03 `01-09`, `01-10`, `03-18`, `03-19`; AFTER `06-01` cho topology reconciliation nâng cao. **Đóng:** shape/editability đúng scope, geometry/Undo/save-reopen/plugin E2E, runtime native target. Ghi batch openNURBS vào roadmap chỉ khi thật sự chạy measurement mới.

#### Checklist: Project, 3DM, STL, user data và file lifecycle

- [ ] Tách pure codec/import preparation khỏi active-document lifecycle và publish file. Host chọn thời điểm đổi document; không cho nested open phá command đang chạy.
- [ ] Plugin import dùng cùng admission, units/layer/object mapping, diagnostics và atomic transaction với File UI; unsupported không biến thành hình giả.
- [ ] Export snapshot immutable qua staging file + atomic publish; lỗi không làm hỏng file đích cũ. Quy định đường dẫn/collision/overwrite rõ trong API.
- [ ] Plugin persistent data có namespace/plugin ID/schema version/migration/size limit; file thiếu plugin vẫn mở được và giữ dữ liệu theo policy được chốt.
- [ ] 3DM tách retained source payload, host-editable geometry và proprietary userdata; public docs giữ giới hạn trim đã ghi trong evidence 2026-10-20.
- [ ] STL nối owner `OM9-L3-01-12-008` đích M07; kiểm tra tessellation, mm, winding, selected export và external viewer.

**Owner:** M01 `01-12`, `03-15`; M03 `01-13`; M07 manufacturing. **Đóng:** cold save/open, import/export scope fixtures, error byte preservation, plugin absence/version mismatch, không claim mọi định dạng ở AFTER đã hỗ trợ.

Các checklist project/exchange ở đây phân công: M01 sở hữu codec/lifecycle nền tảng; M03 sở hữu 3DM import/export operators; M07 thực thi STL ở owner được chỉ rõ. Không trì hoãn việc SDK của một operator sang lúc toàn bộ M03 hoàn thành.

### Theo dõi L3 trong milestone

#### OM9-L2-01-09 — 9 L3

Owner: [01-be-occt-geometry-creation-ops](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L2.json>). Đối chiếu curve/surface/native primitives; export operator đã đạt, làm mới operator thiếu kèm SDK.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-09-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L3-001 - Analytic Solid Primitive Factories/l3.json>) — Analytic Solid Primitive Factories | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-09-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L3-002 - Curve To Solid Extrusion/l3.json>) — Curve To Solid Extrusion | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-09-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L3-003 - Revolve Profile And Rail Revolve/l3.json>) — Revolve Profile And Rail Revolve | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-09-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L3-004 - Pipe From Curve/l3.json>) — Pipe From Curve | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-09-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L3-005 - Planar Boundary Surface Generator/l3.json>) — Planar Boundary Surface Generator | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-09-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L3-006 - Loft Through Crosssections/l3.json>) — Loft Through Crosssections | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-09-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L3-007 - Sweep Single And Double Rail/l3.json>) — Sweep Single And Double Rail | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-09-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L3-008 - Surface From Edge Curves/l3.json>) — Surface From Edge Curves | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-09-009](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-09 - 01-be-occt-geometry/L3-009 - General Profile To B-Rep Adapter/l3.json>) — General Profile To B-Rep Adapter | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-01-10 — 9 L3

Owner: [02-be-occt-boolean-fillet-shell-ops](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L2.json>). Chứng minh linked boolean/fillet/shell trên input tùy biến trước; fixture measurement chưa đủ.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-10-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-001 - B-Rep Boolean Union/l3.json>) — B-Rep Boolean Union | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-10-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-002 - B-Rep Boolean/l3.json>) — B-Rep Boolean Difference | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-10-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-003 - B-Rep Boolean/l3.json>) — B-Rep Boolean Intersection | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-10-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-004 - Curve And Face Split/l3.json>) — Curve And Face Split Intersections | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-10-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-005 - Planar Cap And Close Solid/l3.json>) — Planar Cap And Close Solid | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-10-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-006 - Fillet And Chamfer B-Rep/l3.json>) — Fillet And Chamfer B-Rep Edges | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-10-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-007 - Offset Face And Thicken/l3.json>) — Offset Face And Thicken Shell | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-10-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-008 - Face Extract Unjoin/l3.json>) — Face Extract Unjoin And Rejoin | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-10-009](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-10 - 02-be-occt-boolean-fillet-shell-ops/L3-009 - Geometry Health/l3.json>) — Geometry Health And Naked Edge Query | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-01-11 — 8 L3

Owner: [03-be-occt-transforms-and-pattern-ops](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L2.json>). Matrix delta đã xuất; mirror/array/frames/orient/copy phải audit rồi xuất từng capability.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-11-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L3-001 - Translation Rotation Scaling Matrices/l3.json>) — Translation Rotation Scaling Matrices | EVIDENCE-SCOPED subset Windows | SDK-SUBSET: world matrix delta |
| [OM9-L3-01-11-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L3-002 - Mirror Across Plane/l3.json>) — Mirror Across Plane | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-11-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L3-003 - Nonuniform Scaling And Units/l3.json>) — Nonuniform Scaling And Units | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-11-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L3-004 - Polar And Linear Instance Pattern/l3.json>) — Polar And Linear Instance Pattern | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-11-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L3-005 - Curve Sampled Placement Frames/l3.json>) — Curve Sampled Placement Frames | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-11-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L3-006 - Surface UV Normal Placement Frames/l3.json>) — Surface UV Normal Placement Frames | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-11-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L3-007 - Orientation Two Three Point Solver/l3.json>) — Orientation Two Three Point Solver | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-11-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-11 - 03-be-occt/L3-008 - Array Bake Vs Instance Policy/l3.json>) — Array Bake Vs Instance Policy | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-01-13 — 12 L3

Owner: [14-febe-import-rhino-3dm](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L2.json>). Export native exchange service hiện có; giữ strict admission, subtype và trim limitations.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-13-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-001 - 3dm File Signature And Version Probe/l3.json>) — .3dm File Signature And Version Probe | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-002 - Opennurbs Reader Bridge Contract/l3.json>) — Opennurbs Reader Bridge Contract | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-003 - 3dm Document Units Tolerance Mapping/l3.json>) — .3dm Document Units Tolerance Mapping | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-004 - 3dm Layer And Object Attribute Mapping/l3.json>) — .3dm Layer And Object Attribute Mapping | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-005 - 3dm Analytic And NURBS Curves/l3.json>) — .3dm Analytic And NURBS Curves | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-006 - 3dm NURBS Surface And Trimmed B-Rep/l3.json>) — .3dm NURBS Surface And Trimmed B-Rep | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-007 - 3dm Mesh And Instance Blocks/l3.json>) — .3dm Mesh And Instance Blocks | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-008 - 3dm Unsupported Proprietary Object Policy/l3.json>) — .3dm Unsupported Proprietary Object Policy | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-009](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-009 - 3dm Import Atomic Scene Transaction/l3.json>) — .3dm Import Atomic Scene Transaction | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-010](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-010 - 3dm File Picker And Options Dialog/l3.json>) — .3dm File Picker And Options Dialog | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-011](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-011 - 3dm Import Summary Warning UI/l3.json>) — .3dm Import Summary Warning UI | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-13-012](<../OM9_Milestones/M03 - Geometry 3DM/L2-01-13 - 14-febe-import-rhino-3dm/L3-012 - 3dm Golden Roundtrip Fixtures/l3.json>) — .3dm Golden Roundtrip Fixtures | RUNTIME-CLAIM | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-02-04 — 12 L3

Owner: [04-febe-freeform-curve-modeling](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L2.json>). Line/polyline/full circle đã có; NURBS/evaluation/edit export có oracle; phần thiếu làm kèm SDK.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-02-04-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-001 - Line Polyline Point Creation/l3.json>) — Line Polyline Point Creation | EVIDENCE-SCOPED subset Windows | SDK-SUBSET: Point/Line/Polyline |
| [OM9-L3-02-04-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-002 - Circle Arc Ellipse Polygon Creation/l3.json>) — Circle Arc Ellipse Polygon Creation | EVIDENCE-SCOPED subset Windows; SDK subset Windows (2026-10-21) | SDK-SUBSET: full Circle (XY + arbitrary frame) + `add_arc` 3-point + `add_ellipse` + `add_rectangle` + `add_polygon`; spiral/helix mở |
| [OM9-L3-02-04-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-003 - Interpolated And Controlpoint NURBS Curve/l3.json>) — Interpolated And Controlpoint NURBS Curve | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `add_interpolated_curve` (odd degree, chord/uniform) + `add_nurbs_curve` (control-point record); periodic/seam và edit mở |
| [OM9-L3-02-04-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-004 - Curve Edit Points And Grips/l3.json>) — Curve Edit Points And Grips | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-04-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-005 - Join Explode Close And Split Curves/l3.json>) — Join Explode Close And Split Curves | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-04-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-006 - Offset Fillet Chamfer Curves/l3.json>) — Offset Fillet Chamfer Curves | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-04-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-007 - Blend Arcblend And Match Curve Continuity/l3.json>) — Blend Arcblend And Match Curve Continuity | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-04-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-008 - Rebuild Refit And Knot Operations/l3.json>) — Rebuild Refit And Knot Operations | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-04-009](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-009 - Project Pullback And Curve Intersections/l3.json>) — Project Pullback And Curve Intersections | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-04-010](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-010 - Duplicate Edge Border Isocurve/l3.json>) — Duplicate Edge Border Isocurve | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-04-011](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-011 - Curve Seams Directions And Divisions/l3.json>) — Curve Seams Directions And Divisions | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-04-012](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-04 - 04/L3-012 - Sketch On Surface Mesh And Two Views/l3.json>) — Sketch On Surface Mesh And Two Views | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-02-05 — 10 L3

Owner: [05-febe-freeform-surface-modeling](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L2.json>). Bọc evaluate/loft/sweep selected path; trim/network/patch và UI workflow audit riêng.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-02-05-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-001 - Sweep1 Sweep2 And Profile Sweep Workflows/l3.json>) — Sweep1 Sweep2 And Profile Sweep Workflows | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-05-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-002 - Loft And Curve Network Workflows/l3.json>) — Loft And Curve Network Workflows | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-05-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-003 - Revolve Railrevolve And Extrude Workflows/l3.json>) — Revolve Railrevolve And Extrude Workflows | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-05-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-004 - Edge Planar And Patch Surface Workflows/l3.json>) — Edge Planar And Patch Surface Workflows | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-05-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-005 - Trim Split Untrim And Shrink Surface/l3.json>) — Trim Split Untrim And Shrink Surface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-05-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-006 - Surface Rebuild Refit And Softedit/l3.json>) — Surface Rebuild Refit And Softedit | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-05-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-007 - Surface Match Blend And Join Continuity/l3.json>) — Surface Match Blend And Join Continuity | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-05-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-008 - Surface Offset Fillet Chamfer/l3.json>) — Surface Offset Fillet Chamfer | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-05-009](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-009 - Surface UV Direction And Isocurve Handling/l3.json>) — Surface UV Direction And Isocurve Handling | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-05-010](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-05 - 05/L3-010 - Surface Normal Extrude And Unroll/l3.json>) — Surface Normal Extrude And Unroll | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-02-06 — 6 L3

Owner: [06-febe-rhino-transform-orient-workflows](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-06 - 06-febe-rhino/L2.json>). Transform/orient sessions gọi shared geometry service; options/pivot/copy/preview qua SDK contract.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-02-06-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-06 - 06-febe-rhino/L3-001 - Interactive Move Copy And Rotate/l3.json>) — Interactive Move Copy And Rotate | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-06-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-06 - 06-febe-rhino/L3-002 - Interactive Scale 1d 2d 3d/l3.json>) — Interactive Scale 1d 2d 3d | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-06-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-06 - 06-febe-rhino/L3-003 - Interactive Mirror And Array/l3.json>) — Interactive Mirror And Array | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-06-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-06 - 06-febe-rhino/L3-004 - Orient By Two Three Reference Points/l3.json>) — Orient By Two Three Reference Points | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-06-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-06 - 06-febe-rhino/L3-005 - Align And Setpoints/l3.json>) — Align And Setpoints | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-02-06-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-02-06 - 06-febe-rhino/L3-006 - Transform Cancel Preview And Undo/l3.json>) — Transform Cancel Preview And Undo | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-08 — 6 L3

Owner: [07-febe-matrix-top11-and-shortcut-actions](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-08 - 07-febe-matrix-top11/L2.json>). Top11/shortcuts là command adapter; công bố callable command khi shared implementation pass.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-08-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-08 - 07-febe-matrix-top11/L3-001 - Top11 Duplicate Action/l3.json>) — Top11 Duplicate Action | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-08-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-08 - 07-febe-matrix-top11/L3-002 - Top11 Editpoints And Controlpoints/l3.json>) — Top11 Editpoints And Controlpoints | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-08-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-08 - 07-febe-matrix-top11/L3-003 - Top11 Mirror Move Rotate Actions/l3.json>) — Top11 Mirror Move Rotate Actions | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-08-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-08 - 07-febe-matrix-top11/L3-004 - Top11 Explode Join Split Trim Actions/l3.json>) — Top11 Explode Join Split Trim Actions | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-08-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-08 - 07-febe-matrix-top11/L3-005 - Top11 Ringrail Entrypoint/l3.json>) — Top11 Ringrail Entrypoint; **đích M04** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-08-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-08 - 07-febe-matrix-top11/L3-006 - Top11 Toolbar Availability State/l3.json>) — Top11 Toolbar Availability State | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-16 — 58 L3

Owner: [08-febe-matrix-curve-menu-workflows](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L2.json>). Từng Curve menu L3 gọi shared Curve và đo đạc service; API và menu/CMD parity khi triển khai.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-16-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-001 - Curve · PolyLine - menu binding/l3.json>) — Curve · PolyLine — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-002 - Curve · Line - menu binding/l3.json>) — Curve · Line — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-003 - Curve · Interp Curve - menu binding/l3.json>) — Curve · Interp Curve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-004 - Curve · Rectangle - menu binding/l3.json>) — Curve · Rectangle — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-005 - Curve · Circle - menu binding/l3.json>) — Curve · Circle — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-006 - Curve · Ellipse - menu binding/l3.json>) — Curve · Ellipse — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-007 - Curve · Arc - menu binding/l3.json>) — Curve · Arc — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-008 - Curve · Arc Direction - menu binding/l3.json>) — Curve · Arc Direction — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-009](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-009 - Curve · Rebuild - menu binding/l3.json>) — Curve · Rebuild — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-010](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-010 - Curve · Refit to Tolerance - menu binding/l3.json>) — Curve · Refit to Tolerance — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-011](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-011 - Curve · Quick Curve Blend - menu binding/l3.json>) — Curve · Quick Curve Blend — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-012](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-012 - Curve · Adjustable Curve Blend - menu binding/l3.json>) — Curve · Adjustable Curve Blend — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-013](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-013 - Curve · Curve from 2 Views - menu binding/l3.json>) — Curve · Curve from 2 Views — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-014](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-014 - Curve · Curve 2 Views with History - menu binding/l3.json>) — Curve · Curve 2 Views with History — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-015](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-015 - Curve · Pullback - menu binding/l3.json>) — Curve · Pullback — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-016](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-016 - Curve · Project - menu binding/l3.json>) — Curve · Project — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-017](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-017 - Curve · Intersection - menu binding/l3.json>) — Curve · Intersection — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-018](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-018 - Curve · Offset Curve - menu binding/l3.json>) — Curve · Offset Curve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-019](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-019 - Curve · Offset Curve on Surface - menu binding/l3.json>) — Curve · Offset Curve on Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-020](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-020 - Curve · Extract Isocurve - menu binding/l3.json>) — Curve · Extract Isocurve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-021](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-021 - Curve · Fillet Curves - menu binding/l3.json>) — Curve · Fillet Curves — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-022](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-022 - Curve · Fillet Corners - menu binding/l3.json>) — Curve · Fillet Corners — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-023](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-023 - Curve · Chamfer Curves - menu binding/l3.json>) — Curve · Chamfer Curves — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-024](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-024 - Curve · Create UV Curves - menu binding/l3.json>) — Curve · Create UV Curves — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-025](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-025 - Curve · Apply UV Curves - menu binding/l3.json>) — Curve · Apply UV Curves — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-026](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-026 - Curve · Through Points - menu binding/l3.json>) — Curve · Through Points — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-027](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-027 - Curve · Extend Curve - menu binding/l3.json>) — Curve · Extend Curve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-028](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-028 - Curve · Polygon - menu binding/l3.json>) — Curve · Polygon — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-029](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-029 - Curve · Spiral - menu binding/l3.json>) — Curve · Spiral — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-030](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-030 - Curve · Helix - menu binding/l3.json>) — Curve · Helix — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-031](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-031 - Curve · Point - menu binding/l3.json>) — Curve · Point — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-032](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-032 - Curve · Mark Curve Start - menu binding/l3.json>) — Curve · Mark Curve Start — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-033](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-033 - Curve · Mark Curve End - menu binding/l3.json>) — Curve · Mark Curve End — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-034](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-034 - Curve · Adjust Closed Curve Seam - menu binding/l3.json>) — Curve · Adjust Closed Curve Seam — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-035](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-035 - Curve · Continue Interpolated Curve - menu binding/l3.json>) — Curve · Continue Interpolated Curve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-036](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-036 - Curve · Divide Curve - menu binding/l3.json>) — Curve · Divide Curve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-037](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-037 - Curve · Curve Boolean - menu binding/l3.json>) — Curve · Curve Boolean — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-038](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-038 - Curve · Tween Curves - menu binding/l3.json>) — Curve · Tween Curves — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-039](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-039 - Curve · Duplicate Edge - menu binding/l3.json>) — Curve · Duplicate Edge — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-040](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-040 - Curve · Duplicate Border - menu binding/l3.json>) — Curve · Duplicate Border — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-041](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-041 - Curve · Curve - menu binding/l3.json>) — Curve · Curve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-042](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-042 - Curve · Continue Control Point Curve - menu binding/l3.json>) — Curve · Continue Control Point Curve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-043](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-043 - Curve · Extract Angle Isocurve - menu binding/l3.json>) — Curve · Extract Angle Isocurve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-044](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-044 - Curve · Cross Section Profiles - menu binding/l3.json>) — Curve · Cross Section Profiles — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-045](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-045 - Curve · Sketch - menu binding/l3.json>) — Curve · Sketch — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-046](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-046 - Curve · Sketch on Surface - menu binding/l3.json>) — Curve · Sketch on Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-047](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-047 - Curve · Sketch on Mesh - menu binding/l3.json>) — Curve · Sketch on Mesh — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-048](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-048 - Curve · Polyline on Surface - menu binding/l3.json>) — Curve · Polyline on Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-049](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-049 - Curve · Interp Curve on Surface - menu binding/l3.json>) — Curve · Interp Curve on Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-050](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-050 - Curve · Curve to Lines - menu binding/l3.json>) — Curve · Curve to Lines — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-051](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-051 - Curve · Section - menu binding/l3.json>) — Curve · Section — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-052](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-052 - Curve · Match - menu binding/l3.json>) — Curve · Match — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-053](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-053 - Curve · Silhouette - menu binding/l3.json>) — Curve · Silhouette — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-054](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-054 - Curve · Extract Wire Frame - menu binding/l3.json>) — Curve · Extract Wire Frame — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-055](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-055 - Curve · Soft Edit - menu binding/l3.json>) — Curve · Soft Edit — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-056](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-056 - Curve · Offset Normal to Surface - menu binding/l3.json>) — Curve · Offset Normal to Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-057](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-057 - Curve · Arc Blend - menu binding/l3.json>) — Curve · Arc Blend — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-16-058](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-16 - 08/L3-058 - Curve · Intersect Two Sets - menu binding/l3.json>) — Curve · Intersect Two Sets — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-17 — 38 L3

Owner: [09-febe-matrix-surface-menu-workflows](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L2.json>). Từng Surface menu L3 gọi shared Surface service; không nhân đôi sweep/loft/trim.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-17-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-001 - Surface · Sweep 1 - menu binding/l3.json>) — Surface · Sweep 1 — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-002 - Surface · Sweep 1 History - menu binding/l3.json>) — Surface · Sweep 1 History — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-003 - Surface · Sweep 2 - menu binding/l3.json>) — Surface · Sweep 2 — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-004 - Surface · Sweep 2 History - menu binding/l3.json>) — Surface · Sweep 2 History — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-005 - Surface · Profile Sweep - menu binding/l3.json>) — Surface · Profile Sweep — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-006 - Surface · Rebuild - menu binding/l3.json>) — Surface · Rebuild — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-007 - Surface · Blend Surface - menu binding/l3.json>) — Surface · Blend Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-008 - Surface · Variable Blend Surfaces - menu binding/l3.json>) — Surface · Variable Blend Surfaces — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-009](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-009 - Surface · Loft - menu binding/l3.json>) — Surface · Loft — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-010](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-010 - Surface · Curve Network - menu binding/l3.json>) — Surface · Curve Network — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-011](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-011 - Surface · Shrink Trimmed Surface - menu binding/l3.json>) — Surface · Shrink Trimmed Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-012](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-012 - Surface · Patch - menu binding/l3.json>) — Surface · Patch — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-013](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-013 - Surface · Planar Curves - menu binding/l3.json>) — Surface · Planar Curves — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-014](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-014 - Surface · Edge Curves - menu binding/l3.json>) — Surface · Edge Curves — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-015](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-015 - Surface · Plane from Corners - menu binding/l3.json>) — Surface · Plane from Corners — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-016](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-016 - Surface · Untrim - menu binding/l3.json>) — Surface · Untrim — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-017](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-017 - Surface · Surface Extrude All - menu binding/l3.json>) — Surface · Surface Extrude All — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-018](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-018 - Surface · Rail Revolve - menu binding/l3.json>) — Surface · Rail Revolve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-019](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-019 - Surface · Sweep 4 - menu binding/l3.json>) — Surface · Sweep 4 — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-020](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-020 - Surface · Fillet Surfaces - menu binding/l3.json>) — Surface · Fillet Surfaces — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-021](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-021 - Surface · Variable Fillet Surfaces - menu binding/l3.json>) — Surface · Variable Fillet Surfaces — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-022](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-022 - Surface · Chamfer Surfaces - menu binding/l3.json>) — Surface · Chamfer Surfaces — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-023](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-023 - Surface · Variable Chamfer Surfaces - menu binding/l3.json>) — Surface · Variable Chamfer Surfaces — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-024](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-024 - Surface · Offset Surface - menu binding/l3.json>) — Surface · Offset Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-025](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-025 - Surface · Variable Offset Surface - menu binding/l3.json>) — Surface · Variable Offset Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-026](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-026 - Surface · Tween Surfaces - menu binding/l3.json>) — Surface · Tween Surfaces — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-027](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-027 - Surface · Merge Surfaces - menu binding/l3.json>) — Surface · Merge Surfaces — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-028](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-028 - Surface · Match Surfaces - menu binding/l3.json>) — Surface · Match Surfaces — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-029](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-029 - Surface · Drape - menu binding/l3.json>) — Surface · Drape — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-030](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-030 - Surface · Sweep Multi - menu binding/l3.json>) — Surface · Sweep Multi — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-031](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-031 - Surface · Split at Isocurve - menu binding/l3.json>) — Surface · Split at Isocurve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-032](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-032 - Surface · Extend Surface - menu binding/l3.json>) — Surface · Extend Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-033](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-033 - Surface · Extrude Normal to Surface - menu binding/l3.json>) — Surface · Extrude Normal to Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-034](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-034 - Surface · Unroll Surface - menu binding/l3.json>) — Surface · Unroll Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-035](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-035 - Surface · Soft Edit - menu binding/l3.json>) — Surface · Soft Edit — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-036](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-036 - Surface · Cutting Plane - menu binding/l3.json>) — Surface · Cutting Plane — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-037](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-037 - Surface · Adjust Closed Surface Seam - menu binding/l3.json>) — Surface · Adjust Closed Surface Seam — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-17-038](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-17 - 09-febe/L3-038 - Surface · Set Surface Tangent - menu binding/l3.json>) — Surface · Set Surface Tangent — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-18 — 21 L3

Owner: [10-febe-matrix-solid-menu-workflows](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L2.json>). Từng Solid menu L3 nối selected native BRep/Solid ops và SDK; không quảng cáo cả nhóm từ box/sphere.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-18-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-001 - Solid · Boolean Union - menu binding/l3.json>) — Solid · Boolean Union — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-002 - Solid · Boolean Difference - menu binding/l3.json>) — Solid · Boolean Difference — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-003 - Solid · Boolean Intersection - menu binding/l3.json>) — Solid · Boolean Intersection — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-004 - Solid · Boolean Two Objects - menu binding/l3.json>) — Solid · Boolean Two Objects — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-005 - Solid · Cap Planar - menu binding/l3.json>) — Solid · Cap Planar — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-006 - Solid · Extract Surface - menu binding/l3.json>) — Solid · Extract Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-007 - Solid · Fillet Edge - menu binding/l3.json>) — Solid · Fillet Edge — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-008 - Solid · Text - menu binding/l3.json>) — Solid · Text — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-009](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-009 - Solid · Extrude Curve Straight - menu binding/l3.json>) — Solid · Extrude Curve Straight — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-010](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-010 - Solid · Solid Extrude All - menu binding/l3.json>) — Solid · Solid Extrude All — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-011](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-011 - Solid · Pipe - menu binding/l3.json>) — Solid · Pipe — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-012](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-012 - Solid · Box - menu binding/l3.json>) — Solid · Box — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-013](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-013 - Solid · Sphere - menu binding/l3.json>) — Solid · Sphere — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-014](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-014 - Solid · Ellipsoid - menu binding/l3.json>) — Solid · Ellipsoid — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-015](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-015 - Solid · Torus - menu binding/l3.json>) — Solid · Torus — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-016](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-016 - Solid · Cylinder - menu binding/l3.json>) — Solid · Cylinder — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-017](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-017 - Solid · Tube - menu binding/l3.json>) — Solid · Tube — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-018](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-018 - Solid · Pyramid - menu binding/l3.json>) — Solid · Pyramid — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-019](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-019 - Solid · Cone - menu binding/l3.json>) — Solid · Cone — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-020](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-020 - Solid · Truncated Cone - menu binding/l3.json>) — Solid · Truncated Cone — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-18-021](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-18 - 10-febe/L3-021 - Solid · Shell - menu binding/l3.json>) — Solid · Shell — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-19 — 7 L3

Owner: [11-febe-matrix-utilities-and-mesh-health](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-19 - 11-febe-matrix/L2.json>). Geometry/mesh health report DTO; ShowEdges visualization là command/input/UI, backend validity theo native BRep/Solid.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-19-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-19 - 11-febe-matrix/L3-001 - Check Object Validity And Bad Objects/l3.json>) — Check Object Validity And Bad Objects | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-19-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-19 - 11-febe-matrix/L3-002 - Show And Classify Naked Edges/l3.json>) — Show And Classify Naked Edges | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-19-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-19 - 11-febe-matrix/L3-003 - Bounding Box And Center Object/l3.json>) — Bounding Box And Center Object | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-19-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-19 - 11-febe-matrix/L3-004 - Group And Ungroup Selection/l3.json>) — Group And Ungroup Selection | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-19-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-19 - 11-febe-matrix/L3-005 - Mesh From NURBS For Production/l3.json>) — Mesh From NURBS For Production | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-19-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-19 - 11-febe-matrix/L3-006 - Mesh Reduce Basic/l3.json>) — Mesh Reduce Basic | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-19-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-19 - 11-febe-matrix/L3-007 - Object Health Report Panel/l3.json>) — Object Health Report Panel | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-20 — 5 L3

Owner: [12-febe-matrix-measure-and-model-statistics](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-20 - 12-febe-matrix/L2.json>). Xuất distance/angle trước; length/area/volume/statistics theo backend và exact owner.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-20-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-20 - 12-febe-matrix/L3-001 - Distance And Length Measurement/l3.json>) — Distance And Length Measurement | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `measure_distance` + `measure_curve_length` (sampled); exact/adaptive length mở |
| [OM9-L3-03-20-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-20 - 12-febe-matrix/L3-002 - Angle And Radius Measurement/l3.json>) — Angle And Radius Measurement | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `measure_angle` + `measure_curve_radius` (circle record); ellipse radius mở |
| [OM9-L3-03-20-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-20 - 12-febe-matrix/L3-003 - Bounding Size And Center Measurement/l3.json>) — Bounding Size And Center Measurement | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `object_bounds` (point/curve-sample/mesh); B-Rep/surface/cloud và group bounds mở |
| [OM9-L3-03-20-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-20 - 12-febe-matrix/L3-004 - Jewelry Object Count And Type Summary/l3.json>) — Jewelry Object Count And Type Summary | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `object_summary` (counts by geometry kind); jewelry class/weight summary mở |
| [OM9-L3-03-20-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-03-20 - 12-febe-matrix/L3-005 - Measurement Label And Command Feedback/l3.json>) — Measurement Label And Command Feedback | EVIDENCE-SCOPED core; SDK subset Windows (2026-10-21) | SDK-SUBSET: `report_measurement` (Progress line); structured record/UI acceptance mở |

#### OM9-L2-04-01 — 42 L3

Owner: [13-febe-matrix-transform-toolbar-bridges](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L2.json>). Toolbar Transform là adapter selection và context/command/input/UI; tái sử dụng exact operator và transaction.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-01-001](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-001 - Transform · Scale 3D - menu binding/l3.json>) — Transform · Scale 3D — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-002](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-002 - Transform · Scale2D - menu binding/l3.json>) — Transform · Scale2D — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-003](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-003 - Transform · Scale1D - menu binding/l3.json>) — Transform · Scale1D — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-004](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-004 - Transform · Scale Non-Uniform - menu binding/l3.json>) — Transform · Scale Non-Uniform — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-005](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-005 - Transform · Orient 2 Points - menu binding/l3.json>) — Transform · Orient 2 Points — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-006](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-006 - Transform · Orient 3 Points - menu binding/l3.json>) — Transform · Orient 3 Points — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-007](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-007 - Transform · Rotate3D - menu binding/l3.json>) — Transform · Rotate3D — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-008](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-008 - Transform · Bend - menu binding/l3.json>) — Transform · Bend — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-009](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-009 - Transform · Array Polar - menu binding/l3.json>) — Transform · Array Polar — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-010](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-010 - Transform · Project To C-Plane - menu binding/l3.json>) — Transform · Project To C-Plane — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-011](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-011 - Transform · Smart Flow - menu binding/l3.json>) — Transform · Smart Flow — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-012](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-012 - Transform · Smart Flow Rigid - menu binding/l3.json>) — Transform · Smart Flow Rigid — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-013](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-013 - Transform · Smart Flow Break - menu binding/l3.json>) — Transform · Smart Flow Break — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-014](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-014 - Transform · Smart Flow Pull Down Curves - menu binding/l3.json>) — Transform · Smart Flow Pull Down Curves — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-015](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-015 - Transform · Orient on Surface - menu binding/l3.json>) — Transform · Orient on Surface — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-016](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-016 - Transform · Orient Perpendicular to Curve - menu binding/l3.json>) — Transform · Orient Perpendicular to Curve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-017](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-017 - Transform · Orient on Curve - menu binding/l3.json>) — Transform · Orient on Curve — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-018](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-018 - Transform · Symmetry - menu binding/l3.json>) — Transform · Symmetry — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-019](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-019 - Transform · Solid Points ON - menu binding/l3.json>) — Transform · Solid Points ON — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-020](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-020 - Transform · Cage Edit - menu binding/l3.json>) — Transform · Cage Edit — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-021](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-021 - Transform · ReleaseFromCage - menu binding/l3.json>) — Transform · ReleaseFromCage — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-022](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-022 - Transform · Array Along Curve - menu binding/l3.json>) — Transform · Array Along Curve — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-023](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-023 - Transform · Array Linear - menu binding/l3.json>) — Transform · Array Linear — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-024](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-024 - Transform · Array Along Surface - menu binding/l3.json>) — Transform · Array Along Surface — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-025](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-025 - Transform · Array Curve on Surface - menu binding/l3.json>) — Transform · Array Curve on Surface — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-026](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-026 - Transform · Array - menu binding/l3.json>) — Transform · Array — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-027](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-027 - Transform · Flow Along Curve - menu binding/l3.json>) — Transform · Flow Along Curve — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-028](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-028 - Transform · Flow Along Surface - menu binding/l3.json>) — Transform · Flow Along Surface — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-029](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-029 - Transform · Splop - menu binding/l3.json>) — Transform · Splop — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-030](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-030 - Transform · Set Points - menu binding/l3.json>) — Transform · Set Points — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-031](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-031 - Transform · Scale By Plane - menu binding/l3.json>) — Transform · Scale By Plane — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-032](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-032 - Transform · Stretch - menu binding/l3.json>) — Transform · Stretch — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-033](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-033 - Transform · Taper - menu binding/l3.json>) — Transform · Taper — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-034](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-034 - Transform · Shear - menu binding/l3.json>) — Transform · Shear — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-035](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-035 - Transform · Twist - menu binding/l3.json>) — Transform · Twist — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-036](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-036 - Transform · Maelstrom - menu binding/l3.json>) — Transform · Maelstrom — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-037](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-037 - Transform · Make Down Facing - menu binding/l3.json>) — Transform · Make Down Facing — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-038](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-038 - Transform · Remap to C-Plane - menu binding/l3.json>) — Transform · Remap to C-Plane — menu binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-039](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-039 - Transform · MoveUVN - menu binding/l3.json>) — Transform · MoveUVN — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-040](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-040 - Transform · Soft Move - menu binding/l3.json>) — Transform · Soft Move — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-041](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-041 - Transform · Create Cage - menu binding/l3.json>) — Transform · Create Cage — menu binding; **đích M08** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-01-042](<../OM9_Milestones/M03 - Geometry 3DM/L2-04-01 - 13-febe/L3-042 - Transform menu action availability/l3.json>) — Transform menu action availability | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

## M04 — Builder, History và Ring

Nguồn: [L1 canonical](<../OM9_Milestones/M04 - Builder and History/L1.json>). 7 L2 / 51 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M01.

DAG/History/Builder/preset/ring/weight tích hợp contract plugin ngay trong owner hiện tại; không để Builder hoạt động xong rồi lập một dự án SDK khác.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-L2-01-04](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L2.json>) — DAG/history tối thiểu cho rail/profile/gems/settings/cutters, dirty propagation, rollback, stable semantic references. | SOURCE-PRESENT history/provenance | EXPORT-BACKLOG + CO-IMPLEMENT gaps | Feature schema/DAG/rebuild dùng core; không suy Builder UI hoàn tất từ history backend. |

| [OM9-L2-03-11](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L2.json>) — Builder panel, live preview, viewport control handles | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Builder parameters/preview service + UI contribution, lifetime và cancellation theo command/input/UI/History và Builders. |

| [OM9-L2-03-12](<../OM9_Milestones/M04 - Builder and History/L2-03-12 - 03-febe-matrix-builder-styles-and-presets/L2.json>) — Preset/Style theo Builder, snapshot các tham số thiết kế | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Versioned style/preset DTO, serialization/validation; Builder UI và plugin dùng cùng dữ liệu. |

| [OM9-L2-03-13](<../OM9_Milestones/M04 - Builder and History/L2-03-13 - 04-febe-matrix-history/L2.json>) — UI bật/tắt và chỉnh association của builder history | SOURCE-PRESENT core; UI audit | EXPORT-BACKLOG + AUDIT-THEN-ROUTE | History service public trước; UI toggle/update và plugin feature persistence cùng contract. |

| [OM9-L2-04-02](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L2.json>) — Ring rail, outside rail, profile placer, ring resizer | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Ring rail/profile/resize service; history/units và SDK cùng lượt khi thực thi. |

| [OM9-L2-04-03](<../OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L2.json>) — Eternity/Signet/Raised/Award ring builders theo NURBS | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Ring-family typed builders theo từng L3; không dùng private UI algorithm để tạo geometry. |

| [OM9-L2-04-19](<../OM9_Milestones/M04 - Builder and History/L2-04-19 - 07-be-jewelry-metal-weight-calculations/L2.json>) — Tính thể tích, vật liệu, trọng lượng kim loại trang sức | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Volume/material/density/alloy/weight DTO với đơn vị rõ; audit công thức và native volume riêng. |

### Checklist tích hợp khi thực hiện milestone

#### Checklist: History, Builders, presets và domain-neutral metadata

- [ ] Audit core DAG/provenance/results/workers và nối feature DTO thay vì serialize Rust implementation structs.
- [ ] Feature registration có input/output refs, parameters schema, plugin/version và deterministic rebuild contract; graph cycle/missing dependency được báo rõ.
- [ ] Source revision/permissions/context checked lúc publish; stale async output bị bỏ, không ghi đè sửa đổi mới.
- [ ] Parent delete/rebuild failure/plugin missing/schema migration có policy rõ; không tự xóa downstream hoặc giả geometry thành công.
- [ ] Presets/styles tách dữ liệu khỏi UI; Builder parameter edits và live handles dùng cùng service như SDK.
- [ ] Ring rail/profile/resize/ring-family và metal weight audit độc lập. Core volume không chứng minh đã có bảng mật độ, alloy hay toàn bộ ring builder.

**Owner:** toàn bộ 7 L2 M04, đặc biệt `01-04`, `03-11`, `03-12`, `03-13`. L3 `OM9-L3-01-04-001`–`009` là xương sống History. **Đóng:** chained rebuild, failed rebuild atomic, plugin unload/missing/reload, Undo/Redo/save-open và UI/plugin parity; domain-specific owner chỉ đóng theo test riêng.

### Theo dõi L3 trong milestone

#### OM9-L2-01-04 — 9 L3

Owner: [01-be-associative-dependency-rebuild](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L2.json>). Feature schema/DAG/rebuild dùng core; không suy Builder UI hoàn tất từ history backend.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-01-04-001](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L3-001 - Feature Record And Typed Dependency Edges/l3.json>) — Feature Record And Typed Dependency Edges | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-04-002](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L3-002 - Feature Provenance From Command/l3.json>) — Feature Provenance From Command | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-04-003](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L3-003 - Dirty Propagation And Topological Order/l3.json>) — Dirty Propagation And Topological Order | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-04-004](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L3-004 - Stable Semantic Reference Linking/l3.json>) — Stable Semantic Reference Linking | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-04-005](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L3-005 - Incremental Recompute And Cache/l3.json>) — Incremental Recompute And Cache | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-04-006](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L3-006 - Builder Preview Vs Committed Features/l3.json>) — Builder Preview Vs Committed Features | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-04-007](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L3-007 - History Recompute Failure Rollback/l3.json>) — History Recompute Failure Rollback | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-04-008](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L3-008 - Delete Parent Or Change Input Policy/l3.json>) — Delete Parent Or Change Input Policy | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-01-04-009](<../OM9_Milestones/M04 - Builder and History/L2-01-04 - 01-be-associative-dependency-rebuild/L3-009 - Associative Rebuild Geometry Fixtures/l3.json>) — Associative Rebuild Geometry Fixtures | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-11 — 9 L3

Owner: [02-febe-matrix-builder-ui-and-live-handles](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L2.json>). Builder parameters/preview service + UI contribution, lifetime và cancellation theo command/input/UI/History và Builders.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-11-001](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L3-001 - Builder Typed Input Boxes/l3.json>) — Builder Typed Input Boxes | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-11-002](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L3-002 - Builder Mode Tabs And Context/l3.json>) — Builder Mode Tabs And Context | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-11-003](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L3-003 - Builder Parameter Controls And Sliders/l3.json>) — Builder Parameter Controls And Sliders | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-11-004](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L3-004 - Builder Live Preview Lifecycle/l3.json>) — Builder Live Preview Lifecycle | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-11-005](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L3-005 - Builder Viewport Control Handles/l3.json>) — Builder Viewport Control Handles | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-11-006](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L3-006 - Builder Handle And Panel Bidirectional Sync/l3.json>) — Builder Handle And Panel Bidirectional Sync | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-11-007](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L3-007 - Builder Profile Selector Window/l3.json>) — Builder Profile Selector Window | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-11-008](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L3-008 - Builder Accept Cancel Reset And Undo/l3.json>) — Builder Accept Cancel Reset And Undo | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-11-009](<../OM9_Milestones/M04 - Builder and History/L2-03-11 - 02-febe-matrix-builder-ui-and-live-handles/L3-009 - Builder F6 And Commandline Input Binding/l3.json>) — Builder F6 And Commandline Input Binding | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-12 — 6 L3

Owner: [03-febe-matrix-builder-styles-and-presets](<../OM9_Milestones/M04 - Builder and History/L2-03-12 - 03-febe-matrix-builder-styles-and-presets/L2.json>). Versioned style/preset DTO, serialization/validation; Builder UI và plugin dùng cùng dữ liệu.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-12-001](<../OM9_Milestones/M04 - Builder and History/L2-03-12 - 03-febe-matrix-builder-styles-and-presets/L3-001 - Builder Style Param Snapshot Schema/l3.json>) — Builder Style Param Snapshot Schema | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-12-002](<../OM9_Milestones/M04 - Builder and History/L2-03-12 - 03-febe-matrix-builder-styles-and-presets/L3-002 - Builder Style Browser And Thumbnail/l3.json>) — Builder Style Browser And Thumbnail | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-12-003](<../OM9_Milestones/M04 - Builder and History/L2-03-12 - 03-febe-matrix-builder-styles-and-presets/L3-003 - Builder Style Apply And Preview/l3.json>) — Builder Style Apply And Preview | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-12-004](<../OM9_Milestones/M04 - Builder and History/L2-03-12 - 03-febe-matrix-builder-styles-and-presets/L3-004 - Builder Style Save New Name/l3.json>) — Builder Style Save New Name | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-12-005](<../OM9_Milestones/M04 - Builder and History/L2-03-12 - 03-febe-matrix-builder-styles-and-presets/L3-005 - Builder Style Mode And Shape Filter/l3.json>) — Builder Style Mode And Shape Filter | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-12-006](<../OM9_Milestones/M04 - Builder and History/L2-03-12 - 03-febe-matrix-builder-styles-and-presets/L3-006 - Builder Style Match Attributes/l3.json>) — Builder Style Match Attributes | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-03-13 — 6 L3

Owner: [04-febe-matrix-history-associative-updates](<../OM9_Milestones/M04 - Builder and History/L2-03-13 - 04-febe-matrix-history/L2.json>). History service public trước; UI toggle/update và plugin feature persistence cùng contract.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-03-13-001](<../OM9_Milestones/M04 - Builder and History/L2-03-13 - 04-febe-matrix-history/L3-001 - Matrix History Record Toggle/l3.json>) — Matrix History Record Toggle | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-13-002](<../OM9_Milestones/M04 - Builder and History/L2-03-13 - 04-febe-matrix-history/L3-002 - Matrix History Update Toggle/l3.json>) — Matrix History Update Toggle | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-13-003](<../OM9_Milestones/M04 - Builder and History/L2-03-13 - 04-febe-matrix-history/L3-003 - Matrix History Broken Link Report/l3.json>) — Matrix History Broken Link Report | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-13-004](<../OM9_Milestones/M04 - Builder and History/L2-03-13 - 04-febe-matrix-history/L3-004 - Matrix Clear Single Object History/l3.json>) — Matrix Clear Single Object History | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-13-005](<../OM9_Milestones/M04 - Builder and History/L2-03-13 - 04-febe-matrix-history/L3-005 - Edit Existing Builder By History/l3.json>) — Edit Existing Builder By History | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-03-13-006](<../OM9_Milestones/M04 - Builder and History/L2-03-13 - 04-febe-matrix-history/L3-006 - History Commandline And Preview Status/l3.json>) — History Commandline And Preview Status | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-02 — 9 L3

Owner: [05-febe-ring-rail-profile-and-resize-tools](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L2.json>). Ring rail/profile/resize service; history/units và SDK cùng lượt khi thực thi.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-02-001](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L3-001 - Ring Rail Jewelry Size Standard/l3.json>) — Ring Rail Jewelry Size Standard | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-02-002](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L3-002 - Outside Ring Rail Generator/l3.json>) — Outside Ring Rail Generator | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-02-003](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L3-003 - Profile Placer On Rail/l3.json>) — Profile Placer On Rail | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-02-004](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L3-004 - Custom Rail And Advanced Rail/l3.json>) — Custom Rail And Advanced Rail | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-02-005](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L3-005 - Four Rail Profile Tool/l3.json>) — Four Rail Profile Tool | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-02-006](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L3-006 - Profile Edit Merge And Endcap/l3.json>) — Profile Edit Merge And Endcap | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-02-007](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L3-007 - Ring Resizer Builder/l3.json>) — Ring Resizer Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-02-008](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L3-008 - Rail Profile Sweep Associative Output/l3.json>) — Rail Profile Sweep Associative Output | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-02-009](<../OM9_Milestones/M04 - Builder and History/L2-04-02 - 05-febe-ring-rail-profile-and-resize-tools/L3-009 - Rail And Profile Viewport Handles/l3.json>) — Rail And Profile Viewport Handles | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-03 — 7 L3

Owner: [06-febe-jewelry-ring-builder-family](<../OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L2.json>). Ring-family typed builders theo từng L3; không dùng private UI algorithm để tạo geometry.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-03-001](<../OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L3-001 - Eternity Band Builder/l3.json>) — Eternity Band Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-03-002](<../OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L3-002 - Signet Ring Builder/l3.json>) — Signet Ring Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-03-003](<../OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L3-003 - Raised Band Builder/l3.json>) — Raised Band Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-03-004](<../OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L3-004 - Award Ring Builder/l3.json>) — Award Ring Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-03-005](<../OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L3-005 - Ring Builder Style Presets/l3.json>) — Ring Builder Style Presets | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-03-006](<../OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L3-006 - Ring Builder History Updates/l3.json>) — Ring Builder History Updates | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-03-007](<../OM9_Milestones/M04 - Builder and History/L2-04-03 - 06-febe-jewelry-ring-builder-family/L3-007 - Ring Builder Final B-Rep Validation/l3.json>) — Ring Builder Final B-Rep Validation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-19 — 5 L3

Owner: [07-be-jewelry-metal-weight-calculations](<../OM9_Milestones/M04 - Builder and History/L2-04-19 - 07-be-jewelry-metal-weight-calculations/L2.json>). Volume/material/density/alloy/weight DTO với đơn vị rõ; audit công thức và native volume riêng.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-19-001](<../OM9_Milestones/M04 - Builder and History/L2-04-19 - 07-be-jewelry-metal-weight-calculations/L3-001 - Metal Volume From Closed B-Rep/l3.json>) — Metal Volume From Closed B-Rep | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-19-002](<../OM9_Milestones/M04 - Builder and History/L2-04-19 - 07-be-jewelry-metal-weight-calculations/L3-002 - Metal Material Density Registry/l3.json>) — Metal Material Density Registry | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-19-003](<../OM9_Milestones/M04 - Builder and History/L2-04-19 - 07-be-jewelry-metal-weight-calculations/L3-003 - Metal Estimated Mass Calculation/l3.json>) — Metal Estimated Mass Calculation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-19-004](<../OM9_Milestones/M04 - Builder and History/L2-04-19 - 07-be-jewelry-metal-weight-calculations/L3-004 - Multi Material Model Mass Breakdown/l3.json>) — Multi Material Model Mass Breakdown | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-19-005](<../OM9_Milestones/M04 - Builder and History/L2-04-19 - 07-be-jewelry-metal-weight-calculations/L3-005 - Metal Weight Error And Open B-Rep Policy/l3.json>) — Metal Weight Error And Open B-Rep Policy | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

## M05 — Gems và Layout

Nguồn: [L1 canonical](<../OM9_Milestones/M05 - Gems and Layout/L1.json>). 5 L2 / 43 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M03.

Gem assets, instances, placement/layout và báo cáo có typed public service khi triển khai từng L3. Audit standalone trước, giữ các bằng chứng FreeCAD ở phạm vi lịch sử.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-L2-04-09](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L2.json>) — Gem shapes/cuts, size, loader/instances | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-10](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L2.json>) — Gems on Curve/Surface, spacing, orientation, history | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-11](<../OM9_Milestones/M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster/L2.json>) — Halo/Cluster/Baguette Channel/Between Curves | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-12](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L2.json>) — Auto Pavé/Pavé/Azure layout | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-13](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L2.json>) — Gem Control/Map/Reporter, gemstone editing | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

### Checklist tích hợp khi thực hiện milestone

Audit standalone service trước. Mỗi dòng sau là nhiệm vụ SDK của owner thuộc milestone này, thực hiện cùng feature, không chuyển sang hàng đợi SDK bên ngoài.

| Milestone/owner | Service SDK đi cùng implementation | Điều kiện tích hợp |
|---|---|---|

| M05 `04-09` | Gem cut/size metadata, catalog asset loading, instance creation. | Asset provenance/license, dimensions, units, shared instance placement và round-trip. |

| M05 `04-10` | Gem layout on curve/surface, spacing/orientation options, result IDs. | Curve/surface frames, placement/collision policy, deterministic preview/commit/history. |

| M05 `04-11`, `04-12` | Halo/cluster/baguette/pavé/azure services. | Exact owner parameters and constraints; invalid inputs rollback, no UI-only layout algorithm. |

| M05 `04-13` | Gem query/edit/map/report DTO. | Không đếm mesh triangles như số viên; nested instance và quantity/size đúng dữ liệu. |

- [ ] Dùng đúng dependency geometry/history của owner; app và plugin gọi cùng service.
- [ ] Plugin ngoài repository tạo/chỉnh workflow thật, bảo toàn units/permissions/Undo/Cancel/save-open.
- [ ] Ghi capability và target evidence; phần chưa pass giữ mở tại chính owner.

### Theo dõi L3 trong milestone

#### OM9-L2-04-09 — 8 L3

Owner: [01-febe-gem-asset-shapes-and-loader](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-09-001](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L3-001 - Gem Shape Cut Dimension Schema/l3.json>) — Gem Shape Cut Dimension Schema | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-09-002](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L3-002 - Gem Reference Mesh And B-Rep Assets/l3.json>) — Gem Reference Mesh And B-Rep Assets | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-09-003](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L3-003 - Gem Loader Shape Cut Size Picker/l3.json>) — Gem Loader Shape Cut Size Picker | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-09-004](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L3-004 - Gem Custom Shape From Profile/l3.json>) — Gem Custom Shape From Profile | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-09-005](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L3-005 - Gem Emerald And Baguette Builders/l3.json>) — Gem Emerald And Baguette Builders | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-09-006](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L3-006 - Gem Instance Create Rotate And Position/l3.json>) — Gem Instance Create Rotate And Position | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-09-007](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L3-007 - Gem Asset Properties And Layer Defaults/l3.json>) — Gem Asset Properties And Layer Defaults | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-09-008](<../OM9_Milestones/M05 - Gems and Layout/L2-04-09 - 01-febe-gem-asset-shapes-and-loader/L3-008 - Gem Facet Outline Girdle Reference API/l3.json>) — Gem Facet Outline Girdle Reference API | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-10 — 11 L3

Owner: [02-febe-gem-on-curves-and-surfaces](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-10-001](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-001 - Gem On Curve Basic/l3.json>) — Gem On Curve Basic | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-002](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-002 - Gem On Curve Advanced/l3.json>) — Gem On Curve Advanced | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-003](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-003 - Gem Count On Curve/l3.json>) — Gem Count On Curve | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-004](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-004 - Gem List On Curve/l3.json>) — Gem List On Curve | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-005](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-005 - Gem On Curve Multi/l3.json>) — Gem On Curve Multi | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-006](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-006 - Gem Profile Curve And Guides/l3.json>) — Gem Profile Curve And Guides | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-007](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-007 - Gem On Surface/l3.json>) — Gem On Surface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-008](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-008 - Gem Follow Underlying Curve/l3.json>) — Gem Follow Underlying Curve | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-009](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-009 - Gem Flow Placement/l3.json>) — Gem Flow Placement | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-010](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-010 - Gem Splop On Surface/l3.json>) — Gem Splop On Surface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-10-011](<../OM9_Milestones/M05 - Gems and Layout/L2-04-10 - 02-febe-gem-on-curves-and-surfaces/L3-011 - Gem To Curve Surface History Links/l3.json>) — Gem To Curve Surface History Links | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-11 — 7 L3

Owner: [03-febe-gem-halo-cluster-baguette-layouts](<../OM9_Milestones/M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-11-001](<../OM9_Milestones/M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster/L3-001 - Halo Builder/l3.json>) — Halo Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-11-002](<../OM9_Milestones/M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster/L3-002 - Cluster Builder/l3.json>) — Cluster Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-11-003](<../OM9_Milestones/M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster/L3-003 - Baguette Channel Layout/l3.json>) — Baguette Channel Layout | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-11-004](<../OM9_Milestones/M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster/L3-004 - Tapered Baguettes Between Two Curves/l3.json>) — Tapered Baguettes Between Two Curves | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-11-005](<../OM9_Milestones/M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster/L3-005 - Gems Between Two Curves/l3.json>) — Gems Between Two Curves | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-11-006](<../OM9_Milestones/M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster/L3-006 - Orient To Gem/l3.json>) — Orient To Gem | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-11-007](<../OM9_Milestones/M05 - Gems and Layout/L2-04-11 - 03-febe-gem-halo-cluster/L3-007 - Halo Cluster Edit Parameters/l3.json>) — Halo Cluster Edit Parameters | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-12 — 8 L3

Owner: [04-febe-gem-pave-layout-and-azure](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-12-001](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L3-001 - Auto Pave Builder/l3.json>) — Auto Pave Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-12-002](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L3-002 - Manual Pave Builder/l3.json>) — Manual Pave Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-12-003](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L3-003 - Pave Azure Builder/l3.json>) — Pave Azure Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-12-004](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L3-004 - Pave Prong Layout/l3.json>) — Pave Prong Layout | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-12-005](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L3-005 - Pave Sphere Layout/l3.json>) — Pave Sphere Layout | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-12-006](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L3-006 - Gem Springs And Relaxation/l3.json>) — Gem Springs And Relaxation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-12-007](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L3-007 - Pave Region Controls And Preview/l3.json>) — Pave Region Controls And Preview | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-12-008](<../OM9_Milestones/M05 - Gems and Layout/L2-04-12 - 04-febe-gem-pave-layout-and-azure/L3-008 - Pave History Geometry Refresh/l3.json>) — Pave History Geometry Refresh | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-13 — 9 L3

Owner: [05-febe-gem-instance-edits-and-reports](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-13-001](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L3-001 - Gem Positioner MSR/l3.json>) — Gem Positioner MSR | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-13-002](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L3-002 - Gem Control And Size Edits/l3.json>) — Gem Control And Size Edits | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-13-003](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L3-003 - Gem Match Attributes/l3.json>) — Gem Match Attributes | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-13-004](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L3-004 - Gem Style Save/l3.json>) — Gem Style Save | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-13-005](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L3-005 - Gem Style Load/l3.json>) — Gem Style Load | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-13-006](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L3-006 - Gem Update And Regenerate/l3.json>) — Gem Update And Regenerate | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-13-007](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L3-007 - Gem Map Viewport Overlay/l3.json>) — Gem Map Viewport Overlay | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-13-008](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L3-008 - Gem Reporter Count And Weight/l3.json>) — Gem Reporter Count And Weight | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-13-009](<../OM9_Milestones/M05 - Gems and Layout/L2-04-13 - 05-febe-gem-instance-edits-and-reports/L3-009 - Gem Attribute History And ID Preservation/l3.json>) — Gem Attribute History And ID Preservation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

## M06 — Settings và Prongs

Nguồn: [L1 canonical](<../OM9_Milestones/M06 - Settings and Prongs/L1.json>). 2 L2 / 19 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M03, M05.

Head/bezel/prong/channel/bead dùng chung domain service cho app và plugin; adapter SDK và native geometry proof là đầu việc của chính owner settings.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-L2-04-14](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L2.json>) — Head Builder, Bezel Builder với NURBS/BRep | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-15](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L2.json>) — Prong/Channel/Metal Piece/Bead settings | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

### Checklist tích hợp khi thực hiện milestone

Audit standalone service trước. Mỗi dòng sau là nhiệm vụ SDK của owner thuộc milestone này, thực hiện cùng feature, không chuyển sang hàng đợi SDK bên ngoài.

| Milestone/owner | Service SDK đi cùng implementation | Điều kiện tích hợp |
|---|---|---|

| M06 `04-14` | Head/bezel construction và editable result refs. | BRep validity, size/clearance inputs, history, expected ring/gem relationship. |

| M06 `04-15` | Prong/channel/metal/bead services. | Placement/clearance/material settings đúng milestone; preview không thay committed doc. |

- [ ] Dùng đúng dependency geometry/history của owner; app và plugin gọi cùng service.
- [ ] Plugin ngoài repository tạo/chỉnh workflow thật, bảo toàn units/permissions/Undo/Cancel/save-open.
- [ ] Ghi capability và target evidence; phần chưa pass giữ mở tại chính owner.

### Theo dõi L3 trong milestone

#### OM9-L2-04-14 — 9 L3

Owner: [01-febe-head-and-bezel-brep-settings](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-14-001](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L3-001 - Head Builder Basket/l3.json>) — Head Builder Basket | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-14-002](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L3-002 - Head Builder Half Bezel Mode/l3.json>) — Head Builder Half Bezel Mode | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-14-003](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L3-003 - Head Builder Prong Rail Level Edit/l3.json>) — Head Builder Prong Rail Level Edit | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-14-004](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L3-004 - Bezel Builder Full/l3.json>) — Bezel Builder Full | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-14-005](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L3-005 - Bezel Builder Under Bezel And Gallery/l3.json>) — Bezel Builder Under Bezel And Gallery | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-14-006](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L3-006 - Scallop Bezel Builder/l3.json>) — Scallop Bezel Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-14-007](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L3-007 - Head Bezel Profile Browser/l3.json>) — Head Bezel Profile Browser | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-14-008](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L3-008 - Head Bezel Girdle Seat Attachment/l3.json>) — Head Bezel Girdle Seat Attachment | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-14-009](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-14 - 01-febe-head-and-bezel-brep-settings/L3-009 - Head Bezel Geometry Validity And Update/l3.json>) — Head Bezel Geometry Validity And Update | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-15 — 10 L3

Owner: [02-febe-prong-channel-metal-and-bead-settings](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-15-001](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-001 - Prong Builder Gem Line/l3.json>) — Prong Builder Gem Line | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-15-002](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-002 - Prong Editor Selected/l3.json>) — Prong Editor Selected | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-15-003](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-003 - Prong On Surface/l3.json>) — Prong On Surface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-15-004](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-004 - Metal Piece From Gems/l3.json>) — Metal Piece From Gems | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-15-005](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-005 - Channel Border Curve/l3.json>) — Channel Border Curve | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-15-006](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-006 - Bead On Curve Milgrain/l3.json>) — Bead On Curve Milgrain | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-15-007](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-007 - Bead On Surface/l3.json>) — Bead On Surface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-15-008](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-008 - Pull Object To Rail Setting/l3.json>) — Pull Object To Rail Setting | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-15-009](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-009 - Setting Row End Modes And Prong Spacing/l3.json>) — Setting Row End Modes And Prong Spacing | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-15-010](<../OM9_Milestones/M06 - Settings and Prongs/L2-04-15 - 02-febe-prong-channel-metal-and-bead/L3-010 - Settings Associative Edit And Mass/l3.json>) — Settings Associative Edit And Mass | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

## M07 — Cutters và kiểm tra chế tạo

Nguồn: [L1 canonical](<../OM9_Milestones/M07 - Cutters/L1.json>). 3 L2 / 19 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M03.

Cutter generation/apply, manufacturing checks và STL có SDK service cùng implementation. L3 STL OM9-L3-01-12-008 giữ owner gốc M01 nhưng thực thi ở M07; không nhân bản owner.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-L2-04-16](<../OM9_Milestones/M07 - Cutters/L2-04-16 - 01-febe-single-gem-seat-cutter-workflows/L2.json>) — Gem Seat/Gem Cutters cho 1 viên đá | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-17](<../OM9_Milestones/M07 - Cutters/L2-04-17 - 02-febe-channel-microprong-brightcut-cutters/L2.json>) — Channel/MicroProng/Bright Cut cutters | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-18](<../OM9_Milestones/M07 - Cutters/L2-04-18 - 03-febe-ring-finger-plane-quad-cutters/L2.json>) — Cut to finger rail/plane/cube/quad cut jewelry | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

### Checklist tích hợp khi thực hiện milestone

Audit standalone service trước. Mỗi dòng sau là nhiệm vụ SDK của owner thuộc milestone này, thực hiện cùng feature, không chuyển sang hàng đợi SDK bên ngoài.

| Milestone/owner | Service SDK đi cùng implementation | Điều kiện tích hợp |
|---|---|---|

| M07 `04-16` | Gem seat/cutter generation và tùy chọn apply. | Tách tạo cutter với boolean apply; explicit target/result IDs và volume oracle. |

| M07 `04-17` | Channel/microprong/bright-cut cutter services. | Reuse native ops, boolean failure atomic, consistent units/tolerance. |

| M07 `04-18` | Finger/plane/quad cutters. | Correct frame/plane, target set, empty/invalid intersection và Undo/save-reopen. |

- [ ] Dùng đúng dependency geometry/history của owner; app và plugin gọi cùng service.
- [ ] Plugin ngoài repository tạo/chỉnh workflow thật, bảo toàn units/permissions/Undo/Cancel/save-open.
- [ ] Ghi capability và target evidence; phần chưa pass giữ mở tại chính owner.

### Theo dõi L3 trong milestone

#### OM9-L2-04-16 — 7 L3

Owner: [01-febe-single-gem-seat-cutter-workflows](<../OM9_Milestones/M07 - Cutters/L2-04-16 - 01-febe-single-gem-seat-cutter-workflows/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-16-001](<../OM9_Milestones/M07 - Cutters/L2-04-16 - 01-febe-single-gem-seat-cutter-workflows/L3-001 - Gem Cutter Azure For Single Stone/l3.json>) — Gem Cutter Azure For Single Stone | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-16-002](<../OM9_Milestones/M07 - Cutters/L2-04-16 - 01-febe-single-gem-seat-cutter-workflows/L3-002 - Gem Cutter Multiple Stones/l3.json>) — Gem Cutter Multiple Stones | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-16-003](<../OM9_Milestones/M07 - Cutters/L2-04-16 - 01-febe-single-gem-seat-cutter-workflows/L3-003 - Gem Cutter Library Profiles/l3.json>) — Gem Cutter Library Profiles | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-16-004](<../OM9_Milestones/M07 - Cutters/L2-04-16 - 01-febe-single-gem-seat-cutter-workflows/L3-004 - Bezel Cutter Builder/l3.json>) — Bezel Cutter Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-16-005](<../OM9_Milestones/M07 - Cutters/L2-04-16 - 01-febe-single-gem-seat-cutter-workflows/L3-005 - Cutter Pavilion Clearance And Depth Controls/l3.json>) — Cutter Pavilion Clearance And Depth Controls | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-16-006](<../OM9_Milestones/M07 - Cutters/L2-04-16 - 01-febe-single-gem-seat-cutter-workflows/L3-006 - Interactive Boolean On Metal/l3.json>) — Interactive Boolean On Metal | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-16-007](<../OM9_Milestones/M07 - Cutters/L2-04-16 - 01-febe-single-gem-seat-cutter-workflows/L3-007 - Cutter Body Preserve Remove Option/l3.json>) — Cutter Body Preserve Remove Option | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-17 — 6 L3

Owner: [02-febe-channel-microprong-brightcut-cutters](<../OM9_Milestones/M07 - Cutters/L2-04-17 - 02-febe-channel-microprong-brightcut-cutters/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-17-001](<../OM9_Milestones/M07 - Cutters/L2-04-17 - 02-febe-channel-microprong-brightcut-cutters/L3-001 - Channel Builder Cutter/l3.json>) — Channel Builder Cutter | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-17-002](<../OM9_Milestones/M07 - Cutters/L2-04-17 - 02-febe-channel-microprong-brightcut-cutters/L3-002 - Microprong Cutter/l3.json>) — Microprong Cutter | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-17-003](<../OM9_Milestones/M07 - Cutters/L2-04-17 - 02-febe-channel-microprong-brightcut-cutters/L3-003 - Bright Cut Channel Builder/l3.json>) — Bright Cut Channel Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-17-004](<../OM9_Milestones/M07 - Cutters/L2-04-17 - 02-febe-channel-microprong-brightcut-cutters/L3-004 - Bright Cutter Single Gem/l3.json>) — Bright Cutter Single Gem | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-17-005](<../OM9_Milestones/M07 - Cutters/L2-04-17 - 02-febe-channel-microprong-brightcut-cutters/L3-005 - Channel Cutter Profile And End Controls/l3.json>) — Channel Cutter Profile And End Controls | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-17-006](<../OM9_Milestones/M07 - Cutters/L2-04-17 - 02-febe-channel-microprong-brightcut-cutters/L3-006 - Multiple Stone Interactive Boolean/l3.json>) — Multiple Stone Interactive Boolean | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-18 — 6 L3

Owner: [03-febe-ring-finger-plane-quad-cutters](<../OM9_Milestones/M07 - Cutters/L2-04-18 - 03-febe-ring-finger-plane-quad-cutters/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-18-001](<../OM9_Milestones/M07 - Cutters/L2-04-18 - 03-febe-ring-finger-plane-quad-cutters/L3-001 - Cut To Finger Rail/l3.json>) — Cut To Finger Rail | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-18-002](<../OM9_Milestones/M07 - Cutters/L2-04-18 - 03-febe-ring-finger-plane-quad-cutters/L3-002 - Plane And Cube Cutters/l3.json>) — Plane And Cube Cutters | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-18-003](<../OM9_Milestones/M07 - Cutters/L2-04-18 - 03-febe-ring-finger-plane-quad-cutters/L3-003 - Quad Flip Routines/l3.json>) — Quad Flip Routines | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-18-004](<../OM9_Milestones/M07 - Cutters/L2-04-18 - 03-febe-ring-finger-plane-quad-cutters/L3-004 - Boolean Builder Operations/l3.json>) — Boolean Builder Operations | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-18-005](<../OM9_Milestones/M07 - Cutters/L2-04-18 - 03-febe-ring-finger-plane-quad-cutters/L3-005 - Cutting Plane Mirror History/l3.json>) — Cutting Plane Mirror History | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-18-006](<../OM9_Milestones/M07 - Cutters/L2-04-18 - 03-febe-ring-finger-plane-quad-cutters/L3-006 - Cutters Geometry Watertight Check/l3.json>) — Cutters Geometry Watertight Check | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

## M08 — Advanced Jewelry Modeling

Nguồn: [L1 canonical](<../OM9_Milestones/M08 - Jewelry Modeling/L1.json>). 8 L2 / 51 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M03, M05, M06.

SmartFlow, placement, deformation, ornament, pattern, rope và library được mở SDK theo từng owner. Không kéo phạm vi AFTER vào M08.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-L2-04-04](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-04 - 01-febe-smart-target-curve-blend-network/L2.json>) — Smart Target/MSR/Blend curve networks phục vụ form nhẫn | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-05](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L2.json>) — Smart Flow auto-base/surface mapping, rigid objects, associative history | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-06](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-06 - 03-febe-jewelry-ornament-builder-family/L2.json>) — Knot/FreeformKnot/JumpRing/Nautilus và pattern ornaments | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-07](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L2.json>) — Orient/array và đặt chi tiết trên curve/surface trang sức | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-04-08](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L2.json>) — Bend/Twist/Taper/CageEdit/SoftMove dùng thiết kế đường nét trang sức | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-05-01](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-01 - 06-febe-matrix9-smart-pattern-builder/L2.json>) — Smart Pattern trên surface: phân phối chi tiết trang sức, group/mesh, attraction và preview. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-05-02](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-02 - 07-febe-matrix9-rope-builder/L2.json>) — Rope Builder: braided rope quanh rail curve; cỡ sợi, xoắn, spacing, cap. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

| [OM9-L2-05-03](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-03 - 08-febe-user-design-libraries/L2.json>) — User Library chứa cả mẫu thiết kế; import/open model và phân loại thư mục riêng. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → EXPORT/CO-IMPLEMENT | Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt. |

### Checklist tích hợp khi thực hiện milestone

#### Checklist: Advanced jewelry M08

- [ ] `04-04`: target/blend/network service, giữ continuity và exact owner constraints.
- [ ] `04-05`: SmartFlow mapping, rigid/deform options, source/target refs, history rebuild; không coi orient đơn giản là morph hoàn chỉnh.
- [ ] `04-06`: knot/jump-ring/ornament builders với typed parameters và shared construction.
- [ ] `04-07`: curve/surface placement; tái dùng frame/orient/instance APIs selection và context–Surface thay vòng lặp UI riêng.
- [ ] `04-08`: bend/twist/taper/cage/softmove audit backend từng operator; constraints và geometry validity trước publish.
- [ ] `05-01`: Smart Pattern distribution/group/mesh/attraction/preview cùng service; budgets cho object count.
- [ ] `05-02`: Rope rail/strand/twist/spacing/cap và geometry/history tests.
- [ ] `05-03`: user design library dùng file/asset services project và exchange, namespace/metadata và missing-data policy.

**Đóng:** từng owner đạt automated + manual gates đúng spec, API capability đúng scope, round-trip/history và community plugin test; không kéo Clayoo/SubD/Emboss/Render từ AFTER vào phạm vi M08.

### Theo dõi L3 trong milestone

#### OM9-L2-04-04 — 5 L3

Owner: [01-febe-smart-target-curve-blend-network](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-04 - 01-febe-smart-target-curve-blend-network/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-04-001](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-04 - 01-febe-smart-target-curve-blend-network/L3-001 - Smart Target Place And Add/l3.json>) — Smart Target Place And Add | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-04-002](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-04 - 01-febe-smart-target-curve-blend-network/L3-002 - Smart Target Curve End/l3.json>) — Smart Target Curve End | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-04-003](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-04 - 01-febe-smart-target-curve-blend-network/L3-003 - Smart Blend Curve Network/l3.json>) — Smart Blend Curve Network | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-04-004](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-04 - 01-febe-smart-target-curve-blend-network/L3-004 - Smart MSR Control Handles/l3.json>) — Smart MSR Control Handles | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-04-005](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-04 - 01-febe-smart-target-curve-blend-network/L3-005 - Smart Target Selection And Update/l3.json>) — Smart Target Selection And Update | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-05 — 8 L3

Owner: [02-febe-matrix-smartflow-history-morph](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-05-001](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L3-001 - Smartflow Autobase Surface Generation/l3.json>) — Smartflow Autobase Surface Generation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-05-002](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L3-002 - Smartflow Mapping Base To Target/l3.json>) — Smartflow Mapping Base To Target | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-05-003](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L3-003 - Smartflow Rigid Object Placement/l3.json>) — Smartflow Rigid Object Placement | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-05-004](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L3-004 - Smartflow History Recompute/l3.json>) — Smartflow History Recompute | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-05-005](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L3-005 - Smartflow Break And Reconnect/l3.json>) — Smartflow Break And Reconnect | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-05-006](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L3-006 - Smartflow Pulldown Reference Curves/l3.json>) — Smartflow Pulldown Reference Curves | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-05-007](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L3-007 - Smartflow UV Normal Align Assistance/l3.json>) — Smartflow UV Normal Align Assistance | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-05-008](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-05 - 02-febe-matrix-smartflow-history-morph/L3-008 - Smartflow Overhang Tolerance Controls/l3.json>) — Smartflow Overhang Tolerance Controls | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-06 — 6 L3

Owner: [03-febe-jewelry-ornament-builder-family](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-06 - 03-febe-jewelry-ornament-builder-family/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-06-001](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-06 - 03-febe-jewelry-ornament-builder-family/L3-001 - Knot Builder/l3.json>) — Knot Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-06-002](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-06 - 03-febe-jewelry-ornament-builder-family/L3-002 - Freeform Knot Builder/l3.json>) — Freeform Knot Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-06-003](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-06 - 03-febe-jewelry-ornament-builder-family/L3-003 - Jump Ring Builder/l3.json>) — Jump Ring Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-06-004](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-06 - 03-febe-jewelry-ornament-builder-family/L3-004 - Nautilus Builder/l3.json>) — Nautilus Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-06-005](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-06 - 03-febe-jewelry-ornament-builder-family/L3-005 - Ornament Pattern Builder/l3.json>) — Ornament Pattern Builder | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-06-006](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-06 - 03-febe-jewelry-ornament-builder-family/L3-006 - Ornament Builder Style And History/l3.json>) — Ornament Builder Style And History | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-07 — 9 L3

Owner: [04-febe-jewelry-surface-object-placement-tools](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-07-001](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L3-001 - Orient Object On Curve/l3.json>) — Orient Object On Curve | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-07-002](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L3-002 - Orient Object On Surface/l3.json>) — Orient Object On Surface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-07-003](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L3-003 - Array Along Curve Jewelry/l3.json>) — Array Along Curve Jewelry | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-07-004](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L3-004 - Array Along Surface UV/l3.json>) — Array Along Surface UV | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-07-005](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L3-005 - Array Curve On Surface/l3.json>) — Array Curve On Surface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-07-006](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L3-006 - Flow Along Curve Jewelry/l3.json>) — Flow Along Curve Jewelry | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-07-007](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L3-007 - Flow Along Surface Jewelry/l3.json>) — Flow Along Surface Jewelry | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-07-008](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L3-008 - Splop Object On Surface/l3.json>) — Splop Object On Surface | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-07-009](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-07 - 04-febe-jewelry-surface-object-placement/L3-009 - Object On Curve Tool/l3.json>) — Object On Curve Tool | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-04-08 — 8 L3

Owner: [05-febe-matrix-advanced-deformation-ux](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-04-08-001](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L3-001 - Bend Deformation Command/l3.json>) — Bend Deformation Command | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-08-002](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L3-002 - Stretch And Taper Deformation/l3.json>) — Stretch And Taper Deformation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-08-003](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L3-003 - Twist And Maelstrom Deformation/l3.json>) — Twist And Maelstrom Deformation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-08-004](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L3-004 - Shear And Softmove Deformation/l3.json>) — Shear And Softmove Deformation | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-08-005](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L3-005 - Cage Edit And Release Command/l3.json>) — Cage Edit And Release Command | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-08-006](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L3-006 - Move Uvn And Setpoints/l3.json>) — Move Uvn And Setpoints | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-08-007](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L3-007 - Symmetry And Live Mirror/l3.json>) — Symmetry And Live Mirror | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-04-08-008](<../OM9_Milestones/M08 - Jewelry Modeling/L2-04-08 - 05-febe-matrix-advanced-deformation-ux/L3-008 - Deformation Preview And Undo/l3.json>) — Deformation Preview And Undo | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-05-01 — 5 L3

Owner: [06-febe-matrix9-smart-pattern-builder](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-01 - 06-febe-matrix9-smart-pattern-builder/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-05-01-001](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-01 - 06-febe-matrix9-smart-pattern-builder/L3-001 - Smartpattern Source Pattern Choice/l3.json>) — Smartpattern Source Pattern Choice | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-01-002](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-01 - 06-febe-matrix9-smart-pattern-builder/L3-002 - Smartpattern UV Distribution/l3.json>) — Smartpattern UV Distribution | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-01-003](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-01 - 06-febe-matrix9-smart-pattern-builder/L3-003 - Smartpattern Group And Segment Controls/l3.json>) — Smartpattern Group And Segment Controls | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-01-004](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-01 - 06-febe-matrix9-smart-pattern-builder/L3-004 - Smartpattern Attractor Region Controls/l3.json>) — Smartpattern Attractor Region Controls | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-01-005](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-01 - 06-febe-matrix9-smart-pattern-builder/L3-005 - Smartpattern History And Viewport Preview/l3.json>) — Smartpattern History And Viewport Preview | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-05-02 — 5 L3

Owner: [07-febe-matrix9-rope-builder](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-02 - 07-febe-matrix9-rope-builder/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-05-02-001](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-02 - 07-febe-matrix9-rope-builder/L3-001 - Rope Builder Curve Input/l3.json>) — Rope Builder Curve Input | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-02-002](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-02 - 07-febe-matrix9-rope-builder/L3-002 - Rope Builder Thread Count And Diameter/l3.json>) — Rope Builder Thread Count And Diameter | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-02-003](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-02 - 07-febe-matrix9-rope-builder/L3-003 - Rope Builder Twists And Spacing/l3.json>) — Rope Builder Twists And Spacing | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-02-004](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-02 - 07-febe-matrix9-rope-builder/L3-004 - Rope Builder End Cap Modes/l3.json>) — Rope Builder End Cap Modes | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-02-005](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-02 - 07-febe-matrix9-rope-builder/L3-005 - Rope Builder Interactive Preview/l3.json>) — Rope Builder Interactive Preview | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-05-03 — 5 L3

Owner: [08-febe-user-design-libraries](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-03 - 08-febe-user-design-libraries/L2.json>). Audit Rust service và evidence đúng L3; phần có xuất public adapter, phần thiếu triển khai shared service + SDK theo Gems/Settings/Cutters/Advanced Jewelry cùng lượt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-05-03-001](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-03 - 08-febe-user-design-libraries/L3-001 - User Library Folder Catalog/l3.json>) — User Library Folder Catalog | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-03-002](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-03 - 08-febe-user-design-libraries/L3-002 - User Library Asset Register And Preview/l3.json>) — User Library Asset Register And Preview | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-03-003](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-03 - 08-febe-user-design-libraries/L3-003 - User Library Insert Model/l3.json>) — User Library Insert Model | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-03-004](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-03 - 08-febe-user-design-libraries/L3-004 - User Library Search And Select/l3.json>) — User Library Search And Select | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-05-03-005](<../OM9_Milestones/M08 - Jewelry Modeling/L2-05-03 - 08-febe-user-design-libraries/L3-005 - User Library Save And Import/l3.json>) — User Library Save And Import | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

## M09 — Cổng nghiệm thu và phát hành (không phát sinh L2 owner)

Nguồn: [L1 canonical](<../OM9_Milestones/M09 - Acceptance and Release Gates/L1.json>). 0 L2 / 0 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M00, M01, M02, M03, M04, M05, M06, M07, M08.

Thu thập evidence SDK của các milestone đã triển khai trong G-MVP/G-FULL hiện có. Không thêm release gate/L2/L3; kế hoạch này không sửa trạng thái 30 L4 hoặc ký nghiệm thu.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

M09 không có L2/L3. Thu thập public SDK/plugin/target proof cùng hồ sơ nghiệm thu của owner upstream; G-MVP/G-FULL và 30 L4 giữ nguyên trạng thái hiện có.

### Checklist tích hợp khi thực hiện milestone

#### Checklist: Nghiệm thu SDK theo capability và phát hành

- [ ] Mỗi capability có test public DTO/host route, service parity, transaction failure, limits và external consumer.
- [ ] Một gói `.om9p` ngoài checkout chỉ phụ thuộc SDK công khai; build bằng archive SDK phát hành, không path tới core/app.
- [ ] Giữ compatibility fixture cho SDK cũ; ABI/capability mismatch fail rõ, không load rồi crash do đoán struct layout.
- [ ] Update package version/schema/docs/example/API inventory và bảng L3 trong file này cùng thay đổi.
- [ ] Release ghi target support, known gaps, third-party notices, source/build SHA và bằng chứng. Không lấy artifact gitignored làm nơi duy nhất giữ kết luận.
- [ ] Thực hiện public source audit và kiểm tra licenses, không đưa proprietary .rhp/source/manual vào SDK bundle.
- [ ] M09 vẫn chỉ có G-MVP/G-FULL. Đưa evidence SDK liên quan vào hồ sơ owner/gate thích hợp; file này không thêm L2, không tự đổi 30 L4 và không tuyên bố hai gate PASS.

**Nghiệm thu SDK** là bổ sung cho năng lực công khai; **nghiệm thu milestone** vẫn theo đúng automated/manual/business contract. Hai trạng thái phải được báo riêng.

### Theo dõi L3 trong milestone

Không tạo L3 mới cho SDK tại M09.

## AFTER — Phạm vi mở rộng sau các release gate chính

Nguồn: [L1 canonical](<../OM9_Milestones/ZAFTER - Extensions/L1.json>). 34 L2 / 170 L3 vật lý; số lượng inventory, không phải completion.

Dependency milestone hiện có: M09.

Mỗi extension tiếp tục dùng owner canonical và dependency sau M09. Khi đến lượt triển khai, public adapter là checklist của cùng extension; tên legacy không tạo dependency Rhino runtime.

### Owner và việc SDK nằm trong milestone

| Owner canonical và phạm vi | Baseline nội bộ | Phần SDK cần tích hợp tại owner | Việc tiếp theo trong cùng milestone |
|---|---|---|---|

| [OM9-L2-06-01](<../OM9_Milestones/ZAFTER - Extensions/L2-06-01 - 01-be-advanced-topology-reconciliation/L2.json>) — Remap tham chiếu faces/edges khi hình học thay đổi phức tạp. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Topology remap/query tokens có revision/ambiguity policy; không stable ID theo native index. |

| [OM9-L2-06-02](<../OM9_Milestones/ZAFTER - Extensions/L2-06-02 - 02-febe-additional-jewelry-cad-formats/L2.json>) — Các adapter bổ sung như STEP/IGES khi cần trao đổi với xưởng/đối tác trang sức. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | STEP/IGES adapters qua shared import/export admission; capability theo format/runtime. |

| [OM9-L2-06-03](<../OM9_Milestones/ZAFTER - Extensions/L2-06-03 - 03-febe-advanced-preferences-i18n-keymap/L2.json>) — Cấu hình giao diện chi tiết, hệ phím tắt và bản dịch mở rộng. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Preferences/i18n/keymap registry có namespace/version; không expose widget internals. |

| [OM9-L2-06-04](<../OM9_Milestones/ZAFTER - Extensions/L2-06-04 - 04-be-profiling-benchmark-lab/L2.json>) — Profiling GPU/OCCT long-run; benchmark chuyên sâu, không phải kiểm thử bắt buộc L4. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Profiler/benchmark reports qua optional diagnostics; hot-path tracing không thành geometry API. |

| [OM9-L2-07-01](<../OM9_Milestones/ZAFTER - Extensions/L2-07-01 - 01-febe-advanced-rhino-macros-and-scripting/L2.json>) — Scripting/macro nâng cao để tự động hóa modeling sau. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | OM9 macro/script service dùng SDK/command registry; giữ target độc lập Rhino, không Rhino runtime. |

| [OM9-L2-07-02](<../OM9_Milestones/ZAFTER - Extensions/L2-07-02 - 02-febe-advanced-selection-filters/L2.json>) — Rhino detailed selection filters beyond jewelry Super Select. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Selection filter expression/DTO tái dùng selection và context permissions và occurrence semantics. |

| [OM9-L2-07-03](<../OM9_Milestones/ZAFTER - Extensions/L2-07-03 - 03-febe-named-views-and-matrix-viewmanager/L2.json>) — Một nơi lưu/phục hồi camera/view names và UI Matrix View Manager. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Named view save/restore service trên selection và context view DTO và project và exchange persistence. |

| [OM9-L2-07-04](<../OM9_Milestones/ZAFTER - Extensions/L2-07-04 - 04-febe-zebra-curvature-surface-analysis/L2.json>) — Zebra, EMap, heatmap curvature và các visual analysis nâng cao. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Surface analysis samples/diagnostics trên Surface; overlay/render là host-owned command/input/UI. |

| [OM9-L2-07-05](<../OM9_Milestones/ZAFTER - Extensions/L2-07-05 - 05-febe-advanced-layers-groups-blocks/L2.json>) — Rhino manager nâng cao cho layers/groups/blocks. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Layers/groups/blocks manager services mở rộng document, units và attributes, instance refs và shared validation. |

| [OM9-L2-08-01](<../OM9_Milestones/ZAFTER - Extensions/L2-08-01 - 01-be-shared-offline-render/L2.json>) — Offline renderer, scene lights and PBR material engine. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Offline render job/material/light/scene snapshot API; tách GPU viewport ownership. |

| [OM9-L2-08-02](<../OM9_Milestones/ZAFTER - Extensions/L2-08-02 - 02-febe-jewelry-render-lookdev-and-light/L2.json>) — Jewelry lighting/metal/gem lookdev. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Jewelry lookdev/light-rig data service trên shared render API; asset license/provenance. |

| [OM9-L2-08-03](<../OM9_Milestones/ZAFTER - Extensions/L2-08-03 - 03-febe-jewelry-animation-and-moviemaker/L2.json>) — Animations / movie maker. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Animation timeline/track/job APIs, cancellation và frame-output lifecycle. |

| [OM9-L2-08-04](<../OM9_Milestones/ZAFTER - Extensions/L2-08-04 - 04-febe-jewelry-batch-render/L2.json>) — Batch render and postprocess. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Batch render/postprocess jobs có budgets/staging/atomic output, không block UI. |

| [OM9-L2-08-05](<../OM9_Milestones/ZAFTER - Extensions/L2-08-05 - 05-febe-design-report-render-layout-and-presentation/L2.json>) — Rendered images in catalogs/layout/report. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Report/layout/render output DTO dùng data/query services và export pipeline. |

| [OM9-L2-09-01](<../OM9_Milestones/ZAFTER - Extensions/L2-09-01 - 01-febe-tsplines-legacy-hud-command/L2.json>) — Legacy T-Splines HUD and command workflow. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Legacy HUD/command là OM9 adapter; không hứa proprietary T-Splines binary compatibility. |

| [OM9-L2-09-02](<../OM9_Milestones/ZAFTER - Extensions/L2-09-02 - 02-febe-tsplines-legacy-primitive-builder/L2.json>) — T-Splines primitive builders incl. T-Spline Signet, Bezel. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | T-spline primitive services chỉ xuất sau own backend, licensing và shape validation. |

| [OM9-L2-09-03](<../OM9_Milestones/ZAFTER - Extensions/L2-09-03 - 03-be-tsplines-legacy-topology-tjunction-parity/L2.json>) — T-spline topological model including T-junctions. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | T-junction/topology model SDK độc lập, validate edits và revision-scoped refs. |

| [OM9-L2-09-04](<../OM9_Milestones/ZAFTER - Extensions/L2-09-04 - 04-be-tsplines-legacy-import-export-compat/L2.json>) — Legacy T-spline conversion/interoperability. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Authorized legacy format conversion qua import/export service; unsupported báo rõ. |

| [OM9-L2-09-05](<../OM9_Milestones/ZAFTER - Extensions/L2-09-05 - 05-febe-clayoo-hud-gumball-selection-modes/L2.json>) — Clayoo sub-object HUD/controls. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | SubD selection/handle/context services; HUD vẫn host UI adapter. |

| [OM9-L2-09-06](<../OM9_Milestones/ZAFTER - Extensions/L2-09-06 - 06-febe-clayoo-subd-primitive-generators/L2.json>) — Clayoo SubD primitives. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | SubD primitive generators với explicit cage/topology DTO và shared service. |

| [OM9-L2-09-07](<../OM9_Milestones/ZAFTER - Extensions/L2-09-07 - 07-febe-clayoo-local-topology-construction/L2.json>) — Clayoo topology editing. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Local topology edit operators, validity, selection remap và atomic history. |

| [OM9-L2-09-08](<../OM9_Milestones/ZAFTER - Extensions/L2-09-08 - 08-febe-clayoo-curve-driven-shapes/L2.json>) — Clayoo Sweep/Loft/Pipe curve-driven meshes. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Curve-driven SubD shapes reuse curve/frame services; mesh không tự thành NURBS. |

| [OM9-L2-09-09](<../OM9_Milestones/ZAFTER - Extensions/L2-09-09 - 09-febe-clayoo-jewelry-subd-builders/L2.json>) — Clayoo Ring, Signet, Bezel builders. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | SubD jewelry builders trên topology service; typed parameters và geometry tests. |

| [OM9-L2-09-10](<../OM9_Milestones/ZAFTER - Extensions/L2-09-10 - 10-febe-clayoo-crease-weld-symmetry/L2.json>) — Clayoo crease/weld/symmetry. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Crease/weld/symmetry operations và topology validity/remap contract. |

| [OM9-L2-09-11](<../OM9_Milestones/ZAFTER - Extensions/L2-09-11 - 11-be-clayoo-nurbs-tspline-mesh-conversion/L2.json>) — Clayoo SubD to mesh/NURBS/legacy TSpline conversions. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Explicit conversion types/error bounds; không mặc định mọi SubD thành exact NURBS. |

| [OM9-L2-09-12](<../OM9_Milestones/ZAFTER - Extensions/L2-09-12 - 12-febe-clayoo-selection-set/L2.json>) — Clayoo selection sets and growth/loop operations. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | SubD loop/grow/selection-set service tách khỏi root-object selection selection và context. |

| [OM9-L2-09-13](<../OM9_Milestones/ZAFTER - Extensions/L2-09-13 - 13-febe-clayemboss-procedural-relief/L2.json>) — ClayEmboss profile/relief procedural operations. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Procedural relief/heightfield services dùng owned data, unit và memory budgets. |

| [OM9-L2-09-14](<../OM9_Milestones/ZAFTER - Extensions/L2-09-14 - 14-febe-clayemboss-brush-stamp-sculpt/L2.json>) — ClayEmboss brush/sculpt/stamp. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Brush/stamp operations và interaction sessions; preview/cancel/commit bounded. |

| [OM9-L2-09-15](<../OM9_Milestones/ZAFTER - Extensions/L2-09-15 - 15-febe-clayemboss-texture-heightfield-decimation/L2.json>) — ClayEmboss texture, heightfield and decimation. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Texture/heightfield/decimation workers, error bounds và source provenance. |

| [OM9-L2-09-16](<../OM9_Milestones/ZAFTER - Extensions/L2-09-16 - 16-febe-clayemboss-text-on-curve/L2.json>) — Command gốc ClayEmbossTextOnCurve, text 2D/3D chạy theo curve. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Text-on-curve service gồm layout/font/source/geometry semantics và persistence. |

| [OM9-L2-09-17](<../OM9_Milestones/ZAFTER - Extensions/L2-09-17 - 17-febe-clay-blueprint/L2.json>) — Blueprint tham chiếu ảnh 3D theo các mặt, command gốc ClayBluePrint. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Blueprint image/calibration data service, asset lifecycle và view placement. |

| [OM9-L2-10-01](<../OM9_Milestones/ZAFTER - Extensions/L2-10-01 - 01-febe-matrix-art-and-surface-texture-builder/L2.json>) — Bitmap/heightfield Matrix Art + Texture Builder mesh-on-surface theo grayscale, dùng shared heightfield utilities. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Shared heightfield/texture-on-surface services, không tạo engine riêng cho MatrixArt. |

| [OM9-L2-10-02](<../OM9_Milestones/ZAFTER - Extensions/L2-10-02 - 02-febe-advanced-jewelry-mesh-repair/L2.json>) — Advanced repair/decimation for casting or jewelry additive manufacture. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Mesh repair/decimation report/result APIs với topology/error oracle và rollback. |

| [OM9-L2-10-03](<../OM9_Milestones/ZAFTER - Extensions/L2-10-03 - 03-febe-jewelry-print-supports/L2.json>) — Supports, sprues and manufacturing aids for casting/printing jewelry. | AUDIT-REQUIRED standalone | AUDIT-THEN-ROUTE → CO-IMPLEMENT nếu thiếu | Support/sprue generation và manufacturing checks, editable result IDs và export. |

### Checklist tích hợp khi thực hiện milestone

Các việc trong bảng owner là checklist chi tiết theo extension. Chỉ bắt đầu theo dependency milestone chính thức; service hiện có được mở public tại owner, service thiếu được làm cùng API và plugin proof.

### Theo dõi L3 trong milestone

#### OM9-L2-06-01 — 5 L3

Owner: [01-be-advanced-topology-reconciliation](<../OM9_Milestones/ZAFTER - Extensions/L2-06-01 - 01-be-advanced-topology-reconciliation/L2.json>). Topology remap/query tokens có revision/ambiguity policy; không stable ID theo native index.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-06-01-001](<../OM9_Milestones/ZAFTER - Extensions/L2-06-01 - 01-be-advanced-topology-reconciliation/L3-001 - Advanced Topological Name Recovery/l3.json>) — Advanced Topological Name Recovery; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-01-002](<../OM9_Milestones/ZAFTER - Extensions/L2-06-01 - 01-be-advanced-topology-reconciliation/L3-002 - Shape History Graph Reconciliation/l3.json>) — Shape History Graph Reconciliation; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-01-003](<../OM9_Milestones/ZAFTER - Extensions/L2-06-01 - 01-be-advanced-topology-reconciliation/L3-003 - Subshape Geometric Signature Matching/l3.json>) — Subshape Geometric Signature Matching; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-01-004](<../OM9_Milestones/ZAFTER - Extensions/L2-06-01 - 01-be-advanced-topology-reconciliation/L3-004 - Ambiguous Reference User Remapping Policy/l3.json>) — Ambiguous Reference User Remapping Policy; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-01-005](<../OM9_Milestones/ZAFTER - Extensions/L2-06-01 - 01-be-advanced-topology-reconciliation/L3-005 - Topology Remap Regression Library/l3.json>) — Topology Remap Regression Library; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-06-02 — 4 L3

Owner: [02-febe-additional-jewelry-cad-formats](<../OM9_Milestones/ZAFTER - Extensions/L2-06-02 - 02-febe-additional-jewelry-cad-formats/L2.json>). STEP/IGES adapters qua shared import/export admission; capability theo format/runtime.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-06-02-001](<../OM9_Milestones/ZAFTER - Extensions/L2-06-02 - 02-febe-additional-jewelry-cad-formats/L3-001 - Step Part Model Import Adapter/l3.json>) — Step Part Model Import Adapter; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-02-002](<../OM9_Milestones/ZAFTER - Extensions/L2-06-02 - 02-febe-additional-jewelry-cad-formats/L3-002 - Iges Surface Import Adapter/l3.json>) — Iges Surface Import Adapter; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-02-003](<../OM9_Milestones/ZAFTER - Extensions/L2-06-02 - 02-febe-additional-jewelry-cad-formats/L3-003 - Additional Format File Dialog Integration/l3.json>) — Additional Format File Dialog Integration; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-02-004](<../OM9_Milestones/ZAFTER - Extensions/L2-06-02 - 02-febe-additional-jewelry-cad-formats/L3-004 - Exchange Format Compatibility Report/l3.json>) — Exchange Format Compatibility Report; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-06-03 — 5 L3

Owner: [03-febe-advanced-preferences-i18n-keymap](<../OM9_Milestones/ZAFTER - Extensions/L2-06-03 - 03-febe-advanced-preferences-i18n-keymap/L2.json>). Preferences/i18n/keymap registry có namespace/version; không expose widget internals.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-06-03-001](<../OM9_Milestones/ZAFTER - Extensions/L2-06-03 - 03-febe-advanced-preferences-i18n-keymap/L3-001 - Advanced Preferences Dialog/l3.json>) — Advanced Preferences Dialog; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-03-002](<../OM9_Milestones/ZAFTER - Extensions/L2-06-03 - 03-febe-advanced-preferences-i18n-keymap/L3-002 - Hotkey Mapping Editor/l3.json>) — Hotkey Mapping Editor; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-03-003](<../OM9_Milestones/ZAFTER - Extensions/L2-06-03 - 03-febe-advanced-preferences-i18n-keymap/L3-003 - Language Resource Localization/l3.json>) — Language Resource Localization; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-03-004](<../OM9_Milestones/ZAFTER - Extensions/L2-06-03 - 03-febe-advanced-preferences-i18n-keymap/L3-004 - Custom Toolbar And Menu Workspaces/l3.json>) — Custom Toolbar And Menu Workspaces; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-03-005](<../OM9_Milestones/ZAFTER - Extensions/L2-06-03 - 03-febe-advanced-preferences-i18n-keymap/L3-005 - Preference Profile Import Export/l3.json>) — Preference Profile Import Export; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-06-04 — 4 L3

Owner: [04-be-profiling-benchmark-lab](<../OM9_Milestones/ZAFTER - Extensions/L2-06-04 - 04-be-profiling-benchmark-lab/L2.json>). Profiler/benchmark reports qua optional diagnostics; hot-path tracing không thành geometry API.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-06-04-001](<../OM9_Milestones/ZAFTER - Extensions/L2-06-04 - 04-be-profiling-benchmark-lab/L3-001 - OCCT Performance Microbenchmarks/l3.json>) — OCCT Performance Microbenchmarks; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-04-002](<../OM9_Milestones/ZAFTER - Extensions/L2-06-04 - 04-be-profiling-benchmark-lab/L3-002 - GPU Tessellation Frame Profiling/l3.json>) — GPU Tessellation Frame Profiling; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-04-003](<../OM9_Milestones/ZAFTER - Extensions/L2-06-04 - 04-be-profiling-benchmark-lab/L3-003 - Massive Gem Instance Stress Suite/l3.json>) — Massive Gem Instance Stress Suite; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-06-04-004](<../OM9_Milestones/ZAFTER - Extensions/L2-06-04 - 04-be-profiling-benchmark-lab/L3-004 - Repeatable Cad Baseline Benchmark Data/l3.json>) — Repeatable Cad Baseline Benchmark Data; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-07-01 — 4 L3

Owner: [01-febe-advanced-rhino-macros-and-scripting](<../OM9_Milestones/ZAFTER - Extensions/L2-07-01 - 01-febe-advanced-rhino-macros-and-scripting/L2.json>). OM9 macro/script service dùng SDK/command registry; giữ target độc lập Rhino, không Rhino runtime.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-07-01-001](<../OM9_Milestones/ZAFTER - Extensions/L2-07-01 - 01-febe-advanced-rhino-macros-and-scripting/L3-001 - Rhino Command Alias Script Editor/l3.json>) — Rhino Command Alias Script Editor; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-01-002](<../OM9_Milestones/ZAFTER - Extensions/L2-07-01 - 01-febe-advanced-rhino-macros-and-scripting/L3-002 - Macro Recording And Replay/l3.json>) — Macro Recording And Replay; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-01-003](<../OM9_Milestones/ZAFTER - Extensions/L2-07-01 - 01-febe-advanced-rhino-macros-and-scripting/L3-003 - Macro Permission And Reentrancy Boundary/l3.json>) — Macro Permission And Reentrancy Boundary; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-01-004](<../OM9_Milestones/ZAFTER - Extensions/L2-07-01 - 01-febe-advanced-rhino-macros-and-scripting/L3-004 - Custom Menu To Macro Binding/l3.json>) — Custom Menu To Macro Binding; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-07-02 — 4 L3

Owner: [02-febe-advanced-selection-filters](<../OM9_Milestones/ZAFTER - Extensions/L2-07-02 - 02-febe-advanced-selection-filters/L2.json>). Selection filter expression/DTO tái dùng selection và context permissions và occurrence semantics.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-07-02-001](<../OM9_Milestones/ZAFTER - Extensions/L2-07-02 - 02-febe-advanced-selection-filters/L3-001 - Selection Filter By Rhino Object Class/l3.json>) — Selection Filter By Rhino Object Class; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-02-002](<../OM9_Milestones/ZAFTER - Extensions/L2-07-02 - 02-febe-advanced-selection-filters/L3-002 - Selection Filter By Color Name Layer/l3.json>) — Selection Filter By Color Name Layer; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-02-003](<../OM9_Milestones/ZAFTER - Extensions/L2-07-02 - 02-febe-advanced-selection-filters/L3-003 - Subobject Face Edge Select Filters/l3.json>) — Subobject Face Edge Select Filters; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-02-004](<../OM9_Milestones/ZAFTER - Extensions/L2-07-02 - 02-febe-advanced-selection-filters/L3-004 - Save Filter Presets/l3.json>) — Save Filter Presets; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-07-03 — 4 L3

Owner: [03-febe-named-views-and-matrix-viewmanager](<../OM9_Milestones/ZAFTER - Extensions/L2-07-03 - 03-febe-named-views-and-matrix-viewmanager/L2.json>). Named view save/restore service trên selection và context view DTO và project và exchange persistence.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-07-03-001](<../OM9_Milestones/ZAFTER - Extensions/L2-07-03 - 03-febe-named-views-and-matrix-viewmanager/L3-001 - Named View Camera Save/l3.json>) — Named View Camera Save; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-03-002](<../OM9_Milestones/ZAFTER - Extensions/L2-07-03 - 03-febe-named-views-and-matrix-viewmanager/L3-002 - Named View Manager Palette/l3.json>) — Named View Manager Palette; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-03-003](<../OM9_Milestones/ZAFTER - Extensions/L2-07-03 - 03-febe-named-views-and-matrix-viewmanager/L3-003 - Named View Restore And Rename/l3.json>) — Named View Restore And Rename; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-03-004](<../OM9_Milestones/ZAFTER - Extensions/L2-07-03 - 03-febe-named-views-and-matrix-viewmanager/L3-004 - Matrix View Manager Command Bindings/l3.json>) — Matrix View Manager Command Bindings; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-07-04 — 5 L3

Owner: [04-febe-zebra-curvature-surface-analysis](<../OM9_Milestones/ZAFTER - Extensions/L2-07-04 - 04-febe-zebra-curvature-surface-analysis/L2.json>). Surface analysis samples/diagnostics trên Surface; overlay/render là host-owned command/input/UI.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-07-04-001](<../OM9_Milestones/ZAFTER - Extensions/L2-07-04 - 04-febe-zebra-curvature-surface-analysis/L3-001 - Zebra Surface Continuity Analysis/l3.json>) — Zebra Surface Continuity Analysis; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-04-002](<../OM9_Milestones/ZAFTER - Extensions/L2-07-04 - 04-febe-zebra-curvature-surface-analysis/L3-002 - Curvature Heatmap Analysis/l3.json>) — Curvature Heatmap Analysis; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-04-003](<../OM9_Milestones/ZAFTER - Extensions/L2-07-04 - 04-febe-zebra-curvature-surface-analysis/L3-003 - Curvature Graph Analysis/l3.json>) — Curvature Graph Analysis; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-04-004](<../OM9_Milestones/ZAFTER - Extensions/L2-07-04 - 04-febe-zebra-curvature-surface-analysis/L3-004 - Environment Reflection Surface Analysis/l3.json>) — Environment Reflection Surface Analysis; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-04-005](<../OM9_Milestones/ZAFTER - Extensions/L2-07-04 - 04-febe-zebra-curvature-surface-analysis/L3-005 - Draft Angle Visual Analysis/l3.json>) — Draft Angle Visual Analysis; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-07-05 — 4 L3

Owner: [05-febe-advanced-layers-groups-blocks](<../OM9_Milestones/ZAFTER - Extensions/L2-07-05 - 05-febe-advanced-layers-groups-blocks/L2.json>). Layers/groups/blocks manager services mở rộng document, units và attributes, instance refs và shared validation.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-07-05-001](<../OM9_Milestones/ZAFTER - Extensions/L2-07-05 - 05-febe-advanced-layers-groups-blocks/L3-001 - Advanced Layer Hierarchy Editor/l3.json>) — Advanced Layer Hierarchy Editor; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-05-002](<../OM9_Milestones/ZAFTER - Extensions/L2-07-05 - 05-febe-advanced-layers-groups-blocks/L3-002 - Rhino Group Management Panel/l3.json>) — Rhino Group Management Panel; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-05-003](<../OM9_Milestones/ZAFTER - Extensions/L2-07-05 - 05-febe-advanced-layers-groups-blocks/L3-003 - Block Definition And Instances Manager/l3.json>) — Block Definition And Instances Manager; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-07-05-004](<../OM9_Milestones/ZAFTER - Extensions/L2-07-05 - 05-febe-advanced-layers-groups-blocks/L3-004 - Worksession Unsupported Deferred Import/l3.json>) — Worksession Unsupported Deferred Import; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-08-01 — 5 L3

Owner: [01-be-shared-offline-render-material-light-pipeline](<../OM9_Milestones/ZAFTER - Extensions/L2-08-01 - 01-be-shared-offline-render/L2.json>). Offline render job/material/light/scene snapshot API; tách GPU viewport ownership.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-08-01-001](<../OM9_Milestones/ZAFTER - Extensions/L2-08-01 - 01-be-shared-offline-render/L3-001 - Offline Renderer Scene Translation/l3.json>) — Offline Renderer Scene Translation; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-01-002](<../OM9_Milestones/ZAFTER - Extensions/L2-08-01 - 01-be-shared-offline-render/L3-002 - PBR Material And Shader Schema/l3.json>) — PBR Material And Shader Schema; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-01-003](<../OM9_Milestones/ZAFTER - Extensions/L2-08-01 - 01-be-shared-offline-render/L3-003 - Scene Light And Environment Model/l3.json>) — Scene Light And Environment Model; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-01-004](<../OM9_Milestones/ZAFTER - Extensions/L2-08-01 - 01-be-shared-offline-render/L3-004 - Offline Render Job Lifecycle/l3.json>) — Offline Render Job Lifecycle; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-01-005](<../OM9_Milestones/ZAFTER - Extensions/L2-08-01 - 01-be-shared-offline-render/L3-005 - Render Output Image Buffer And Color Management/l3.json>) — Render Output Image Buffer And Color Management; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-08-02 — 5 L3

Owner: [02-febe-jewelry-render-lookdev-and-light-rigs](<../OM9_Milestones/ZAFTER - Extensions/L2-08-02 - 02-febe-jewelry-render-lookdev-and-light/L2.json>). Jewelry lookdev/light-rig data service trên shared render API; asset license/provenance.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-08-02-001](<../OM9_Milestones/ZAFTER - Extensions/L2-08-02 - 02-febe-jewelry-render-lookdev-and-light/L3-001 - Jewelry Metal Lookdev Material Picker/l3.json>) — Jewelry Metal Lookdev Material Picker; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-02-002](<../OM9_Milestones/ZAFTER - Extensions/L2-08-02 - 02-febe-jewelry-render-lookdev-and-light/L3-002 - Gem Refraction And Caustic Lookdev/l3.json>) — Gem Refraction And Caustic Lookdev; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-02-003](<../OM9_Milestones/ZAFTER - Extensions/L2-08-02 - 02-febe-jewelry-render-lookdev-and-light/L3-003 - Studio Light Rig And Reflector Setup/l3.json>) — Studio Light Rig And Reflector Setup; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-02-004](<../OM9_Milestones/ZAFTER - Extensions/L2-08-02 - 02-febe-jewelry-render-lookdev-and-light/L3-004 - Render Camera Backdrop And Props/l3.json>) — Render Camera Backdrop And Props; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-02-005](<../OM9_Milestones/ZAFTER - Extensions/L2-08-02 - 02-febe-jewelry-render-lookdev-and-light/L3-005 - Material Preview And Style Application/l3.json>) — Material Preview And Style Application; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-08-03 — 4 L3

Owner: [03-febe-jewelry-animation-and-moviemaker](<../OM9_Milestones/ZAFTER - Extensions/L2-08-03 - 03-febe-jewelry-animation-and-moviemaker/L2.json>). Animation timeline/track/job APIs, cancellation và frame-output lifecycle.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-08-03-001](<../OM9_Milestones/ZAFTER - Extensions/L2-08-03 - 03-febe-jewelry-animation-and-moviemaker/L3-001 - Animation Turntable Builder/l3.json>) — Animation Turntable Builder; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-03-002](<../OM9_Milestones/ZAFTER - Extensions/L2-08-03 - 03-febe-jewelry-animation-and-moviemaker/L3-002 - Animation Keyframe Timeline/l3.json>) — Animation Keyframe Timeline; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-03-003](<../OM9_Milestones/ZAFTER - Extensions/L2-08-03 - 03-febe-jewelry-animation-and-moviemaker/L3-003 - Matrix Movie Maker Output/l3.json>) — Matrix Movie Maker Output; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-03-004](<../OM9_Milestones/ZAFTER - Extensions/L2-08-03 - 03-febe-jewelry-animation-and-moviemaker/L3-004 - Animation Preview And Cancel/l3.json>) — Animation Preview And Cancel; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-08-04 — 5 L3

Owner: [04-febe-jewelry-batch-render-and-postprocessing](<../OM9_Milestones/ZAFTER - Extensions/L2-08-04 - 04-febe-jewelry-batch-render/L2.json>). Batch render/postprocess jobs có budgets/staging/atomic output, không block UI.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-08-04-001](<../OM9_Milestones/ZAFTER - Extensions/L2-08-04 - 04-febe-jewelry-batch-render/L3-001 - Batch Render Design Variants/l3.json>) — Batch Render Design Variants; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-04-002](<../OM9_Milestones/ZAFTER - Extensions/L2-08-04 - 04-febe-jewelry-batch-render/L3-002 - Render Job Queue And Scheduler/l3.json>) — Render Job Queue And Scheduler; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-04-003](<../OM9_Milestones/ZAFTER - Extensions/L2-08-04 - 04-febe-jewelry-batch-render/L3-003 - Render Editor Crop Watermark And Alpha/l3.json>) — Render Editor Crop Watermark And Alpha; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-04-004](<../OM9_Milestones/ZAFTER - Extensions/L2-08-04 - 04-febe-jewelry-batch-render/L3-004 - Four Viewport Capture And Layout/l3.json>) — Four Viewport Capture And Layout; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-04-005](<../OM9_Milestones/ZAFTER - Extensions/L2-08-04 - 04-febe-jewelry-batch-render/L3-005 - Save Render To Image Formats/l3.json>) — Save Render To Image Formats; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-08-05 — 4 L3

Owner: [05-febe-design-report-render-layout-and-presentation](<../OM9_Milestones/ZAFTER - Extensions/L2-08-05 - 05-febe-design-report-render-layout-and-presentation/L2.json>). Report/layout/render output DTO dùng data/query services và export pipeline.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-08-05-001](<../OM9_Milestones/ZAFTER - Extensions/L2-08-05 - 05-febe-design-report-render-layout-and-presentation/L3-001 - Design Report Cover And Template/l3.json>) — Design Report Cover And Template; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-05-002](<../OM9_Milestones/ZAFTER - Extensions/L2-08-05 - 05-febe-design-report-render-layout-and-presentation/L3-002 - Report Render Images And Gem Table/l3.json>) — Report Render Images And Gem Table; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-05-003](<../OM9_Milestones/ZAFTER - Extensions/L2-08-05 - 05-febe-design-report-render-layout-and-presentation/L3-003 - Catalog Layout Pages And Labels/l3.json>) — Catalog Layout Pages And Labels; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-08-05-004](<../OM9_Milestones/ZAFTER - Extensions/L2-08-05 - 05-febe-design-report-render-layout-and-presentation/L3-004 - Report Export And Print/l3.json>) — Report Export And Print; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-01 — 6 L3

Owner: [01-febe-tsplines-legacy-hud-command-compat](<../OM9_Milestones/ZAFTER - Extensions/L2-09-01 - 01-febe-tsplines-legacy-hud-command/L2.json>). Legacy HUD/command là OM9 adapter; không hứa proprietary T-Splines binary compatibility.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-01-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-01 - 01-febe-tsplines-legacy-hud-command/L3-001 - T-Splines Legacy Hud Activation/l3.json>) — T-Splines Legacy Hud Activation; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-01-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-01 - 01-febe-tsplines-legacy-hud-command/L3-002 - T-Splines Select Vertex Edge Face Object Mode/l3.json>) — T-Splines Select Vertex Edge Face Object Mode; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-01-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-01 - 01-febe-tsplines-legacy-hud-command/L3-003 - T-Splines Gumball Translate Scale Rotate/l3.json>) — T-Splines Gumball Translate Scale Rotate; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-01-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-01 - 01-febe-tsplines-legacy-hud-command/L3-004 - T-Splines Box Smooth Toggle/l3.json>) — T-Splines Box Smooth Toggle; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-01-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-01 - 01-febe-tsplines-legacy-hud-command/L3-005 - T-Splines X Y Symmetry Toggles/l3.json>) — T-Splines X Y Symmetry Toggles; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-01-006](<../OM9_Milestones/ZAFTER - Extensions/L2-09-01 - 01-febe-tsplines-legacy-hud-command/L3-006 - T-Splines Compatibility Mode Settings/l3.json>) — T-Splines Compatibility Mode Settings; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-02 — 6 L3

Owner: [02-febe-tsplines-legacy-primitive-builder-compat](<../OM9_Milestones/ZAFTER - Extensions/L2-09-02 - 02-febe-tsplines-legacy-primitive-builder/L2.json>). T-spline primitive services chỉ xuất sau own backend, licensing và shape validation.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-02-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-02 - 02-febe-tsplines-legacy-primitive-builder/L3-001 - T-Spline Box Sphere Cylinder Cone Torus/l3.json>) — T-Spline Box Sphere Cylinder Cone Torus; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-02-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-02 - 02-febe-tsplines-legacy-primitive-builder/L3-002 - T-Spline Plane And Tube From Curves/l3.json>) — T-Spline Plane And Tube From Curves; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-02-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-02 - 02-febe-tsplines-legacy-primitive-builder/L3-003 - T-Spline Bezel Builder/l3.json>) — T-Spline Bezel Builder; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-02-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-02 - 02-febe-tsplines-legacy-primitive-builder/L3-004 - T-Spline Under Bezel Builder/l3.json>) — T-Spline Under Bezel Builder; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-02-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-02 - 02-febe-tsplines-legacy-primitive-builder/L3-005 - T-Spline Signet Ring Builder/l3.json>) — T-Spline Signet Ring Builder; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-02-006](<../OM9_Milestones/ZAFTER - Extensions/L2-09-02 - 02-febe-tsplines-legacy-primitive-builder/L3-006 - T-Spline Skin Loft Pipe/l3.json>) — T-Spline Skin Loft Pipe; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-03 — 7 L3

Owner: [03-be-tsplines-legacy-topology-tjunction-parity](<../OM9_Milestones/ZAFTER - Extensions/L2-09-03 - 03-be-tsplines-legacy-topology-tjunction-parity/L2.json>). T-junction/topology model SDK độc lập, validate edits và revision-scoped refs.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-03-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-03 - 03-be-tsplines-legacy-topology-tjunction-parity/L3-001 - T-Spline Control Mesh Topology/l3.json>) — T-Spline Control Mesh Topology; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-03-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-03 - 03-be-tsplines-legacy-topology-tjunction-parity/L3-002 - T-Spline Tjunction And Extraordinary Points/l3.json>) — T-Spline Tjunction And Extraordinary Points; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-03-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-03 - 03-be-tsplines-legacy-topology-tjunction-parity/L3-003 - T-Spline Insert Edges And Points/l3.json>) — T-Spline Insert Edges And Points; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-03-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-03 - 03-be-tsplines-legacy-topology-tjunction-parity/L3-004 - T-Spline Subdivide And Unweld Weld/l3.json>) — T-Spline Subdivide And Unweld Weld; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-03-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-03 - 03-be-tsplines-legacy-topology-tjunction-parity/L3-005 - T-Spline Bridge Append Fill And Extrude/l3.json>) — T-Spline Bridge Append Fill And Extrude; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-03-006](<../OM9_Milestones/ZAFTER - Extensions/L2-09-03 - 03-be-tsplines-legacy-topology-tjunction-parity/L3-006 - T-Spline Crease Weight And Smoothness/l3.json>) — T-Spline Crease Weight And Smoothness; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-03-007](<../OM9_Milestones/ZAFTER - Extensions/L2-09-03 - 03-be-tsplines-legacy-topology-tjunction-parity/L3-007 - T-Spline Topology Geometry Regression/l3.json>) — T-Spline Topology Geometry Regression; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-04 — 4 L3

Owner: [04-be-tsplines-legacy-import-export-compat](<../OM9_Milestones/ZAFTER - Extensions/L2-09-04 - 04-be-tsplines-legacy-import-export-compat/L2.json>). Authorized legacy format conversion qua import/export service; unsupported báo rõ.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-04-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-04 - 04-be-tsplines-legacy-import-export-compat/L3-001 - T-Spline To Rhino Surface Converter/l3.json>) — T-Spline To Rhino Surface Converter; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-04-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-04 - 04-be-tsplines-legacy-import-export-compat/L3-002 - Rhino Surface To T-Spline Eligibility/l3.json>) — Rhino Surface To T-Spline Eligibility; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-04-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-04 - 04-be-tsplines-legacy-import-export-compat/L3-003 - T-Spline To Mesh Tessellation Export/l3.json>) — T-Spline To Mesh Tessellation Export; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-04-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-04 - 04-be-tsplines-legacy-import-export-compat/L3-004 - Legacy T-Spline File Data Bridge/l3.json>) — Legacy T-Spline File Data Bridge; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-05 — 6 L3

Owner: [05-febe-clayoo-hud-gumball-selection-modes](<../OM9_Milestones/ZAFTER - Extensions/L2-09-05 - 05-febe-clayoo-hud-gumball-selection-modes/L2.json>). SubD selection/handle/context services; HUD vẫn host UI adapter.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-05-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-05 - 05-febe-clayoo-hud-gumball-selection-modes/L3-001 - Clayoo Clayedit Enable Hud/l3.json>) — Clayoo Clayedit Enable Hud; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-05-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-05 - 05-febe-clayoo-hud-gumball-selection-modes/L3-002 - Clayoo Object Vertex Edge Face Modes/l3.json>) — Clayoo Object Vertex Edge Face Modes; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-05-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-05 - 05-febe-clayoo-hud-gumball-selection-modes/L3-003 - Clayoo Gumball Translation Scale Rotation/l3.json>) — Clayoo Gumball Translation Scale Rotation; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-05-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-05 - 05-febe-clayoo-hud-gumball-selection-modes/L3-004 - Clayoo Box Smooth Visualization/l3.json>) — Clayoo Box Smooth Visualization; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-05-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-05 - 05-febe-clayoo-hud-gumball-selection-modes/L3-005 - Clayoo Hotkey Profiles/l3.json>) — Clayoo Hotkey Profiles; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-05-006](<../OM9_Milestones/ZAFTER - Extensions/L2-09-05 - 05-febe-clayoo-hud-gumball-selection-modes/L3-006 - Clayoo Points On Surface Overlay/l3.json>) — Clayoo Points On Surface Overlay; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-06 — 5 L3

Owner: [06-febe-clayoo-subd-primitive-generators](<../OM9_Milestones/ZAFTER - Extensions/L2-09-06 - 06-febe-clayoo-subd-primitive-generators/L2.json>). SubD primitive generators với explicit cage/topology DTO và shared service.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-06-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-06 - 06-febe-clayoo-subd-primitive-generators/L3-001 - Clayoo Box And Plane Generators/l3.json>) — Clayoo Box And Plane Generators; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-06-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-06 - 06-febe-clayoo-subd-primitive-generators/L3-002 - Clayoo Sphere Quad And Polygonal/l3.json>) — Clayoo Sphere Quad And Polygonal; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-06-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-06 - 06-febe-clayoo-subd-primitive-generators/L3-003 - Clayoo Cylinder Cone Generators/l3.json>) — Clayoo Cylinder Cone Generators; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-06-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-06 - 06-febe-clayoo-subd-primitive-generators/L3-004 - Clayoo Shape Viewport Control Handles/l3.json>) — Clayoo Shape Viewport Control Handles; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-06-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-06 - 06-febe-clayoo-subd-primitive-generators/L3-005 - Clayoo Subdivision Level Preview/l3.json>) — Clayoo Subdivision Level Preview; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-07 — 7 L3

Owner: [07-febe-clayoo-local-topology-construction](<../OM9_Milestones/ZAFTER - Extensions/L2-09-07 - 07-febe-clayoo-local-topology-construction/L2.json>). Local topology edit operators, validity, selection remap và atomic history.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-07-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-07 - 07-febe-clayoo-local-topology-construction/L3-001 - Clayoo Divide And Split Sides/l3.json>) — Clayoo Divide And Split Sides; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-07-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-07 - 07-febe-clayoo-local-topology-construction/L3-002 - Clayoo Inset And Knife/l3.json>) — Clayoo Inset And Knife; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-07-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-07 - 07-febe-clayoo-local-topology-construction/L3-003 - Clayoo Extrude And Offset/l3.json>) — Clayoo Extrude And Offset; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-07-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-07 - 07-febe-clayoo-local-topology-construction/L3-004 - Clayoo Bridge And Append Face/l3.json>) — Clayoo Bridge And Append Face; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-07-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-07 - 07-febe-clayoo-local-topology-construction/L3-005 - Clayoo Fill And Shell/l3.json>) — Clayoo Fill And Shell; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-07-006](<../OM9_Milestones/ZAFTER - Extensions/L2-09-07 - 07-febe-clayoo-local-topology-construction/L3-006 - Clayoo Collapse And Merge/l3.json>) — Clayoo Collapse And Merge; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-07-007](<../OM9_Milestones/ZAFTER - Extensions/L2-09-07 - 07-febe-clayoo-local-topology-construction/L3-007 - Clayoo Topology Operation Undo/l3.json>) — Clayoo Topology Operation Undo; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-08 — 6 L3

Owner: [08-febe-clayoo-curve-driven-shapes](<../OM9_Milestones/ZAFTER - Extensions/L2-09-08 - 08-febe-clayoo-curve-driven-shapes/L2.json>). Curve-driven SubD shapes reuse curve/frame services; mesh không tự thành NURBS.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-08-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-08 - 08-febe-clayoo-curve-driven-shapes/L3-001 - Clayoo Pipe From Curves/l3.json>) — Clayoo Pipe From Curves; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-08-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-08 - 08-febe-clayoo-curve-driven-shapes/L3-002 - Clayoo Sweep One Rail/l3.json>) — Clayoo Sweep One Rail; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-08-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-08 - 08-febe-clayoo-curve-driven-shapes/L3-003 - Clayoo Sweep Two Rails/l3.json>) — Clayoo Sweep Two Rails; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-08-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-08 - 08-febe-clayoo-curve-driven-shapes/L3-004 - Clayoo Loft And Revolve/l3.json>) — Clayoo Loft And Revolve; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-08-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-08 - 08-febe-clayoo-curve-driven-shapes/L3-005 - Clayoo Wrap Curves And Mesh/l3.json>) — Clayoo Wrap Curves And Mesh; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-08-006](<../OM9_Milestones/ZAFTER - Extensions/L2-09-08 - 08-febe-clayoo-curve-driven-shapes/L3-006 - Clayoo Create By One Two Curves/l3.json>) — Clayoo Create By One Two Curves; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-09 — 4 L3

Owner: [09-febe-clayoo-jewelry-subd-builders](<../OM9_Milestones/ZAFTER - Extensions/L2-09-09 - 09-febe-clayoo-jewelry-subd-builders/L2.json>). SubD jewelry builders trên topology service; typed parameters và geometry tests.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-09-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-09 - 09-febe-clayoo-jewelry-subd-builders/L3-001 - Clayoo Ring Builder/l3.json>) — Clayoo Ring Builder; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-09-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-09 - 09-febe-clayoo-jewelry-subd-builders/L3-002 - Clayoo Signet Ring Builder/l3.json>) — Clayoo Signet Ring Builder; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-09-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-09 - 09-febe-clayoo-jewelry-subd-builders/L3-003 - Clayoo Bezel Around Gem/l3.json>) — Clayoo Bezel Around Gem; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-09-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-09 - 09-febe-clayoo-jewelry-subd-builders/L3-004 - Clayoo Jewelry Builder Handles/l3.json>) — Clayoo Jewelry Builder Handles; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-10 — 6 L3

Owner: [10-febe-clayoo-crease-weld-symmetry](<../OM9_Milestones/ZAFTER - Extensions/L2-09-10 - 10-febe-clayoo-crease-weld-symmetry/L2.json>). Crease/weld/symmetry operations và topology validity/remap contract.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-10-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-10 - 10-febe-clayoo-crease-weld-symmetry/L3-001 - Clayoo Crease And Uncrease Edges/l3.json>) — Clayoo Crease And Uncrease Edges; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-10-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-10 - 10-febe-clayoo-crease-weld-symmetry/L3-002 - Clayoo Weld Vertices With Tolerance/l3.json>) — Clayoo Weld Vertices With Tolerance; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-10-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-10 - 10-febe-clayoo-crease-weld-symmetry/L3-003 - Clayoo Match Edges To Curve/l3.json>) — Clayoo Match Edges To Curve; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-10-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-10 - 10-febe-clayoo-crease-weld-symmetry/L3-004 - Clayoo Symmetry Plane And Loop/l3.json>) — Clayoo Symmetry Plane And Loop; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-10-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-10 - 10-febe-clayoo-crease-weld-symmetry/L3-005 - Clayoo Project To Plane/l3.json>) — Clayoo Project To Plane; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-10-006](<../OM9_Milestones/ZAFTER - Extensions/L2-09-10 - 10-febe-clayoo-crease-weld-symmetry/L3-006 - Clayoo Flip Normals And Winding/l3.json>) — Clayoo Flip Normals And Winding; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-11 — 6 L3

Owner: [11-be-clayoo-nurbs-tspline-mesh-conversion](<../OM9_Milestones/ZAFTER - Extensions/L2-09-11 - 11-be-clayoo-nurbs-tspline-mesh-conversion/L2.json>). Explicit conversion types/error bounds; không mặc định mọi SubD thành exact NURBS.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-11-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-11 - 11-be-clayoo-nurbs-tspline-mesh-conversion/L3-001 - Clayoo Controlmesh To SubD Evaluator/l3.json>) — Clayoo Controlmesh To SubD Evaluator; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-11-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-11 - 11-be-clayoo-nurbs-tspline-mesh-conversion/L3-002 - Clayoo SubD To NURBS Surfaces/l3.json>) — Clayoo SubD To NURBS Surfaces; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-11-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-11 - 11-be-clayoo-nurbs-tspline-mesh-conversion/L3-003 - Clayoo SubD To Triangle Mesh/l3.json>) — Clayoo SubD To Triangle Mesh; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-11-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-11 - 11-be-clayoo-nurbs-tspline-mesh-conversion/L3-004 - Clayoo NURBS To SubD Mesh Policy/l3.json>) — Clayoo NURBS To SubD Mesh Policy; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-11-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-11 - 11-be-clayoo-nurbs-tspline-mesh-conversion/L3-005 - Clayoo T-Spline Conversion Boundary/l3.json>) — Clayoo T-Spline Conversion Boundary; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-11-006](<../OM9_Milestones/ZAFTER - Extensions/L2-09-11 - 11-be-clayoo-nurbs-tspline-mesh-conversion/L3-006 - Clayoo Conversion Error And Loss Report/l3.json>) — Clayoo Conversion Error And Loss Report; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-12 — 6 L3

Owner: [12-febe-clayoo-selection-set-operations](<../OM9_Milestones/ZAFTER - Extensions/L2-09-12 - 12-febe-clayoo-selection-set/L2.json>). SubD loop/grow/selection-set service tách khỏi root-object selection selection và context.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-12-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-12 - 12-febe-clayoo-selection-set/L3-001 - Clayoo Ring Edge Loop And UV Select/l3.json>) — Clayoo Ring Edge Loop And UV Select; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-12-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-12 - 12-febe-clayoo-selection-set/L3-002 - Clayoo Select All None Invert/l3.json>) — Clayoo Select All None Invert; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-12-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-12 - 12-febe-clayoo-selection-set/L3-003 - Clayoo Paint Selection/l3.json>) — Clayoo Paint Selection; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-12-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-12 - 12-febe-clayoo-selection-set/L3-004 - Clayoo Grow Shrink Selection/l3.json>) — Clayoo Grow Shrink Selection; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-12-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-12 - 12-febe-clayoo-selection-set/L3-005 - Clayoo Crease And Naked Edge Filters/l3.json>) — Clayoo Crease And Naked Edge Filters; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-12-006](<../OM9_Milestones/ZAFTER - Extensions/L2-09-12 - 12-febe-clayoo-selection-set/L3-006 - Clayoo Saved Selection Sets/l3.json>) — Clayoo Saved Selection Sets; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-13 — 5 L3

Owner: [13-febe-clayemboss-procedural-relief](<../OM9_Milestones/ZAFTER - Extensions/L2-09-13 - 13-febe-clayemboss-procedural-relief/L2.json>). Procedural relief/heightfield services dùng owned data, unit và memory budgets.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-13-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-13 - 13-febe-clayemboss-procedural-relief/L3-001 - Clayemboss Main Procedural Relief/l3.json>) — Clayemboss Main Procedural Relief; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-13-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-13 - 13-febe-clayemboss-procedural-relief/L3-002 - Clayemboss By Profile/l3.json>) — Clayemboss By Profile; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-13-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-13 - 13-febe-clayemboss-procedural-relief/L3-003 - Clayemboss Extrude And Revolve/l3.json>) — Clayemboss Extrude And Revolve; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-13-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-13 - 13-febe-clayemboss-procedural-relief/L3-004 - Clayemboss Base On Geometry/l3.json>) — Clayemboss Base On Geometry; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-13-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-13 - 13-febe-clayemboss-procedural-relief/L3-005 - Clayemboss Height Parameter Controls/l3.json>) — Clayemboss Height Parameter Controls; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-14 — 5 L3

Owner: [14-febe-clayemboss-brush-stamp-sculpt](<../OM9_Milestones/ZAFTER - Extensions/L2-09-14 - 14-febe-clayemboss-brush-stamp-sculpt/L2.json>). Brush/stamp operations và interaction sessions; preview/cancel/commit bounded.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-14-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-14 - 14-febe-clayemboss-brush-stamp-sculpt/L3-001 - Clayemboss Stamp Brush/l3.json>) — Clayemboss Stamp Brush; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-14-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-14 - 14-febe-clayemboss-brush-stamp-sculpt/L3-002 - Clayemboss Carbon Copy And Brush/l3.json>) — Clayemboss Carbon Copy And Brush; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-14-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-14 - 14-febe-clayemboss-brush-stamp-sculpt/L3-003 - Clayemboss Brush Sculpt/l3.json>) — Clayemboss Brush Sculpt; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-14-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-14 - 14-febe-clayemboss-brush-stamp-sculpt/L3-004 - Clayemboss Smoothing Brush/l3.json>) — Clayemboss Smoothing Brush; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-14-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-14 - 14-febe-clayemboss-brush-stamp-sculpt/L3-005 - Clayemboss Brush Settings And Preview/l3.json>) — Clayemboss Brush Settings And Preview; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-15 — 5 L3

Owner: [15-febe-clayemboss-texture-heightfield-decimation](<../OM9_Milestones/ZAFTER - Extensions/L2-09-15 - 15-febe-clayemboss-texture-heightfield-decimation/L2.json>). Texture/heightfield/decimation workers, error bounds và source provenance.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-15-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-15 - 15-febe-clayemboss-texture-heightfield-decimation/L3-001 - Clayemboss Texture From Image/l3.json>) — Clayemboss Texture From Image; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-15-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-15 - 15-febe-clayemboss-texture-heightfield-decimation/L3-002 - Clayemboss Heightfield Image/l3.json>) — Clayemboss Heightfield Image; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-15-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-15 - 15-febe-clayemboss-texture-heightfield-decimation/L3-003 - Clayemboss Resources Browser/l3.json>) — Clayemboss Resources Browser; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-15-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-15 - 15-febe-clayemboss-texture-heightfield-decimation/L3-004 - Clayemboss Decimator/l3.json>) — Clayemboss Decimator; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-15-005](<../OM9_Milestones/ZAFTER - Extensions/L2-09-15 - 15-febe-clayemboss-texture-heightfield-decimation/L3-005 - Clayemboss Texture Repeat And Scale/l3.json>) — Clayemboss Texture Repeat And Scale; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-16 — 4 L3

Owner: [16-febe-clayemboss-text-on-curve](<../OM9_Milestones/ZAFTER - Extensions/L2-09-16 - 16-febe-clayemboss-text-on-curve/L2.json>). Text-on-curve service gồm layout/font/source/geometry semantics và persistence.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-16-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-16 - 16-febe-clayemboss-text-on-curve/L3-001 - Clayemboss Text On Curve 2d/l3.json>) — Clayemboss Text On Curve 2d; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-16-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-16 - 16-febe-clayemboss-text-on-curve/L3-002 - Clayemboss Text On Curve 3d/l3.json>) — Clayemboss Text On Curve 3d; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-16-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-16 - 16-febe-clayemboss-text-on-curve/L3-003 - Clayemboss Text Glyph Spacing Direction/l3.json>) — Clayemboss Text Glyph Spacing Direction; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-16-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-16 - 16-febe-clayemboss-text-on-curve/L3-004 - Clayemboss Text On Curve Builder UI/l3.json>) — Clayemboss Text On Curve Builder UI; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-09-17 — 4 L3

Owner: [17-febe-clay-blueprint](<../OM9_Milestones/ZAFTER - Extensions/L2-09-17 - 17-febe-clay-blueprint/L2.json>). Blueprint image/calibration data service, asset lifecycle và view placement.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-09-17-001](<../OM9_Milestones/ZAFTER - Extensions/L2-09-17 - 17-febe-clay-blueprint/L3-001 - Clay Blueprint Multiview Image Placement/l3.json>) — Clay Blueprint Multiview Image Placement; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-17-002](<../OM9_Milestones/ZAFTER - Extensions/L2-09-17 - 17-febe-clay-blueprint/L3-002 - Clay Blueprint Image Alignment/l3.json>) — Clay Blueprint Image Alignment; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-17-003](<../OM9_Milestones/ZAFTER - Extensions/L2-09-17 - 17-febe-clay-blueprint/L3-003 - Clay Blueprint Transparent Overlays/l3.json>) — Clay Blueprint Transparent Overlays; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-09-17-004](<../OM9_Milestones/ZAFTER - Extensions/L2-09-17 - 17-febe-clay-blueprint/L3-004 - Clay Blueprint Preset Save Load/l3.json>) — Clay Blueprint Preset Save Load; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-10-01 — 5 L3

Owner: [01-febe-matrix-art-and-surface-texture-builder](<../OM9_Milestones/ZAFTER - Extensions/L2-10-01 - 01-febe-matrix-art-and-surface-texture-builder/L2.json>). Shared heightfield/texture-on-surface services, không tạo engine riêng cho MatrixArt.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-10-01-001](<../OM9_Milestones/ZAFTER - Extensions/L2-10-01 - 01-febe-matrix-art-and-surface-texture-builder/L3-001 - Matrix Art Bitmap To Heightfield/l3.json>) — Matrix Art Bitmap To Heightfield; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-01-002](<../OM9_Milestones/ZAFTER - Extensions/L2-10-01 - 01-febe-matrix-art-and-surface-texture-builder/L3-002 - Matrix Art Relief Region Mask/l3.json>) — Matrix Art Relief Region Mask; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-01-003](<../OM9_Milestones/ZAFTER - Extensions/L2-10-01 - 01-febe-matrix-art-and-surface-texture-builder/L3-003 - Matrix Texture Builder On Surface/l3.json>) — Matrix Texture Builder On Surface; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-01-004](<../OM9_Milestones/ZAFTER - Extensions/L2-10-01 - 01-febe-matrix-art-and-surface-texture-builder/L3-004 - Matrix Texture Builder UV Layout/l3.json>) — Matrix Texture Builder UV Layout; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-01-005](<../OM9_Milestones/ZAFTER - Extensions/L2-10-01 - 01-febe-matrix-art-and-surface-texture-builder/L3-005 - Heightfield Result To Mesh/l3.json>) — Heightfield Result To Mesh; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-10-02 — 5 L3

Owner: [02-febe-advanced-jewelry-mesh-repair](<../OM9_Milestones/ZAFTER - Extensions/L2-10-02 - 02-febe-advanced-jewelry-mesh-repair/L2.json>). Mesh repair/decimation report/result APIs với topology/error oracle và rollback.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-10-02-001](<../OM9_Milestones/ZAFTER - Extensions/L2-10-02 - 02-febe-advanced-jewelry-mesh-repair/L3-001 - Mesh Weld Normals Repair/l3.json>) — Mesh Weld Normals Repair; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-02-002](<../OM9_Milestones/ZAFTER - Extensions/L2-10-02 - 02-febe-advanced-jewelry-mesh-repair/L3-002 - Mesh Manifold And Hole Repair/l3.json>) — Mesh Manifold And Hole Repair; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-02-003](<../OM9_Milestones/ZAFTER - Extensions/L2-10-02 - 02-febe-advanced-jewelry-mesh-repair/L3-003 - Mesh Intersection And Island Repair/l3.json>) — Mesh Intersection And Island Repair; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-02-004](<../OM9_Milestones/ZAFTER - Extensions/L2-10-02 - 02-febe-advanced-jewelry-mesh-repair/L3-004 - Mesh Reduction Advanced/l3.json>) — Mesh Reduction Advanced; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-02-005](<../OM9_Milestones/ZAFTER - Extensions/L2-10-02 - 02-febe-advanced-jewelry-mesh-repair/L3-005 - Mesh Repair Before After Report/l3.json>) — Mesh Repair Before After Report; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

#### OM9-L2-10-03 — 6 L3

Owner: [03-febe-jewelry-print-supports-and-sprues](<../OM9_Milestones/ZAFTER - Extensions/L2-10-03 - 03-febe-jewelry-print-supports/L2.json>). Support/sprue generation và manufacturing checks, editable result IDs và export.

| L3 / năng lực | Tín hiệu nội bộ | SDK tại ngày chốt / đường xử lý |
|---|---|---|
| [OM9-L3-10-03-001](<../OM9_Milestones/ZAFTER - Extensions/L2-10-03 - 03-febe-jewelry-print-supports/L3-001 - Ring Support Sprue Builder/l3.json>) — Ring Support Sprue Builder; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-03-002](<../OM9_Milestones/ZAFTER - Extensions/L2-10-03 - 03-febe-jewelry-print-supports/L3-002 - Support Base And Support Ring/l3.json>) — Support Base And Support Ring; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-03-003](<../OM9_Milestones/ZAFTER - Extensions/L2-10-03 - 03-febe-jewelry-print-supports/L3-003 - Cylinder And Tapered Support Generators/l3.json>) — Cylinder And Tapered Support Generators; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-03-004](<../OM9_Milestones/ZAFTER - Extensions/L2-10-03 - 03-febe-jewelry-print-supports/L3-004 - Blend Supports Between Surfaces/l3.json>) — Blend Supports Between Surfaces; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-03-005](<../OM9_Milestones/ZAFTER - Extensions/L2-10-03 - 03-febe-jewelry-print-supports/L3-005 - Support Region Analysis/l3.json>) — Support Region Analysis; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |
| [OM9-L3-10-03-006](<../OM9_Milestones/ZAFTER - Extensions/L2-10-03 - 03-febe-jewelry-print-supports/L3-006 - 3d Printing Export Support Workflow/l3.json>) — 3d Printing Export Support Workflow; **đích AFTER** | AUDIT-REQUIRED | AUDIT → export phần có / co-implement phần thiếu |

## Quyết định ghi L3: `sdk_export_plan` (2026-10-21)

Người dùng chốt: khi cập nhật L3, chỉ ghi **phần nào nên đưa vào SDK**, không cần implement ngay.
Vì vậy mỗi L3 đã có slice SDK nhận thêm trường `sdk_export_plan`:

- `status: "exported_in_sdk_v1"` khi bề mặt đã giao và có receipt đo.
- `api`: danh sách API công khai thuộc L3 đó.
- `implemented_slices`: các `runtime_implementation` đã ghi (nếu có).
- `rule`: chỉ dữ liệu/sự kiện/hành động plugin cần mới vào SDK; kernel/UI/adapter host không vào.

Phạm vi áp dụng (người dùng chọn): **8 owner đã có trong plan** — M01 `01-03`, `01-08`, `01-12`,
`01-02`, `02-01`; M02 `01-07`; M03 `02-04`, `03-20`. L2 của các owner này có
`sdk_export_plan_policy` để nhắc quy tắc.

Trạng thái `sdk_export_plan.status` trong 76 L3 của 8 owner:

| status | Số L3 | Nghĩa |
|---|---|---|
| `exported_in_sdk_v1` | 30 | Bề mặt đã giao, có `api` + `implemented_slices` + receipt đo. |
| `planned` | 36 | Plugin cần nhưng **chưa** giao; `candidate_api` + `host_only` nói rõ phần nào vào SDK. |
| `not_in_sdk` | 10 | Codec/state machine/fixture nội bộ host, cố ý không đưa vào SDK. |

Không có L3 nào bị đánh dấu `exported` khi chưa có receipt; `planned` không phải là cam kết đã xong.

**Trạng thái 2026-10-22:** 76/76 L3 của 8 owner có `sdk_export_plan`; 30 `exported` (có receipt), 36 `planned` (có `candidate_api` + `host_only`), 10 `not_in_sdk`. `OM9-L3-01-03-013` (provenance) đã chuyển từ `planned` sang `exported` khi wrap. Bảng đọc nhanh: `docs/validation/2026-10-21-l3-sdk-export-plan.md` (sinh bằng `tools/l3_sdk_export_plan.py`).

## Phụ lục: khoảng trống đã nhận diện trong các owner

Tất cả tên API ở cột “đề xuất” là thiết kế cần thực hiện, **chưa gọi được** trong SDK v0.1.0. Tên symbol nội bộ chỉ giúp tìm code, không phải ABI công khai.

| Nhóm kỹ thuật tham khảo | Owner chính | Code/năng lực nội bộ đã nhận diện | Gap SDK và API đề xuất | Điều kiện đóng |
|---|---|---|---|---|
| document, units và attributes | M01 `01-08` | `om9-core/src/units.rs`, `coords.rs`: LengthMm/AngleDeg/AngleRad, world/local, parse/resolve. `SOURCE-PRESENT`; parser đã xuất qua SDK (2026-10-21). | Đã xuất `document_units`, `parse_length`, `parse_angle`, `parse_coordinates`. Còn `resolve_coordinates` qua CPlane, tolerance setter và typed builder parameters. | Round-trip unit, mm/local/world rõ ràng, số không hữu hạn bị từ chối, plugin parity. |
| document, units và attributes | M01 `01-03`, `03-14` | `model.rs`: IDs, permissions, attributes, layers, groups/extensions. SDK chỉ đọc một phần. | `query_objects`, `object_attributes`, `set_object_attributes`, layer CRUD/current layer. | Lock/Hide/Selectable và Undo/persistence đúng contract; không nhân đôi DB layer trong plugin. |
| selection và context | M02 `01-07` | Selection, subobject/occurrence và drag có evidence M02 2026-10-19; review ngoài còn mở. | `query_selection`, `set_selection`, typed `SubobjectRef`, `pick_query`. | Cùng filter/permission/scene revision với UI; stale và ambiguity bị từ chối. |
| selection và context | M01 `03-01`; M02 `02-02`, `03-02` | Camera/CPlane, project/ray/pan/orbit/zoom nội bộ; navigation có automated evidence, performance acceptance còn mở. | `active_view`, `view_snapshot`, `cplane`, projection/ray và dịch vụ chỉnh view giới hạn. | Không giữ UI reference; đổi view không sửa geometry; không dùng API availability làm bằng chứng FPS. |
| Curve và đo đạc | M01 `02-01`; M03 `02-04` | `geometry.rs`, `evaluation.rs`, `curve_build.rs`, `offset_curve.rs`: NURBS, evaluate/derivatives, interpolate, arc/ellipse, rebuild, knots, degree, fit. `SOURCE-PRESENT`, semantics có evidence riêng. | Curve DTO và `evaluate_curve`, `curve_derivatives`, build/edit curve capability. | Domain, rational weights/knots, closure/seam, tolerance; geometry parity và failure rollback. |
| Surface | M03 `02-05`, `01-09` | `surface_kernel.rs`: evaluate/normal/isocurve/refit, plane/cylinder/loft/path frames/sweep1/sweep2. `SOURCE-PRESENT`. | Surface DTO; evaluate/derivatives/normals; plane/loft/sweep service. | UV/trim/normal semantics; numerical oracle; không gọi mọi Network/Patch đã hoàn thành. |
| Curve và đo đạc/Surface | M03 `03-20` | `measure.rs`: distance, distance_mm, angle_degrees. | `measure_distance`, `measure_angle`; thêm length/area/volume khi backend được chứng minh. | Units/tolerance, degenerate cases, scaled/instance placement; không suy thể tích từ display mesh tùy tiện. |
| native BRep/Solid | M01 `01-05`; M03 `01-09`, `01-10` | `om9-kernel/src/brep.rs`: box/sphere/read BRep/source volume/3DM conversion có selected native path. | Opaque BRep IDs, `create_box`, `create_sphere`, validity/volume theo capability. Boolean/fillet/shell tổng quát phải kiểm tra riêng. | Native lifetime/budget/crash containment + runtime đúng target; không export fixture-only probe. |
| project và exchange | M01 `01-12`, `03-15` | `om9-archive` encode/decode/save/open_restored; app File lifecycle. | Project operations thông qua host lifecycle và plugin-data persistence. | Atomic files, validated load, dirty state/context invalidation; không thay active doc lén giữa command. |
| project và exchange | M03 `01-13` | `om9-exchange`, `om9-3dm`, native exchange; 12 L3 có runtime metadata; editable BRep có evidence M03 2026-10-20. | `import_3dm`, `export_3dm`, structured diagnostics/options qua service hiện có. | Đúng loại hình/units/layer/blocks; strict atomic failure; giữ giới hạn trim và manual review. |
| project và exchange | M07 qua `01-12-008` | `om9-exchange` STL writer; phải kiểm tra đường tessellation đầy đủ trước khi xuất. | `export_stl` với tolerance/mm/winding/selection và publish file atomic. | External viewer/manifold theo fixture kỳ vọng; không xuất mesh xem nhanh như geometry chuẩn. |
| History và Builders | M04 `01-04`, `03-13` | Core history/provenance/results/workers có implementation nội bộ. Chưa chứng minh toàn bộ Builder UI. | Feature references, feature registration/update/query, persisted plugin schema. | DAG/cycle/stale/deletion/rebuild/error/cancel/Undo/reload cùng host. |
| command/input/UI | M01 `01-06`, `01-02`, `03-07`, `03-10` | Scene primitive batches, command dispatch/session, menu/CMD và host plugin command có một phần. | Prompt/input sessions, preview overlay, menu/toolbar contribution, message service. | Shared command implementation, cleanup khi cancel/unload/context change; GPU vẫn host-owned. |

Chi tiết nguồn đã đọc nằm trong [API-INVENTORY](../sdk/API-INVENTORY.md). Không đổi `SOURCE-PRESENT` thành `EVIDENCE-SCOPED` chỉ vì hàm compile hoặc vì có bài test ở host cũ.

### Những hàm tuyệt đối không được xuất vội

- `om9-kernel/src/lib.rs` có các probe như `topology`, `transform_box`, `curve_point_and_derivative`, `surface_point_and_normal`, `line_face_hits`, các phép đo drill/fillet/offset trên fixture cố định. Chúng không phải dịch vụ sửa hình tổng quát.
- `brep_kernel::KernelBackend` có `UnlinkedBackend`; trait public không chứng minh OCCT operation đã được nối.
- Không đưa internal `Document`, `TransactionCoordinator`, egui/wgpu structs hoặc con trỏ C++ qua Rust ABI của dynamic library. DTO/C ABI phải kiểm soát version, ownership và allocation.
- Polycurve hiển thị/picking đã kiểm chứng chỉ một tập direct-line kết nối đúng hướng. Không quảng cáo mọi Polycurve hay global curve parameter đã hỗ trợ.

## Quy trình cập nhật owner khi triển khai tính năng

1. **Chọn owner:** đọc M01 foundation và exact L2/L3 của tính năng. Tra hàng tương ứng trong phụ lục.
2. **Kiểm tra phần đã làm:** tìm command, Rust service, native adapter và evidence. Nếu chỉ có FreeCAD/QymCAD/probe thì ghi đúng loại nguồn; không đánh dấu standalone đã có.
3. **Chọn đường:** code đạt scope → export; code một phần → export phần đạt và co-implement phần thiếu; không có → làm shared service + SDK cùng lượt; host internal → ghi lý do và façade cần thiết.
4. **Định nghĩa public contract trước adapter:** DTO, units, coordinate space, IDs/revision, errors, budgets, thread/cancel/transaction, capability/version; tên đề xuất chưa thành API available.
5. **Triển khai một nghiệp vụ:** Rust service được cả app và host SDK gọi. C++ ngoại lệ phải ghi lý do; không nhân đôi thuật toán trong plugin host.
6. **Kết nối đủ đường:** DTO → SDK wrapper → ABI request/result → host dispatch → shared service → atomic publish → scene/persistence. Types-only không được gọi là đã merge SDK.
7. **Kiểm chứng:** service oracle; SDK validation; app menu/CMD nơi có UI; native plugin bên ngoài; Undo/Cancel/Redo; round-trip; native target checks khi cần.
8. **Cập nhật bằng chứng:** scoped validation record, owner `runtime_implementation`, traceability/status theo schema đang có, README/API inventory, live ledger và hàng kế hoạch này. Không rewrite imported planning_state để che lịch sử.
9. **Đóng phần SDK trong owner:** ghi exact symbols/capability, supported types/options và target proof. Chỉ phần scope pass được chuyển `SDK-SUBSET`/merged; phần thiếu vẫn ở backlog.
10. **Skill:** mapping migration chỉ dùng public APIs đã được chứng minh. Nếu cần thay business rule tái sử dụng, trình before/after và xin consent theo AGENTS; cập nhật evidence kỹ thuật không tự thành approval.

### Điều kiện “đã merge SDK” cho một năng lực

- [ ] Có canonical owner và phạm vi API cụ thể, không dùng tên milestone lớn thay scope.
- [ ] Public SDK crate/bundle dùng được độc lập; không phụ thuộc internal crates hoặc upstream private paths.
- [ ] Request/response/DTO + wrapper + host dispatch hoạt động thực tế và capability khai báo đúng.
- [ ] Shared service cùng semantics với app; validation/permissions/units/revision/budget được giữ.
- [ ] Mutations atomic, Undo/Redo/Cancel và failed command giữ old Redo theo contract.
- [ ] Native plugin `.om9p` build từ SDK ngoài repo chạy được; native target evidence tách Windows/macOS.
- [ ] Geometry/metadata/save-reopen đúng oracle của owner; UI/manual gates ghi riêng.
- [ ] API inventory, example, supported targets, evidence và live ledger đồng bộ.

Một tính năng đã xong nội bộ nhưng chưa đạt các mục trên được ghi **“core có; SDK chưa tích hợp”**. Một tính năng mới chưa có public adapter không được ghi **“SDK ready”**. Không bắt mọi chi tiết nội bộ phải có API; `HOST-INTERNAL` cần lý do ownership rõ và service thay thế nếu có nhu cầu plugin.

### Mẫu cập nhật một hàng công việc

```yaml
canonical_owner: OM9-L3-xx-yy-zzz
core_state: SOURCE-PRESENT | EVIDENCE-SCOPED | AUDIT-REQUIRED | NOT-IMPLEMENTED-VERIFIED
sdk_state: EXPORT-BACKLOG | CO-IMPLEMENT | SDK-SUBSET | HOST-INTERNAL
scope: "Đúng loại hình, options và giới hạn; không viết hoàn thành cả milestone"
source_symbols: ["crate::module::symbol"]
public_capability: "proposed cho đến khi runtime pass"
public_api: ["symbols thực sự xuất sau khi triển khai"]
shared_service: "một owner Rust, không nhân đôi nghiệp vụ"
evidence: ["record bền vững trong docs/ hoặc implementation/"]
target_proof: {windows_x64: unverified, macos_arm64: unverified, macos_x64: unverified}
remaining: ["scope/gate còn thiếu"]
next_action: "Một thay đổi cụ thể và oracle để đóng"
```

Mẫu này là thông tin cần ghi trong kế hoạch/evidence, không yêu cầu tự đổi schema tất cả JSON owner. Trước khi thêm field vào milestone phải theo cấu trúc hiện hữu và tooling liên quan.

## Kiểm chứng kỹ thuật trong từng milestone

Khi có coding thật, chạy tập kiểm thử phù hợp với capability; không lặp toàn workspace nếu không có lý do. Build từ root với `CARGO_TARGET_DIR=L:\om9-target` và `-j 2`; không tạo lại thư mục `standalone/`. Ví dụ tập SDK nền tảng:

```powershell
rtk cargo test -p om9-sdk -p om9-plugin-host -j 2
rtk python tools/public_source_audit.py
```

`CARGO_TARGET_DIR` phải được đặt vào môi trường trước khi gọi cargo. Thêm đúng core/app/native tests của owner, external plugin test và runtime target khi có native thay đổi. Đọc [build-validation skill](../skills/openmatrix9-build-validation/SKILL.md) khi chuyển sang thực thi; lệnh minh họa trên không thay procedure của từng native adapter.

Sau một validation batch có artifact UI, chạy `rtk python tools/prune_build_artifacts.py --apply`; giữ 10 run mới nhất và các run được dẫn chứng theo tool. Measurement openNURBS mới phải ghi named batch, fixtures/results, phần chứng minh, prepared batches, open packages và next action vào roadmap hiện có. Tổng batch tương lai còn lại chưa được enumerate đầy đủ thì ghi unknown; không đổi số test thành phần trăm.

Đợt viết kế hoạch này chỉ kiểm tra tài liệu, catalog coverage, IDs/links và public-source audit; không chạy lại hoặc nâng trạng thái các kết quả runtime kế thừa.

## Điểm bắt đầu trong milestone hiện có

Khi bắt đầu lượt implementation tiếp theo, chọn owner M01 `OM9-L2-01-08` (units/tolerance) hoặc `OM9-L2-01-03` (document/attributes), đọc các L3 tương ứng và đóng một việc SDK cụ thể tại owner đó. Tận dụng service đã có và kiểm chứng bằng plugin ngoài repo. NURBS/Curve tiếp tục ở owner M03; audit native portability theo M00/M01. Không tạo các phase SDK độc lập hoặc bắt đầu đồng loạt 791 L3.

## Nhật ký thực hiện

Các lượt thực hiện trong ngày 2026-10-21, đúng owner và đúng “Điểm bắt đầu” bên dưới:

| Ngày | Owner/L3 | Việc đã làm | Bằng chứng đo | Còn mở |
|---|---|---|---|---|
| 2026-10-21 | M01 `OM9-L2-01-08`; `OM9-L3-01-08-001/002/003/007` | Retrofit SDK v1: DTO `DocumentUnits`/`DocumentTolerance`/`LengthValue`/`AngleValue`/`Coordinates`; method `document_units`, `parse_length`, `parse_angle`, `parse_coordinates`; host dispatch gọi `om9_core::units` và `coords::parse_local` (không parser thứ hai); capability list; lệnh example `UnitsReport`; test host + test DLL native. | `docs/validation/2026-10-21-m01-08-units-sdk.md` và `.json`: 32 test SDK/host, 4 test Controller, 4 test DLL native, batch `validate_plugin_sdk.py` PASS (10 command, `.om9p`/`.om9`/SDK ZIP hash ổn định, cancel/unknown giữ file), public source audit 0 lỗi. | `L3-004/005/006` (typed builder parameter/range/serialization), tolerance setter, CPlane `resolve_coordinates`, macOS runtime, manual gate, UI acceptance. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-002/003` | Retrofit SDK v1: `query_objects` (filter + paging + generation/revision), `object_attributes`, `set_object_attributes` (color/visible/hidden/selectable/locked), `rename_object`; host đánh giá bản staged bằng `Document::permissions_for_attributes` và phát `Mutation::Attributes`/`Mutation::Label`; bảng `ObjectInfo` thêm generation/revision; lệnh example `AttributesDemo`; test host + test DLL native. | `docs/validation/2026-10-21-m01-03-attributes-sdk.md` và `.json`: 35 test SDK/host, 399 test core (47 suite), 5 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Layer create/update/current/assign, groups/blocks, change notification, snapshot/restore, provenance; layer tạo trong cùng command chưa nhận object mới (transaction insert admission); macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-004/005` | Retrofit SDK v1 layer service: `layer_create`, `layer_update`, `set_current_layer`, `assign_objects_to_layer`; host stage `LayerAction`/`LayerPlan` qua coordinator, đọc bằng `layer_permissions_with`/`permissions_for_attributes_with` (tách từ core, hành vi không đổi); giữ rule một layer write/layer/command và rule chống đọc permission cũ; lệnh example `LayerDemo`; test host + test DLL native. | `docs/validation/2026-10-21-m01-03-layers-sdk.md` và `.json`: 37 test SDK/host, 399 test core, 6 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Layer delete/reparent/preset-slot (khoảng trống “tạo layer rồi thêm object” đã đóng ở lượt sau); groups/blocks, change notification, snapshot, provenance; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-02`; `OM9-L3-01-02-006` (hỗ trợ `OM9-L2-01-03` L3-004/005) | Đóng khoảng trống “tạo layer rồi thêm object trong cùng command”: `Mutation::Insert` phân giải layer được đề xuất trong request + parent chain; `check_layer_reads_with` loại trừ đúng các layer do request tạo khỏi guard đọc/ghi; giữ nguyên rule từ chối output vào layer đang đổi cờ. Example `LayerDemo` tạo layer current, thêm point, gán object trong một command. | `docs/validation/2026-10-21-m01-02-created-layer-output.md` và `.json`: 403 test core (47 suite, +4 test admission), 38 test SDK/host, 6 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | `Mutation::Publish`/cross-document `Move` vẫn validate trên base layer; layer delete/reparent/preset-slot, groups/blocks, notification, snapshot, provenance; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-02`; `OM9-L3-01-02-010` | Export command feedback: `Host::progress`/`warning` + `MessageLevel`/`PluginMessage`; host stage 32×512 và trả trong `PluginRun`; Controller phát `Progress`/`Warn` vào EventLog hiện có trước khi publish; example `FeedbackDemo`; test host + test DLL native cho cả nhánh commit và refused. | `docs/validation/2026-10-21-m01-02-command-feedback.md` và `.json`: 43 test SDK/host, 403 test core, 8 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Interactive prompt/input sessions, option tokens, preview overlay, dynamic menu, observer callback; báo cáo trên nhánh lỗi; macOS, manual gate. |
| 2026-10-21 | M02 `OM9-L2-01-07`; `OM9-L3-01-07-002` | Export selection write: `set_selection`/`add_to_selection`; host kiểm tra id canonical/tồn tại/effective selectable và budget; Controller kiểm tra scene root + pick geometry trước commit; `selected_objects` đọc bản staged; example `SelectionDemo`; test host + Controller + DLL native. | `docs/validation/2026-10-21-m02-selection-sdk.md` và `.json`: 45 test SDK/host, 403 test core, 5 test Controller, 9 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Remove/toggle, typed subobject/occurrence, pick query, window/crossing, drag/highlight; macOS, manual gate. |
| 2026-10-21 | M02 `OM9-L2-01-07`; `OM9-L3-01-07-002` | Hoàn tất add/remove/toggle: `remove_from_selection`/`toggle_selection`; leaving không cần selectable, joining vẫn cần root live/selectable và Controller validate trước commit; example `SelectionDemo` mở rộng; test host + DLL native. | `docs/validation/2026-10-21-m02-selection-toggle.md` và `.json`: 46 test SDK/host, 403 test core, 5 test Controller, 9 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Typed subobject/occurrence, pick query, window/crossing, drag/highlight; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-02-01`; `OM9-L3-02-01-002` | Export curve read: `curve_info` (kind/domain/degree/poles/periodic/reversed) và `evaluate_curve` (native parameter, D1-D3, object-local mm, ≤1024 mẫu) dùng `evaluation::evaluate_curve`; polycurve bị từ chối; example `CurveDemo`; test host + DLL native. | `docs/validation/2026-10-21-m03-curve-evaluation.md` và `.json`: 47 test SDK/host, 403 test core, 5 test Controller, 10 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | NURBS weights/knots write, polycurve segment, curve edit/offset/fillet/blend/rebuild/intersections, surface/trim/blocks; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-02-01`; `OM9-L3-02-01-002/003` | Export `curve_definition` (NURBS poles/weights/knots/seam, arc angle domain, bounded) và `curve_semantics` (closed/planar dưới tolerance tường minh hoặc document; từ chối thiếu tolerance/degenerate) dùng `rhino_semantics::classify_curve`; example `CurveDemo` mở rộng; test host + DLL native. | `docs/validation/2026-10-21-m01-02-01-curve-definition.md` và `.json`: 48 test SDK/host, 403 test core, 5 test Controller, 10 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Control write, curve construction/edit/offset/fillet/blend/rebuild/intersections, polycurve segment, surface/trim/blocks; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-02-04`; `OM9-L3-02-04-002/003` | Export curve construction: `add_arc` (arc_through, angle domain) và `add_interpolated_curve` (interpolate, odd degree, chord/uniform) qua shared output path; example `CurveBuildDemo`; test host + DLL native. | `docs/validation/2026-10-21-m03-curve-construction.md` và `.json`: 49 test SDK/host, 403 test core, 5 test Controller, 11 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Control-point NURBS, ellipse/polygon, edit/offset/fillet/blend/rebuild/intersections, polycurve segment, surface/BRep; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-02-04`; `OM9-L3-02-04-003` | Export `add_nurbs_curve` (control-point record: weight/pole, knot count, clamped domain, `NurbsCurve::validate`); periodic/seam không mở; example `CurveBuildDemo` thêm curve thứ ba; test host + DLL native. | `docs/validation/2026-10-21-m03-nurbs-construction.md` và `.json`: 50 test SDK/host, 403 test core, 5 test Controller, 11 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Periodic/seam NURBS, ellipse/polygon, edit/offset/fillet/blend/rebuild/intersections, polycurve segment, surface/BRep; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-03-20`; `OM9-L3-03-20-001/002` | Export `measure_distance`/`measure_angle` dùng service analysis của owner; DTO typed; example `MeasureDemo`; test host + DLL native; đo không đổi document/Undo. | `docs/validation/2026-10-21-m03-measure.md` và `.json`: 51 test SDK/host, 403 test core, 5 test Controller, 12 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Object-to-object, length/radius/area/volume, bounding/statistics, label/feedback; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-02-04`; `OM9-L3-02-04-002` | Export `add_ellipse` (rational quadratic NURBS) và `add_rectangle` (closed 4-segment polycurve) từ frame trực chuẩn của caller; example `CurveShapeDemo`; test host + DLL native; một transaction/Undo. | `docs/validation/2026-10-21-m03-curve-shapes.md` và `.json`: 52 test SDK/host, 403 test core, 5 test Controller, 13 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Polygon-by-points, spiral/helix, edit/offset/fillet/blend/rebuild, periodic/seam NURBS; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-02-04`; `OM9-L3-02-04-002` | Export `add_polygon` (3+ điểm, khép kín một lần, không cạnh suy biến) qua `curve_build::polyline`; example `CurveShapeDemo` mở rộng; test host + DLL native; một transaction/Undo. | `docs/validation/2026-10-21-m03-polygon.md` và `.json`: 53 test SDK/host, 403 test core, 5 test Controller, 13 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Spiral/helix, edit/offset/fillet/blend/rebuild, periodic/seam NURBS; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-12`; `OM9-L3-01-12-005` | File lifecycle: `project_state` (path/dirty) + `save_project` (atomic save sau commit qua `begin_file_operation(Save)`); untitled bị từ chối trước commit; save fail sau commit là warning; không open/create/replace. Example `ProjectDemo`; test host + DLL native (Save As thật, mở lại file). | `docs/validation/2026-10-21-m01-01-12-project-state.md` và `.json`: 54 test SDK/host, 403 test core, 5 test Controller, 14 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Open/create/Save As, write-failure recovery qua plugin path, load validation/reload; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-12`; `OM9-L3-01-12-005` | Save As request: `save_project_as` mở host Save As dialog sau commit; user chọn đích; atomic save như Save; save/save-as không kết hợp; example `ProjectDemo` thêm `save_as`; test host + DLL native (dialog→accept path→mở lại file). | `docs/validation/2026-10-21-m01-01-12-save-as.md` và `.json`: 55 test SDK/host, 403 test core, 5 test Controller, 15 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Interactive path-dialog UI acceptance, write-failure recovery qua plugin path, load validation/reload; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-009` | Extension slots: `object_extensions`/`set_object_extension`/`clear_object_extension` qua `Mutation::Extension`, `Extension::new` validation; one-source-write-per-object; object tạo trong cùng command bị từ chối. Example `ExtensionDemo`; test host + DLL native; Undo/Redo. | `docs/validation/2026-10-21-m01-01-03-extensions.md` và `.json`: 56 test SDK/host, 403 test core, 5 test Controller, 16 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Extension schema interpretation, same-command tagging, change notification feed; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-002` | Placement write: `set_object_placement` qua `Mutation::Placement`, `Transform::validate`; one-source-write; object tạo trong cùng command giữ placement qua insert. Example `PlacementDemo`; test host + DLL native; Undo/Redo. | `docs/validation/2026-10-21-m01-01-03-placement.md` và `.json`: 57 test SDK/host, 403 test core, 5 test Controller, 17 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Nested/instance transform, gumball drag, created-in-command placement, change feed; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-003` | Mesh creation: `add_mesh` qua `Mesh::validate`; example `MeshDemo`; test host + DLL native; một transaction/Undo. | `docs/validation/2026-10-21-m01-01-03-mesh.md` và `.json`: 59 test SDK/host, 403 test core, 5 test Controller, 18 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Mesh normals/colors/UV, edit/boolean/repair, change feed; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-011` | Recorded change feed: Controller ghi `ChangeBatch::between` sau mỗi plugin command; `last_change` trả batch cho command sau; host-command feed ngoài ranh giới là open. Example `ChangeDemo`; test host + DLL native. | `docs/validation/2026-10-21-m01-01-03-change-feed.md` và `.json`: 60 test SDK/host, 403 test core, 5 test Controller, 19 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Host-command feed, multi-batch history, group/feature producer entries; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-02-04`; `OM9-L3-02-04-002` | Framed circle: `add_circle_on_frame` dựng `Curve::Circle` trên frame trực chuẩn (domain [0,tau]), validate record; example `CurveShapeDemo` thêm circle XZ; test host + DLL native; một transaction/Undo. | `docs/validation/2026-10-21-m03-framed-circle.md` và `.json`: 58 test SDK/host, 403 test core, 5 test Controller, 17 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Arbitrary-frame ellipse, spiral/helix, edit/offset/fillet; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-02-04`; `OM9-L3-02-04-005` | Curve segments: `curve_segments` + `evaluate_curve_segment` (polycurve segment list/eval, nested/index refused); example `CurveShapeDemo` thêm rect segments; test host + DLL native. | `docs/validation/2026-10-21-m03-curve-segments.md` và `.json`: 61 test SDK/host, 403 test core, 5 test Controller, 19 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Segment explode/split edit, nested flatten, arc-direction edit; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-03-20`; `OM9-L3-03-20-003` | Object bounds: `object_bounds` từ point/33-sample curve/mesh vertices; example `BoundsDemo`; test host + DLL native; read-only. | `docs/validation/2026-10-21-m03-bounds.md` và `.json`: 62 test SDK/host, 403 test core, 5 test Controller, 20 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Exact analytic bounds, B-Rep/surface/cloud, group bounds/count summary; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-03-20`; `OM9-L3-03-20-001` | Curve length: `measure_curve_length` (33 sampled chords); example `LengthDemo`; test host + DLL native. | `docs/validation/2026-10-21-m03-length.md` và `.json`: 63 test SDK/host, 403 test core, 5 test Controller, 21 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Exact/adaptive length, radius/area/volume, statistics; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-03-20`; `OM9-L3-03-20-002` | Circle radius: `measure_curve_radius` từ record; example `RadiusDemo`; test host + DLL native. | `docs/validation/2026-10-21-m03-radius.md` và `.json`: 64 test SDK/host, 403 test core, 5 test Controller, 22 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Ellipse radius, area/volume, statistics; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-03-20`; `OM9-L3-03-20-004` | Object summary: `object_summary` theo geometry kind, staged-consistent; example `SummaryDemo`; test host + DLL native. | `docs/validation/2026-10-21-m03-summary.md` và `.json`: 65 test SDK/host, 403 test core, 5 test Controller, 23 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Jewelry class/weight summary; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-03-20`; `OM9-L3-03-20-005` | Measurement label: `report_measurement` (Progress line); example `LabelDemo`; test host + DLL native (label trong event log). | `docs/validation/2026-10-21-m03-label.md` và `.json`: 66 test SDK/host, 403 test core, 5 test Controller, 24 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Structured measurement record, UI acceptance; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-004` | Layer order read: `layer_order` (bottom→top, kèm layer tạo cùng command); example `LayerOrderDemo`; test host + DLL native. | `docs/validation/2026-10-21-m01-01-03-layer-order.md` và `.json`: 67 test SDK/host, 403 test core, 5 test Controller, 25 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Layer reorder, group records, block instance graph; macOS, manual gate. |
| 2026-10-21 | M03 `OM9-L2-03-20`; `OM9-L3-03-20-001` | Mesh area: `measure_mesh_area` (fan triangles); example `AreaDemo`; test host + DLL native. | `docs/validation/2026-10-21-m03-area.md` và `.json`: 68 test SDK/host, 403 test core, 5 test Controller, 26 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Volume, NURBS/surface area; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-011` | Feed mở rộng: host Undo/Redo ghi batch với cause `undo`/`redo`; native test chứng minh host undo hiện trong `ChangeDemo`. | `docs/validation/2026-10-21-m01-01-03-change-undo.md` và `.json`: 68 test SDK/host, 403 test core, 5 test Controller, 27 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Host edit paths khác, multi-batch history; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-007` | Groups read: `groups` (id/name/members/revision); example `GroupsDemo`; test host + DLL native. | `docs/validation/2026-10-21-m01-01-03-groups.md` và `.json`: 13+57 test SDK/host, 403 test core, 5 test Controller, 28 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Group write, block instance graph; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-007` | Group create: `create_group` (members tồn tại, một Undo); example `GroupCreateDemo`; test host + DLL native. | `docs/validation/2026-10-21-m01-01-03-group-create.md` và `.json`: 13+62 test SDK/host, 403 test core, 5 test Controller, 33 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Group edit/dissolve, member add/remove; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-007` | Group delete: `delete_group` (record only, một Undo); example `GroupDeleteDemo`; test host + DLL native. | `docs/validation/2026-10-21-m01-01-03-group-delete.md` và `.json`: 13+63 test SDK/host, 403 test core, 5 test Controller, 34 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Group edit, member add/remove; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-004` | Single layer read: `layer_info` dùng chung helper với `layers`; test host (equality + refusal); chưa có native command. | `docs/validation/2026-10-21-m01-01-03-layer-info.md` và `.json`: 13+64 test SDK/host, 403 core, 5 Controller, batch `validate_plugin_sdk.py` PASS (34 native), public source audit 0 lỗi. | Native command, layer delete/reorder; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-004` | Native `LayerInfoDemo` + DLL test đóng gap host-only: đọc layer vừa tạo, assert id/current/locked; native suite 35. | `docs/validation/2026-10-21-m01-01-03-layer-info-native.json`: 13+64 test SDK/host, 403 core, 5 Controller, 35 native, batch PASS. | Layer delete/reorder; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-08`; `OM9-L3-01-08-003` | Tolerance write: `set_document_tolerance` (host-tested, `Mutation::Tolerance`); chưa có native command. | `docs/validation/2026-10-21-m01-01-08-tolerance-write.md` và `.json`: 13+65 SDK/host, 403 core, 5 Controller, batch PASS 35 native, audit 0 lỗi. | Native command, per-object tolerance; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-08`; `OM9-L3-01-08-003` | Native `ToleranceDemo` + DLL test: set 0.01 mm, Undo khôi phục default 0.001 mm; native suite 36. | `docs/validation/2026-10-21-m01-01-08-tolerance-native.json`: 13+65 SDK/host, 403 core, 5 Controller, 36 native, batch PASS. | Per-object tolerance; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-008` | Block definitions read: `block_definitions`; example `BlocksDemo`; test host + DLL native. | `docs/validation/2026-10-21-m01-01-03-blocks.md` và `.json`: 13+58 test SDK/host, 403 test core, 5 test Controller, 29 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Instance insertion, definition editing, graph; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-004` | Clear current layer: `clear_current_layer` (no-op khi trống, Undo khôi phục); example `ClearLayerDemo`; test host + DLL native. | `docs/validation/2026-10-21-m01-01-03-clear-layer.md` và `.json`: 13+59 test SDK/host, 403 test core, 5 test Controller, 30 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Layer delete/reorder; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-002` | Placement read: `object_placement` (staged/base); example `PlacementReadDemo`; test host + DLL native. | `docs/validation/2026-10-21-m01-01-03-placement-read.md` và `.json`: 13+60 test SDK/host, 403 test core, 5 test Controller, 31 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Instance composition, gumball drag; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-03`; `OM9-L3-01-03-002` | Object record read: `object_info`; example `InfoDemo`; test host + DLL native. | `docs/validation/2026-10-21-m01-01-03-object-info.md` và `.json`: 13+61 test SDK/host, 403 test core, 5 test Controller, 32 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Instance composition, gumball drag; macOS, manual gate. |
| 2026-10-21 | M01 `OM9-L2-01-08`; `OM9-L3-01-08-004/005/006` | Export typed builder/command parameters: `Dimension`/`ParameterDomain`/`ParameterSpec`/`ParameterValue`; `validate_parameter`/`apply_parameters` chuyển sang `request::OptionDomain`/`CommandSchema::apply` (bounds, step, count, choice, text, default, required/unknown); wire form lấy `kind` làm đơn vị; example `ParameterDemo`; test host + SDK wire + DLL native. | `docs/validation/2026-10-21-m01-08-builder-parameters.md` và `.json`: 41 test SDK/host, 403 test core, 7 test DLL native, batch `validate_plugin_sdk.py` PASS, public source audit 0 lỗi. | Builder parameter editor/UI parity (M04/M01-02), tolerance editing, CPlane frame; macOS, manual gate. |

Quy tắc: chỉ dòng có bằng chứng đo mới được ghi ở đây; phần chưa đo vẫn nằm trong bảng kế hoạch phía trên.

## Nhật ký điều chỉnh

| Ngày | Thay đổi kế hoạch | Trạng thái |
|---|---|---|
| 2026-10-21 | Theo chỉ đạo người dùng: chuyển từ lộ trình SDK riêng sang tích hợp requirements trực tiếp vào 11 L1 và 102 L2; sắp xếp tài liệu theo milestone và giữ đủ 791 L3. | Chỉ documentation/planning; giữ nguyên runtime evidence, IDs, manual và release gate statuses. |
| 2026-10-21 | Thực hiện lát cắt đầu tiên tại owner M01 `OM9-L2-01-08` (units/tolerance/parse) theo “Điểm bắt đầu”; thêm mục Nhật ký thực hiện và cập nhật 4 dòng L3 đã có SDK subset. | Đã đo trên Windows x64; 4/7 L3 có `SDK-SUBSET`; L3-004/005/006, macOS và manual gate vẫn mở. Không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai tại owner M01 `OM9-L2-01-03` (query/attributes/rename); tách `Document::permissions_for_attributes` để đánh giá bản staged bằng cùng layer chain; thêm quy tắc một source write/object/command. | Đã đo trên Windows x64; 2/13 L3 có `SDK-SUBSET`; layer CRUD/current/membership, groups/blocks, notification, snapshot vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ ba tại owner M01 `OM9-L2-01-03` (layer service); tách `layer_permissions_with`/`permissions_for_attributes_with` để đọc bản staged mà không nhân đôi rule; giữ rule chống đọc permission cũ. | Đã đo trên Windows x64; 4/13 L3 của owner có `SDK-SUBSET`; layer delete/reparent/preset-slot, groups/blocks, notification, snapshot vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ tư tại owner transaction M01 `OM9-L2-01-02`: `Mutation::Insert` chấp nhận layer được đề xuất trong cùng request và `check_layer_reads_with` chỉ loại trừ layer do request tạo; đóng khoảng trống “tạo layer rồi thêm object” cho plugin. | Đã đo trên Windows x64; 403 test core (+4 test admission); rule chống đọc permission cũ cho layer đang đổi cờ vẫn giữ; `Mutation::Publish`/cross-document Move không đổi; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ năm hoàn tất owner units M01 `OM9-L2-01-08`: export typed builder parameters (`validate_parameter`/`apply_parameters`) dùng lại `OptionDomain`/`CommandSchema`; wire form tag = đơn vị. | Đã đo trên Windows x64; 7/7 L3 của owner có `SDK-SUBSET`; UI parity (M04/M01-02), tolerance editing, CPlane frame vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ sáu tại owner command M01 `OM9-L2-01-02`: export feedback `progress`/`warning` vào EventLog dùng chung, giới hạn 32×512, không tạo Undo; command lỗi không phát report. | Đã đo trên Windows x64; 8 test DLL native; UI prompt/input, option token, preview, menu, observer callback vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ bảy tại owner selection M02 `OM9-L2-01-07`: export `set_selection`/`add_to_selection` với validation host + Controller trước commit; selection không tạo transaction/Undo. | Đã đo trên Windows x64; 9 test DLL native; remove/toggle, subobject ref, pick query, window/crossing và drag vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ tám hoàn tất L3-002 (M02 `OM9-L2-01-07`): thêm `remove_from_selection`/`toggle_selection`; leaving không cần selectable, joining giữ rule root live/selectable + Controller validate trước commit. | Đã đo trên Windows x64; 46 test SDK/host, 9 test DLL native; subobject ref, pick query, window/crossing và drag vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ chín tại owner curve M01 `OM9-L2-02-01`: export `curve_info`/`evaluate_curve` (line/circle/NURBS, native parameter, D1-D3, object-local mm) dùng lại evaluator của core; polycurve bị từ chối thay vì đoán tham số. | Đã đo trên Windows x64; 47 test SDK/host, 10 test DLL native; weights/knots write, polycurve segment và curve edit/offset/fillet vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười tại owner curve M01 `OM9-L2-02-01`: export `curve_definition` (poles/weights/knots/seam/arc, bounded) và `curve_semantics` (closed/planar dưới tolerance tường minh hoặc document; từ chối thiếu tolerance/degenerate). | Đã đo trên Windows x64; 48 test SDK/host, 10 test DLL native; control write, curve construction/edit/offset/fillet, polycurve segment và surface vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười một tại owner curve construction M03 `OM9-L2-02-04`: export `add_arc` (3-point) và `add_interpolated_curve` (odd degree, chord/uniform) qua shared output path; một transaction/Undo. | Đã đo trên Windows x64; 49 test SDK/host, 11 test DLL native; control-point NURBS, ellipse/polygon, edit/offset/fillet vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười hai hoàn tất construction của L3-02-04-003: thêm `add_nurbs_curve` (control-point record, clamped domain, `NurbsCurve::validate`); periodic/seam vẫn không mở. | Đã đo trên Windows x64; 50 test SDK/host, 11 test DLL native; periodic/seam, ellipse/polygon, edit/offset/fillet vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười ba tại owner measurement M03 `OM9-L2-03-20`: export `measure_distance`/`measure_angle` dùng `measure::distance_mm`/`angle_degrees`; DTO typed, không stage mutation. | Đã đo trên Windows x64; 51 test SDK/host, 12 test DLL native; object-to-object, length/radius/area/volume, bounding/statistics và label vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười bốn tại owner curve M03 `OM9-L2-02-04`: export `add_ellipse`/`add_rectangle` từ frame trực chuẩn; khép creation kinds của L3-002 (còn polygon-by-points). | Đã đo trên Windows x64; 52 test SDK/host, 13 test DLL native; polygon-by-points, spiral/helix, edit/offset/fillet vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười lăm tại owner curve M03 `OM9-L2-02-04`: export `add_polygon`; khép creation kinds của L3-002 (circle/arc/ellipse/rectangle/polygon). | Đã đo trên Windows x64; 53 test SDK/host, 13 test DLL native; spiral/helix, edit/offset/fillet vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười sáu tại owner file lifecycle M01 `OM9-L2-01-12`: export `project_state`/`save_project`; atomic save sau commit, untitled từ chối trước commit, save fail là warning; kiểm chứng Save As + mở lại file thật. | Đã đo trên Windows x64; 54 test SDK/host, 14 test DLL native; plugin-driven open/create/Save As và write-failure qua plugin path vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười bảy tại owner file lifecycle M01 `OM9-L2-01-12`: thêm `save_project_as` (host dialog sau commit); save/save-as loại trừ nhau; kiểm chứng dialog→accept→atomic save→mở lại file. | Đã đo trên Windows x64; 55 test SDK/host, 15 test DLL native; UI dialog acceptance và write-failure qua plugin path vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười tám tại owner document M01 `OM9-L2-01-03`: export extension slot read/write qua `Mutation::Extension`, validation của core, one-source-write; object tạo trong cùng command bị từ chối. | Đã đo trên Windows x64; 56 test SDK/host, 16 test DLL native; schema interpretation, same-command tagging và change notification feed vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ mười chín tại owner document M01 `OM9-L2-01-03`: export placement write qua `Mutation::Placement` + `Transform::validate`; one-source-write; object tạo trong cùng command giữ placement qua insert. | Đã đo trên Windows x64; 57 test SDK/host, 17 test DLL native; nested/instance transform, gumball drag và change feed vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi tại owner curve M03 `OM9-L2-02-04`: thêm `add_circle_on_frame` (circle trên frame trực chuẩn); example `CurveShapeDemo` thêm circle XZ. | Đã đo trên Windows x64; 58 test SDK/host, 17 test DLL native; arbitrary-frame ellipse, spiral/helix, edit/offset vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi mốt tại owner document M01 `OM9-L2-01-03`: export `add_mesh` (vertices/faces qua `Mesh::validate`); example `MeshDemo`. | Đã đo trên Windows x64; 59 test SDK/host, 18 test DLL native; mesh normals/colors/UV, edit/boolean/repair và change feed vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi hai tại owner document M01 `OM9-L2-01-03`: recorded change feed tại ranh giới plugin (`ChangeBatch::between`, `last_change`); example `ChangeDemo`. | Đã đo trên Windows x64; 60 test SDK/host, 19 test DLL native; host-command feed và multi-batch history vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi ba tại owner curve M03 `OM9-L2-02-04`: export `curve_segments`/`evaluate_curve_segment` (polycurve segment list/eval). | Đã đo trên Windows x64; 61 test SDK/host, 19 test DLL native; segment explode/split edit và nested flatten vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi bốn tại owner measurement M03 `OM9-L2-03-20`: export `object_bounds` (point/curve-sample/mesh); example `BoundsDemo`. | Đã đo trên Windows x64; 62 test SDK/host, 20 test DLL native; exact analytic bounds, B-Rep/surface/cloud và group bounds vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi lăm tại owner measurement M03 `OM9-L2-03-20`: export `measure_curve_length` (sampled 33 chords); example `LengthDemo`. | Đã đo trên Windows x64; 63 test SDK/host, 21 test DLL native; exact/adaptive length, radius/area/volume và statistics vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi sáu tại owner measurement M03 `OM9-L2-03-20`: thêm `measure_curve_radius` (record circle/arc); example `RadiusDemo`. | Đã đo trên Windows x64; 64 test SDK/host, 22 test DLL native; ellipse radius, area/volume và statistics vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi bảy tại owner measurement M03 `OM9-L2-03-20`: thêm `object_summary` (counts by geometry kind, staged-consistent); example `SummaryDemo`. | Đã đo trên Windows x64; 65 test SDK/host, 23 test DLL native; jewelry class/weight summary, area/volume vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi tám tại owner measurement M03 `OM9-L2-03-20`: thêm `report_measurement` (Progress label); example `LabelDemo`; owner nay 5/5 L3 SDK-SUBSET. | Đã đo trên Windows x64; 66 test SDK/host, 24 test DLL native; structured measurement record, area/volume và UI acceptance vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ hai mươi chín tại owner document M01 `OM9-L2-01-03`: thêm `layer_order` (bottom→top, kèm layer tạo cùng command); example `LayerOrderDemo`. | Đã đo trên Windows x64; 67 test SDK/host, 25 test DLL native; layer reorder, group records, block graph và host-command change feed vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ ba mươi tại owner measurement M03 `OM9-L2-03-20`: thêm `measure_mesh_area` (fan triangles); example `AreaDemo`. | Đã đo trên Windows x64; 68 test SDK/host, 26 test DLL native; volume, NURBS/surface area và UI acceptance vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ ba mươi mốt tại owner document M01 `OM9-L2-01-03`: feed ghi thêm host Undo/Redo (cause/revision thật); native test xác nhận. | Đã đo trên Windows x64; 68 test SDK/host, 27 test DLL native; host edit paths khác và multi-batch history vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ ba mươi ba tại owner document M01 `OM9-L2-01-03`: export `groups` (read-only group records); example `GroupsDemo`. | Đã đo trên Windows x64; 57 test host + 13 SDK, 28 test DLL native; group write, block graph và host edit paths vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ ba mươi bốn tại owner document M01 `OM9-L2-01-03`: export `block_definitions` (read-only); example `BlocksDemo`. | Đã đo trên Windows x64; 58 test host + 13 SDK, 29 test DLL native; instance graph và definition write vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ ba mươi lăm tại owner document M01 `OM9-L2-01-03`: thêm `clear_current_layer` (no-op khi trống); example `ClearLayerDemo`. | Đã đo trên Windows x64; 59 test host + 13 SDK, 30 test DLL native; layer delete/reorder vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ ba mươi bảy tại owner document M01 `OM9-L2-01-03`: thêm `object_placement` (staged/base); example `PlacementReadDemo`. | Đã đo trên Windows x64; 60 test host + 13 SDK, 31 test DLL native; instance composition và gumball drag vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ ba mươi tám tại owner document M01 `OM9-L2-01-03`: thêm `object_info` (single record); example `InfoDemo`. | Đã đo trên Windows x64; 61 test host + 13 SDK, 32 test DLL native; instance composition và gumball drag vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ ba mươi chín tại owner document M01 `OM9-L2-01-03`: thêm `create_group` (mutation tại base revision); example `GroupCreateDemo`. | Đã đo trên Windows x64; 62 test host + 13 SDK, 33 test DLL native; group edit/dissolve và member add/remove vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ bốn mươi tại owner document M01 `OM9-L2-01-03`: thêm `delete_group` (record only); example `GroupDeleteDemo`. | Đã đo trên Windows x64; 63 test host + 13 SDK, 34 test DLL native; group edit và member add/remove vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ bốn mươi mốt tại owner document M01 `OM9-L2-01-03`: thêm `layer_info` (host-tested, chưa có native command). | Đã đo trên Windows x64; 64 test host + 13 SDK, batch PASS 34 native; cần native command ở vòng sau; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ bốn mươi hai tại owner document M01 `OM9-L2-01-03`: thêm native command `LayerInfoDemo` + DLL test cho `layer_info`. | Đã đo trên Windows x64; 64 test host + 13 SDK, 35 native; layer delete/reorder vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ bốn mươi ba tại owner units M01 `OM9-L2-01-08`: mở tolerance **write** (`set_document_tolerance`, host-tested). | Đã đo trên Windows x64; 65 test host + 13 SDK, batch PASS 35 native; cần native command; không nâng runtime status của L2/L1. |
| 2026-10-21 | Lát cắt thứ bốn mươi bốn tại owner units M01 `OM9-L2-01-08`: thêm native `ToleranceDemo` + DLL test (set 0.01 mm, Undo về 0.001 mm). | Đã đo trên Windows x64; 65 test host + 13 SDK, 36 native; per-object tolerance vẫn mở; không nâng runtime status của L2/L1. |
| 2026-10-22 | Lát cắt thứ bốn mươi lăm: wrap `object_provenance` (L3-013) — `planned` → `exported`, example `ProvenanceDemo`. | Đã đo trên Windows x64; 66 test host + 13 SDK, 37 native; ghi provenance vẫn là flow import của host; không nâng runtime status L2/L1. |
| 2026-10-22 | Lát cắt thứ bốn mươi sáu: wrap `is_object_pickable` (L3-006) — `planned` → `exported`, example `PickableDemo`. | Đã đo trên Windows x64; 67 test host + 13 SDK, 38 native; pick geometry vẫn mở; không nâng runtime status L2/L1. |

## Kiểm tra tài liệu sau hợp nhất

Đợt kiểm tra lập kế hoạch, 2026-10-21:

- **11 L1 và 102 L2** có `sdk_integration_plan` ngay trong JSON canonical; giữ nguyên mọi field đã có trước khi bổ sung.
- **791 L3** giữ nguyên bytes tại thời điểm lập kế hoạch; checklist L2 bao phủ đúng 791 ID, không trùng và không thiếu. Điều kiện SDK áp dụng tại owner khi thực hiện từng L3.
- Bảng kế hoạch có đủ 102 L2 / 791 L3, đúng ID và file; **1.019 local links, không có đường dẫn hỏng**.
- L3 STL `OM9-L3-01-12-008` giữ owner `OM9-L2-01-12` và đích thực thi M07. Không còn phase SDK độc lập.
- `tools/public_source_audit.py`: **8.849 source files / 511 icon bindings / 510 SVG, 0 lỗi** tại thời điểm chạy; `git diff --check` với cấu hình CRLF: PASS.
- Đợt đó không sửa Rust/native runtime, không chạy measurement mới, không nâng trạng thái runtime/manual/M09.

Đợt thực hiện đầu tiên, 2026-10-21 (sau khi lập kế hoạch):

- Lượt thực hiện 2026-10-21 chỉ thêm/ghi `runtime_implementation` cho các L3 có `SDK-SUBSET`, mỗi entry trỏ tới record đo tương ứng: 7 L3 của M01 `OM9-L2-01-08` (001–007), 8 L3 của M01 `OM9-L2-01-03` (002/003/004/005/007/008/009/011), 2 L3 của M01 `OM9-L2-01-02` (006/010), 1 L3 của M02 `OM9-L2-01-07` (002), 2 L3 của M01 `OM9-L2-02-01` (002/003), 3 L3 của M03 `OM9-L2-02-04` (002/003/005), 5 L3 của M03 `OM9-L2-03-20` (001–005) và 1 L3 của M01 `OM9-L2-01-12` (005) — tổng 29 L3; ID, `specification`, `execution_design`, `task_ids` và L4 của chúng không đổi; các L3 còn lại giữ nguyên bytes.
- `L2.json` của hai owner thêm `sdk_implementation_progress` (M01-03 có hai lượt ghi); không đổi `planning_state`, `sdk_integration_plan` hay gate/manual status. `Document::permissions_for_attributes`, `layer_permissions_with` và `permissions_for_attributes_with` được tách trong core (hành vi không đổi) để đánh giá bản staged bằng cùng rule.
- Kiểm tra lại: toàn bộ JSON của tám owner parse hợp lệ; `tools/public_source_audit.py` báo **8.863 source files / 511 icon bindings / 510 SVG, 0 lỗi**; `cargo test -p om9-core` 403 pass/47 suite; `git diff --check` PASS; rustfmt sạch trên các file Rust đã sửa.

