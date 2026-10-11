---
id: OM9-INFO-020
name: Gumball
command: Gumball
domain: 01-core
module: Info & Settings
kind: command
implementation_status: implemented
approved_scope_complete: true
stock_ci_status: pending
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-INFO-020 — Gumball

Alias tương thích: `Gumball`. Nhóm: `01-core`. Trạng thái: `implemented` trong approved OM9 scope.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-INFO-020` — Gumball.**

Gumball là lệnh bật/tắt bộ tay nắm Move/Scale/Rotate của selection. Khi bật,
cùng một Gumball phải hiện đồng thời trong cả bốn viewport của document;
mọi bản hiển thị dùng chung origin, hệ trục, operation và preview. Mũi tên
dịch một trục, ô mặt phẳng dịch hai trục, tâm dịch tự do, cung xoay quanh tâm
và ô vuông Scale một trục. X đỏ, Y xanh lá, Z xanh dương theo ảnh Rhino/Matrix
người dùng cung cấp. Hình thức và thao tác chi tiết dưới đây là mục tiêu
nghiệm thu; implementation notes chỉ ghi phạm vi đã chạy được.

### Nguồn và phạm vi revision 2026-10-11

- Ảnh bốn viewport do người dùng gửi là ảnh tham chiếu Rhino 5/Matrix,
  không phải bằng chứng OM9 đã đạt. Ảnh xác nhận hiển thị đồng thời, Scale
  ở biên selection, ô mặt phẳng hai màu và menu ball vòng trắng có đuôi chấm.
- Bản trích Matrix 8 Book 1 gồm PDF 1–3 / trang in 35–37 đã đọc cả chữ và
  hình. Nó mô tả axes/planes/free origin, arcs, Scale, nhập số, Alt Copy,
  Ctrl Extrude, Relocate và Alignment. Số PDF45–47 trong evidence cũ thuộc
  quyển đầy đủ; không dùng chúng để mở bản trích ba trang.
- PDF dùng X đỏ/Y xanh lam hoặc cyan/Z vàng; mục tiêu màu OM9 lấy theo ảnh
  X đỏ/Y xanh lá/Z xanh dương. Không coi khác biệt hai nguồn là lỗi đọc PDF.
- [Rhino 5 Gumball Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/gumball.htm)
  và [Gumball Options](https://docs.mcneel.com/rhino/5/help/en-us/options/modeling_aids_gumball.htm)
  bổ sung Shift Scale3D, menu và các tùy chọn hiển thị. Các kích thước dưới
  đây là lựa chọn OM9 để tái hiện ảnh, không phải default Rhino đã xác minh.
- Chỉ ba spec lõi `OM9-INFO-020/021/022`; Smart Targets/Smart MSR và các mode
  riêng của Clayoo/SubD nằm ngoài revision này. Không đổi nền, màu selection,
  tên hoặc thứ tự viewport qua spec Gumball.

### Bật/tắt và hiển thị đồng thời

1. `Gumball On` bật, `Gumball Off` tắt, `Gumball Toggle` hoặc `Gumball` đổi
   trạng thái. Menu/icon hiện có đi qua cùng handler và cùng trạng thái checked.
   Profile mới mặc định Off; profile đã có giữ Enabled đã lưu. Chọn object
   khi Off không tự bật. Khi On nhưng selection rỗng thì ẩn tay nắm, vẫn giữ On.
2. Khi có selection hợp lệ, cả bốn native viewport đang tồn tại đều có overlay.
   Không đợi kích hoạt từng ô; không chỉ gắn overlay vào active view. Các camera
   nhìn cùng một frame thế giới, không tạo bốn pivot hoặc bốn phiên biến đổi.
3. Một lần nhấn tay nắm ở ô chưa active phải vừa chọn ô thao tác vừa bắt đúng
   tay nắm trong cùng sự kiện; không cần click lần hai hoặc chờ timer refresh.
4. Drag ở bất kỳ ô nào đồng bộ preview sang mọi ô còn mở. Mouse capture,
   editor số và menu chỉ thuộc ô phát sinh; một gesture chỉ commit một batch
   và một transaction Undo. Copy/Extrude không nhân kết quả theo số viewport.
5. Off, deselect, permission/source change hoặc document/workbench close
   dọn mọi overlay/editor/preview và mouse capture phù hợp. Đổi active view
   khi idle giữ widget xuất hiện ở tất cả ô. Đổi view/navigation trong drag
   hủy phiên thống nhất, không để preview hoặc capture ở ô khác.
6. Maximize rồi trở lại bốn ô giữ selection, frame và Enabled; không thay
   geometry, Undo hoặc camera ngoài thao tác view được yêu cầu. Đóng một ô
   dọn đúng overlay của nó; nếu đó là ô đang sở hữu gesture thì hủy gesture.

### Hình dáng và bố trí tay nắm

| Thành phần | Mục tiêu theo ảnh | Quy tắc OM9 để nghiệm thu |
|---|---|---|
| Move | Mũi tên thanh, đầu nhọn, nằm phía dương của trục | X đỏ/Y xanh lá/Z xanh dương; màu theo trục Gumball đã chọn, không theo tọa độ màn hình |
| Rotate | Cung khoảng một phần tư vòng ở phía âm | Màu của trục xoay; nhìn Top/Front/Right ở World không để cung vuông góc chồng tâm |
| Scale | Ô vuông rỗng, nối tâm bằng nét chấm, ở biên phía âm | Vị trí theo bounds selection trong frame Gumball, không dùng độ dài cố định của mũi tên |
| Plane | Ô hai màu có dấu chia ở giữa, tách khỏi tâm | Hai màu đúng cặp XY/YZ/ZX; hình nằm trên mặt phẳng đó và project theo camera |
| Origin | Ô rất nhỏ tương phản trắng/đen | Free Move độc lập; không nhầm với menu ball hoặc trục nhìn thẳng vào camera |
| Menu ball | Vòng trắng rỗng phía dưới-trái tâm, có đuôi chấm | Theo camera, không xoay theo frame object; mở menu, không bắt đầu transform |

Profile hình thức OM9 ở DPI100%: từ tâm đến đầu Move và bán kính cung khoảng
80 logical px; nét axes/arcs 1–2 px; đầu mũi tên khoảng 18×9 px; Scale square
8 px; origin 4 px; plane square 20 px, cách tâm khoảng 50 px trên mỗi trục
phẳng; menu ring đường kính 10 px, offset màn hình khoảng (-30,+30) px.
Các trị số này chuẩn hóa ảnh tham chiếu để có fixture đo được; mọi kích thước
screen-space phải giữ theo logical px khi zoom và DPI thay đổi. Chúng không
ấn định khoảng cách Scale tới tâm, vì khoảng đó phụ thuộc geometry.

Bounds dùng selection thực sự và world placement, gồm group/instance theo
capability đã hỗ trợ. Rust nhận bounds/extent độc lập từ adapter, tính theo
frame Gumball. Với pivot nằm trong bounds, Scale mỗi trục neo tại biên âm
của extent trục đó, hai tọa độ còn lại theo pivot; nét chấm nối về pivot.
Đây là thuật toán OM9 tái hiện vị trí trong ảnh, chưa phải thuật toán nội bộ
Rhino đã chứng minh cho mọi representation.

Pivot ngoài bounds hoặc extent bằng zero cần fallback được ghi rõ: giữ phía
âm và ít nhất 12 logical px tách khỏi origin để có thể pick; offset hiển thị
không được đổi pivot, factor hay geometry. Tại Begin phải có reference extent
hữu hạn và factor ban đầu bằng 1; trường hợp không đủ dữ liệu thì báo lý do
và không mở transaction. Grip ngoài viewport được camera clip, không kéo cả
bộ Scale về gần tâm. Relocate phải cập nhật lại bounds/frame/leader ở mọi ô.

Ở World và camera orthographic chuẩn, hiển thị như sau; xét camera thực tế,
không suy từ tên hoặc vị trí ô trong workspace:

| Góc nhìn | Move/Scale | Rotate | Plane |
|---|---|---|---|
| Top (XY) | X, Y | Z | XY |
| Front (XZ) | X, Z | Y | XZ |
| Right (YZ) | Y, Z | X | YZ |
| Perspective | Các trục/cung có projection dùng được | Các cung không suy biến | Các mặt phẳng không suy biến |

Object/CPlane xoay thì chọn handles theo projection của basis hiện hành,
không hard-code bảng World. Trục/cung/mặt phẳng suy biến khi chiếu phải ẩn
cả hình lẫn hit target; không để chúng bắt nhầm Origin. Guides và preview
không pickable, không góp vào FitAll bounds. Hình nhỏ có vùng pick khoảng
5 logical px; khi chồng nhau ưu tiên menu/origin rồi Scale, Move, Plane,
Rotate. ID operation vẫn ổn định dù một số handle bị ẩn ở camera cụ thể.

### Chuột, phím và nhập số

| Tương tác | Kết quả yêu cầu |
|---|---|
| Drag mũi tên / plane / origin | Move theo một trục / hai trục / tự do; preview đồng bộ bốn ô |
| Drag cung / ô Scale | Rotate quanh pivot / Scale1D; kéo Scale ra xa pivot làm nở, kéo gần làm co |
| Giữ Shift khi Scale | Scale3D đồng đều theo cùng pivot; nhả Shift trở lại Scale1D trong cùng phiên |
| Click tay nắm | Editor số gọn cạnh điểm click, nằm trong ô đó; distance theo unit document, góc độ, factor không đơn vị |
| Alt tap sau khi bắt đầu drag | Toggle Copy; tap lần nữa tắt, bỏ qua auto-repeat; giữ nguyên nguồn khi Copy |
| Alt + click rồi nhập số | Áp dụng trị số cho bản sao; chỉ một batch Copy |
| Gõ số + Enter khi đang giữ drag | Ràng buộc distance hoặc bước góc trên Command, tiếp tục preview; mouse release mới commit |
| Ctrl có từ mouse-down | Chỉnh widget theo `OM9-INFO-022`, không dịch/scale/xoay geometry |
| Ctrl thêm sau khi drag arrow, còn giữ lúc release | Extrude planar curve/surface có capability; không áp dụng ngầm cho solid, arc hoặc Scale handle |
| Esc | Đóng numeric editor trước nếu có; Esc tiếp hủy phiên và trả trạng thái trước gesture ở mọi ô |

Distance/angle/factor phải hữu hạn, factor khác zero; dấu âm và unit theo
parser/operation policy hiện hữu. Input sai không commit. Thay selection,
source incarnation/revision, units, permission hoặc frame trong gesture
hủy hoặc reject batch trước publication; không commit theo snapshot stale.

### Menu và tùy chọn Gumball lõi

Menu ring có Relocate, Reset và Align CPlane/Object/World; trạng thái checked
đồng bộ giữa menu và Command. Reset trả placement/orientation mặc định của
selection theo alignment, không reset toàn bộ settings hoặc geometry.
Popup nhận mouse release đúng chủ; đóng popup không nuốt click transform sau.

Mục tiêu bổ sung trong `OM9-INFO-020`: Snappy/Smooth Dragging, Drag Strength
(100% bình thường; finite >0, Reset về 100), Settings và Rotate View Around
Gumball. Snappy dùng Osnap đã bật, Smooth bỏ qua Osnap; default Smooth là
lựa chọn OM9 giữ drag không snap hiện hành, không suy từ ảnh. Rotate View
Around Gumball chỉ đổi tâm orbit của camera thao tác, không tạo Undo geometry.
Settings quản lý màu, kích thước screen-space, độ dày, vị trí plane/menu;
zero ở kích thước Scale/Plane/Menu hoặc độ dày Move/Rotate tắt đúng nhóm đó.
Restore Defaults chỉ reset settings Gumball. Tùy chọn chưa có adapter phải
disabled có lý do, không dựng menu hoạt động rỗng rồi coi là đã hỗ trợ.

### Ranh giới triển khai

Một Rust session sở hữu Enabled, alignment/frame, extents, modifiers,
validation và operation plan. Native Qt/Coin giữ collection overlay theo
identity/generation của từng view, capture camera/ray/pick và dựng primitives
từ cùng snapshot; FreeCAD/OCCT adapters giữ geometry/transaction. Không giữ
raw native pointer quá lifetime trong Rust hoặc viết lại transform policy
riêng cho từng viewport. Áp dụng `RCORE-01/02/03/04/05/06/10/12` theo phạm vi
owner, units, coordinates, command, pick, selection, persistence và display;
một slice đạt không chứng nhận toàn bộ các chapter đó.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `selected_objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Enabled` | `boolean` | Hiện gizmo đồng thời ở các viewport | Default OM9 Off cho profile mới; giữ preference đã lưu; Command/menu cùng handler. |
| `Operation` | `choice` | Move, Rotate, Scale hoặc Extrude | Theo tay nắm và modifiers; không tự Extrude object không có capability. |
| `Axis` | `choice` | X, Y, Z, Plane hoặc Free | Theo tay nắm trong frame chung; không có trục mặc định độc lập camera. |
| `Distance` | `number` | Dịch/extrude theo unit document | Hữu hạn, có dấu; input không có unit dùng document units; không tự điền distance. |
| `Angle` | `number` | Góc, độ tại UI | Hữu hạn; khi giữ drag là bước góc; không tự điền góc. |
| `Scale Factor` | `number` | Hệ số không đơn vị | Hữu hạn, khác zero; Begin factor=1, cho phép âm theo capability; numeric phải nhập rõ. |
| `Copy` | `boolean` | Giữ originals | Off khi bắt đầu gesture thường; Alt toggle/Alt numeric theo bảng thao tác. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-INFO-020`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust gizmo operation state và constrained transform/extrude plan; Qt hit-test handles, native adapter placement/shape/extrude, copy giữ gốc.
3. Tích hợp `Gumball` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Axis", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Alt copy không di chuyển gốc; scale X không đổi Y/Z; Ctrl planar curve tạo extrude đúng distance; Esc drag giữ object ban đầu.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

