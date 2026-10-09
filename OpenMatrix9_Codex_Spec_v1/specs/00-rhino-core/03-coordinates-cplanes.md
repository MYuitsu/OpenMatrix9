---
id: RCORE-03
title: Tọa độ, parser điểm và CPlane theo viewport
priority: P0
evidence_level: source_inspected
runtime_validation: not_run
evidence_scope: initial_source_audit_2026-10-09
date: 2026-10-09
---

# RCORE-03 — Tọa độ, parser điểm và CPlane

## Cập nhật triển khai P0 — 2026-10-09

Baseline source audit phía dưới được giữ nguyên về phạm vi; evidence triển khai
mới theo slice nằm ở [báo cáo P0](../../../docs/validation/2026-10-09-rhino-core-p0.md).
Rust có `PointInput/ResolvedPoint` và frame thuận tay phải đã validate, dùng cho
CurveSession và Box/Sphere: `x,y[,z]`, world `w`, relative `r/R/@`, world-relative
`wr`; thiếu previous point, số không hữu hạn và overflow đều reject. Input scale
được truyền vào parser, không lấy lại tùy ý khi command đang chạy.

Registry Rust/native lưu CPlane theo document/view slot, current/named planes
qua Meta save/reload; previous/next có branching nhưng chưa serialize stack.
Frame điều khiển cả origin và rotation của grid, kể cả lúc grid đang ẩn. UI Set
CPlane đã nối World XY/XZ/YZ, Previous/Next, named Save/Restore. Replay native
đã pass **162 assertions**, gồm thao tác menu bằng phím Home/Down/Right/Return,
named dialog, giữ Circle đang nhập, frame/grid và save/reload. Môi trường này
không chuyển con trỏ đúng qua QtTest, nên không dùng lần hover chuột bị lỗi làm
bằng chứng pass. Camera giữ nguyên vị trí/hướng/projection; near/far clipping
do renderer tính lại theo scene không thuộc phép so sánh view, raw snapshot
vẫn được lưu. Cold restore trong process mới pass **6 assertions**; đường dẫn
kết quả nằm trong báo cáo P0.
Tên CPlane bị giới hạn 4096 records và tổng 512 KiB UTF-8/document, kiểm tra
trước mutation; embedded NUL trong metadata bị reject, không truncate tên.

**P0 còn thiếu:** View/Curve/Surface và interactive 3Point menu; propagation
mọi family; polar/spherical/surveyor/locale đầy đủ; provenance document/view/
revision trong History. Latch vẫn theo slice hiện hữu: Circle/Rectangle chụp
frame ở điểm đầu; Line/Polyline dùng frame theo input, chưa đổi thành policy chung.

## Phạm vi và mức bằng chứng

Đây là hợp đồng nền cho `OM9-COORD`, `OM9-VIEWPORT-001`, Line/Polyline,
Circle, Rectangle, Box, Sphere và các thao tác transform nhận điểm.
Không thêm command ID, không nâng supported slice của các feature đó.
Nguồn Rhino: User's Guide Rhino 5, PDF 49–51 / trang in 41–43;
phạm vi CPlane mở rộng kế thừa [audit ngày 2026-10-09][audit] và yêu cầu gốc.
PDF xác nhận CPlane độc lập theo viewport, world cố định, cả `x,y` và
`x,y,z` là tọa độ CPlane, cùng cú pháp tương đối `rx,y`.
Đã đọc thêm [Rhino 5 Help — Entering Numbers][rhino-numbers] ngày 2026-10-09;
trang ghi Rhinoceros 5, ngày 17-Sep-2015. Cú pháp dưới đây dựa đúng trang này,
không suy từ help Rhino đời mới hoặc một URL chưa đọc được.

Đọc source tại `H:/FreeCAD-src`, FreeCAD HEAD
`21d36cfa1eb110a1d0667050ff31706298805bbd`, OM9 HEAD
`52e887ab706dcd5d80e23979fb1f1b62d8d869b0`.
Working tree có thay đổi người dùng; HEAD không phải hash mọi file đang đọc.
Các nhận định của baseline audit nguồn ban đầu là `source_inspected`; lần audit đó chưa chạy FreeCAD/Rhino hay native tests, tách với cập nhật triển khai ở trên.

## Hợp đồng input và output

1. Input parser gồm chuỗi nguyên bản, locale, expected kind của phase,
   units/tolerance context, previous point, document/view/frame identity và revision.
   Kết quả phải là tagged value, không trả một bộ ba số mất nguồn gốc.
