---
id: RCORE-10
title: FCStd, trao đổi 3DM và clipboard
priority: P0
review_date: '2026-10-09'
evidence_level: source_inspected
runtime_validation: not_run
evidence_scope: initial_source_audit_2026-10-09
freecad_commit: 21d36cfa1eb110a1d0667050ff31706298805bbd
openmatrix9_commit: 52e887ab706dcd5d80e23979fb1f1b62d8d869b0
---

# RCORE-10 — FCStd, trao đổi 3DM và clipboard

## Cập nhật triển khai P0 — 2026-10-09

Metadata/source claims phía dưới là baseline audit ban đầu. Rust nay kiểm bounded
export manifest trước Qt/native conversion: tuple size, finite values, color/
boolean types, tolerance và mesh indices; retained/mixed payload không bị bỏ
âm thầm. Retained namespaces phải là UUID canonical; records/components phải
không trùng identity, resolve đủ references và không có cycle. Kiểm graph chạy
trước preservation staging và trước retained replay sau restore.

Native History/I/O fixture đã pass **7 checks**, gồm từ chối malformed export
trước khi chạm destination; mixed CAD/Mesh, repeated namespace, import Undo/Redo
đã qua 56 checks, cold retained FCStd qua 28 checks và stale-state restore qua
2 checks; mọi process exit 0. Kết quả/hash theo slice xem
[báo cáo P0](../../../docs/validation/2026-10-09-rhino-core-p0.md); không đóng
RCORE-10.T01–T12 chỉ từ parser reread hoặc version 5.

**P0 còn thiếu:** current-state merged preservation writer và table remapping;
ma trận type/field read/edit/retain/export đầy đủ; Rhino binary geometry clipboard
theo từng chiều; toàn pipeline allocation/cancellation budgets, disk-full và
late-result fixtures. Python exporter đã stage/replace; low-level native writer
vẫn cần caller cấp staging path. Rhino 5 open/edit/save/reimport vẫn phải chạy
trong ứng dụng đích, không suy từ FreeCAD clipboard hoặc openNURBS.

## Phạm vi và mức bằng chứng

Chi tiết hóa [nhóm nền Rhino](README.md).
Liên quan `OM9-FILE-001..010`, `OM9-FILE-012`, `OM9-OBJECT`, `OM9-ORGANIZE`, `OM9-HISTORY`.
`OM9-FILE-012` đã xuất hiện trong source; không đặt một ID khác cho Import3dm/Export3dm.
Tài liệu này không tự đăng ký feature thứ 608 hoặc thay implementation status catalog.
Mã nguồn FreeCAD có persistence FCStd và clipboard object; Part có STEP/IGES exchange.
3DM được thấy trong adapter openNURBS riêng của OM9, không gán thành chức năng Part tiêu chuẩn.
Standard FreeCAD Ctrl+C/V không được gọi là tương thích định dạng clipboard nhị phân Rhino.
Kết luận baseline là source-inspected ngày 2026-10-09 trên working tree có thay đổi, khi đó chưa runtime-validated.
Audit nguồn ban đầu không chạy FreeCAD, Rhino 5, export/import fixture hay kiểm tra clipboard OS; cập nhật triển khai ở trên ghi evidence mới riêng.

## Bốn khả năng cần khai báo riêng

| Khả năng | Bằng chứng cần có | Điều không thể suy ra |
|---|---|---|
| Read/inventory | Archive parser đọc thành công và kê đủ loại/field cần kiểm tra. | Chưa chứng minh objects editable hoặc được hiển thị. |
| Display/edit | Host representation và editor có supported operation cụ thể. | Hình nhìn được chưa chứng minh giữ nguyên NURBS/trim/metadata. |
| Retain source | Snapshot/records gốc có identity và kiểm hash khi lưu. | Giữ bản gốc chưa chứng minh xuất được trạng thái đã sửa. |
| Export current state | Writer merge/edit/delete tạo archive hợp lệ và kiểm lại trong ứng dụng đích. | Version 5/50 hoặc parser đọc lại chưa chứng minh Rhino 5 mở/edit đúng. |