2026-10-12: `implemented` / `validated_supported_slice`; `approved_scope_complete=true`.
Approved OM9 core scope completed against the exact30 native suite: native primitive dimensions/expressions, standard FreeCAD Link/Scale with native OM9 Compound-derived helpers preserving admitted definitions/dependencies/subelements/Body trees and presentation across Undo/Redo/FCStd, bounded native input settling before fresh gestures after Undo with unchanged history and callback cancellation, actual Mesh Ctrl+Shift component picking, typed document Camera/Light physical frames, shared four-view Relocate and process-local cache. Native host limits and unsupported graph/frame cases remain explicit. Clayoo/SmartTargets, Rhino internal roll, full original-host parity, full Render and live host camera widgets are excluded. Stock GitHub Actions CI is pending.

Plugin SHA256 `d2da3db05211a12e5455725fb0007d313ae8d4729b50b1a81b48fc4b2787f1a1`; host SHA256 `98eef899354a8aa84cfa2934e6e1a3c8558c7e6fff3b89c347c06731f06018a8`.
[Final native evidence](../../../docs/validation/2026-10-12-gumball-core-completion.md).

### Approved core closure — 2026-10-12, final native gates passed

Native primitive Scale keeps native types and edits their dimensional properties
when the affine transform is representable: Box/Plane/Ellipsoid/Wedge by owner
axes, Sphere/Torus uniformly, and Cylinder/Cone/Prism with equal radial factors
and a separate axial factor. A bound dimension retains its expression and
dependency by multiplying the expression; coupled changed dimensions,
attachment/bound Placement, oblique primitive limits and unrepresentable owner
frames use an admitted associative result instead of overwriting definitions.

