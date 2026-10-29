# Phiếu điều chỉnh từng owner

Đọc `SPEC.vi.md` trước. Đây là yêu cầu migration, không phải bản ghi đã chạy. File `COMMAND-MATRIX.vi.md` cho đường dẫn, hàm provider và ID lấy được từ module; `source-inventory.json` giữ điều kiện availability hiện tại. Mỗi phiếu phải được bổ sung stage table chính xác sau khi đọc parser và exact milestone, trước khi sửa code.

## 1. Phiếu mẫu phải điền cho từng command/mode

```text
Command / aliases / feature ID:
Menu / toolbar / F6 / CMD routes thực tế:
Domain owner + pending field/variant:
Selection và context bắt buộc trước khi start:
Stage A: prompt; required input; options; current/default values; allowed transitions:
Stage B: ...
Mỗi option: stable ID; kind; current value getter; shortcut; constraints; unit;
  begin-entry; valid submit; invalid retry; point pick; unavailable stages:
Mỗi choice: set hay cycle; ảnh hưởng đến điểm/constraint/preview đã có:
Finish / Cancel / pending Undo / document Undo boundary:
Existing test targets + fixture dựng geometry hợp lệ:
Runtime evidence đúng binary:
Unknown / unsupported / baseline failure:
```

Không điền default bằng trí nhớ CAD. Đọc constructor/dispatch của owner. Nếu prompt và parser mâu thuẫn, ghi cả hai, chọn theo local contract có thẩm quyền và bổ sung regression cho quyết định; không sửa ngầm theo cảm tính.

## 2. Polyline — mẫu đối chiếu bắt buộc

Nguồn: `session/polyline.rs`, `polyline_options.rs`. ID `OM9-CURVE-001`.

| Stage | Tham số/action khả dụng | Kết quả bắt buộc |
|---|---|---|
| Chưa có điểm | PersistentClose, Mode, Helpers | P/H đổi flag; M đổi Line/Arc; không có Length, Close, Undo, Direction, Center hoặc BothSides |
| Có điểm, Mode Line | P/M/H, Length, Undo; Close khi đủ 3 điểm | L mở value-entry; giá trị hợp lệ hiện Length=value; C đóng đúng chain |
| Có điểm, Mode Arc | P/M/H, Direction, Center, Undo | D/C yêu cầu input owner; không lẫn C Close hoặc L Length |
| Length entry | Nhập length hữu hạn >0 | Snapshot option thông thường rỗng; lỗi giữ stage/draft; không commit wire |
| Direction/Center entry | Point/vector theo owner | Viewport và text dùng cùng owner; hiển thị giá trị đã nhận, preview tương ứng |
| Finish/Cancel | Enter/Close hoặc Escape theo owner | Enter commit whole wire một transaction; Escape bỏ pending, không xóa object có trước |

Không lấy flag Helpers làm bằng chứng tracking đã xong. Không chứng nhận continuously closed preview của PersistentClose, major/antipodal Center arcs hoặc mixed-Polycurve 3DM re-export nếu chưa có bằng chứng riêng. Migration các owner khác không được làm hỏng Line–Arc–Line analytic geometry và regression Close/PersistentClose.

## 3. Line, Rectangle và Arc

### 3.1. Line

Nguồn `session.rs` (`CommandSession`, `submit_geometry_text`) và `session/command_options.rs`. ID `OM9-CURVE-002`.

- BothSides là flag của Line; contract đã duyệt đặt việc chọn ở trước first point. Adapter hiện expose rộng hơn stage này: audit và thu hẹp theo contract, không copy từ trạng thái `both_sides()`/repeat của run cũ.
- Length là constraint khoảng cách, lưu mm; hiển thị theo unit của session. Xác định stage cho phép theo owner contract; giá trị finite, dương theo parser. Empty/default và first-point behavior phải có test, không suy từ Polyline.
- B/click/full name/assignment phải dùng owner, không `set_both_sides` trực tiếp trong UI riêng. Kích hoạt Length đi qua entry; owner đổi endpoint theo direction pick như cũ.
- Test: tài liệu inch Length=2.5 → khoảng cách 63.5 mm; BothSides trước point; hủy; geometry và một transaction. Test Line → Polyline/Circle/Rectangle/Idle và quay Line mới, không rò flag, Length hoặc accelerator.

