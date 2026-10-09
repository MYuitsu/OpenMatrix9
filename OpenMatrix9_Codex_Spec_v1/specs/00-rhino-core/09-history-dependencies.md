---
id: RCORE-09
title: Associative History và identity bền vững
priority: P0
review_date: '2026-10-09'
evidence_level: source_inspected
runtime_validation: not_run
evidence_scope: initial_source_audit_2026-10-09
freecad_commit: 21d36cfa1eb110a1d0667050ff31706298805bbd
openmatrix9_commit: 52e887ab706dcd5d80e23979fb1f1b62d8d869b0
---

# RCORE-09 — Associative History và identity bền vững

## Cập nhật triển khai P0 — 2026-10-09

Metadata và bảng source phía dưới ghi baseline audit ban đầu. Policy Rust đã
tách Record (links mới) khỏi Update (graph đã ghi); Record Off không đóng băng
descendants khi Update On. Scheduler kiểm cycle/duplicate/null IDs và bounds cả
khi Update Off; native dùng document IDs thay address. Join có `HistoryDirty`
persistent cho output stale/failed; record thiếu parent không trả success cùng
shape cũ. Detach giữ policy Undo riêng của OM9.

Fixture History/I/O mới đã pass **7 checks** trong phạm vi được ghi ở
[báo cáo P0](../../../docs/validation/2026-10-09-rhino-core-p0.md); hồi quy History
83, Circle History 89, Builder History 43 và cold restore stale-state 2 checks
đã qua, process exit 0. Không coi các assertions
là nghiệm thu đầy đủ RCORE-09.T01–T12.

**P0 còn thiếu:** record schema chung với generation/revisions, typed units/frame,
command/backend/schema version và migration; stale/failed state thống nhất cho
Surface/Circle/Builder/Cage; pre-mutation DAG/placement checks mọi đường; semantic
subelement identity; cross-family replay/error propagation và async cancellation.
Affine-template Builder không chứng minh mọi Matrix Builder/Style đã replay được.

## Phạm vi và mức bằng chứng

Đặc tả chi tiết hóa [nhóm nền Rhino](README.md).
Đây là yêu cầu nền dùng chung; không tạo thêm command hoặc nâng trạng thái 607 feature.
Liên quan `OM9-HISTORY-001`, `OM9-INFO-015..018`, Builder, Styles và các family có History.
FreeCAD đã có document graph, property links, recompute và transactions trong mã nguồn.
Các cơ chế đó là backend khả dụng, chưa chứng minh toàn bộ workflow History Rhino hay Matrix.
Các nhận định dưới đây dựa trên source hiện có ngày 2026-10-09; working tree có thay đổi.
Lần audit nguồn ban đầu không chạy FreeCAD, native tests, cold reload hoặc Rhino; evidence triển khai mới được ghi riêng ở phần cập nhật.