2. Kiểu kết quả tối thiểu: absolute CPlane point, explicit world point,
   relative vector, scalar distance, one-shot angle, option token hoặc lỗi.
   Scalar chỉ là điểm khi phase có quy tắc dựng điểm từ scalar đã công bố.
3. CPlane `x,y` tương đương `(x,y,0)`; CPlane `x,y,z` giữ đủ ba thành phần.
   Với origin `O` và basis `X,Y,Z`, world point là `O+xX+yY+zZ`.
   Relative vector không cộng origin; cộng vector vào previous accepted point.
4. Grammar phải phân biệt `w` world, `r/R/@` relative CPlane và `wr` relative world
   theo bảng đã đối chiếu help bên dưới; locale và biến thể chưa có evidence giữ riêng.
   Không suy frame từ số thành phần; không coi ví dụ chưa kiểm chứng là default Rhino.
5. Parser phải từ chối NaN/Infinity, số vượt domain của feature, unit sai dimension,
   thiếu thành phần, trailing garbage và relative khi chưa có previous point.
   Lỗi giữ nguyên phase, các điểm đã nhận, option và document.
6. Units là RCORE-02: điểm và displacement đổi sang host units đúng một lần;
   angle không qua nhánh length. Precision hiển thị không làm tròn dữ liệu dựng.
7. Output cho solver gồm world point, original frame/value, frame version,
   input kind và provenance; History lưu đủ dữ liệu tái dựng theo RCORE-09.
   Snap vốn đã trả world point không bị áp CPlane transform lần nữa.

## Grammar đã đối chiếu help Rhino 5

`x,y[,z]` là ký hiệu schema; không nhập dấu ngoặc vuông. `P` là previous point.
Các ví dụ số bên dưới được tự đặt để kiểm parser; expected world là phép tính
độc lập với runtime. Nguồn xác nhận cú pháp: [Entering Numbers][rhino-numbers].

| Input | Frame / phép giải | Ví dụ |
|---|---|---|
| `x,y` hoặc `x,y,z` | CPlane; thiếu z dùng 0 | `1,2,3` |
| `wx,y,z` | World absolute; không cộng origin CPlane | `w1,2,3` |
| `rx,y[,z]`, `Rx,y[,z]`, `@x,y[,z]` | Relative CPlane; `P+xX+yY+zZ` | `r1,2,3` |
| `wrx,y[,z]` | Relative world; `P+(x,y,z)` | `wr1,2,3` |
| `rw4<45` | Help có ví dụ relative polar world | Giữ nhánh polar riêng |

Help còn có ví dụ `@w` trong dạng surveyor; chưa suy mọi hoán vị prefix đều hợp lệ.
Help cấm khoảng trắng bên trong number/angle/point; có decimal dấu chấm,
scientific notation `e/E`, fractions và length-unit suffix đổi sang model units.
Locale decimal-comma, danh sách unit aliases và mọi biến thể prefix vẫn cần fixture.
Có grammar được tài liệu xác nhận không có nghĩa OM9 đã implement hoặc runtime pass.

## CPlane, state và ownership

- `CPlaneState` gồm origin, basis thuận tay phải trực chuẩn, frame ID/version,
  viewport ID, mode và tên tùy chọn; camera orientation là state khác.
  Đổi CPlane không di chuyển object, đổi camera không mặc nhiên đổi CPlane cố định.
- Hỗ trợ phải tách theo World Top/Front/Right, Origin, 3Point, View,
  Curve, Surface, named save/restore/import và previous/next CPlane.
  Ba điểm thẳng hàng, axis zero hoặc basis suy biến bị từ chối nguyên tử.
- Curve/Surface cần quy tắc lấy tiếp tuyến/pháp tuyến, orientation, seam,
  singularity và vị trí trên miền trim; tên option chưa xác định thuật toán đó.
- Mỗi command khai báo latch tại start, first point hoặc từng phase.
  Lựa chọn OM9 đề xuất: latch construction frame tại first accepted point,
  giữ frame đó đến khi phase dựng kết thúc; trước latch dùng viewport hiện hành.
  Đây là policy cần nghiệm thu, không áp ngầm lên mọi command hiện có.
- Chuyển viewport vẫn cho pan/zoom/pick; screen ray dùng camera mới,
  construction frame dùng latch đã công bố. Prompt cho biết frame đang dùng.
  View song song CPlane không có giao hữu hạn: giữ session và cho nhập số/đổi view.
- Session Undo một điểm phục hồi frame/latch tương ứng checkpoint;
  restart/cancel xóa previous point và latch tạm theo schema command.
  CPlane undo/view undo không được tự dùng document geometry Undo.
