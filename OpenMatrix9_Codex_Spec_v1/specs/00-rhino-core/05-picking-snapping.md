---
id: RCORE-05
title: Pick resolver, object snaps và constraints
priority: P0
evidence_level: source_inspected
runtime_validation: not_run
evidence_scope: initial_source_audit_2026-10-09
date: 2026-10-09
---

# RCORE-05 — Pick, snapping và constraints

## Cập nhật triển khai P0 — 2026-10-09

Metadata và rà nguồn phía dưới là baseline ban đầu. Rust nay tách persistent
preferences khỏi one-shot/suspension: `Osnap Once/Only/Suspend/Resume/Clear`;
accepted point mới consume one-shot, hover/options/error không consume. Cancel
phiên active dọn transient và giữ persistent preferences. Curve, Solid, Distance,
Angle và PictureFrame đã nối consumer; malformed CPlane bị reject trước native
point resolution/geometry trên các đường đã phủ.

Fixtures `core_snap_lifecycle_smoke` và `core_invalid_cplane_tools_smoke` đã qua
lần lượt 10 và 14 checks native, process exit 0; xem phạm vi và bằng chứng tại
[báo cáo P0](../../../docs/validation/2026-10-09-rhino-core-p0.md). State lifecycle
tests không chứng minh thuật toán geometric snapping hoặc mọi T01–T10 đã pass.

**P0 còn thiếu:** resolver chung có source/subelement provenance và revision,
hover/click revalidation đồng nhất; advanced tangent/perpendicular/special snaps,
Grid/SmartTrack/Elevator và SnapToLocked; policy ranking/visibility/lock và
constraint composition xuyên mọi command/viewport.

## Phạm vi và mức bằng chứng

Dịch vụ dùng chung cho `OM9-PICK`, `OM9-SNAP-001..016`, `OM9-INFO-013`
và mọi command nhận point. Không thay solver Tangent riêng của Circle bằng một
snap marker, không coi 16 spec/icon là 16 snap đã chạy được.
Nguồn Rhino: User's Guide Rhino 5 PDF 45–51 / in 37–43 và PDF 97 / in 89;
marker biểu diễn điểm sẽ được nhận, persistent/one-shot snaps có vòng đời khác,
grid snap là lưới tưởng tượng vô hạn, Elevator theo CPlane Z, SmartTrack tồn tại
trong command. Phần chưa có help riêng tiếp tục ghi chưa chốt; xem [audit][audit].

Đối chiếu source ngày 2026-10-09, FreeCAD HEAD
`21d36cfa1eb110a1d0667050ff31706298805bbd`, OM9 HEAD
`52e887ab706dcd5d80e23979fb1f1b62d8d869b0` tại working tree có thay đổi người dùng.
Audit nguồn ban đầu không chạy runtime/native test; cập nhật triển khai ở trên tách biệt, source có hàm vẫn không bằng evidence pass.

## Input, candidate và output

1. `PickRequest` có screen position/logical pixel ratio, viewport camera revision,
   CPlane snapshot, previous accepted point, document revision, active modes,
   phase constraints, excluded preview IDs và visibility/reference policy.
2. `SnapCandidate` phải có document/object/subelement identity, instance path,
   world point, parameter/UV nếu cần, source revision, loại snap, distance screen,
   residual geometry, origin là native geometry hay transient smart point.
   Không chỉ gửi vị trí marker hoặc world XYZ rời nguồn.
3. Enumeration candidates và kiểm geometry phải giữ miền hữu hạn của edge/trim face.
   Bounded endpoint, underlying infinite curve và projected point là loại khác nhau.
   Không snap vào display mesh thay CAD khi contract yêu cầu edge/face chính xác.
4. Resolver trả Accepted(point, provenance, satisfied constraints), Ambiguous,
   NoCandidate, IncompatibleConstraints hoặc StaleReference. Chỉ Accepted được commit.
   Free pick không có object snap cũng phải mang frame/constraint provenance.
