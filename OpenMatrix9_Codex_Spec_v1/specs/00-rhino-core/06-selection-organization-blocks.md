---
id: RCORE-06
title: Selection, layers, groups, block instances và external references
priority: P0/P1/P2
evidence_level: source_inspected
runtime_validation: not_run
evidence_scope: initial_source_audit_2026-10-09
date: 2026-10-09
---

# RCORE-06 — Selection và tổ chức mô hình

## Cập nhật triển khai P0 — 2026-10-09

Các bảng source bên dưới ghi baseline audit ban đầu. Đợt P0 này tăng kiểm identity
ở phạm vi cụ thể: native History scheduler dùng document object ID với lookup
tạm thay address; retained import dùng namespace UUID riêng, phát hiện duplicate,
missing và cyclic dependency giữa records/components. Đây không phải stable
handle/generation registry chung cho selection hoặc block editing.

Layer/lock và native block adapters hiện có giữ supported slice riêng; chưa có
thay đổi nào ở chương này chứng minh policy xuyên mọi family. Native repeated
import/Undo/Redo/FCStd và mixed geometry fixtures đang nghiệm thu; tham chiếu
[báo cáo P0](../../../docs/validation/2026-10-09-rhino-core-p0.md), không tự đóng
fixtures selection vì import thành công.

**P0 còn thiếu:** unified whole/subelement selection snapshots có revision/
generation; intent edit/reference và layer visibility/lock/SnapToLocked thống
nhất; output active-layer policy xuyên command; selection revalidation sau
source/view change. Block editing/definition manager và Worksession vẫn thuộc
P1/P2, không được gộp thành P0 đã hoàn tất.

## Phạm vi và bằng chứng

P0: selection/identity, visibility, lock và layer output xuyên command.
P1: group, Block/Insert/BlockEdit/BlockManager/Explode.
P2: Worksession/external reference. Liên quan `OM9-SELECT`, `OM9-ORGANIZE`,
`OM9-INFO-012`, `OM9-LAYER-001`, `OM9-UTIL-012/015`, `OM9-TOP11-005`.
Không tạo command catalog mới, không nâng trạng thái vì App::Link có sẵn.

Nguồn Rhino: User's Guide Rhino 5 PDF 33–38 / in 25–30 và PDF 97–98 / in 89–90;
tham chiếu [audit][audit]. Rhino phân biệt group selection, block definition dùng
chung và instance transform; locked layer vẫn cho snap; Worksession attachment
không edit trực tiếp nhưng dùng làm input cho một số creation commands được.
Checkout đọc ngày 2026-10-09: FreeCAD HEAD
`21d36cfa1eb110a1d0667050ff31706298805bbd`, OM9 HEAD
`52e887ab706dcd5d80e23979fb1f1b62d8d869b0` với thay đổi người dùng còn trong tree.
Đây là baseline source inspection ban đầu, khi đó chưa chạy native/Rhino/FCStd acceptance; evidence mới được tách ở phần cập nhật.

## Selection input/output và state

1. Request xác định intent: edit target, read-only reference, whole object,
   face/edge/vertex/grip hoặc instance component. Filter gồm representation,
   cardinality, visibility, lock, document scope và command phase.
2. Result là tập có thứ tự ổn định của `(document ID, object ID, subelement,
   instance path, revision, world transform, selection role)`; display label
   không làm identity. `EdgeN` có thể tồn tại nhưng trỏ geometry khác sau recompute.
3. Preselection được validate lại lúc start; postselection dùng cùng filter.
   Add/remove/clear/invert/ambiguity đều cập nhật highlight và state atomically.
   Hover preselection không thêm thành viên hoặc tạo document Undo.
   Tập đã chọn trước command trong nghĩa Rhino khác hover `setPreselect` của
   FreeCAD; adapter phải đọc selected set và phase, không dùng hover làm selection.
4. Theo PDF, click/Shift-add/Ctrl-remove/Ctrl+Shift-subobject cần mapping tương
   ứng OM9; không thay shortcut text field hoặc navigation ngoài active pick.
   Các modifier khi geometry tool/grips đang active cần hợp đồng riêng.