- Named CPlane cần persistent identity độc lập label, conflict policy khi đổi tên/import,
  document scope, save/reload và mapping viewport. Không lưu pointer view trong document.
- Native host sở hữu Qt view, Coin camera, FreeCAD document và placement handles.
  Rust sở hữu frame snapshot, grammar, validation, history stack và session revision.
  Khi đóng view/document, hủy request chờ và bỏ callback trước khi handle hết lifetime.
- Placement của object lồng group/link giải riêng bằng RCORE-06.
  Local geometry → world placement và CPlane input → world là hai phép khác nhau.

## FreeCAD hiện có và khoảng tích hợp OM9

Nhãn đánh giá mô tả source/UI/API đã thấy, không phải kết quả tương thích runtime.

| Capability ID | Năng lực cần | FreeCAD | Bằng chứng / giới hạn | OM9 hiện tại / việc còn lại |
|---|---|---|---|---|
| RCORE-03.C01 | Frame local↔world | API | [WorkingPlane][fc-wp] `PlaneBase.get_global_coords/get_local_coords` dùng matrix và inverse | Có frame trong [Session][om9-curve]; cần tagged parser dùng chung |
| RCORE-03.C02 | CPlane riêng từng viewport | UI+API | `get_working_plane` lưu `PlaneGui` theo view; Auto có thể bám camera | Chưa chứng minh OM9 đồng bộ với Draft; cần nguồn state duy nhất |
| RCORE-03.C03 | World/Origin/3Point | UI+API | [Draft_SelectPlane][fc-ui], `align_to_3_points`, `align_to_point_and_axis`, top/front/side | Menu Set CPlane đang disabled tại [CoreWorkspace][om9-workspace]; không suy thiếu host |
| RCORE-03.C04 | Theo curve/face/placement | API | `align_to_edge_or_wire`, `align_to_face`, `align_to_obj_placement` | Cần mapping option và singular/trim policy theo command |
| RCORE-03.C05 | CPlane previous/next | UI+API | `PlaneGui._previous/_next`; task panel có nút nối handler | Chưa kiểm tương tác với session latch và OM9 view undo |
| RCORE-03.C06 | Named frame persistence | Một phần | [WorkingPlaneProxy][fc-proxy] và `align_to_wp_proxy` cung cấp object/restore | Chưa chứng minh equivalent NamedCPlane/import Rhino hoặc persistence mọi viewport |
| RCORE-03.C07 | Grammar điểm Rhino dùng chung | Chưa thấy trong phạm vi rà | WorkingPlane là phép chuyển frame; Draft task field không chứng minh grammar toàn host | `Session::input` có comma coordinates và `r/R`; cần adapter cho `w/wr/@` đã có help, cùng locale fixtures |
| RCORE-03.C08 | `x,y,z` theo CPlane | Một phần | Host có API frame nhưng không áp grammar Rhino cho mọi workbench | Source [curve.rs][om9-curve] transform cả 2/3 số khi `frame` có giá trị; record Circle khác |
| RCORE-03.C09 | Units/angle/scalar xuyên lệnh | Một phần | Draft panel dùng `Units.Quantity`; không phải parser point session thống nhất | Các family có slice riêng; hợp nhất không được đổi threshold đã duyệt |
| RCORE-03.C10 | Đổi view/đóng doc giữa nhập điểm | Chưa kiểm chứng | `PlaneGui` có view observer; `Draft_SelectPlane.Activated` finish active Draft command | [CurveController][om9-controller] kiểm document/updateInputFrame; cần native fixtures |

## Khác biệt Circle phải giải quyết

[Native record Circle][circle-record] mô tả hai thành phần theo CPlane,
ba thành phần world XYZ và latch axes tại điểm đầu.
Source hiện đọc `Session::input` trong [curve.rs][om9-curve] dòng 552–584
áp basis cho cả hai và ba thành phần khi có frame; relative bỏ origin rồi cộng điểm trước.
`CurveController::submit` cập nhật frame, thử `circleNativeInput`, sau đó gọi Rust input.
Do đó có khác biệt giữa record và source hiện tại; chưa có replay chứng minh mọi
nhánh Circle/runtime binary đang theo một trong hai cách. Không sửa production ở audit này.
Phải lưu fixture legacy, fixture mục tiêu Rhino và hash binary/source của lần chạy;
quyết định migration explicit trước khi sửa record hoặc gọi chức năng tương thích.

## Adapter plan và phụ thuộc