5. Preview và click dùng cùng resolver, tolerance và phase snapshot.
   Nếu source/view đổi giữa hover và click, resolve lại rồi cập nhật marker;
   không nhận điểm khác marker mà không phản hồi rõ.
6. Aperture pixel là điều kiện chọn candidate; absolute/angular tolerance là điều
   kiện geometry. Zoom/DPI có thể đổi candidate được chọn, không nới độ đúng hình học.
   Candidate tie có thứ tự ổn định và cơ chế cycle/ambiguity theo policy đã nghiệm thu.

## Vòng đời modes và constraint composition

Các policy lifecycle, ranking và giải quyết xung đột chưa được nguồn xác nhận
là hợp đồng mục tiêu OM9 cần nghiệm thu, không phải defaults Rhino đã phục hồi.

- Persistent set thuộc preference/context đã công bố; one-shot override chỉ áp
  một accepted pick. Invalid click không tiêu thụ override; cancel dọn override.
  Suspend giữ set để restore, Clear xóa set, exclusive enable tắt các mode còn lại.
- Không dùng một priority tuyệt đối cho mọi tổ hợp. Resolver phải báo constraint
  nào thắng, constraint nào dùng đồng thời, constraint nào bị vô hiệu theo policy.
  Không sửa point sau snap rồi vẫn gắn nhãn End/Tangent dù residual đã sai.
- Ortho cần base point và angle basis đã khai báo; base thường là previous point
  nhưng có thể đổi qua From/SetBasePoint. Shift là modifier tạm theo
  RCORE-04/focus. Circle size và các family có policy riêng phải được giữ explicit.
- Planar quyết định mặt phẳng điểm sau; Project biến đổi candidate 3D lên CPlane.
  Candidate gốc và điểm chiếu được lưu riêng; không gọi projected intersection là
  giao thật 3D. Cần help/fixture cho precedence chính xác từng tổ hợp Rhino 5.
- Distance khóa bán kính quanh base point; one-shot angle khóa hướng/nhánh góc.
  Kết hợp hai constraint dùng cùng units/frame; click chọn nhánh còn mơ hồ.
  Conflict với object snap phải báo hoặc theo policy explicit, không âm thầm lệch.
- Tangent/Perpendicular tới curve khác “from curve” với base point bị ràng buộc.
  Kiểm tangent vector/normal, finite parameter, residual và nhánh nghiệm; không
  dùng projection trực quan thay nghiệm. Curve degenerate có lỗi recoverable.
- Intersection phân biệt thật 3D, apparent theo view/CPlane và extension vô hạn.
  Surface snap phải xét trim loops/hole và orientation; underlying surface vẫn
  có điểm trong lỗ không có nghĩa face có điểm đó.
- Grid hiển thị hữu hạn và grid snap là hai policy độc lập. Mục tiêu từ PDF:
  grid snap tiếp tục ngoài vùng lưới nhìn thấy; ẩn grid không tự tắt mode snap.
  Grid spacing/angle/origin theo RCORE-02/03, không đổi geometry khi chỉ đổi display.
- Elevator có phase pick chân rồi pick/nhập độ cao theo CPlane Z; số âm được phép
  nếu feature cho phép. Dữ liệu chân+height+frame được giữ cho Undo/preview.
  Chuyển view để thấy chiều cao không làm reset chân hoặc biến Z thành world Z.
- SmartTrack giữ smart points/lines với source revision và session generation;
  có thể snap giao/vuông góc/trực tiếp theo supported modes. Đây là transient data,
  không tạo document object, undo entry, selectable node hoặc export geometry.
- Knot và special snap phải có contract theo loại knot/curve domain và repeated
  knot; không suy từ Knot Builder trang sức hoặc từ icon giống nhau.

## FreeCAD, Draft và OM9: bảng đối chiếu