5. Window trái→phải yêu cầu object hoàn toàn nằm trong miền chọn; crossing
   phải→trái nhận object nằm trong hoặc giao miền. Filter/type/hierarchy vẫn áp.
   Bbox chỉ là broad phase; không dùng tâm bbox thay điều kiện fully enclosed.
6. Các lệnh select theo name/layer/color/type/group cần quy tắc label trùng,
   hidden/locked và type mapping RCORE-01. Surface không đồng nghĩa mọi Part::Feature.
7. Grips/CV/mesh vertex/face/edge giữ identity riêng, không giả thành document
   object độc lập. Recompute làm subelement unresolved thì báo, không nối EdgeN ngẫu nhiên.
8. Session giữ selection snapshot và policy restore-on-cancel; document selection
   có thể đổi do UI khác nhưng commit phải revalidate intent/revision của request.
   Esc/close/deactivate tháo gate/callback thuộc session, không làm mất gate khác.

## Layer, visibility và editability

- Layer schema tối thiểu: persistent ID, name/parent, order, visible, locked,
  color/material inheritance, current flag và document version.
  Object assignment theo ID; đổi tên/reorder không đổi membership.
- Effective visibility xét object, layer và hierarchy/instance path.
  Effective editability xét lock object/layer/parent và external-reference state.
  Snap/read-reference permission là trường khác, không suy từ selectable/editable.
- Locked layer: không chọn như edit target; có thể snap khi visible theo Rhino
  PDF 97. Command đọc reference phải nêu rõ cho phép; unlock không đổi geometry.
  `locked=True` khi khai báo property FreeCAD không tự chứng minh Rhino layer lock.
- New output mặc định vào current layer; semantic layer của feature chỉ dùng
  khi spec đó yêu cầu và ghi rõ override. Current-layer ID được chụp/revalidate;
  layer bị xóa/lock giữa preview và commit không được âm thầm rơi về layer khác.
- Layer rename/delete/move/copy cần conflict và migration policy; delete layer
  có object phải nêu delete/move/cancel. Không để orphan layer ID sau Undo/reload.
- Material/color “by layer”, “by object”, “by parent/instance” là modes explicit;
  display override không đồng nghĩa đổi source material trong shared definition.

## Group, block và Worksession là ba hợp đồng khác

Group là tập để chọn/thao tác cùng nhau; không tự tạo shared geometry definition.
Group/Ungroup/AddToGroup/RemoveFromGroup/SetGroupName/SelGroup cần membership ID,
nested-group/cycle policy, add/remove selection semantics và Undo persistence.
PDF mô tả đổi hai group về cùng tên có thể hợp nhóm; mapping này cần kiểm riêng
vì label FreeCAD không phải unique group key và không nên silently merge geometry.

Block definition sở hữu member geometry trong definition-local frame; instance
sở hữu definition ID, placement/scale, attribute overrides và layer assignment.
Nested instance phải resolve transform có thứ tự, kể cả reflection/nonuniform scale.
Backend không hỗ trợ transform/representation thì reject hoặc explicit conversion,
không làm phẳng rồi gọi đó là giữ cấu trúc.

| Operation | Input/output và ownership bắt buộc |
|---|---|
| Block | Selected editable objects + base point/name → definition + instance; policy giữ/xóa source explicit, một transaction |
| Insert | Definition hoặc file resolve units + transform → instance; source definition không bị copy/edit ngoài policy |
| BlockEdit | Definition ID/version → edit session tạm; commit sửa definition cập nhật mọi instance; cancel không đổi definition |
| BlockManager | Liệt kê definitions, counts, nesting, source file; export/update/delete phải xác định đúng dependency closure |
| Explode | Một instance → owned independent members đúng world placement/attributes; instance khác và definition còn dùng giữ nguyên |
| Missing/cycle | Definition thiếu, external file mất, cycle hoặc depth budget vượt → lỗi/placeholder rõ, không dùng geometry stale như hợp lệ |

Worksession quản lý external attachments, active editable file, read-only sources,
refresh, path resolution và missing-file state. Refresh cập nhật snapshot/revision,
invalidate selection/snap/History liên quan; không ghi ngược source ngoài scope.
Project Manager hoặc external `App::Link` chỉ cung cấp một phần cơ chế;
chưa chứng minh toàn bộ Worksession file ownership/refresh semantics.