Rust trước: tách grammar/version, typed coordinate values, frame math/checks,
named-frame registry và latch/undo rules khỏi các controller riêng.
Native C++ tối thiểu lấy ray/camera/placement và render CPlane; Qt chỉ chuyển event
và hiển thị prompt. Nếu tiếp cận Draft qua Python, đó là adapter API hiện hữu có
owner rõ, không đặt thêm business logic mới trong Python.
Không gọi `Draft_SelectPlane` trực tiếp trong active OM9 session nếu việc đó kết
thúc command ngoài ý muốn; dùng API snapshot có adapter và kiểm callback ownership.
Native exception chuyển thành typed error, không xuyên ABI; Rust không giữ raw pointer.
Phụ thuộc: RCORE-02 units/tolerance; RCORE-04 session; RCORE-05 pick;
RCORE-06 placement/identity; RCORE-09 replay; RCORE-12 view lifetime.

## Chi tiết bổ sung từ code decompile

Các chi tiết dưới đây vẫn ở mức `source_inspected`, chưa kiểm chứng runtime.
“Thân managed” là nhánh xử lý, tham số hoặc giá trị gán thấy trực tiếp;
mô tả SDK là hợp đồng trong chú thích đi kèm code. Phần bridge native bổ sung
xác nhận được một số nhánh tại ranh giới API, nhưng các lời gọi vào SDK bên
dưới vẫn cần kiểm chứng riêng. Giữ nguyên đánh giá FreeCAD và phạm vi native
OM9 đã nghiệm thu ở các phần trên.

| Hành vi và dữ liệu | Giới hạn và yêu cầu tích hợp |
|---|---|
| Viewport có cả API lấy Plane hình học và ConstructionPlane đầy đủ; Set và Push gửi cờ khác nhau xuống native, còn Pop/Next/Previous có entrypoint riêng. Chú thích SDK mô tả stack CPlane thuộc viewport. | Không thấy thuật toán stack, kích thước stack, quan hệ với document Undo hay thời điểm latch của từng command trong lớp managed này. |
| Constructor DTO mới gán WorldXY, GridSpacing 1, GridLineCount 70, ThickLineFrequency 5, DepthBuffered/ShowGrid/ShowAxes bật. GridSpacing và SnapSpacing là hai trường riêng; CopyToNative truyền riêng từng trường. | Đây là giá trị tạo DTO managed, không phải default document/template đang mở. Không lấy 70 làm kích thước lưới bắt buộc của OM9 hoặc coi snap spacing luôn bằng grid spacing. |
| Named CPlane table giữ RhinoDoc owner và truy cập bằng index/name. SDK mô tả Add với tên rỗng sinh tên, tên đã có thay plane cũ; Delete(name) thực tế Find rồi Delete(index). | Replacement/name generation do native xử lý. Không thấy persistent UUID độc lập label trong API này; ID bền của OM9 vẫn là quyết định thiết kế, không là default Rhino đã chứng minh. |
| SetToPlanView và SetProjection nhận cờ đổi CPlane riêng; SetProjection từ chối projection None ngay trong managed. Có property UniversalConstructionPlaneMode chuyển đọc/ghi xuống native. | Không được suy mọi thao tác đổi camera đều đổi CPlane, cũng không coi CPlane luôn độc lập trong mọi mode. Cơ chế truyền CPlane giữa views và default universal mode chưa được thân managed xác nhận. |

Thân bridge native bổ sung phân biệt rõ đổi plane hình học và thay toàn bộ
ConstructionPlane. Những nhánh sau là bằng chứng ở ranh giới API, chưa phải
nghiệm thu hành vi viewport.

| Nhánh bridge native đã đọc | Giới hạn và yêu cầu tích hợp |
|---|---|
| Setter chỉ nhận Plane sao chép ConstructionPlane hiện có, thay phần plane rồi gửi bản sao tới SDK SetConstructionPlane. Các trường ngoài plane được giữ trong bản sao tại boundary này. | Không tạo lại CPlane từ DTO mặc định khi chỉ cần đổi origin/basis, để tránh tự mất grid/snap/display fields. Việc SDK chấp nhận hoặc điều chỉnh state sau đó vẫn chưa chạy kiểm chứng. |
| Setter nhận ConstructionPlane đầy đủ kiểm tra viewport và dữ liệu khác null, sau đó chọn Push hoặc Set theo cờ. Selector stack tách Pop, Next và Previous; handle null hoặc selector không được nhận trả false. | Nhánh dispatch đã thấy trực tiếp; thuật toán stack, capacity, universal propagation, frame latch và quan hệ với document Undo vẫn thuộc SDK, chưa được các bridge này xác nhận. |

