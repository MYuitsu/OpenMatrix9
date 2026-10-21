# Document scene entity contract (OM9-L2-01-03)

Nghiệp vụ đã duyệt cho module `OM9-L2-01-03` (Be Document Scene Entities, M01), owner duyệt
2026-10-17. Đây là hợp đồng hành vi của tài liệu/cảnh; evidence kỹ thuật nằm ở
`implementation/design/M01/OM9-L2-01-03/api.md` (§7) và `implementation/evidence/M01/OM9-L2-01-03/`.

Phạm vi: identity của đối tượng/layer/definition/feature/group, envelope entity, layer records,
membership, luật hiển thị/khoá, group, block definition/instance, extension slot, hợp đồng selection,
thông báo thay đổi, snapshot/restore và provenance nhập. Không gồm UI quản lý Layers.

## 1. Định danh bền vững

- `ObjectId`, `LayerId`, `DefinitionId`, `FeatureId`, `GroupId` là UUID có kiểu và là định danh bền
  vững của đối tượng trong tài liệu; không dùng chỉ số mảng hay vị trí danh sách làm định danh.
- ID đã nghỉ (đối tượng bị xoá, group bị xoá) không bao giờ được cấp lại: chèn lại một ID đã dùng trả
  `Invalid("allocated object UUID" | "allocated group UUID" | "allocated feature UUID")`.
- Undo/Redo khôi phục đúng định danh cũ, không cấp định danh mới; save/reopen giữ nguyên ID.
- Một `SourceStamp` (object + generation + geometry/attributes revision) chỉ resolve khi còn đúng là
  bản hiện tại; tem cũ trả `Stale("source stamp")`.

## 2. Envelope entity có kiểu

- Sáu loại hình học được hỗ trợ: `Point`, `Curve`, `Surface`, `Mesh`, `Cloud`, `Instance`. Kind được
  giữ nguyên qua lưu/mở.
- Kind không dựng được trong bản này (`Brep`/`BRep`/`B-rep`) trả `Unsupported("brep geometry")`; kind
  lạ trả `Unsupported("geometry kind")`. Không bao giờ hạ cấp âm thầm (BRep không được đọc thành Mesh).
- `GeometryHandle` (object + generation + geometry_revision) là tem hiển thị; handle cũ/treo trả
  `Stale("geometry handle")` chứ không trỏ vào đối tượng đang sống.
- `display_attributes()` chỉ đọc: trả name/color cùng các cờ hiệu lực (visible/selectable/editable)
  sau chuỗi layer, không sửa bản ghi.

## 3. Layer, membership và luật hiển thị

- Layer có ID bền vững; đổi tên/đổi màu/đổi cờ giữ nguyên ID. `parent` phải tồn tại (`Missing("layer")`),
  `layer_order` liệt kê mỗi layer đúng một lần, `current_layer` phải nằm trong order.
- Entity thuộc tối đa một layer; đổi layer giữ nguyên EntityId và chỉ tăng `attributes_revision`.
- Xoá layer để lại membership treo là lỗi tường minh `Missing("entity layer")`; không tự remap sang
  layer hiện hành.
- Layer ẩn: đối tượng không được vẽ và không pick. Layer khoá: vẫn vẽ nhưng không chọn/không sửa; mọi
  ghi vào nguồn thuộc layer khoá trả `Permission("source edit")` và không thay đổi tài liệu.

## 4. Group là lớp phủ thành viên

- `Group { id, name, members, revision }` chỉ là thành viên: group/ungroup không đổi hình học, không
  đổi ID và không gộp định danh hình học.
- Thành viên phải tồn tại và không lặp trong cùng một group; group rỗng bị từ chối; một đối tượng có
  thể nằm trong nhiều group.
- Xoá đối tượng thì nó bị gỡ khỏi mọi group; group mất thành viên cuối cùng tự bị xoá; group còn thành
  viên khác giữ nguyên GroupId và tăng revision.

## 5. Block definition và instance

- `Definition` sở hữu topology (members); mỗi occurrence chỉ giữ `placement` và layer riêng, không sao
  chép topology.
- `Document::member_transform(root, path)` = placement của occurrence hợp với placement các member
  trên đường dẫn; occurrence này đổi placement không làm occurrence khác đổi theo.
- `Mutation::DefinitionInsert` tạo definition + các occurrence trong một giao dịch: một Undo xoá cả
  definition lẫn mọi occurrence vừa tạo.
