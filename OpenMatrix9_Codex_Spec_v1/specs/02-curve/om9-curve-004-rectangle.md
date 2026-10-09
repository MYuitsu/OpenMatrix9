---
id: OM9-CURVE-004
name: Rectangle
command: Rectangle
domain: 02-curve
module: Curve Tools
kind: command
implementation_status: partially_implemented
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-CURVE-004 — Rectangle

Alias tương thích: `Rectangle`. Nhóm: `02-curve`. Trạng thái: `partially_implemented` — supported slice đã kiểm chứng ngày 2026-10-09.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-004` — Rectangle.**

Dựng rectangle từ hai góc đối; có thể nhập chiều dài rồi chọn góc, Shift ràng buộc square. 3Point chọn hai góc kề xác định cạnh đầu và điểm/giá trị width, dùng cho mặt phẳng nghiêng. Vertical dựng mặt phẳng vuông góc CPlane từ một cạnh và width. Center đặt tâm rồi góc/chiều dài. Rounded dựng khung rồi chọn bán kính góc. Corner=Arc tạo bốn đoạn thẳng và bốn cung, Corner=Conic tạo bốn phần conic; giữ khác biệt phân đoạn khi explode. Chuột phải vào nhánh Rounded.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `picked_points` | `point` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `3Point` | `boolean` | Khung từ cạnh và width | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Vertical` | `boolean` | Plane vuông góc CPlane | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Center` | `reference` | Tâm khung | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Rounded` | `boolean` | Bo góc | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Corner` | `choice` | Arc hoặc Conic | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Radius` | `number` | Bán kính góc theo đơn vị document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Length` | `number` | Chiều dài khung theo đơn vị document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Width` | `number` | Chiều rộng khung theo đơn vị document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-004` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Xây hệ trục từ điểm/hướng đã nhập; dựng khung kín rồi giải các góc Arc hoặc Conic và kiểm tra bán kính có nghiệm.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-004` tới command native `Rectangle` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "3Point", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Rounded", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Corner", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Khung 10×6 đúng cạnh và vuông góc; Shift cho cạnh bằng nhau; Rounded Arc có tám phần còn Conic có bốn phần khi tách.
- Cancel ở giữa phiên không thay object/layer; lỗi solver không commit output một phần. Undo/Redo phục hồi toàn bộ thao tác; save/reload bảo toàn geometry, placement và links theo hợp đồng.
- Kiểm tra NaN/Infinity bị từ chối và UI cùng CMD cho cùng kết quả với cùng input; các options chưa có mặc định yêu cầu xác nhận giá trị đang hiển thị.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

### Source evidence

Matrix 8 Book 1, printed pp.127–128 / PDF pp.137–138, was read on 2026-10-09
from `OpenMatrix9_Codex_Spec_v1/matrix8_book_1.pdf`. Rectangle begins on PDF137;
its Rounded options continue on PDF138 before Circle. The source describes
two opposite corners, a numeric length followed by a corner, Shift square,
3Point from an edge and opposite side/width, Vertical, Center and Rounded.
Rounded Arc and Conic have different decomposition; neither is implemented
by this slice. Exact Matrix defaults for numeric Center dimensions are not
established by those pages.

### OpenMatrix9 decisions

- Start in Corners mode, no fixed Length or Width, sharp corners. Choose
  `3Point`, `Vertical`, `Center` or `Corners` before the first point.
- Inputs are world points from mouse/snaps; CMD `x,y[,z]` and `rX,Y[,Z]` use
  the construction frame. The frame is latched at the first point. Modes and
  dimensions are reset on a new command. No preselection is required.
- Units are mm, with explicit `mm`, `cm`, `in` suffixes. Finite coordinates
  and output corners must stay within ±1e9 mm. Each side is at least 1e-7 mm.
