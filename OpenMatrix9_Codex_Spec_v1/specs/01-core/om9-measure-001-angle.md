---
id: OM9-MEASURE-001
name: Angle
command: Angle
domain: 01-core
module: Measure
kind: command
implementation_status: partially_implemented
implementation_validation_status: validated_supported_slice
implementation_verified_at: 2026-10-09
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MEASURE-001 — Angle

Alias tương thích: `Angle`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-MEASURE-001` — Angle.**

Angle thu hai cặp endpoints xác định hai lines rồi báo angle feedback, không tạo annotation. TwoObjects nhận hai planar surfaces hoặc hai lines; dùng normals/hướng có orientation policy rõ để chọn angle chính/supplement. Snaps cho points, vector zero không hợp lệ.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `picked_points` | `point` | 4 | 4 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `TwoObjects` | `boolean` | Đo từ hai objects | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-MEASURE-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Native vector/plane angle evaluation, Rust point/object branch and unit formatting.
3. Tích hợp `Angle` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-001",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 4, max: Some(4) },
    ],
    parameters: &[
        Parameter { name: "TwoObjects", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Hai vectors vuông cho 90 độ; zero-length bị từ chối; TwoObjects surfaces tính từ normals theo policy; không thêm objects.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Supported slice:** native four-point Angle measurement, shared Cartesian point input and accepted-coordinate reporting. The TwoObjects branch remains unimplemented.

### Inputs

Enter or pick the start/end of the first directed line, then the start/end of the second. No geometry selection is required. Typed input uses the shared Rust parser: CPlane `x,y` / `x,y,z`, world `wX,Y,Z`, relative CPlane `r`, `R` or `@`, and relative world `wr`. Relative input starts from the last accepted world point across all four phases. Explicit length units and the captured document input scale are supported. Native picks, available snaps and F4 origin are already world coordinates and are not transformed a second time.

### Parameters and defaults

The supported branch reports degrees from 0 through 180 between the two directed line vectors. Both line lengths must exceed `1e-12` mm; world coordinate components must be finite and within ±`1e9` mm. `TwoObjects` and `Unit=` currently return an explicit unsupported-option message. Object selection, planar-face normals and object orientation/principal-versus-supplement behavior from the full contract remain open.

### Output

The final Angle result and each accepted point in world millimeters appear in command/report history. A completed, owned coordinate report is also exposed through the main-window `om9MeasurementReport` string property. The command creates no document object, dimension or annotation and does not modify document geometry, selection, units or Undo state.

### Preview / commit / cancel

The command retains accepted inputs until the fourth valid point. A rejected zero-length endpoint keeps the current phase and previous accepted point; failed inputs do not publish a completed report. The command-start CPlane is latched. Document owner/UID and the exact captured unit context are checked before input acceptance and completion; document switch/close or an invalidated context ends the session. Esc, command replacement and workbench cancellation clear pending measurement state.

### History / dependency model

Command/report history and the owned UI report record accepted inputs, frame revision, logical view slot, units and source kind. They are not an associative History feature, persisted measurement object, or dependency link to snapped geometry. Snap object identity is explicitly unavailable in the present report schema.

### Error / invalid-input behavior

The shared parser rejects malformed, ambiguous, non-finite and overflowing input without advancing accepted state. Rust computes the angle; the native adapter manages FreeCAD/Qt lifecycle and reporting. The final matching-module fixtures verify failed retries, lifetime and read-only invariants. Broader grammar and TwoObjects remain open.

Validation: [shared point-consumer report](../../../docs/validation/2026-10-09-rhino-core-point-consumers.md) and its hash-bound evidence. Status applies only to the supported slice above.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-MEASURE-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-MEASURE-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