Mỗi row support matrix phải ghi cả bốn khả năng cho point, cloud, curve, surface, BRep,
extrusion, mesh, block/instance, layer, annotation, material/light và unknown userdata.
Mỗi field units, UUID, visibility, lock, color, material và parent relation cũng cần row rõ.
Không gộp read thành full support; không đổi CAD thành mesh để né converter thất bại.
CAD có display mesh vẫn ở nhánh CAD; mesh thực và cloud là representations độc lập.

## Hợp đồng input/output, units và placement

Import nhận path hoặc clipboard payload, source format/version và mode `geometry`/`preserve`.
Selection export/copy phải đóng băng identity và revision tại thời điểm bắt đầu thao tác.
Export lấy geometry hiện tại sau edit, transform và delete; không hồi sinh snapshot nguyên bản.
Whole object/subelement selection được hỗ trợ phải được công bố; không âm thầm copy toàn object.
Mỗi lần paste/import tạo namespace identity riêng; UUID nguồn là provenance, không thay host ID.
Hai lần import cùng source phải không đè object lần trước hoặc nhầm retained records.
World placement được resolve từ hierarchy một lần; geometry và metadata phải dùng cùng frame.
Units source khai báo rõ; unitless/custom cần hệ số hoặc lựa chọn có nghĩa trước mutation.
Scale geometry, tolerance và tham số độ dài theo cùng policy; display precision không là scale.
Archive giữ lại source unit system ngay cả khi host geometry chuyển về mm.
Output geometry-only phải liệt kê dữ liệu bị bỏ; preservation export cần merge writer riêng.
Snapshot-only export phải có tên/diễn giải đúng; không masquerade thành current-state export.

Block geometry mode được phép flatten theo supported contract OM9 hiện có.
Definition member chỉ nhập qua instance đang dùng; không thêm bản gốc vào top level.
Nested transform ghép parent × child; thử cả reflection và nonuniform scale.
Color từ parent, visibility và lock phải resolve theo hierarchy khi flatten.
Flatten không giữ definition/instance structure; người dùng cần BlockEdit phải biết giới hạn này.
Preservation mode lưu nguồn chưa có editor không được gắn nhãn object có thể sửa đầy đủ.

## Lỗi, transaction, clipboard và lifecycle

Parse, unit resolution, dependency resolution và conversion phải xong trước mutation document.
Một paste/import thành công tạo một transaction; Undo gỡ toàn bộ lần đó, Redo phục hồi đúng IDs.
Cancel/lỗi không để retained objects, group rỗng, file staging hoặc undo entry một phần.
Missing definition/member, cycle, singular transform, corrupt archive phải nêu record lỗi.
Unknown userdata được coi là dữ liệu; file/clipboard không tự chạy macro hoặc plugin payload.
Writer ghi file tạm, kiểm kết quả rồi thay output theo policy; lỗi không phá file đích đang tốt.
Retained source cần hash, path/embedded policy và missing/relink behavior khi cold reload.
Clipboard có capability theo chiều Rhino→OM9, OM9→Rhino và OM9→OM9 riêng.
Qt MIME FreeCAD và Windows clipboard format Rhino là hai protocol phải khảo sát độc lập.
Ctrl+C/V ở viewport/tree gọi geometry khi capability có; text editor giữ hành vi văn bản.
Clipboard không được giữ pointer tới object có thể bị xóa sau Copy; payload có ownership rõ.
Paste kiểm budget và bounds trước allocation; worker không sửa App document hoặc Qt widgets.
Đóng document/hủy thao tác loại kết quả muộn theo document generation và revision.

## Chi tiết bổ sung từ code decompile

Các chi tiết dưới đây được đọc từ code decompile; chưa chạy binary, UI hoặc fixture runtime.
Phân biệt logic managed, wrapper gọi native và phần triển khai native khi xác định phạm vi đã hiểu.
Các phát hiện này không nâng trạng thái triển khai hoặc đóng nghiệm thu FreeCAD/OM9.