### 3.2. Rectangle

Nguồn `session/rect.rs`; test `rect.rs`.

- Mode choices khai báo: Vertical, 3Point, Center trước first point. Default mode đọc `RectPending::new`; không thêm Corners/Mode cycle nếu parser chưa định nghĩa đường chọn đó.
- Square là toggle; chọn lại Square phải đảo flag, không set Yes cố định. Length chỉ available khi parser đã có corner cần thiết.
- Length/Width/Height có ý nghĩa khác theo mode: ThreePoint/Vertical có nhánh nhập chiều rộng/chiều cao trong local edge frame. Giữ commit timing và mapping của owner, không gán tất cả vào `width`.
- Giá trị width/height cần dương, finite; nếu parser hiện thiếu finite/budget guard thì thêm ở domain input boundary và có regression. Không tạo hình ở bước first corner hoặc toggle.
- Test mọi mode: typed dimensions và viewport points cho cùng shape/frame, Square hai lần, invalid rồi sửa, Undo point/constraint và Esc. Không đánh dấu Vertical xong chỉ từ default two-corner.

### 3.3. Arc / Arc Direction

Nguồn `session/arc.rs`; test `arc.rs`; module có hơn một feature ID.

- 3Point, Center, StartPoint là choice **set**, không toggle. Mode change được owner giới hạn trước first point; sau đó option phải unavailable và stale activation bị từ chối.
- Radius/Diameter chỉ thuộc Center mode và stage owner đang chấp nhận. Dùng đúng `arc_option_tokens` và parser để đối chiếu stage; không bật Radius ở mọi mode chỉ vì Arc có field radius.
- Diameter nhập 10 phải tương đương radius 5 khi mode/geometry đủ điều kiện. Undo trả về point stage đúng; không gọi document Undo.
- Test start direction collinear, center/radius mismatch, repeated points; lỗi không làm mất arc points. Kiểm actual analytic arc/frame và menu/CMD của từng ID.

## 4. Circle và Ellipse — nhiều mode, nhiều nghĩa của một chữ

### 4.1. Circle

Nguồn `session/circle.rs`; test `circle.rs`.

Các nhóm tham số cần audit độc lập:

| Nhóm | Option/value | Quy tắc migration |
|---|---|---|
| Construction | Center, 2Point, 3Point, Vertical, FitPoints, AroundCurve, Tangent | Chọn mode cụ thể. Parser hiện có nhánh reset points/normal khi đổi mode; giữ semantics có bằng chứng, không áp luật Arc trước-first-point một cách máy móc |
| Approximation | Deformable, Approx | Cùng một owner flag, hai alias. Không hiển thị Yes/No đối nghịch và không giữ hai bool UI |
| Approximation inputs | Degree, PointCount | Chỉ khi deformable. Degree 1..11; PointCount parser hiện nhận >=4, không tự gán cap 256 của Ellipse sang Circle; vẫn áp domain budgets |
| Size kind | Radius, Diameter, Circumference, Area | Typed Choice của Size; lựa chọn chỉ set kind. Parser nhận cả bốn; provider hiện thiếu nút Size=Radius: audit gap, không mặc định không hỗ trợ |
| Size input | Radius=, Diameter=, Circumference=, Area= | Số finite dương; conversion về radius do owner. Đọc nhánh Center/Vertical sau center vì provider hiện chỉ expose một phần nhánh Center |
| Tangent | Radius, Point, Solution | Point đúng 3 thành phần finite; Solution 1..4 hiện tại; khả năng dựng nghiệm phụ thuộc constraint/solver thật |
| AroundCurve | OnCurve | Fraction 0..1 và curve source phải tồn tại; giữ normal/tangent của curve owner |
| Input control | Undo, Cancel | Theo points/owner stage; không tự thêm document commit |

