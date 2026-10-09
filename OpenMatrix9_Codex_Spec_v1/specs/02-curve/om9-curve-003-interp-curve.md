---
id: OM9-CURVE-003
name: Interp Curve
command: InterpCrv
domain: 02-curve
module: Curve Tools
kind: command
implementation_status: partially_implemented
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-06'
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-CURVE-003 — Interp Curve

Alias tương thích: `InterpCrv`. Nhóm: `02-curve`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-003` — Interp Curve.**

Thu điểm đi qua curve rồi Enter; điểm nhập là điểm nội suy, không phải control point. Đóng gần điểm đầu hoạt động cả khi snaps tắt; Alt đình chỉ. Degree điều khiển bậc; phạm vi tương thích được mô tả là 1–11, nhưng chính sách mẫu không áp đặt phạm vi chưa có solver. Knots chọn Uniform, Chord hoặc SqrtChrd; Chord là lựa chọn ban đầu được mô tả. Dùng khoảng tham số đều, theo độ dài dây hoặc căn bậc hai độ dài dây; không diễn giải Uniform thành khoảng cách vật lý 1 mm. Start Tangent và End Tangent nhận hướng hoặc curve tham chiếu. PersistentClose cập nhật dạng kín khi tiếp tục thêm điểm. Close tạo nối tuần hoàn trơn; Sharp tạo nối không tuần hoàn có góc; Undo bỏ lần nhập cuối. Quan hệ trái/phải của hình minh họa không dùng để xác định Sharp.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `picked_points` | `point` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Degree` | `number` | Bậc curve | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Knots` | `choice` | Uniform, Chord hoặc SqrtChrd | Chord — hành vi khởi tạo được mô tả; mẫu operation plan không tự điền khi thiếu. |
| `PersistentClose` | `boolean` | Giữ kín khi bổ sung | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Start Tangent` | `reference` | Hướng tiếp tuyến đầu | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `End Tangent` | `reference` | Hướng tiếp tuyến cuối | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Close` | `boolean` | Đóng periodic | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Sharp` | `boolean` | Đóng có kink | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Undo` | `boolean` | Bỏ thao tác nhập cuối | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-003` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Nội suy B-spline với phân bố tham số được chọn và điều kiện tiếp tuyến, phân biệt closure periodic và kink trong dữ liệu Rust.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-003` tới command native `InterpCrv` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-003",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "Knots", kind: ParameterKind::Choice, required: false },
        Parameter { name: "PersistentClose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Start Tangent", kind: ParameterKind::Reference, required: false },
        Parameter { name: "End Tangent", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sharp", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Điểm nội suy nằm trên kết quả trong dung sai; điểm không đều cho Chord khác Uniform; Close trơn tại seam còn Sharp cho phép gián đoạn tiếp tuyến.
- Cancel ở giữa phiên không thay object/layer; lỗi solver không commit output một phần. Undo/Redo phục hồi toàn bộ thao tác; save/reload bảo toàn geometry, placement và links theo hợp đồng.
- Kiểm tra NaN/Infinity bị từ chối và UI cùng CMD cho cùng kết quả với cùng input; các options chưa có mặc định yêu cầu xác nhận giá trị đang hiển thị.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-06. **Scope:** Native B-spline interpolation, Degree/Knots, smooth/sharp close, AutoClose/Alt, preview, Undo; tangent constraints remain unsupported.

Command `InterpCrv`, icon `CurveFreeFormInterpolatePoints`, Curve menu.
Implemented slice: native point interpolation, parameterization, closure and
command lifecycle. This is not a claim of complete Matrix/Rhino compatibility.

### Evidence

Requested catalog: `OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-003-interp-curve.md`.
The former `ref/matrix9/` package is now outside the public source layout.
Catalog evidence describes point picking, Enter, automatic close and Alt.
`TODO_EVIDENCE`: the registered PDF path was unavailable during implementation;
been re-checked against it. The next
that boundary, not a verified manual read. Original references are preserved.

Supplementary primary documentation, read 2026-10-06:
[Rhino 5 InterpCrv](https://docs.mcneel.com/rhino/5/help/en-us/commands/interpcrv.htm).
It supports the meanings of Uniform, Chord, SqrtChrd, smooth periodic Close,
nonperiodic Sharp, Undo and Alt suspension. This is supplemental evidence,
not recovery of Matrix defaults.

### OpenMatrix9 decisions

- Inputs: ordered world points, entered through the shared command frame or
  viewport picker. CMD coordinates/relative coordinates use the active CPlane;
  mouse coordinates use the existing native snaps and Ortho/Shift rules.
  Selection is not required; unrelated selected objects remain untouched.
- Defaults: degree 3, Uniform knots, PersistentClose=No; millimetres. Accepted
  degrees are 1, 3, 5, 7, 9, 11. With too few points, open degree is reduced to
  `min(requested, points-1)`; periodic degree is additionally reduced to odd.
  This bounded support is a host decision, not a verified Matrix degree range.
- Limits: at most 256 picked points, finite coordinates within 1e9 mm,
  consecutive points at least 1e-7 mm apart. Failed/ill-conditioned fits give an
  editable error, never nonfinite geometry.
- Close: at least three picked points; creates a periodic B-spline. AutoClose
  uses a 10 logical-pixel radius around the first point and works without Osnap.
  Alt suspends it. Sharp repeats the start point in an open knot-vector fit to
  create a closed, nonperiodic curve. Enter commits the current open fit unless
  PersistentClose is enabled. PersistentClose previews a closed curve from three
  distinct points; the two-point Rhino variant is not implemented.
- Degree/Knots support both `Option=value` and pending value prompts; Undo removes
  the last picked point. Esc/Cancel removes temporary geometry. StartTangent and
  EndTangent are explicitly unsupported and return an error.
- Output: one valid native `Part::Feature` with one non-rational B-spline edge,
  constructed by Part/OpenCascade from Rust's poles/knots/multiplicities.
  Output uses document-root ownership and the established Curve line color.
- Preview is unpickable scene geometry in the document's 3D views. It uses the
  same fit and explicit close state as commit, including Alt and CMD repeated
  endpoints. It does not add a document object or transaction.
- Commit is one document transaction. Undo/Redo and FCStd serialization use
  native FreeCAD geometry. Menu, CMD and repeat share the same session. Success
  history is recorded only after geometry commit. Changing/replacing/closing the
  document or leaving the workbench cancels the session.
- Builder/Styles/associative History are not used: this slice outputs standalone
  geometry. Unrelated surface, solid, trim, volume and selection-window foundation
  requirements do not apply to this point-drawing command.

### Validation and continuation

`rust/tests/spline_geometry.rs`, `rust/tests/curve_session.rs` and
`rust/tests/curve_command_permissions.rs` cover interpolation, closed seam,
invalid values, option flow, preview/commit consistency and write permissions.
`tests/curve_spline_smoke.FCMacro` tests actual menu/CMD/mouse events, native
interpolation within 1e-7 mm, AutoClose/Alt, cancellation and persistence.
Existing `tests/curve_smoke.FCMacro` protects Line/Polyline behavior.

Remaining compatibility: Matrix manual/default verification, tangent workflows,
two-point PersistentClose, specialist foundation workflows beyond the existing

Implementation record: [docs/features/OM9-CURVE-003.md](../../../docs/features/OM9-CURVE-003.md).

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented (OpenMatrix9 choices; giá trị tương thích còn chưa xác định).
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested for the supported slice.
- [x] History/dependency scope documented; native snapshot persistence tested.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-CURVE-003`.


Kiểm chứng đầy đủ hợp đồng và tương thích hình học vẫn chưa hoàn tất.
Checked items apply only to the supported implementation slice above.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CURVE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CURVE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CURVE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CURVE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CURVE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-CURVE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-CURVE-003` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