| Hành vi và hệ quả hợp đồng | Giới hạn |
|---|---|
| File3dmSettings tách model/page units và tolerance; RhinoDoc tách setter UnitSystem với AdjustModelUnitSystem/AdjustPageUnitSystem có cờ scale. Native settings writer gọi writer units/tolerances, còn writer đó ghi riêng precision và dữ liệu units. Import/export phải kê cả metadata units và thao tác scale; đổi metadata không được tự coi là đã rescale geometry. | Các setter/Adjust là native API boundary, chưa thực nghiệm hậu quả scale hoặc custom units. Nhánh precision ngoài 0..20 ghi giá trị thay thế 3 là xử lý serialization, không chứng minh precision UI mặc định. |
| File3dmWriteOptions constructor đặt Version=5 và bật SaveRenderMeshes, SaveAnalysisMeshes, SaveUserData; File3dm.Write chuyển các option xuống writer native. FileWriteOptions của RhinoDoc là API khác, có IncludeHistory, WriteGeometryOnly, WriteUserData, selected-only và transform. Default constructor managed không phải default hộp thoại Save/Export hoặc mọi caller Matrix. | Constructor FileWriteOptions lấy state từ native; chưa chứng minh default native, thực thi writer, toàn bộ version downgrade hay output mở được trong Rhino 5. |
| File3dm layer/material tables là dữ liệu độc lập, có insert/remove và lookup index trả wrapper theo UUID của archive. Native ON_Layer.Write và ON_Material.Write có thân serializer, gồm identity, giá trị thuộc tính và dữ liệu rendering liên quan. Rebuild archive chỉ từ geometry không đủ để kết luận giữ layers/materials. | Không gán tên cho mọi offset không rõ trong decompile; chưa kiểm round-trip từng field, remap index sau xóa hoặc nội dung shader/plugin riêng. |
| File3dmHistoryRecordTable trong export này chỉ expose Dump, Count và Clear. GemvisionUserDataBase.Read/Write lưu ArchivableDictionary qua archive. Có thể nhận diện/lưu record hoặc userdata mà vẫn chưa có bộ thực thi History; đếm record không phải acceptance replay. | Không kết luận toàn bộ native API thiếu editor/replay; đây là phạm vi wrapper đã đọc. Không chạy hoặc deserialize payload plugin trong lần audit. |

Các yêu cầu về schema state/identity, worker, transaction và xử lý lỗi của OM9 vẫn là quyết định
thiết kế, trừ hành vi gốc được nêu rõ ở trên. Không suy defaults toàn ứng dụng từ một caller.

## FreeCAD đã có đến đâu; OM9 cần bổ sung gì

Các nhãn phản ánh source, không phải chứng nhận chạy ứng dụng hoặc mọi add-on của FreeCAD.