## Ma trận FreeCAD và OM9

Nhãn `UI+API`/`API` là nền tảng thấy trong source; mọi hàng vẫn cần runtime fixtures.

| Capability ID | Năng lực | FreeCAD | Bằng chứng / giới hạn | OM9 hiện tại / phần còn lại |
|---|---|---|---|---|
| RCORE-06.C01 | Object/subelement selection | UI+API | [Selection][fc-selection] `addSelection`, `setPreselect`, `SelectionObject`, resolve modes | [OM9 Command][om9-command] dùng Selection keys; cần identity/revision contract toàn family |
| RCORE-06.C02 | Filter/gate theo phase | API | `SelectionGate`, `SelectionGateFilterExternal`, `addSelectionGate/rmvSelectionGate` | Cần adapter owns/restores gate và edit-vs-reference role |
| RCORE-06.C03 | Window/crossing | Một phần | [BoxSelection][fc-box] `applyBoxSelection`, CENTER/INTERSECT; CENTER có nhánh tâm bbox | Chưa equivalent fully enclosed Rhino; cần exact fixture và policy riêng |
| RCORE-06.C04 | Visible/selectable | UI+API | [ViewProviderGeometryObject][fc-selectable] `Selectable`; [ViewProviderDocumentObject][fc-visible] `Visibility` | Không suy selectable=False thành locked-but-snappable |
| RCORE-06.C05 | Layer membership/style/UI | UI+API | [Layer][fc-layer] `Group` PropertyLinkListHidden; [Layer UI][fc-layer-ui] manager; [view layer][fc-layer-view] overrides | Cần persistent layer ID/mapping nhập-xuất và per-command assignment |
| RCORE-06.C06 | Current-layer creation | Một phần | `ViewProviderLayer.activate` gọi Draft_AutoGroup làm active layer | [Circle record][circle-record] ghi active-layer mapping chưa implement; không áp AutoGroup mặc nhiên cho OM9 |
| RCORE-06.C07 | Layer lock còn snap được | Chưa thấy trong phạm vi rà | Layer source đã đọc không chứng minh Rhino lock; Draft snap loại Selectable=False | Cần lock policy Rust, gate edit và reference permission tách biệt |
| RCORE-06.C08 | Group/nested membership | API | [GroupExtension][fc-group] `Group`, `addObject/removeObject`, `isChildOf` | Cần semantics select-as-one, Ungroup và name collision Rhino; group không là block |
| RCORE-06.C09 | Shared instances/transforms | UI+API | [App::Link][fc-link] LinkedObject/LinkPlacement/ScaleVector; [CommandLink][fc-link-ui] tạo/chọn link | Nền phù hợp để khảo sát block mapping, chưa là BlockEdit/Manager hoàn chỉnh |
| RCORE-06.C10 | External object references | API | App::Link dùng PropertyXLink, có liên kết sang document khác | Cần Worksession active-file/read-only/refresh/missing-file contract độc lập |
| RCORE-06.C11 | Block authoring workflow | Chưa kiểm chứng | Link UI/API không tự chứng minh Block/Insert/Edit/Manager Rhino semantics | Chưa thấy equivalent đầy đủ trong OM9 Command và exchange files rà; cần implementation riêng |
| RCORE-06.C12 | Explode embedded/nested blocks | Một phần | Link là cơ sở instance; không đồng nhất với file-exchange explode | [ThreeDmExplode][om9-explode] decode instance references, depth/cycle/flatten; không là preservation roundtrip |

## Các phát hiện source phải giữ khi triển khai

`applyBoxSelection` chọn CENTER khi kéo trái→phải; leaf whole-object branch có
đường chấp nhận bbox center. Đây là khác biệt thuật toán cụ thể với fully enclosed,
không phải suy đoán từ tên menu. Subelement branch có projection/line checks riêng.
Không dùng đổi icon hoặc đổi tên Box Selection để tuyên bố đạt Rhino window selection.