Test conversion ở center hợp lệ: Radius=5, Diameter=10, Circumference=10π, Area=25π đều cho radius 5, cùng normal/frame và một commit theo owner. Test choice Size=Area rồi Size=Diameter qua ba route: selected value đổi đúng, chưa tạo object chỉ vì chọn kind.

Tên bắt đầu trùng C/D/A/P không được lấy shortcut bằng `chars().next()` mà không resolve uniqueness. Cancel dùng Escape; số 2/3 vẫn gõ được trong numeric/coordinate input. Cases Center, 2Point, 3Point, Vertical, FitPoints, AroundCurve, từng constraint Tangent phải tách trạng thái/evidence; một Circle mặc định không chứng minh hết.

### 4.2. Ellipse

Nguồn `session/ellipse.rs`; test `ellipse.rs`.

- Mode choices: Center, Corner, Diameter, Vertical, FromFoci, AroundCurve ở stage chưa có point/center theo provider; kiểm parser trước khi quyết định có cho đổi mode sau point.
- OnCurve chỉ khi AroundCurve cần center và source curve hợp lệ; MarkFoci theo FromFoci hoặc center đã có. MarkFoci là toggle của owner, không tự thêm geometry marker thật vào document khi chỉ preview.
- Deformable/Approx cùng flag; Degree 1..11, PointCount 4..256 theo parser hiện tại. Không copy PointCount bounds từ Circle.
- RadiusX/RadiusY cần center và giá trị dương; snapshot lấy từng radius đúng trục, không hiển thị cả hai từ radius_x. Point/diameter/foci cần không suy biến theo solver.
- Test mỗi mode/frame, radius X/Y khác nhau, MarkFoci hai lần, deformable bật/tắt, invalid radius và retry, relation point_count/degree ở geometry admission.

## 5. Đường qua điểm / control points

| Owner / file | Tham số | Điều kiện, lỗi và điều cần giữ | Test riêng tối thiểu |
|---|---|---|---|
| `interp` / `interp.rs` | Degree, Close, Undo, Cancel | Degree **lẻ** 1..11; Close từ 4 điểm theo provider; cần đủ điểm cho degree khi commit | Degree=2 bị từ chối và giữ draft; Degree=3/5 hợp lệ với đủ điểm; closed/open và Undo point |
| `control_curve` / `control_curve.rs` | Degree, Undo, Cancel | Degree 1..11; nhận control points; commit cần ít nhất 2 control points và constraints domain | Đổi degree trước/sau point theo parser; không xóa points khi chỉ chọn Degree |
| `continue_curve` / `continue_curve.rs` | Degree, Undo, Cancel | Selection curve bắt buộc; giữ poles/source samples; cần ít nhất một control point mới để commit | Invalid source/stale selection, degree retry, extension đầu/cuối theo owner, source không đổi trước commit |
| `continue_interp` / `continue_interp.rs` | Degree, Undo, Cancel | Degree 1..11 trong parser; cần ba điểm mới theo error owner hiện tại | Không gán luật odd của InterpCrv cho owner này khi contract không yêu cầu; đủ/thiếu điểm và continuity |
| `through_points` / `through_points.rs` | Degree, Closed, Open, Cancel | Selected points đủ; Degree 1..11; Closed/Open là hai **set choices** của cùng closed flag; cần nhiều points hơn degree | Chọn Closed hai lần vẫn Closed; Open trả Open; không toggle khi chọn lại cùng choice |

Các branch nhận điểm text phải giữ thứ tự và dimension; không biến một chuỗi ba tọa độ thành ba accelerator. Mỗi owner có test fixture selection/continuity của chính nó, không reuse một constructor empty state để tuyên bố geometry xong.

## 6. Polygon, Helix, Spiral