The fallback uses standard FreeCAD `App::Link` and `Part::Scale`, with native
`OpenMatrix9Gui::GumballScaleCompound` helpers derived from `Part::Compound`
for occurrence normalization and legacy Part owners. The helper fixes its
stock non-hiding Part view provider so native Undo/Redo and FCStd reopening
retain the same presentation. It retains source definitions,
supported incoming native links/subelements, group coordinates and Body/Tip
trees. The visible result recomputes from that native graph; Alt creates a
private result while keeping the original and its consumers intact. This is
FreeCAD dependency preservation, not full Matrix History or Clayoo/SmartTargets.
Read-only/foreign-document/unresolved graph cases must reject before mutation;
publication callbacks must abort the whole batch. Native warm/cold checks passed for the final compiled plugin in the exact30 cohort.

Typed `OpenMatrix9Gui::GumballCamera` and `GumballLight` are document objects with
physical Placement and unit-bearing dimensions. Gumball moves/rotates those
objects and scales admitted camera/light parameters. This does not establish
renderer support or parity with the host's live modeling-camera widgets.

[Validated baseline](../../../docs/validation/2026-10-11-gumball-four-view-completion.md).
[Candidate scope and applicability](../../../docs/validation/2026-10-12-gumball-core-release-candidate.md).
[Inputs/units/output/lifecycle and limits](../../../docs/features/OM9-INFO-020.md).
Source design sections and compiled request-planner examples above remain unchanged.