`App::Link` có shared target, nested resolution, LinkTransform và ScaleVector;
những cơ chế này đáng dùng lại. Cần quyết định definition container, ID, source units,
attribute inheritance và edit propagation để trở thành OM9 block workflow.
`ThreeDmExplode` có metadata layer visibility/lock và flattened nested state;
flatten đúng vị trí vẫn mất shared-definition semantics, phải công bố đúng loại output.
`Circle` record còn ghi root output và layer inheritance chưa nhận nghiệm thu.

## Rust-first adapter, cancel và lỗi

Rust sở hữu selection policy, type filters, layer/group/definition registry,
effective permissions, transform plan, cycle checks và edit-session state.
C++ tối thiểu nối `Gui::Selection`, ViewProvider, App::Link/Group và transactions;
native shape extraction/transform dùng backend RCORE-01. Python Draft layer chỉ
là API bridge nếu cần, không đặt thêm registry/behavior mới vào Python.
Rust giữ IDs/generations và owned snapshots; FreeCAD sở hữu document objects/links.
Commit revalidate definition/source revision, apply mutation+metadata một transaction.
Cancel BlockEdit/Explode preview không sinh members hoặc sửa shared source;
failure abort toàn transaction. Undo/Redo/reload phục hồi memberships, IDs và overrides.
Phụ thuộc RCORE-01 types; RCORE-02 units; RCORE-03 transforms; RCORE-04 session;
RCORE-05 reference snap; RCORE-09 topology/dependency; RCORE-10 exchange; RCORE-12 display.

## Chi tiết bổ sung từ code decompile

Các chi tiết dưới đây vẫn ở mức `source_inspected`, chưa kiểm chứng runtime.
“Thân managed” là nhánh xử lý, tham số hoặc giá trị gán thấy trực tiếp;
mô tả SDK là hợp đồng trong chú thích đi kèm code. Phần bridge native bổ sung
xác nhận được một số nhánh tại ranh giới API, nhưng các lời gọi vào SDK bên
dưới vẫn cần kiểm chứng riêng. Giữ nguyên đánh giá FreeCAD và phạm vi native
OM9 đã nghiệm thu ở các phần trên.

| Hành vi và dữ liệu | Giới hạn và yêu cầu tích hợp |
|---|---|
| SDK mô tả valid preselection được trả ngay; không có valid preselection thì cho postselect. ignoreUnacceptable=true cho bỏ phần không hợp lệ khi có phần hợp lệ, false buộc postselect nếu có phần không hợp lệ. DisablePreSelect() thực tế truyền (false,true). | Constructor giao native; không có literal xác nhận ignoreUnacceptable hoặc deselect-before-post default. Policy OM9 không tự bỏ input là lựa chọn rõ của spec, không phải hành vi bắt buộc của mọi getter Rhino. |
| SDK mô tả subobject/reference được phép, group không tự mở rộng và ưu tiên phần tử trên cùng khi có nhiều cấp hợp lệ; đây là những cờ độc lập. Managed Get() dùng min=max=1; helper GetMultipleObjects dùng (1,0). SDK phân biệt max=0 chờ Enter, -1 dừng khi đủ min, max dương chặn lượt chọn vượt giới hạn. | Default các cờ lấy từ chú thích SDK, cardinality enforcement là native. Không suy group đã có thì mọi command chọn cả group, reference selectable thành editable, hoặc UI box selection dùng cùng thuật toán FreeCAD. |
| Getter trả được nguồn preselected và có cờ riêng giữ danh sách khi vào get tiếp theo, giữ selected state khi thoát với kết quả khác Object. SDK mô tả mặc định xóa list lúc vào và unselect lúc thoát không phải Object. | Các cờ/state đều ở native. Cancel/option/retry phải có policy per command; không coi restore-on-cancel OM9 là default Rhino cho mọi command. |
| IsSelectable() thực tế không bật bốn bypass selection/grips/layer-lock/layer-visibility. Select(on,sync) mặc định persistent=true và không bypass grips/lock/visibility; highlight có cờ riêng. SDK phân biệt trạng thái selected/persistent/subobject và reference read-only trên reference layer. | Kết quả kiểm lock/hierarchy nằm native. Selection, highlight và edit permission không là một bool; không được map Rhino preselection thành FreeCAD hover hoặc cho reference edit chỉ vì getter chọn được. |
| Indexer layer ngoài phạm vi thực tế rơi về current layer trong managed. SDK mô tả current layer không hidden/locked/deleted, đặt current đưa mode về Normal; persistent visibility/locking quyết định trạng thái con sau khi parent bật/unlock trở lại. | Bộ máy parent/child và current-layer enforcement là native. OM9 giữ lỗi rõ cho stale layer ID theo contract hiện có, không bắt chước fallback index thành lỗi âm thầm. Cần lưu explicit/unset persistent state thay vì chỉ effective bool. |
| Managed InsertionPoint biến đổi origin bằng InstanceXform; definition members và references có API riêng. Explode trả ba mảng cùng số phần tử: objects, attributes và transforms; SDK mô tả cờ recurse nested hoặc giữ InstanceObject trong pieces. | Wrapper không thấy thêm/xóa document objects; không được coi API trả pieces là toàn bộ command Explode đã commit. Thuật toán transform/inheritance và quyền sở hữu native còn phải kiểm. App::Link chỉ là nền cho mapping definition/instance. |
| Ví dụ Matrix cụ thể bật GroupSelect, tắt SubObjectSelect, giữ list/selection qua lượt get và không deselect trước postselect. Vòng lặp sau Option hoặc sau lần trả ObjectsWerePreselected sẽ tắt preselect rồi get tiếp. | Chỉ xác nhận policy của MeshRepairCommand trong export; không nhân rộng thành default Rhino/Matrix hoặc suy command đã chạy thành công. Không port thuật toán thương mại vào repository. |