`UI+API` ở bảng là UI/code path có trong source, chưa kiểm runtime trên checkout.

| Capability ID | Năng lực | FreeCAD | Bằng chứng / giới hạn | OM9 hiện tại / adapter cần |
|---|---|---|---|---|
| RCORE-05.C01 | Snapper chung/marker | UI+API | [Draft Snapper][fc-snap] `snap`, `constrain`, `getPoint`; snap tuple real/marker/visual | [CoreSnaps][om9-snap] đang có resolver native riêng; cần protocol chung preview/commit |
| RCORE-05.C02 | End/Mid/Near/Center | UI+API | `snapToEndpoints/Midpoint/Near/Center` và toolbar modes | OM9 Rust [State][om9-state] chỉ công bố master/End/Mid/Point tại đây |
| RCORE-05.C03 | Perpendicular/face | UI+API | `snapToPerpendicular`, `snapToPerpendicularFace` | Cần residual/trim/from-vs-to contract, không coi Draft tương thích trọn Rhino |
| RCORE-05.C04 | Intersection thật/biểu kiến | Một phần | `snapToIntersection` có nhánh projected WorkingPlane cho line/segment | Cần loại candidate explicit và test lệch Z; chưa có general OM9 path tại CoreSnaps |
| RCORE-05.C05 | Knot snap | API | `snapToBSplineKnots` trong mode Special dùng `getKnots`/`value` | Có nền tảng thật, không kết luận thiếu vì khác tên; bounded trim/domain cần xác minh |
| RCORE-05.C06 | Grid snap độc lập hiển thị | Một phần | `snapToGrid` kiểm `self.grid.Visible` trước mode Grid | Không đáp ứng ngay policy ẩn grid vẫn snap; cần resolver Rust riêng/adapter tách state |
| RCORE-05.C07 | Ortho/angle/extensions | UI+API | `snapToOrtho`, `snapToAngles`, `snapToPolar`, `snapToExtensions` | Cần bảng tổ hợp với modes OM9, distance và per-command exceptions |
| RCORE-05.C08 | Tangent snap tổng quát | Chưa thấy trong phạm vi rà | Không thấy tangent mode/hàm tương đương trong Snapper đã đọc | Circle Tangent có solver riêng, không thay global tangent snap |
| RCORE-05.C09 | Persistent/one-shot/suspend/clear | Một phần | `active_snaps`, preferences và mode switches có sẵn | `State`/CoreSnaps có bitmask/toggle; chưa thấy đủ one-shot lifecycle trong hai file |
| RCORE-05.C10 | Locked nhưng vẫn snappable | Một phần | `snapToObject` bỏ `ViewObject.Selectable=False` | Không thể map lock→Selectable rồi dùng nguyên Draft snap; tách edit và reference permission |
| RCORE-05.C11 | SmartTrack/Elevator Rhino | Một phần | Draft có extension/hold/constraints; không đủ chứng minh semantics Rhino | Chưa thấy session model tương đương trong CoreSnaps; cần state/fixtures riêng |
| RCORE-05.C12 | Candidate identity/revision | Một phần | Draft `snapInfo` có object/component/parent path | Rust `Candidate` chỉ world+screen, CoreSnaps trả point; cần identity/provenance và stale checks |

## Giới hạn source cụ thể

`CoreSnaps::pick` xét object và geo-parent visibility, lấy end/mid/point candidates,
project lên screen rồi gọi Rust với aperture `8.` logical pixels.
Nó gộp các mode thành cùng mảng điểm và trả `Base::Vector3d`; tại boundary này
chưa truyền object ID, subelement, loại snap hoặc revision của candidate.
[CoreSnapGeometry][om9-geometry] có extraction native riêng, là cơ sở dùng lại,
không là bằng chứng đủ locked-layer, instance path hay all representations.
Không đánh giá hiệu năng chỉ từ vòng lặp qua object; phải đo scene/budget fixture.