| Capability | Yêu cầu | FreeCAD | Bằng chứng/phạm vi | OM9 riêng |
|---|---|---|---|---|
| RCORE-10.C01 | FCStd model persistence | UI+API | [Document.cpp](../../../../../src/App/Document.cpp): `saveAs`, `Save`, `Restore`, `Document.xml`; [CommandDoc.cpp](../../../../../src/Gui/CommandDoc.cpp): `StdCmdSave/StdCmdSaveAs`. | Cần native property serialization cho schema OM9 và cold reload fixture. |
| RCORE-10.C02 | GUI state/object appearance persistence | API | [Gui Document.cpp](../../../../../src/Gui/Document.cpp): `Save`, `Restore`, `exportObjects`, `importObjects`. | Appearance/selection transient phải phân biệt field cần lưu. |
| RCORE-10.C03 | Object clipboard FreeCAD | UI+API | [CommandDoc.cpp](../../../../../src/Gui/CommandDoc.cpp): `StdCmdCopy/StdCmdPaste`; [MainWindow.cpp](../../../../../src/Gui/MainWindow.cpp): MIME/export/import. | Có thể tái dùng host transaction; không chứng minh Rhino binary clipboard. |
| RCORE-10.C04 | Clipboard Rhino hai chiều | Chưa thấy trong phạm vi rà | Rà CmdCopy/Paste và MainWindow MIME chỉ thấy protocol host, URL/image. | Rà `Gui`/`rust/src` chỉ thấy clipboard text ở CommandConsole; chưa thấy geometry protocol Rhino trong phạm vi đó. |
| RCORE-10.C05 | STEP/IGES làm exchange native | API | [ImportStep.cpp](../../../../../src/Mod/Part/App/ImportStep.cpp), [ImportIges.cpp](../../../../../src/Mod/Part/App/ImportIges.cpp): OCCT readers. | Không dùng sự có mặt của các reader này để đánh dấu 3DM native. |
| RCORE-10.C06 | Đọc/ghi geometry 3DM | Chưa thấy trong phạm vi rà | Không thấy openNURBS ở các entry Part exchange vừa rà. | [ThreeDmArchive.cpp](../../../Gui/ThreeDmArchive.cpp): `readArchive`, `writeArchive5`; actual runtime chưa chạy. |
| RCORE-10.C07 | Units 3DM và block flatten | Một phần | Host placement/shape có nền; không tự hiểu units/definition 3DM. | Reader xử lý scale mm, nested blocks, cycle/missing/singular errors; cần fixture số. |
| RCORE-10.C08 | Inventory/immutable source retention | Chưa kiểm chứng | FCStd có khả năng lưu properties/files; chưa kiểm preservation 3DM trong host chuẩn. | [ThreeDmPython.cpp](../../../Gui/ThreeDmPython.cpp): `prepare3dmArchive`; [core_3dm_archive.rs](../../../rust/src/core_3dm_archive.rs) có identity/export policy. |
| RCORE-10.C09 | Current-state merged preservation export | Chưa kiểm chứng | Không suy từ native save. | [ThreeDm.py](../../../ThreeDm.py) có chặn legacy export khi retained/preserved data có thể bị bỏ; chưa chứng minh merged writer đầy đủ. |
| RCORE-10.C10 | Geometry-only export có thông báo mất dữ liệu | Một phần | Host exchange là theo format riêng; cần contract OM9. | `export_file` có `geometry_only` và warning; `writeArchive5` từ chối retained record. |
| RCORE-10.C11 | Atomic paste/import + Undo | API | App document transaction và object import/export. | Python chuẩn bị trước native commit; cần kiểm failure từng phase và Redo. |
| RCORE-10.C12 | Rhino 5 mở/edit + FCStd cold reload | Chưa kiểm chứng | Source/API không chứng minh binary đang chạy. | Bằng chứng checkout khác không chuyển thành pass cho checkout này. |

## Phát hiện source cụ thể

[CoreThreeDm.cpp](../../../Gui/CoreThreeDm.cpp) định tuyến dialog Import và Export Selected Rhino 5.
[core_3dm.rs](../../../rust/src/core_3dm.rs) giữ command policy của exact ID `OM9-FILE-012`.
[ThreeDmArchive.cpp](../../../Gui/ThreeDmArchive.cpp) có giới hạn 512 MiB, depth 64 và 1.000.000 items.
Đó là giới hạn code hiện có; không gọi là giới hạn Rhino hoặc ngân sách toàn pipeline đã nghiệm thu.
Reader geometry mode nhận point/curve/BRep/mesh/surface và từ chối loại chưa chuyển đổi.
Reader preservation mode có nhánh `retained` và inventory RenderLight, không đồng nghĩa editor light.
Writer tạo archive version 5 rồi đọc lại/count check; đây là kiểm nội bộ, chưa phải ứng dụng Rhino.
[ThreeDm.py](../../../ThreeDm.py) kiểm selection và source-retained content trước geometry export.
Nhánh chuẩn bị snapshot trong [ThreeDmPython.cpp](../../../Gui/ThreeDmPython.cpp) so archive hash.
[MainWindow.cpp](../../../../../src/Gui/MainWindow.cpp) export object vào buffer/file MIME của FreeCAD.
Việc clipboard host gọi cùng `exportObjects` không tạo chứng cứ cho MIME/format Rhino.
[Audit nguồn](../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md) ghi các record nghiệm thu ở workspace khác chưa có tại checkout này.

## Kế hoạch adapter Rust trước