## Acceptance checklist

Các mục `[x]` dưới đây có evidence baseline ngày 2026-10-11 và đã được chạy lại trong cohort30processes cuối ngày 2026-10-12. Các mục mới dẫn trực tiếp đến evidence cuối; không xác nhận mọi representation của Rhino.

- [x] Command `Gumball On/Off` bật/tắt trên binary visual đã kiểm; không đổi geometry/Undo.
- [x] On với selection hợp lệ tạo overlay đồng thời ở cả bốn view; Off dọn cả bốn, không đợi focus.
- [x] Click handle ở ô chưa active bắt đúng ngay lần đầu; giữ operation IDs qua mọi camera.
- [x] Drag/Copy/Extrude ở từng ô đồng bộ preview tất cả ô và chỉ một transaction/batch kết quả.
- [x] Scale neo biên âm bounds trong frame; đổi kích thước object/zoom không biến thành fixed-offset widget.
- [x] Plane hai màu/dấu chia, origin tương phản và menu vòng trắng/đuôi chấm đúng profile ảnh.
- [x] Top/Front/Right/rotated basis/Perspective lọc handles suy biến và pick đúng mục tiêu.
- [x] Kích thước logical px, pick và numeric/menu đúng ở DPI100%/150%/200%, zoom in/out và maximize/restore.
- [x] Numeric Move/Rotate/Scale, Alt Copy, Shift Scale3D và hai thời điểm Ctrl được replay ở mọi ô.
- [x] Relocate/Reset/Alignment ở một ô đồng bộ frame, extents, leaders và hit targets ở các ô còn lại.
- [x] Close view/document, navigation, stale source/units/permission dọn mọi overlay/preview/editor/capture.
- [x] Snappy/Smooth, Drag Strength, Settings/Restore Defaults và orbit around pivot có native evidence.
- [x] Final compiled native primitive dimensions/expressions and associative Link/Scale/Compound graphs: parametric and associative warm/cold fixtures; approved scope complete; exact Rhino parity remains excluded. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] Typed Camera/Light native Move/Rotate/Scale, Object frames, property types and warm/cold FCStd gates. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).