Hệ quả cho adapter/UI: state CPlane phải mang riêng plane hình học, grid spacing,
snap spacing, visibility và lịch sử CPlane. Rust ID/version của named frame là
policy OM9; thao tác tên trùng cần thể hiện rõ trước khi thay frame. Không đổi
grammar `w/wr/r/@`, policy latch hoặc record Circle từ việc nhìn thấy wrapper.
Thêm cờ explicit cho view command có đổi CPlane; universal mode cần fixture riêng.

| Fixture bổ sung — chưa chạy | Expected invariant / câu hỏi cần ghi nhận |
|---|---|
| RCORE-03.T10 | Đặt grid spacing khác snap spacing, ẩn grid rồi save/reload: hai spacing và visibility không bị gộp; ghi native settings thực tế, không lấy constructor DTO làm default template |
| RCORE-03.T11 | Set/Push/Pop/Previous/Next trên hai viewport và bật/tắt universal mode: ghi stack và propagation thực tế; document geometry/Undo không đổi ngoài policy được duyệt |
| RCORE-03.T12 | Named CPlane Add tên rỗng/tên trùng, restore rồi đổi projection với cờ CPlane true/false: đối chiếu replacement và frame; chưa đặt expected UUID Rhino hoặc thời điểm latch từ tên API |

## Fixtures nghiệm thu cần chạy

Tại baseline audit nguồn, tất cả ca dưới đây **chưa chạy**; cập nhật slice không tự đóng toàn bộ ca. Dùng tolerance đã chốt ở RCORE-02.

| Fixture | Input và expected invariant |
|---|---|
| RCORE-03.T01 | `O=(10,20,30), X=(0,1,0), Y=(0,0,1), Z=(1,0,0)`; `1,2,3` → world `(13,21,32)` cho mỗi command có point input |
| RCORE-03.T02 | Cùng frame T01, `1,2` → `(10,21,32)`; `w1,2,3` → world `(1,2,3)`; `w0,0,0` → world origin; không suy frame bằng số thành phần |
| RCORE-03.T03 | Previous world `(13,21,32)`: mỗi input `r1,2,3`, `R1,2,3`, `@1,2,3` → `(16,22,34)`; `wr1,2,3` → `(14,23,35)`; reset cùng previous trước từng ca, thiếu previous bị từ chối |
| RCORE-03.T04 | Chuyển view trước/sau first pick và CPlane dịch/xoay; latch/version đúng policy; preview và commit đồng điểm |
| RCORE-03.T05 | Named frame save/reload/rename/restore rồi previous/next; không đổi bounds hoặc placement của geometry |
| RCORE-03.T06 | 3Point collinear, NaN, unit sai, locale separator mơ hồ; lỗi không thêm point, object hoặc undo entry |
| RCORE-03.T07 | Pick object trong hai tầng placement/link; output world đúng transform một lần, không cộng CPlane origin hai lần |
| RCORE-03.T08 | Circle replay record legacy và target trên frame T01; ghi riêng source/binary hash, result và quyết định migration |
| RCORE-03.T09 | Esc, Undo điểm, restart, đóng viewport/document trong preview; không còn latch/callback/scene node thuộc session cũ |

## Chưa chốt về tương thích

`w` và `wr` đã có nguồn Rhino 5; còn cần runtime cho locale, unit aliases,
biến thể prefix ngoài ví dụ đã dẫn, polar/spherical/surveyor đầy đủ và lifetime
previous point khi đổi command. CPlane Curve/Surface options vẫn cần help/fixture.
Chưa chứng minh NamedCPlane tương đương WorkingPlaneProxy, hay thời điểm latch
của từng lệnh Rhino. Không tự đổi hành vi đã nghiệm thu vì contract mới.

[rhino-numbers]: https://docs.mcneel.com/rhino/5/help/en-us/user_interface/unit_systems.htm
[audit]: ../../../docs/reviews/2026-10-09-rhino-core-gap-audit.md
[fc-wp]: ../../../../../src/Mod/Draft/WorkingPlane.py
[fc-ui]: ../../../../../src/Mod/Draft/draftguitools/gui_selectplane.py
[fc-proxy]: ../../../../../src/Mod/Draft/draftobjects/wpproxy.py
[om9-curve]: ../../../rust/src/curve.rs
[om9-controller]: ../../../Gui/CurveController.cpp
[om9-workspace]: ../../../Gui/CoreWorkspace.cpp
[circle-record]: ../../../docs/features/OM9-CURVE-005.md