Draft khác yêu cầu ở ít nhất hai điều kiện đọc được trực tiếp: grid visibility
và loại object không selectable. Thay đổi adapter phải giữ supported Draft behavior
cho Draft; không sửa toàn host chỉ để khiến OM9 có policy mong muốn.
Phạm vi negative claims là các file được dẫn, không phải toàn ecosystem FreeCAD.

## Rust-first adapter và ownership

Rust sở hữu mode lifecycle, constraint composition, ranking/ambiguity, candidate
metadata, transient SmartTrack/Elevator và spatial cache độc lập có generation.
C++ tối thiểu đọc native edge/face/placement, gọi OCCT phép cần thiết, project view
và render marker; exception thành typed error. Python Draft API chỉ dùng nếu
cần tương tác component hiện có, không thêm solver/business state mới bằng Python.
Không giữ raw pointer document/shape/view trong Rust; native query dùng handle có
lifetime cho một request hoặc owned snapshot. Worker không đọc mutable document.
Cancel/close/deactivate giải marker/callback/cache session; reply stale bị bỏ.
Mutation chỉ đến ở command commit trên GUI thread; pick service không tự tạo geometry.
Phụ thuộc RCORE-01 representations; RCORE-02 tolerances; RCORE-03 CPlane;
RCORE-04 phases; RCORE-06 visibility/lock/identity; RCORE-12 view/cache lifecycle.

## Chi tiết bổ sung từ code decompile

Các chi tiết dưới đây vẫn ở mức `source_inspected`, chưa kiểm chứng runtime.
“Thân managed” là nhánh xử lý, tham số hoặc giá trị gán thấy trực tiếp;
mô tả SDK là hợp đồng trong chú thích đi kèm code. Phần bridge native bổ sung
xác nhận được một số nhánh tại ranh giới API, nhưng các lời gọi vào SDK bên
dưới vẫn cần kiểm chứng riêng. Giữ nguyên đánh giá FreeCAD và phạm vi native
OM9 đã nghiệm thu ở các phần trên.

| Hành vi và dữ liệu | Giới hạn và yêu cầu tích hợp |
|---|---|
| SDK gắn Ortho/Planar/From với base point; From cho thay base point khi đang get. PermitOrthoSnap điều khiển cả việc tôn trọng Ortho và Planar. SDK mô tả From/constraint options phải nhập đầy đủ và không hiện trên command line; Tab là line constraint, Elevator có mode 0/1/2. | Constructor GetPoint chuyển native; default permit và cơ chế phím ở đây là mô tả SDK, chưa có replay. Không đồng nhất base point với previous accepted point hoặc biến hidden built-in options thành option đã hỗ trợ trong OM9. |
| SDK phân biệt CPlane constraint qua base point: throughBasePoint=true buộc dùng cao độ base ngay khi Planar tắt; false chỉ dịch qua base khi Planar bật. Target plane và virtual CPlane intersection là constraint riêng. ClearConstraints được mô tả gỡ explicit constraints và bật lại built-in options. | Các hàm giải đều chuyển native. Không xác nhận precedence với Project/Osnap hoặc nghiệm khi view/plane suy biến; không áp semantics này cho parser tọa độ bằng suy luận. |
| SDK mô tả distance dương cùng base point khóa khoảng cách; UnsetValue xóa constraint, 0 chặn đặt distance bằng số khi get. AcceptNumber có acceptZero riêng để phân biệt số 0 với điểm gốc trong GetPoint. | Managed chỉ chuyển tham số; chưa xác minh units, giá trị âm/NaN, grammar hay tương tác distance với osnap. Zero không thể áp một nghĩa toàn cục cho mọi phase. |
| Plane constraint có allowElevator. Curve/Surface/Brep/Mesh constraint có allowPickingPointOffObject; SDK mô tả khác biệt cursor/marker khi con trỏ rời object. Brep thêm faceIndex và wireDensity cho phạm vi face và isocurve snaps. | Cờ cho pick ngoài vùng con trỏ không chứng minh kết quả geometry được phép nằm ngoài constraint. Không thấy thuật toán trim, dung sai hay nhánh nghiệm trong wrapper; vẫn phải kiểm residual và trimmed face như contract OM9. |
| GridSnap, Ortho, Planar, Project, SnapToLocked, Osnap mode mask và hai pickbox radius là state riêng. Osnap getter/setter đảo bool native; CreateState dùng cùng phép đảo. API đọc default/current qua native chứ không chứa bộ giá trị mặc định cố định ở đây. | Không biết giá trị SnapToLocked mặc định hoặc thứ tự giải constraint. Locked-visible có thể là nguồn snap theo setting/intent, không phải cam kết luôn snap trong mọi cấu hình; không sao chép polarity native sang public enabled flag OM9. |
| Get() chuyển thành Get(false), rồi Get(false,false); cờ onMouseUp/get2DPoint được gửi native. SDK mô tả false nhận điểm lúc nhấn trái, true nhận lúc thả. | Đây là default overload managed, không chứng minh mọi command dùng overload đó. Chọn thời điểm nhận điểm phải theo phase, không mặc định chỉ xử lý mouse-up hoặc mọi click đều commit command. |