- Corners uses CPlane X/Y components of the diagonal, in a plane parallel to
  the latched CPlane through the first corner. Off-plane second-point height
  is projected out. Center reflects the corner around the picked center;
  entered Length/Width are full side dimensions.
- `Length=N` / `L=N` is positive. `Width=N` / `W=N` is signed and nonzero.
  Bare `Length` or `Width` requests its value. In Corners/Center, after the
  first pick a bare number fixes Length; another fixes Width and completes
  the frame. Edge modes still require a second pick for edge direction;
  after that pick a bare number sets Width.
  Length followed by an opposite-corner pick uses the pick's X sign/Y width.
  Explicit Width sets the side direction by its sign.
- 3Point uses the first two picks as an edge; the opposite side is the third
  pick's component perpendicular to that edge, allowing arbitrary slant.
  Numeric width uses CPlane normal × edge, with CPlane Y as the fallback
  when the edge is parallel to the normal.
- Vertical projects the first edge onto the latched CPlane, then uses its
  normal as the width direction. After two picks, mouse rays intersect the
  vertical session plane; change camera or type width if the ray is parallel.
- Shift makes both sides equal, independently of global Ortho. With a fixed
  Length, it uses that Length; otherwise corner mode uses the larger diagonal
  component and edge modes use the edge length. A pre-entered Width also
  obeys Shift during the final mouse pick and matching preview.

### Native output and lifecycle

Rust owns mode, dimensions, geometry validation, constraints and session state.
C++ adapts the resulting five closed polygon coordinates through the typed
Part API. One root `Part::Feature` contains one valid closed wire with exactly
four straight edges and `OM9FeatureId=OM9-CURVE-004`. No face/solid is implied.
Menu, Curve sidebar and CMD share the handler; the F6 CurveLayout entry uses
the same command.

The Coin preview displays the exact pending outline and picked-point markers
without document objects or transactions. Success creates one object in one
transaction and enters command history only after commit. Invalid coordinates,
degenerate geometry and unsupported options keep the input phase editable.
Native adapter/kernel failures abort the transaction and end the session with
an error; restart Rectangle to retry such a failure.
Undo removes the most recent picked point and clears dimension constraints;
Esc/Cancel, workbench deactivation and document changes clear the preview and
session. Idle prompts return to `Command:` after commit/cancel.

Output is a geometric snapshot with no linked source dependencies; Matrix
History recomputation and active-layer inheritance are unsupported. FreeCAD
Undo/Redo and FCStd save/reload preserve the snapshot and exact feature ID.
Output uses the shared Curve visibility/color policy.

### Validation

- `rust/tests/rectangle_session.rs`: geometry, slant, signed dimensions,
  rotated construction frame, units, square, equivalent preview/click,
  recoverable invalid input, overflow rejection, Undo and Cancel.
- `rust/tests/curve_command_permissions.rs`: document-write permission.
- `tests/rectangle_smoke.FCMacro`: native menu/CMD/sidebar, four-edge wire,
  mouse preview/square/Vertical, errors, Undo/Redo, lifecycle, persistence.

Runtime evidence and build details: `docs/validation/2026-10-09-rectangle.md`.
Native results are recorded there after execution; source/sample presence
does not establish runtime validation.

### Remaining work

Rounded Arc/Conic, the Rounded right-click action, radius picking, active-layer
integration, source-edit History, and exhaustive Matrix/Rhino compatibility.
Unsupported option input reports an error and does not silently create a sharp
rectangle. The next feature in this Curve sequence is OM9-CURVE-005 Circle.

## Acceptance checklist

Các dấu kiểm chỉ áp dụng cho sharp Rectangle slice nêu trên. History source-edit
không áp dụng cho output snapshot; các hợp đồng kỹ thuật chung chưa được
nghiệm thu đầy đủ vẫn để trống.

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] History/dependencies tested if applicable.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-CURVE-004`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CURVE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CURVE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CURVE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CURVE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CURVE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-CURVE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-CURVE-004` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