- Cycle (definition chứa chính nó hoặc vòng lặp lồng nhau) bị từ chối `Invalid("definition cycle")`
  và không để lại ghi một phần; tham chiếu definition không tồn tại trả `Missing("instance definition")`.

## 6. Extension slot (metadata trang sức)

- Slot có tên namespace: `om9.gem`, `om9.rail`, `om9.head`, `om9.cutter` là slot đã biết; slot lạ dạng
  `vendor.futurething` hợp lệ và được giữ nguyên. Tên trần như `Gem` không phải slot →
  `Invalid("extension slot")`.
- `Extension { version, payload }`: `version >= 1`; `payload` là text JSON **object** giữ nguyên từng
  ký tự (field lạ trong payload không bị mất). `version == 0` → `Invalid("extension version")`;
  payload không phải object → `Invalid("extension payload")`; object rỗng →
  `Invalid("empty extension payload")`.
- Đặt/xoá một slot chỉ tăng `attributes_revision`, giữ nguyên ObjectId/generation/geometry_revision.
- `unknown_extension_slots()` **báo** các slot không thuộc bốn slot đã biết; không drop im lặng.

## 7. Hợp đồng selection

- Selection là định danh có kiểu (`ObjectId` + tuỳ chọn `SubshapeRef`), không bao giờ là chỉ số UI.
- `validate()` trả `Stale("selection document")` khi khác document/owner session và
  `Stale("selection epoch")` khi tài liệu đã publish lần khác; `revision == 0` là
  `Invalid("selection revision")`.
- Đối tượng bị xoá (kể cả do Undo) thì entry tương ứng bị prune trước khi dùng lại; Redo khôi phục đối
  tượng nhưng không hồi sinh selection cũ.
- Danh sách publish cho UI chỉ resolve được với đúng thứ tự đã publish; danh sách bị rút ngắn/đổi thứ
  tự trả `Stale("selection index")` thay vì chọn sang đối tượng lân cận.

## 8. Thông báo thay đổi

- Mỗi lần publish sinh **tối đa một** `ChangeBatch`, thứ tự tất định: current layer → layer order →
  layer → entity → group → feature; trong mỗi nhóm, xoá đứng trước tạo/sửa; tài liệu không đổi thì
  không có batch (không yêu cầu vẽ lại).
- `ChangeCause` phân biệt `Edit`/`Undo`/`Redo`/`Replace`/`Restore`; Undo hạ revision nên chuỗi batch là
  liên tục (`next.from_revision == previous.to_revision`), không phải revision luôn tăng.
- `DocumentObserver` chỉ nhận `&ChangeBatch`, không có handle Controller/coordinator, nên callback
  không thể mutation tái nhập (đảm bảo ở tầng kiểu).
- Observer panic hoặc trả lỗi bị **gỡ** sau đúng một diagnostic; các observer còn lại vẫn nhận batch và
  tài liệu vẫn đã publish xong.
- Ghi batch bật từ observer đầu tiên: controller không có observer không giữ baseline (không nhân đôi
  bộ nhớ tài liệu lớn).

## 9. Snapshot và restore

- Snapshot là toàn bộ tài liệu; restore tái tạo đúng ID và bit hình học đã chụp.
- Từ chối tường minh: snapshot của tài liệu khác → `Invalid("snapshot document")`; payload bị sửa →
  `Invalid("snapshot checksum")`; version lạ → `Unsupported("OM9 schema version; no implicit migration")`.
- Restore là nguyên tử: hoặc tài liệu + scene được dựng lại, hoặc không đổi gì. Nó reset Undo (snapshot
  là mốc mới), giữ workspace và đường dẫn tệp, và trả `Busy` khi đang có lệnh chạy.

## 10. Provenance nhập nguồn

- Đối tượng chỉ được coi là "nhập từ nguồn" khi chuỗi bằng chứng đầy đủ: bản ghi namespace trỏ tới đối
  tượng, retained source tồn tại và hash resource nhúng khớp. Thiếu bất kỳ mắt nào → `None`, không bao
  giờ gắn nhãn .3dm native nửa vời.
- `provenance()` trả source path/name, sha256, archive version, native id/index, family và disposition.
- `import_summary()` đếm nguồn/namespace/bản ghi/bound/retained-only/deleted/unsupported;
  `namespace_summary(id)` cho một namespace và trả `Missing("import namespace")` nếu không có.