Thân bridge native bổ sung xác nhận routing selection nhưng chưa phục hồi
bộ máy lựa chọn bên trong SDK. Constructor GetObject vẫn gọi constructor SDK;
các default preselect/group/subobject tiếp tục giữ mức bằng chứng đã ghi ở trên.

| Nhánh bridge native đã đọc | Giới hạn và yêu cầu tích hợp |
|---|---|
| Preselect bridge kiểm getter khác null rồi chuyển hai cờ enabled và ignore-unacceptable. Bộ chuyển bool có nhánh riêng cho postselect, trạng thái preselected, group, subobject, reference, highlight, clear-on-entry và unselect-on-exit. | Cờ không bị gộp thành một quyền selectable. Không suy constructor default, việc bỏ input không hợp lệ hoặc filter/lock behavior từ switch dispatch; các thuật toán này vẫn gọi SDK. |
| GetObjects gắn callback filter, xóa floating-point exception status rồi chuyển min/max tới SDK. Đường trả về bình thường xóa callback slot; getter null cho kết quả số không. | Chưa xác nhận enforcement cardinality, ý nghĩa mọi mã lỗi, thread/re-entry safety hoặc cleanup khi exception. Callback đã được xóa trên đường thường không thay hợp đồng owner/generation và phục hồi selection của OM9. |

Hệ quả cho selection/UI: request cần cấu hình preselect enabled, xử lý input không
hợp lệ, postselect enabled, group/subobject/reference permissions, cardinality và
giữ selection/list qua option/retry. Không có một default áp chung cho mọi command.
Command đọc reference khác command sửa geometry; snap còn phải xét setting `SnapToLocked`
độc lập với quyền chọn và chỉnh sửa. Layer cần tách trạng thái được người dùng đặt với trạng thái hiệu lực do
parent; current layer và stale ID cần revalidate theo policy OM9. Block pieces,
transforms và attributes phải đi cùng nhau tới commit plan; API query không thay
thế workflow BlockEdit/Explode hay chứng minh native FreeCAD parity.