- [x] Hợp đồng và điều kiện hình học được kiểm chứng trong supported native slice; adapter gaps ghi riêng.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] Dated baseline: native Part::Cut recomputes after Box property Scale and Undo.
- [x] Candidate native associative result preserves definitions/dependencies/subelements/Body trees through recompute, Undo/Redo and fresh-process FCStd; Matrix History is outside this scope. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] Undo/redo tested for successful gestures; failed posttransaction callback loses old Redo (host limitation).
- [x] Save/reload persistence tested for native geometry/report/layer/Link/retained 3DM.
- [x] Automated tests use feature ID `OM9-INFO-020`.

## Hợp đồng kỹ thuật chung

Applicability is explicit below. Dated baseline and passing final native gates are
mapped in the [candidate report](../../../docs/validation/2026-10-12-gumball-core-release-candidate.md).

- [x] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): Menu/CMD, numeric input, cancel and modifier timing: dated 2026-10-11 four-view native evidence; final-source replay passed.
- [x] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): Applicable: native identity/topology, typed owner properties and FCStd; baseline evidence exists, new primitive/associative/Camera-Light gates passed. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): Same-document editable ownership, canonical roots and layer permissions: dated ownership/review evidence. New Mesh Ctrl+Shift and dependency graph checks passed.
- [x] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): Shared frame, first-click picks, focus/maximize and source/passive-view cleanup: dated four-view/lifecycle evidence; new-owner replay passed.
- [x] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): Unpickable preview, colors/logical pixels and DPI: dated visual/four-view evidence. New typed Camera/Light view-provider checks passed.
- [x] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): Native End/Mid/Point and source-view rays: dated Relocate pointer evidence. New Mesh Ctrl+Shift identities passed; no full snap-combination claim.
- [x] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): Document units, captured CPlane, w-world, world placement and invalid axes: dated core/four-view evidence; new physical-source frame checks passed.
- [x] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): Applicable: permission, layer/group ownership, native Link definitions, subelements and Body trees; baseline ownership evidence, new graph warm/cold gates passed. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): N/A for document annotations: Gumball creates no text/dimension annotation. Numeric editors are transient viewport UI, with dated DPI evidence.
- [x] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): N/A for a measurement/diagnostic command: no independent analysis output is created. Fixture geometry/volume checks validate transforms only.
- [x] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): Applicable: positive admitted owner transforms, exact affine result, Copy ownership and one atomic batch; final primitive/graph native gates passed. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] Remaining engineering contracts have explicit N/A reasons in the candidate report; Camera/Light names do not certify a renderer.
