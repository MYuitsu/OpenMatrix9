---
id: OM9-CURVE-009
name: Rebuild
command: Rebuild
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

# OM9-CURVE-009 — Rebuild

Alias tương thích: `Rebuild`. Nhóm: `02-curve`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-009` — Rebuild.**

Chọn curve để dựng lại bằng PointCount và Degree, hiển thị số hiện tại bên cạnh số mới. Phân bố lại control point và tạo một curve đơn, không phải polycurve có thể explode. Preview phải tính lại sau thay đổi tùy chọn, báo Maximum deviation và dấu điểm sai lệch lớn nhất. DeleteInput quyết định giữ curve gốc; Create new object on current layer chọn layer hiện hành hoặc layer từng input. Bậc cần đủ số control point; ít điểm thường làm shape giãn nên phải đo sai lệch. Các số trong hộp minh họa không là mặc định và không là ngưỡng chấp nhận.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `input_curves` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `PointCount` | `number` | Số control point | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Degree` | `number` | Bậc curve | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `DeleteInput` | `boolean` | Xóa gốc sau thành công | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Create new object on current layer` | `boolean` | Layer hiện hành thay layer input | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Preview` | `boolean` | Tính preview và độ lệch | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-009` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Lấy mẫu curve theo miền tham số, fit BSpline với số pole/bậc yêu cầu và tính đánh giá sai lệch có kiểm soát giữa các mẫu.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-009` tới command native `Rebuild` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Create new object on current layer", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Rebuild giảm số pole vẫn báo độ lệch; đổi PointCount phải refresh preview; cancel không xóa gốc dù DeleteInput đã bật.
- Cancel ở giữa phiên không thay object/layer; lỗi solver không commit output một phần. Undo/Redo phục hồi toàn bộ thao tác; save/reload bảo toàn geometry, placement và links theo hợp đồng.
- Kiểm tra NaN/Infinity bị từ chối và UI cùng CMD cho cùng kết quả với cùng input; các options chưa có mặc định yêu cầu xác nhận giá trị đang hiển thị.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-06. **Scope:** Curve/wire rebuild, exact PointCount/Degree, sampled preview deviation, batch DeleteInput and persistence; active layers/subedges remain unsupported.

Command `Rebuild`, icon `OthersCurveRebuild`, Curve menu. Implemented slice:
native curve/wire selection, exact degree/pole-count fitting, preview and batch
transactions. Surface Rebuild is a different, unsupported feature.

### Evidence

Requested catalog: `OpenMatrix9_Codex_Spec_v1/specs/02-curve/om9-curve-009-rebuild.md`.
It describes selection, current PointCount/Degree, Preview and OK.
is now present in this package, but p.144 has not been re-checked against it.
catalog boundary; this is not a verified manual-page read.

Supplementary primary documentation, read 2026-10-06:
[Rhino 5 Rebuild](https://docs.mcneel.com/rhino/5/help/en-us/commands/rebuild.htm).
It supports specified control-point count/degree, evenly spaced knots,
DeleteInput, refreshed Preview and reported deviation. It does not establish
Matrix defaults or the OpenMatrix9 fitting algorithm.

### OpenMatrix9 decisions

- Select whole native curve objects before invocation, or select after invocation
  and press Enter. Accept single-edge curves and one connected, complete wire
  per object, including polylines; process at most 16 objects in a batch.
  Reject faces/solids, disconnected curves and subobject selections explicitly.
- PointCount: 2–256, strictly greater than Degree. Degree: 1–11, including even
  degrees. Current B-spline structure is shown per input; analytic/segmented
  curves are labelled separately. Initial target count is the first input's
  pole count clamped to 4–256 (4 if unavailable); initial degree is its degree
  clamped to 1–3. DeleteInput defaults to true. These are host decisions.
- Samples: native OCCT equal-arc-length discretization, `max(513,4*PointCount+1)`
  samples per input in world coordinates. Rust fits a non-rational B-spline with
  uniform knots by bounded least squares; open endpoints are fixed exactly.
  Closed inputs use a periodic fit, with the repeated endpoint removed from the
  solve. This can smooth corners; it does not promise tolerance-constrained refit.
- Preview displays temporary unpickable scene geometry. Maximum deviation is
  explicitly labelled **sampled**: a bidirectional maximum of native nearest
  geometric distances at 129 equal-arc-length points per side. It is an estimate,
  not a certified Hausdorff bound. Changing fit settings clears stale preview
  and deviation. OK recomputes using the current values, even without Preview.
- The current-layer option is visibly disabled because an active layer system
  is not present. Output is created at the document root. Native input line,
  point color and line width are copied. Input-group/layer ownership is not
  claimed to be preserved.
- Accumulate the parent placement when sampling a curve inside App::Part, so
  root-level output and preview retain the displayed world position.
- Before fitting/commit, verify each selected object still exists, its native
  shape is unchanged and its global placement still matches. DeleteInput rejects objects with dependents rather than
  severing their links; clear DeleteInput to keep such inputs.
- Build all results before opening one batch transaction. OK creates one
  `Part::Feature` per input; DeleteInput removes the selected sources only after
  outputs exist. Failure aborts the transaction. Undo/Redo restores the entire
  batch; FCStd saves native splines. No associative History/Builder/Styles model
  is created.
- Invalid options/fits keep the options editable and leave the document intact.
  Cancel/Esc, document switching/deletion and workbench deactivation remove
  preview and the dialog. A rejected selection yields an explicit prompt.
- No point picking, CPlane coordinate entry, snap, window-selection, surface
  trimming or solid-volume behavior is added by Rebuild; existing native
  selection handles object collection.

### Validation and continuation

Rust geometry tests check exact requested structure, uniform knots, endpoint
preservation, periodic circle fit and invalid/ill-conditioned input limits.
`tests/curve_spline_smoke.FCMacro` checks menu/CMD, pre/postselection, preview,
cancel, open/closed/multiple inputs, polyline wires, DeleteInput, invalid degree
and count, Undo/Redo, save/reload and document-switch cancellation.

Remaining compatibility: Matrix manual/default verification, layer mapping,
subedge editing, exact global deviation certification and full original

Implementation record: [docs/features/OM9-CURVE-009.md](../../../docs/features/OM9-CURVE-009.md).

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented (OpenMatrix9 choices; giá trị tương thích còn chưa xác định).
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested for the supported slice.
- [x] History/dependency scope documented; native snapshot persistence tested.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-CURVE-009`.


Kiểm chứng đầy đủ hợp đồng và tương thích hình học vẫn chưa hoàn tất.
Checked items apply only to the supported implementation slice above.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CURVE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CURVE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CURVE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CURVE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CURVE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-CURVE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-CURVE-009` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