| Owner | Tham số | Luật cần giữ | Input/geometry kiểm chứng |
|---|---|---|---|
| `polygon` | Sides; Circumscribed/Inscribed; Undo; Cancel | Sides nguyên 3..1024. Hai kiểu là set choices, không toggle; chọn lại Circumscribed không tự thành Inscribed. Undo khi có center | Sides=3,4; giới hạn 2/1025 từ chối; measured radius/inradius khác đúng hai kiểu; một polygon commit |
| `helix` | Radius, Height, Turns, Axis; Undo; Cancel | Radius/Height/Turns dương; Axis finite nonzero; constructor defaults lấy từ owner. Stage center/radius và Undo không được bị entry Turns làm mất | Turns=2 bằng ba route; axis không trùng default; height/turns thật trên curve; zero/âm retry |
| `spiral` | StartRadius, EndRadius, Turns, Height, Axis; Undo; Cancel | Start/EndRadius dương; Turns dương; Height **không âm**, 0 hợp lệ; Axis nonzero. Không dùng validator Height>0 của Helix | Spiral phẳng Height=0; radii khác nhau; Height=-1 bị từ chối; axis/frame và sampled endpoint đúng |

Axis là vector theo parser của Helix/Spiral; không đổi vector thành world center. Preview có thể lấy kích thước từ cursor khi constraint unset; label unset không được fake thành radius 0.

## 7. Các lệnh chỉnh curve và truy vấn

| Owner / source | Option và kiểu | Ràng buộc/availability quan trọng | Kết quả phải đo/khẳng định |
|---|---|---|---|
| `rebuild` / `rebuild.rs` | Degree integer, Points integer | Selected curve; Degree 1..11; Points >=4 hiện tại; quan hệ degree/points và budget khi build | New curve hợp lệ, degree/count đúng, một commit theo owner, source handling/Undo đúng |
| `fit` / `fit.rs` | Tolerance number | Selected curve; finite >0; không success nếu không đạt tolerance | Sai lệch fit trong tolerance theo fixture owner |
| `offset_curve` / `offset.rs` | Distance number | Selected curve; Distance **khác 0**, âm có nghĩa, không cấm âm chung | Hướng/độ lệch có dấu, source/selection và Undo |
| `offset_on_surface` / `offset_on_surface.rs` | Distance number | Curve + surface selection; Distance khác 0 | Offset trên surface, không thay bằng planar offset |
| `offset_normal` / `offset_normal.rs` | Distance number | Curve + surface selection; Distance khác 0 | Normal/side từ owner surface, không hardcode world Z |
| `project` / `project.rs` | Direction vector | Finite, nonzero; parallel-to-plane invalid theo owner | Projected result trên plane/target thật; không chỉ hiển thị vector |
| `blend` / `blend.rs` | Scale1, Scale2 numbers | Hai curve; mỗi scale >0; dùng đúng start/end | Endpoint/tangent behavior theo solver/fixture; không đổi cả hai scale khi chỉ sửa một |
| `pull` / `pull.rs` | Tolerance number | Curve + surface; tolerance >0 | Result đáp ứng surface/tolerance theo capability hiện có |
| `intersect` / `intersect.rs` | Tolerance number | Hai curve; tolerance >0; không intersection là kết quả/error xác định theo owner | Số/điểm giao thực, không success-no-op |
| `crv2view` / `crv2view.rs` | Cancel, Enter finish | Hiện provider chỉ có Cancel. Đây không phải lỗi thiếu Degree/Length | Chạy đúng two-curve reconstruction; không thêm tham số Polyline |
| `arc_blend` / `arc_blend.rs` | Cancel, Enter finish | Hai curve; collinear direction có thể invalid | Actual arc blend, failure giữ model; không thêm Scale1 từ Blend khác |
| `fillet_corners` / `fillet_corners.rs` | Radius number | Selected polyline có line corners; radius >0 | Arc ở đúng corners, handling radius không vừa theo owner |
| `corner` / `corner_ops.rs` | Radius **hoặc** Distance | Fillet dùng Radius, Chamfer dùng Distance; hai line selection; số >0 | Không show cả hai cho một feature sai; topology và kích thước corner đúng |
| `extend_curve` / `extend_curve.rs` | Length; Start/End | Length >0; Start/End là set side. Provider có thể chỉ đưa side đối diện, snapshot vẫn phải biết current side thật | Thay side không xóa source; extension đúng đầu và chiều dài |
| `divide` / `divide.rs` | Count integer | Count 2..100000; budget áp trước allocation/publish | Số điểm/phân đoạn theo owner, không tạo N command transactions |
| `to_lines` / `to_lines.rs` | Count integer | Count 2..100000; không confuse Count với control-point count | Conversion thực sự thành line representation theo fixture |
| `tween_curves` / `tween_curves.rs` | Count integer | Hai curve; Count 1..64 | Số curve sinh ra/position interpolation theo owner, batch atomicity |
| `crv_seam` / `crv_seam.rs` | Seam fraction | Fraction 0..1; selected curve và closedness theo solver | Seam/parameterization đổi đúng, geometry shape không bị tùy tiện thay |
| `curve_soft_edit` / `curve_soft_edit.rs` | Index integer; Point point | Index phải tồn tại trên owner curve; Point đủ 3 finite values; required target trước commit | Chỉ control point/region dự kiến đổi, invalid index không mutate |