## 11. Lưu trữ

- `groups`, `features`, `extensions` là khoá **additive** trong schema 4 với legacy-key guard: không
  bump schema, không migration; tài liệu không mang dữ liệu mới encode y hệt trước đây.
- `Document.tolerance` là khoá additive optional; tài liệu không khai báo policy thì **không** ghi khoá,
  và reader phải nhận `Missing` thay vì một mặc định bịa ra.

## 12. Quyết định đã chốt (owner, 2026-10-17)

- **ADR-03-5 — GIỮ.** `FeatureId`, `FeatureKind`, `Document.features`, `Mutation::FeatureInsert` và
  `Feature.outputs` giữ nguyên như đã giao. Lý do: oracle L3-001 chứng minh được trong chính module này,
  và M04/M05 gọi `FeatureId` là định danh bền vững cùng nhóm với `EntityId/GemId/LayerId`.
  Phân chia sở hữu: **định danh + output refs nằm trong `Document.features`; đánh giá nằm trong
  `HistoryGraph`**. Khi M04 xuất hiện, `Recipe` phải tham chiếu `FeatureId` mà nó đánh giá; luật chống
  lệch (outputs của feature == outputs của recipe) thêm khi có producer thật đầu tiên.
- **BRep/OCCT**: vẫn `Unsupported("brep geometry")` cho tới khi `OM9-L2-01-05` và quyết định M00-OCCT
  được duyệt; không tính là đã hoàn tất.

## 13. Đơn vị, dung sai và tham số có kiểu (OM9-L2-01-08, owner duyệt 2026-10-17)

Đánh giá lại hành vi sau khi triển khai (W1) để giữ đúng cái đáng giữ:

- **Canonical mm.** Độ dài trong lõi là milimét và chỉ tồn tại dưới kiểu `LengthMm`; không truyền `f64`
  vô nhãn như chiều dài qua biên module. Text `"2.5"` dùng đơn vị tài liệu, `"2.5mm"`/`"0.1 in"` mang
  đơn vị riêng; `Unit::parse` là **bảng đơn vị duy nhất** của OM9 — tầng command dùng lại nó, không có
  bảng thứ hai.
- **Số trần trong lệnh theo đơn vị tài liệu** (hành vi Matrix): `Length=2.5` trong tài liệu inch là
  63,5 mm; hậu tố tường minh luôn thắng. Ở tầng **giá trị bền vững** thì ngược lại: `Value` phải có
  chiều (`Value::length/angle/count/ratio`), một `Value::Number` trần không thoả
  `OptionDomain::Number` và trả `Invalid("missing unit")`.
- **Góc lưu bằng độ**; radian chỉ là kết quả chuyển đổi tường minh (`to_radians`/`to_degrees`) và wire
  form có tag (`{"deg":..}` / `{"rad":..}`) nên không đọc chéo được. Tham số/UI dùng độ; radian vẫn là
  đơn vị toán trong tolerance/replay/geometry.
- **Dung sai là policy của tài liệu, không có mặc định ẩn.** `Document.tolerance: Option<Tolerance>`;
  đọc bằng `effective_tolerance()` và nhận `Missing("document tolerance")` nếu tài liệu không khai báo.
  Ghi đè bằng `Mutation::Tolerance`: một Undo, tăng `revision`, **không** đổi `context_revision` (nên
  tem/stamp và recipe Clean vẫn hợp lệ). `Tolerance::try_new` từ chối theo đúng tên trường;
  `admit_feature` từ chối hình học ≤ dung sai tuyệt đối.
  *Dung sai mặc định cho tài liệu mới là quyết định của template/app, không phải của lõi — chưa chốt.*
- **Toạ độ có kiểu.** `LocalPoint` chỉ thành `WorldPoint` qua một `LocalFrame` tường minh
  (`Missing("coordinate frame")` nếu không có); `CPlane` là frame đó. Ngữ pháp `x,y[,z]` và nhãn lỗi
  giữ nguyên.
- **Miền giá trị.** `OptionDomain::Number` được `validate()` trước mọi giá trị (biên/bước sai bị từ
  chối), chỉ nhận đúng chiều, kiểm tra bước (`Invalid("numeric option step")`), count phải nguyên và ≥ 0.