| Fixture bổ sung — chưa chạy | Expected invariant / câu hỏi cần ghi nhận |
|---|---|
| RCORE-06.T11 | Tập preselect gồm đúng/sai type: so ignoreUnacceptable true/false và pre/post enabled; ghi getter result và tập thực sự thao tác, UI không đổi tập âm thầm ngoài policy OM9 |
| RCORE-06.T12 | Group/subobject/reference flags và max 0/-1/dương, có crossing vượt max: đối chiếu cardinality; reference được chọn không tự có quyền edit |
| RCORE-06.T13 | Preselection → option → get tiếp → Esc, bật/tắt clear-on-entry/unselect-on-exit: kiểm list, selected state và highlight riêng; fixture Matrix chỉ là ví dụ policy, không default toàn host |
| RCORE-06.T14 | Parent layer off/on, lock/unlock với persistent state true/false/unset; đổi current và index stale: không rơi output sang current layer do fallback index không chủ ý |
| RCORE-06.T15 | Query Explode nested true/false: số lượng ba mảng đồng bộ, transform/attributes đúng; chỉ command commit mới tạo độc lập/xóa instance theo policy, query không là bằng chứng đã hoàn tất Explode |

## Fixtures nghiệm thu cần chạy

Tại baseline audit nguồn tất cả fixtures **chưa chạy**; các cập nhật mới phải đánh riêng selection, geometry, metadata và reload.

| Fixture | Expected invariant |
|---|---|
| RCORE-06.T01 | Vật có tâm trong nhưng cắt biên window: window không chọn, crossing chọn; bbox rỗng giao window không tự đủ geometry hit |
| RCORE-06.T02 | Pre/postselect, Shift-add/Ctrl-remove, ambiguous edge/face/instance path: cùng filter và identity; Esc tháo gate/session đúng owner |
| RCORE-06.T03 | Locked visible layer và hidden parent: edit bị chặn, snap/reference đúng policy; không dùng Selectable để vô tình cấm mọi snap |
| RCORE-06.T04 | Tạo Circle trên current layer, đổi tên/reorder, layer bị xóa giữa preview/commit: ID bền hoặc lỗi rõ, không rơi layer mặc định im lặng |
| RCORE-06.T05 | Nested group/add/remove/ungroup/label trùng: không xóa geometry hay đổi placement; Undo/reload giữ membership đúng policy |
| RCORE-06.T06 | Hai instances cùng definition ở transform khác: sửa definition cập nhật cả hai, cancel BlockEdit không thay bất kỳ instance |
| RCORE-06.T07 | Nested reflection/nonuniform scale: world geometry đúng transform một lần; unsupported representation reject trước mutation |
| RCORE-06.T08 | Explode một instance: independent members đúng world/attributes, instance còn lại không đổi; Undo/reload khôi phục definition links |
| RCORE-06.T09 | Missing definition/file hoặc cycle: lỗi/placeholder rõ, không stale geometry thành success; refresh reference invalidate selection đúng revision |
| RCORE-06.T10 | Worksession reference dùng làm creation input nhưng Move bị chặn; refresh/close không ghi source, không giữ dangling handles |

## Điểm chưa chốt

Cần help/fixtures Rhino 5 cho nested block materials, layer inheritance, name collisions,
linked versus embedded definitions, lock propagation và external-file ownership.
Rà giới hạn Selection/BoxSelection, Group/Link, Draft layers/snapper và OM9 files dẫn
dưới đây; không kết luận mọi FreeCAD add-on đều thiếu block hoặc reference tools.

[audit]: ../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md
[fc-selection]: ../../../../../src/Gui/Selection/Selection.h
[fc-box]: ../../../../../src/Gui/Selection/BoxSelection.cpp
[fc-selectable]: ../../../../../src/Gui/ViewProviderGeometryObject.h
[fc-visible]: ../../../../../src/Gui/ViewProviderDocumentObject.h
[fc-layer]: ../../../../../src/Mod/Draft/draftobjects/layer.py
[fc-layer-ui]: ../../../../../src/Mod/Draft/draftguitools/gui_layers.py
[fc-layer-view]: ../../../../../src/Mod/Draft/draftviewproviders/view_layer.py
[fc-group]: ../../../../../src/App/GroupExtension.h
[fc-link]: ../../../../../src/App/Link.h
[fc-link-ui]: ../../../../../src/Gui/CommandLink.cpp
[om9-command]: ../../../Gui/Command.cpp
[om9-explode]: ../../../Gui/ThreeDmExplode.cpp
[circle-record]: ../../../docs/features/OM9-CURVE-005.md