Mọi test Count/Points rất lớn phải chứng minh budget refusal mà không tạo allocation khổng lồ. Không dùng “test chạy lâu nhưng chưa crash” làm nghiệm thu hiệu năng.

## 8. Surface-related providers đã có trong registry

| Owner / source | Tham số | Cách xử lý bắt buộc |
|---|---|---|
| `isocurve` / `isocurve.rs` | U, V numeric | Một `iso` + một `value` trong state hiện tại, không phải hai independent cached values. Chọn U/V đổi axis đúng; kiểm actual surface domain thay vì tự clamp tất cả vào 0..1; finite ở parser chưa chứng minh kernel chấp nhận mọi số |
| `angle_isocurve` / `angle_isocurve.rs` | Angle numeric; Direction **Choice U/V** | Không phân loại Direction thành Point theo tên. Prompt có 0..360 nhưng parser hiện chỉ kiểm finite: ghi discrepancy, tra exact contract trước khi thêm wrap/clamp/range |
| `cross_section` / `cross_section.rs` | Count integer; Spacing number | Surface selection; Count 1..64, Spacing >0; kết quả thực tế có thể không giao: báo owner result, không giả profile |
| `extrude_surface` / `extrude_surface.rs` | Height numeric | Curve selection; Height **khác 0**, âm có nghĩa. Không reuse Helix Height>0; kiểm surface orientation/extent |
| `heightfield` / `heightfield.rs` | File text/path; Height numeric; Size numeric | File bắt buộc trước commit. Height hiện finite, không tự cấm âm/0 khi contract chưa nói; Size finite >0. Giữ Unicode/space/case của path; đọc image theo owner, không diễn giải path thành command |

Các lệnh này có geometry native/capability riêng. Nếu build thiếu capability thì visible-disabled với reason; button/entry mới không được che lỗi backend. Test trên fixture surface/curve đúng owner, không lấy line 2D làm bằng chứng thay thế cho surface operation.

## 9. Solid provider: tách Box và Sphere dù cùng pending type

Nguồn `session/solid.rs`, tests `solid.rs`, `solid_workflows.rs`. Đọc thêm `skills/openmatrix9-feature-port/references/solid-box-sphere-contract.md` trước khi sửa stage/mode.

### 9.1. Box