[Rhino 5 History help](https://docs.mcneel.com/rhino/5/help/en-us/commands/history.htm)
mô tả liên hệ input–output, Record, Update, Lock và cảnh báo đứt liên kết.
Child khóa History vẫn có thể được chọn làm input; đây không phải khóa toàn bộ đối tượng.
Help còn ghi HistoryPurge không Undo. Vì vậy Clear Object History có transaction của OM9
phải được công bố là policy OM9; không gọi nó tương đương mọi semantics của HistoryPurge.
Danh sách lệnh có History trong help cũng không chứng minh builder/plugin Matrix đã được tái tạo.

## Hợp đồng input, output và identity

Input phải là object handle theo document, generation và revision; tên hiển thị chỉ phục vụ UI.
Mỗi cạnh phụ thuộc lưu source identity, subelement selector và lý do phụ thuộc.
Phân biệt nguồn hình học, nguồn placement/hierarchy, giá trị tham số và tài nguyên bên ngoài.
Record cần command ID/version, solver/backend version và schema version để phục hồi có kiểm soát.
Parameters ghi giá trị typed, units, frame và tolerance context đã dùng khi commit.
Output có identity ổn định trong lifecycle của record; recompute không tùy tiện tạo child mới.
Không dùng command transcript, nhãn tree hoặc chuỗi `Edge3` đơn lẻ làm toàn bộ record.

| Trường bắt buộc trong record mục tiêu | Ý nghĩa và bất biến |
|---|---|
| `document_id`, `object_id`, `generation` | Loại bỏ reference trỏ sang document khác hoặc object đã xóa rồi tạo lại. |
| `command_id`, `command_version` | Exact ID gốc và phiên bản hành vi; không suy từ alias. |
| `sources[]` | Identity, subelement và role; giữ thứ tự profile/rail khi solver cần. |
| `parameters`, `units`, `frame` | Đủ để replay độc lập với dialog, camera và units đang hiển thị. |
| `source_revisions[]`, `output_revision` | Chứng minh output ứng với snapshot nào; worker cũ không được commit. |
| `record`, `update`, `lock` | Chính sách thao tác, không gộp với `dirty` hoặc trạng thái khóa layer. |
| `state`, `diagnostic` | Clean/dirty/pending/unresolved/failed/detached; lỗi có source và operation. |
| `output_ids[]` | Một hoặc nhiều kết quả; quan hệ ownership, thay thế và xóa được xác định. |

Units chiều dài/góc không được lấy lại từ UI khi recompute một record đã lưu.
Thay placement parent phải tính world transform đúng một lần, kể cả parent nằm trong hierarchy.
Topology đổi phải resolve bằng cơ chế tham chiếu đã kiểm chứng hoặc trở thành `unresolved`.
Không tự nối child sang cạnh khác chỉ vì `EdgeN` đó vẫn tồn tại sau topology change.
Nếu feature chỉ hỗ trợ whole object, input subelement phải bị từ chối trước commit.

## State, lỗi, Undo và persistence

Record Off: command tạo snapshot output theo contract riêng, không giả có dependency graph.
Record On: geometry, links, parameters và metadata được commit trong cùng transaction.
Update Off theo policy OM9: thay parent đánh dirty; child không được trình bày là đã cập nhật.
Không gán cơ chế giữ dirty này cho Rhino; nhánh native đã đọc xóa pending queue khi Update tắt.
Update On: lập lịch theo dependency order; lỗi một node nêu rõ các downstream bị chặn.
Lock: chặn sửa geometry child theo family; sửa appearance hoặc dùng làm input xét riêng.
Clear/Detach: bỏ liên hệ đã chọn nhưng giữ geometry hiện tại theo policy OM9 đã khai báo.
Xóa parent: family phải chọn từ chối, detach có thông báo hoặc unresolved; không âm thầm rebind.
Cycle: kiểm tra trước mutation; rollback phải khôi phục cả graph lẫn geometry.
Preview không tạo property links bền vững, không thay dirty state của record đã commit.
Cancel/đóng document/deactivate dọn callback, preview và request còn chờ.
Recompute thất bại không được giữ hình cũ rồi gắn trạng thái clean hoặc báo success.
Nếu giữ hình cuối hợp lệ để người dùng quan sát, phải gắn stale/failed rõ ràng.
Undo/Redo phải khôi phục links, parameters, trạng thái, geometry và lock policy cùng nhau.
Lịch sử Undo của phiên và record History lưu trong FCStd là hai vòng đời khác nhau.
Cold reload phải phục hồi qua serializer/module registration; không phụ thuộc raw pointer phiên cũ.
Thiếu type/module khi mở file phải báo capability hoặc lỗi restore; không nâng thành graph hợp lệ.

## Chi tiết bổ sung từ code decompile

Các chi tiết dưới đây được đọc từ code decompile; chưa chạy binary, UI hoặc fixture runtime.
Phân biệt logic managed, wrapper gọi native và phần triển khai native khi xác định phạm vi đã hiểu.
Các phát hiện này không nâng trạng thái triển khai hoặc đóng nghiệm thu FreeCAD/OM9.

| Hành vi và hệ quả hợp đồng | Giới hạn |
|---|---|
| HistoryRecord được tạo bằng command UUID và version; các setter dùng value ID typed, gồm ObjRef và geometry. ReplayHistoryData resolve ObjRef hiện hành, trả version/record ID và mảng kết quả; mỗi kết quả liên hệ ExistingObject và các UpdateTo*. Command.ReplayHistory cơ sở trả false, callback báo exception và trả thất bại. Có record hoặc hook chưa đủ chứng minh command replay được. | Setter, resolver và cập nhật geometry chủ yếu gọi native; chưa kiểm identity của mọi output, rollback hoặc topology rebinding lúc chạy. |
| Native ON_HistoryRecord.Dump phân biệt command ID, version, record ID, antecedent IDs và descendant IDs. Write lưu UUID, version, hai danh sách UUID và các value có kiểu trong chunks. Vì vậy dependency record không đồng nhất với transcript lệnh hoặc stack Undo. | Đã đối chiếu tên field từ Dump với Write; không suy rằng lưu record có nghĩa có solver/plugin phù hợp để replay sau mở lại. |
| HistorySettings có ba cờ riêng RecordingEnabled, UpdateEnabled, ObjectLockingEnabled. Native EnableHistoryUpdate ghi cờ mà UpdateDescendants kiểm; khi tắt, đường đã đọc xóa danh sách UUID đang chờ rồi thoát. Yêu cầu OM9 giữ dirty để cập nhật thủ công là policy OM9, không được mô tả là cơ chế pending của Rhino đã phục dựng. | Không suy default UI từ getters/setters; chưa truy hết các đường thêm record, bật lại Update, lock child và scheduling toàn graph. |
| FourRailProfileBuilder ghi ObjRef các rail vào HistoryRecord, còn rail IDs/count, plane và profile nằm trong object UserDictionary. DoHistory kiểm version, lấy ExistingObject của kết quả đầu, khôi phục builder từ object rồi cập nhật curve. Đây là ví dụ Matrix thực tế kết hợp record và metadata, không phải schema chung mọi Builder. | Chỉ family FourRailProfile; chưa chạy solver hoặc dựng lại UI. Hàm gọi UpdateToCurve nhưng không kiểm bool trả về, nên return true của caller không tự chứng minh cập nhật thành công. |

Các yêu cầu về schema state/identity, worker, transaction và xử lý lỗi của OM9 vẫn là quyết định
thiết kế, trừ hành vi gốc được nêu rõ ở trên. Không suy defaults toàn ứng dụng từ một caller.

## FreeCAD đã có đến đâu; OM9 cần bổ sung gì

`UI+API`/`API` là phát hiện source, không phải runtime pass; `Một phần` chỉ phần nêu trong ô.
`Chưa thấy trong phạm vi rà` không khẳng định thiếu ở mọi workbench/add-on ngoài checkout này.

| Capability | Yêu cầu | FreeCAD | Bằng chứng/phạm vi | OM9 riêng |
|---|---|---|---|---|
| RCORE-09.C01 | Links object và subelement | API | `PropertyLink`, `PropertyLinkList`, `PropertyLinkSub` trong [PropertyLinks.h](../../../../../src/App/PropertyLinks.h). | Đã thấy `Parents`, `SourceCurve`, `SourceCurves`; chưa phải schema chung mọi family. |
| RCORE-09.C02 | Dependency/recompute | UI+API | `Document::recompute`, `DocumentObject::execute/mustExecute` trong [Document.h](../../../../../src/App/Document.h), [DocumentObject.h](../../../../../src/App/DocumentObject.h); `StdCmdRefresh` trong [CommandDoc.cpp](../../../../../src/Gui/CommandDoc.cpp). | Join/Circle/Surface dùng native feature; cần thống nhất dirty/error policy. |
| RCORE-09.C03 | Commit/abort/Undo/Redo | API | `openTransaction`, `commitTransaction`, `abortTransaction`, undo/redo trong [Document.cpp](../../../../../src/App/Document.cpp). | Controller mở transaction; còn cần fixture nguyên tử xuyên family. |
| RCORE-09.C04 | Record/Update/Lock/Clear kiểu Rhino | Một phần | Host graph không tự cung cấp semantics Record một lệnh hoặc khóa child của Rhino. | [HistoryController.cpp](../../../Gui/HistoryController.cpp) và `HistorySettings` có bốn option; cần mapping theo family. |
| RCORE-09.C05 | Kiểm tra DAG/cycle | API | `testIfLinkDAGCompatible` trong [DocumentObject.h](../../../../../src/App/DocumentObject.h). | Phải kiểm cả dependency placement và graph Rust; chưa kết luận mọi đường mutation đều từ chối cycle. |
| RCORE-09.C06 | Identity subelement qua topology change | Một phần | Link API có element-reference support trong [PropertyLinks.h](../../../../../src/App/PropertyLinks.h); không bảo đảm mọi OCCT edit giữ nghĩa của edge. | Phải có unresolved policy; cần fixture đổi số/thứ tự edges. |
| RCORE-09.C07 | Cold FCStd restore | API | `Save/Restore`, `afterRestore` trong [Document.cpp](../../../../../src/App/Document.cpp). | `onDocumentRestored` có ở các native History classes; chưa chạy restore lần này. |
| RCORE-09.C08 | Theo dõi placement hierarchy | Một phần | Native document properties/links và notification là nền. | `PlacementSources` hidden và refresh hook trong [HistoryFeature.h](../../../Gui/HistoryFeature.h), [SurfaceHistory.h](../../../Gui/SurfaceHistory.h). |
| RCORE-09.C09 | Sửa child/xóa parent có policy | Một phần | Host cho feature xử lý lost link/execute failure; không áp một policy Rhino chung. | `detachObjectHistory`, `breakHistoryForEdit`, `onLostLinkToObject` có slice; cần audit từng command gọi. |
| RCORE-09.C10 | Schema command version/units/frame chung | Chưa thấy trong phạm vi rà | Rà App links/transactions không thấy schema OM9/Rhino ở lớp host. | Các field hiện phân tán theo family; cần schema version và migration. |
| RCORE-09.C11 | Toàn bộ Matrix Builder/Styles History | Chưa kiểm chứng | Graph FreeCAD không phải bằng chứng phục dựng builder độc quyền. | Join/Surface/Circle chưa chứng minh mọi builder, Style hoặc plugin. |

## Đọc source và giới hạn kết luận

[HistoryFeature.h](../../../Gui/HistoryFeature.h) khai báo `HistoryJoin::Parents`, `Tolerance`, `Recorded`.
[CircleHistory.h](../../../Gui/CircleHistory.h) dùng `PropertyLinkSubGlobal`, `PathFraction`, `Radius`.
[SurfaceHistory.h](../../../Gui/SurfaceHistory.h) lưu nguồn, serialized options, ID và command.
Các hook restore/Undo cho thấy chủ ý xử lý lifecycle; sự hiện diện hook chưa chứng minh kết quả.
[HistoryController.cpp](../../../Gui/HistoryController.cpp) có Record/Update toggle, Clear và Join session.
Join đang khai báo tolerance riêng trong controller; không đổi ngầm sang default toàn document.
[Spec History hiện có](../01-core/om9-history-001-history-workflow.md) tiếp tục là hợp đồng feature.
Rà này không chứng nhận mọi mutation command đã gọi `breakHistoryForEdit` đúng lúc.

## Kế hoạch adapter Rust trước

Rust sở hữu record schema, validation, command capability, dirty propagation và diagnostic state.
Rust giữ ID/generation, snapshots tham số và cancellation token; không giữ pointer DocumentObject.
Native bridge C++ chỉ resolve App links, snapshot geometry/placement, gọi OCCT và commit host.
`App::Document` vẫn là owner native của objects/properties; Rust không giải phóng object host.
ABI truyền buffer/struct có ownership rõ; panic/exception đổi thành error, không vượt ABI.
Recompute native gọi Rust để lấy kế hoạch family rồi chạy solver tương ứng trên dữ liệu hợp lệ.
Worker chỉ nhận snapshot bất biến hoặc kernel copy đã chứng minh thread safety.
Kết quả về GUI thread phải kiểm document, generation, revision và capability trước transaction.
Migration Join/Surface/Circle cần baseline hành vi; không viết lại ngưỡng và defaults đã nghiệm thu.
Python chỉ làm registration/API bắt buộc và fixture; không thêm graph business logic vào Python.
Phụ thuộc: RCORE-01 representation, RCORE-02 units/tolerance, RCORE-03 frame, RCORE-06 identity.
Phụ thuộc tiếp: RCORE-07 solver contracts và [RCORE-10](10-persistence-3dm-clipboard.md) serialization.

## Fixtures nghiệm thu bắt buộc — chưa chạy tại baseline audit nguồn

| Fixture | Thao tác | Expected invariant |
|---|---|---|
| RCORE-09.T01 — chain hai thế hệ | Curve → surface → derived output; đổi một pole nguồn. | Dirty lan đúng hai thế hệ, mỗi output dùng revision mới; IDs child giữ nguyên. |
| RCORE-09.T02 — placement lồng | Dịch parent group 10 mm và xoay 90°, recompute. | Child cập nhật world geometry đúng một lần, không nhân transform hai lần. |
| RCORE-09.T03 — topology đổi | Tách cạnh nguồn khiến EdgeN cũ khác nghĩa. | Resolve cùng entity có chứng cứ hoặc unresolved; không nối âm thầm sang edge khác. |
| RCORE-09.T04 — Record/Update | Tắt Record tạo A; bật Record tắt Update tạo B rồi sửa parent. | A snapshot; B dirty với geometry cũ được đánh dấu; manual update tạo kết quả mới đúng. |
| RCORE-09.T05 — child lock/clear | Khóa child, thử sửa geometry; Clear rồi sửa. | Khóa chặn mutation; Clear theo policy OM9 giữ shape và Undo phục hồi links. |
| RCORE-09.T06 — parent delete/restore | Xóa parent, Undo, Redo. | Mỗi state đúng policy family; links không trỏ tới object đã chết, không rò callbacks. |
| RCORE-09.T07 — cycle và lỗi solver | Thêm cạnh A→B→A; gây loft invalid. | Cycle bị từ chối nguyên tử; lỗi solver chỉ rõ node, downstream không clean giả. |
| RCORE-09.T08 — cold persistence | Save FCStd, đóng tiến trình, mở khi OM9 chưa active. | Schema/IDs/units/options phục hồi; recompute đúng hoặc báo module thiếu rõ ràng. |
| RCORE-09.T09 — stale job/cancel | Sửa parent/đóng document khi worker chạy. | Output cũ bị bỏ, không commit/undo entry muộn hoặc dereference pointer hết hạn. |
| RCORE-09.T10 — replay version/typed data | Lưu record version cũ, thiếu ObjRef hoặc một output không cập nhật được. | Không gọi generic replay rồi báo success; migration/failure theo family, output và record giữ nhất quán. |
| RCORE-09.T11 — Update Off policy | Sửa parent khi Update Off rồi bật lại và gọi manual update. | Kiểm policy dirty giữ lại của OM9; phân biệt với nhánh Rhino xóa pending queue khi Update tắt. |
| RCORE-09.T12 — metadata ngoài record | Round-trip family cần cả HistoryRecord và object metadata, rồi bỏ riêng một phần. | Replay chỉ bật khi cả schema, references và metadata cần thiết hợp lệ; Count record không đủ. |

## Unknowns và điều kiện đóng

Chưa có runtime evidence của graph chéo family tại các SHA/working tree nêu trên.
Chưa xác minh toàn bộ subelement naming qua mọi OCCT operation và import conversion.
Chưa chốt migration version chung, policy Undo qua file reload hoặc external dependency restore.
Semantics builder/Style phải lấy từ nguồn đúng feature; không suy từ History Rhino tổng quát.
Chỉ đóng capability sau fixture có geometry/state assertions, runtime hash và nhật ký lỗi/cancel.
Kiểm tra link/source của tài liệu không thay thế bất kỳ fixture RCORE-09.T01–RCORE-09.T12 nào.