Rust sở hữu capability matrix, identity namespace, unit policy, source/current-state tracking và jobs.
Rust lập kế hoạch import/export gồm retained/edited/deleted/new records và báo mất dữ liệu.
Rust quản budget CPU/RAM, cancellation, hash validation, path resolution và output staging policy.
Ngân sách worker OM9 mục tiêu là `max(1, floor(0.60 × N))`, giảm theo task độc lập/RAM.
Đây là số worker, không là cam kết chừa chính xác 40% CPU/RAM; không đặt trần cố định 4.
C++ bridge cần vì openNURBS/OCCT/App/Qt là native APIs; bridge không sở hữu policy nghiệp vụ mới.
Buffers có size/owner/lifetime rõ; object native không được giải phóng từ allocator khác.
Không cho exception/panic vượt ABI; convert sang record-scoped error trước rollback/cleanup.
Python hiện là host/bootstrap API path; migration chuyển planning/state về Rust, giữ entry tương thích.
Phụ thuộc RCORE-01 representation, RCORE-02 units, RCORE-06 block/layer identity và RCORE-09 History.
Annotation/render resources dùng [RCORE-11](11-annotation-layout-print.md), [RCORE-12](12-display-render-resources.md).

## Fixtures nghiệm thu bắt buộc — chưa chạy tại baseline audit nguồn

| Fixture | Thao tác | Expected invariant |
|---|---|---|
| RCORE-10.T01 — mixed geometry | Import/export/reimport point, rational arc, face có lỗ, closed BRep và mesh. | Đúng type/count, trim holes/topology/bounds; CAD giữ CAD, không chỉ ảnh giống. |
| RCORE-10.T02 — units | Cùng geometry source mm và inch; thêm unitless/custom. | Cùng kích thước vật lý; unknown units dừng trước mutation; metadata scale nhất quán. |
| RCORE-10.T03 — nested blocks | Hai instance dùng chung definition; nested reflection và scale không đều. | Geometry đúng transform, không có definition duplicate; flatten được báo rõ. |
| RCORE-10.T04 — bad archive | Corrupt file, missing member, cycle, singular transform. | Báo object/dependency lỗi, count/undo stack/document hash không đổi. |
| RCORE-10.T05 — edited source | Import preserve; sửa A, xóa B, tạo C rồi export. | Writer có capability phải xuất A mới/C và bỏ B; nếu chưa hỗ trợ thì từ chối rõ, không hồi sinh source. |
| RCORE-10.T06 — paste hai chiều | Rhino 5→OM9 và OM9→Rhino 5 trên supported types. | Đúng selection/current shape/placement/units; ghi kết quả riêng từng chiều và format. |
| RCORE-10.T07 — clipboard focus/Undo | Copy geometry ở tree rồi paste; Ctrl+V trong text field. | Paste hình học một Undo/Redo; text field chỉ nhận text, không tạo document object. |
| RCORE-10.T08 — cold FCStd | Save, tắt tiến trình, bỏ source ngoài theo policy rồi mở lại. | Geometry/retained references đúng; missing resource được báo, không dùng pointer hoặc temp file cũ. |
| RCORE-10.T09 — output/cancel | Disk-full/permission failure; hủy trước commit và khi worker trả về. | File đích tốt còn nguyên, staging dọn, không partial objects/late commits. |
| RCORE-10.T10 — ứng dụng đích | Mở/edit/save kết quả thực trong Rhino 5 rồi nhập lại. | Đếm/metadata/topology/bounds/deviation đạt ngưỡng fixture; archive version riêng không đủ. |
| RCORE-10.T11 — write-option separation | Lưu cùng scene theo explicit option giữ/bỏ meshes, userdata và History. | Báo đúng phần dữ liệu đã giữ/bỏ; geometry/readability không chứng minh History replay hoặc default UI. |
| RCORE-10.T12 — tables và unit metadata | Đổi layer/material table cùng model/page units; xuất rồi nhập lại. | Identity/remap và metadata từng field đúng; scale geometry chỉ xảy ra theo policy explicit, không dựa vào setter metadata. |

## Unknowns và điều kiện đóng

Chưa xác minh runtime binaries, linked openNURBS version và clipboard format trong Rhino 5 tại máy này.
Chưa có matrix đầy đủ theo type × field × read/edit/retain/export cho working tree hiện tại.
Unknown userdata, annotation, render tables và external blocks cần writer/editor capability riêng.
Adaptive area/volume integration và tolerance regression cần dữ liệu số, không chỉ `Shape.Volume`.
Các thresholds fixture phải xác lập trước chạy; không nới tolerance hoặc đổi CAD sang mesh cho test qua.
Chỉ đóng từng chiều/type sau runtime evidence có hash source, binary, fixture và output.