- Initial choices đang khai báo: Diagonal, 3Point, Vertical, Center, Cube; payload `Mode=...`. Mode sau point chỉ có nhánh riêng được owner cho phép (ví dụ Cube sau first diagonal point). Không bật mọi mode ở mọi stage.
- Length/Width/Height có frame và stage khác nhau theo Box kind. Center sau first point có Length; Width/Height theo point state. `box_length` và `box_width` không phải sphere radius.
- Chọn width/height hoặc viewport height có thể commit ngay; preserve callback transaction và signed height. Không chuyển bộ ba `2 3 4` trong interactive Box thành dimensions một cách tùy tiện vì chuỗi đó còn có nghĩa là một point.
- Baseline `remaining_primitive_menu_prompts_produce_analytic_volumes` đã fail ở assertion Undo count trên mã trước common interface. Không sửa coordinate semantics hoặc xóa assertion để giấu lỗi; tách quyết định workflow đó bằng exact contract/evidence.
- Test mỗi kind, local frame, negative height, default Enter height, dimensions/viewport parity, Undo qua Length/Width, một solid volume/closed topology và atomic commit.

### 9.2. Sphere

- Initial choices: Center, 2Point, 3Point, 4Point, Vertical, AroundCurve, FitPoints, Tangent; choice set, không cycle tất cả bằng một chữ M tự bịa.
- Constraint stages AroundCurve/Tangent/Tangent2/Tangent3 có thể thay toàn bộ mode list bằng Point/Undo/Cancel. Không show mode list cũ từ snapshot cached.
- Radius/Diameter chỉ khi owner đang chấp nhận; Point không có dấu `=` trong một stage là action chọn constraint, khác một numeric field. Không phân loại chỉ bằng suffix của label.
- Solution=1/2 là hai lựa chọn cùng parameter. Current solution đọc private owner qua getter an toàn; không parse text `(Yes)` để suy branch. Chọn lại Solution=2 giữ branch 2.
- Test từng mode và từng tangent constraint được tuyên bố supported; measured center/radius/volume, nghiệm 1/2 khác nhánh đúng, không mất selected curve(s), out-of-plane frame đúng. FitPoints selection batch không được bị generic Enter handler giành mất.

## 10. Những command Solid chưa được 39-group registry bao phủ đầy đủ

`SolidPending::start` còn có chuỗi positional labels ngoài interactive Box/Sphere. Audit `labels`, `values`, `text`, `feature` và `submit_solid_text`; không coi group `solid` đã đồng nghĩa mọi primitive xong.

| Feature / nhóm hiện có | Input cần giữ | Yêu cầu entry đặc thù |
|---|---|---|
| `OM9-SOLID-008` Text | Text rồi Height | Chuỗi có space/chữ số không được parse thành kích thước hoặc shortcut; Height lỗi ở lại Height; Esc giữ document |
| `OM9-SOLID-004` Boolean2Objects | Operation U/D/I hoặc full name | Choice case-insensitive; invalid giữ prompt; hai solid selection; một transaction |
| `OM9-SOLID-013` Ellipsoid | X radius, Y radius, Z radius | Positional chain; mỗi value có stage rõ, preserve already accepted values khi input sau sai |
| `OM9-SOLID-015/016/017/018/019/020` | Ring/tube radius; radius/height; outer/inner/height; sides/base/height; bottom/top radius/height | Không đặt một generic Radius cho tất cả. Validate relations/topology theo solver; không tự cấm height/zero top radius nếu owner có ý nghĩa hợp lệ |
| ExtrudeCrv/ExtrudeAll/Pipe | Height hoặc Radius trên source selection | Preview/commit và number of objects/transaction theo owner batch, không copy Polyline multi-point finish |
| ExtractSrf/FilletEdge/Shell | Face/edge index; radius/thickness | Index nguyên và valid topology, không cast -1/0.5 sang edge/face 0. Giữ source stamps/selection và backend ownership |
| Các immediate Boolean/Cap… | Selection và action hiện có | Phân loại immediate hoặc positional theo route; không thêm parameter UI giả |

Đọc `solid-prompt-contract.md` cho Text/Boolean đã duyệt. Đối với command chỉ nhận positional chain, snapshot chung thể hiện **current input slot**, không ngụ ý cho phép sửa tùy ý mọi slot đã đi qua. Bổ sung Back/Undo slot chỉ khi owner contract đã có; không tự sáng tạo.

## 11. 19 pending field ngoài registry và các variant