Thân bridge native bổ sung đã cho thấy các nhánh chuyển tham số và dọn state
tạm của getter. Constructor vẫn gọi constructor SDK, nên không dùng việc có
bridge để nâng các default từ chú thích thành hành vi runtime đã chứng minh.

| Nhánh bridge native đã đọc | Giới hạn và yêu cầu tích hợp |
|---|---|
| Bridge kiểm getter khác null rồi dispatch riêng quyền Ortho, From, constraint options, Tab, object snap, curve snap và target-plane constraint; Elevator mode được chuyển tiếp. Các constraint curve/surface/Brep/mesh kiểm null và chuyển cờ pick ngoài object; CPlane chuyển cờ through-base. | Chỉ xác nhận routing tại boundary. Default, ranking, trim/solver và thứ tự Ortho/Planar/Project/Osnap còn do SDK xử lý; một số call site có cảnh báo decompiler, không chuyển pseudo-code thành logic OM9 nguyên xi. |
| GetPoint phân biệt callback cho getter điểm và getter transform, chọn đường lấy điểm 3D hoặc 2D theo cờ, rồi chuyển tiếp cờ nhận điểm lúc mouse-up. | Cờ có đi qua native bridge; ý nghĩa event đầy đủ và hành vi từng command vẫn cần replay. Không đồng nhất accepted point với commit command hoặc suy mọi command dùng cùng overload. |
| Sau lời gọi SDK trả về bình thường, bridge xóa callback đang gắn, tắt display conduit tạm và có nhánh regenerate active document khi state tạm yêu cầu. | Đây là cleanup trên đường trả về thấy được, không chứng minh cleanup khi exception, document close hoặc re-entry. OM9 vẫn cần ownership/generation và teardown bảo đảm trên mọi lối thoát của session. |

Hệ quả cho input/UI: cần tách `base_point` khỏi `previous_accepted_point`, trạng thái
Ortho/Planar/Project khỏi quyền dùng chúng trong phase, và khóa distance khỏi nhận
numeric input. UI phải phân biệt một setting đang bật với một getter cho phép dùng.
`allowPickingPointOffObject` mô tả cách nhận pick/cursor, không miễn kiểm nghiệm
geometry. Chưa có bằng chứng xếp hạng osnap, precedence hay one-shot consumption
trong các wrapper này; những contract mục tiêu phía trên vẫn phải nghiệm thu.