Nguồn chính `session.rs`, `session/top11.rs`, `session/surface_edit.rs`. Đây là hàng audit có tên và dữ liệu gợi ý từ struct/comment; thứ tự cú pháp, defaults, ranges và commit phải đọc parser/milestone, không lấy bảng này thay parser.

| Pending field | Input cần xác minh/đưa vào snapshot | Điểm phải kiểm |
|---|---|---|
| `top11_pending` | Duplicate/Move/Rotate/Mirror/Split/Trim variants | Duplicate có Vertical, FromLastPoint, UseLastDistance, UseLastDirection; Mirror có Copy và 3Point; Split/Trim chọn object/cutter thay vì point. Resolve theo variant, không một owner generic mang đủ mọi flag |
| `surface_pending` | Numeric distance của surface operation | Feature quyết định distance/normal effect và signed value; không dùng Curve Offset parser |
| `curve_pair_pending` | Axis origin/direction cho profile/rail | Selection order và local frame; positional vector/point parser |
| `surface_pair_pending` | Source/target edge names | Là topology choice/text, không numeric count; validate feature/source generation |
| `seam_pending` | Axis, tolerance | Axis representation và tolerance default/units theo owner |
| `soft_edit_pending` | UV, radius, delta | UV domain khác world XYZ; vector delta không bị transform hai lần |
| `tween_pending` | Fraction giữa surface pair | Không gán Count của TweenCurves; đúng fraction domain |
| `extend_pending` | Edge, distance | Edge thật của surface, signed extension/solver limits |
| `split_pending` | Axis, parameter | Surface iso parameter domain, giữ type axis |
| `tangent_pending` | Edge, direction | Direction/frame là domain owner, không hardcode Point3 generic |
| `extrude_all_pending` | Height trên nhiều source curves | Batch transaction, một lỗi trong batch không publish một phần |
| `variable_offset_pending` | Hai distances, axis | Không collapse thành một Distance; quan hệ từng đầu |
| `shrink_pending` | UV sub-domain | Bốn bounds/interval order theo parser, không đổi UV thành mm |
| `variable_blend_pending` | Hai tangent scales | Giữ first/second curve semantics, không thay bằng scalar chung |
| `chamfer_pending` | Setback | Phân biệt với `corner_pending` của curve; owner feature riêng |
| `fillet_pending` | Một/hai radii | Bậc/loại geometry và mode theo feature, không bật mọi radius mọi stage |
| `variable_chamfer_pending` | Hai setbacks | Không giữ một scalar UI làm đè hai giá trị |
| `history_options_pending` | Closed, Flip, Simple cho Sweep History | Option/context/History recipe và refs; không mất rail_count/source IDs |
| `slash_pending` | Hai alignment points trên rails | Snapshot phải tiếp tục giữ Closed/Flip/source state từ owner; slash point không phải control point tự do |

Top11 cần test object selection vs point input và repeat-copy transaction boundary. Các option Duplicate hiện có setter; việc comment nói “base-point prompt” chưa chứng minh setter đã enforce stage đó. Audit implementation và milestone để quyết định availability thật, thêm test ở base/destination stages. Không tự copy P/M/H vào Duplicate.

## 12. Entry point không nằm trong các field trên

Audit command registry/catalog và các UI call sites trước khi chốt toàn bộ scope. File lifecycle, View, selection/layers, Gumball numeric, property editor, plugin `PluginLoad`/`PluginRun`, Builder/F6 phải có hàng phân loại dù kết luận là protocol riêng hoặc immediate action. Giữ file dialogs, save/close ownership, JSON/path payload và text focus. Không đếm “không có `_pending`” thành “không tồn tại command”.

Mỗi hàng phát hiện ngoài inventory gốc phải ghi file/method/feature, input protocol, owner, phân loại và testcase/evidence hoặc lý do excluded. Đến cuối migration, không còn hàng `AUDIT_REQUIRED`/`UNCLASSIFIED` mà vẫn tuyên bố hoàn tất tất cả.