| Fixture bổ sung — chưa chạy | Expected invariant / câu hỏi cần ghi nhận |
|---|---|
| RCORE-05.T11 | Previous point và base point khác nhau do From; bật/tắt Ortho/Planar và permit riêng: marker/accepted point dùng đúng base đã khai báo, không lấy previous ngầm |
| RCORE-05.T12 | throughBasePoint true/false × Planar on/off × base có/không, trên CPlane xoay: đo cao độ đúng nhánh SDK mô tả; Project/Osnap conflicts ghi riêng, chưa tự gán precedence |
| RCORE-05.T13 | Distance dương/UnsetValue/0 và acceptZero true/false: phân biệt khóa distance, xóa constraint, vô hiệu numeric distance, số 0 và point input; document không đổi khi chỉ thay input policy |
| RCORE-05.T14 | SnapToLocked on/off với locked-visible/hidden object, Osnap master và mode mask: giữ edit permission độc lập và ghi setting của lần chạy |
| RCORE-05.T15 | Get nhận mouse-down so với mouse-up, kéo chuột và navigation; mỗi phase nhận đúng một điểm, không suy accepted point thành commit cả command |

## Fixtures nghiệm thu cần chạy

Tại baseline audit nguồn, tất cả ca dưới đây **chưa chạy**. Mỗi slice triển khai cần expected point/residual được đo độc lập marker.

| Fixture | Expected invariant |
|---|---|
| RCORE-05.T01 | Nhiều End/Mid cùng aperture/distance: chọn ổn định/cycle rõ, commit cùng marker. Cùng hình học/logical-pixel input và quy đổi DPR đúng giữ candidate; đổi aperture/zoom có thể đổi lựa chọn nhưng không đổi tiêu chuẩn residual geometry |
| RCORE-05.T02 | Locked visible layer: không edit/select như editable, vẫn snap reference theo policy; hidden parent loại candidate |
| RCORE-05.T03 | Face có trim hole: Near/Perp không nhận điểm chỉ nằm trên underlying surface trong lỗ |
| RCORE-05.T04 | Hai line lệch Z nhưng cắt trong view: true Intersect không giả thành công; apparent mode ghi loại/điểm chiếu rõ |
| RCORE-05.T05 | Ortho+End và Project+3D End trên CPlane xoay: constraint policy công bố, marker/label/commit không mâu thuẫn |
| RCORE-05.T06 | Distance `5` và angle `30°` trên frame đã chốt: length 5, angular residual trong tolerance; impossible tangent không nhận point |
| RCORE-05.T07 | One-shot → invalid click → valid click → next pick; override chỉ tiêu thụ sau accept, persistent set khôi phục; suspend khác clear |
| RCORE-05.T08 | Ẩn grid và pick ngoài grid hữu hạn: snap theo spacing vô hạn vẫn tồn tại nếu mode bật; bật/tắt display không đổi state snap |
| RCORE-05.T09 | Elevator height âm/đổi view/Undo rồi SmartTrack; đúng CPlane Z, không document objects/undo entry tạm sau cancel |
| RCORE-05.T10 | Source sửa/xóa giữa hover và click, document close lúc query: re-resolve hoặc stale error, không dùng point cũ/handle hết hạn |

## Chưa chốt về tương thích

Cần help/fixtures Rhino 5 cho special snaps, tangent/perpendicular from/to,
seam/knot và precedence mọi tổ hợp. Decompile bổ sung contract SDK cho
Project/Planar/base-point constraints, chưa xác nhận thuật toán native hoặc runtime.
Không tự áp một algorithm/ranking của Draft như default Rhino, hoặc nhân rộng
Circle solver sang mọi curve snap khi chưa có adapter và evidence độc lập.

[audit]: ../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md
[fc-snap]: ../../../../../src/Mod/Draft/draftguitools/gui_snapper.py
[om9-snap]: ../../../Gui/CoreSnaps.cpp
[om9-geometry]: ../../../Gui/CoreSnapGeometry.cpp
[om9-state]: ../../../rust/src/core_snaps.rs
