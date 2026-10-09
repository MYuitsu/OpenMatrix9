---
id: OM9-MEASURE-002
name: Distance
command: Distance
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

# OM9-MEASURE-002 — Distance

Alias tương thích: `Distance`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-MEASURE-002` — Distance.**

Distance chọn hai points và báo khoảng 3D trong feedback/history. Unit đổi unit display; Model_Units là khởi tạo được mô tả, ngữ cảnh thường mm nhưng không hardcode model luôn mm. Snaps/ProjectOsnap ảnh hưởng points input nên phải báo coordinates đã nhận.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `picked_points` | `point` | 2 | 2 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Unit` | `choice` | Model_Units hoặc unit hiển thị được hỗ trợ | Model_Units — theo document, không hardcode mm. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-MEASURE-002`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust point parser/unit formatter và native Euclidean distance, không thay document unit.
3. Tích hợp `Distance` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MEASURE-002",
    effect: Effect::ReadOnly,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Unit", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Points (0,0,0)/(3,4,0) báo 5 model units; chọn inch đổi số display; model unit không bị sửa.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Supported slice:** native two-point 3D Distance measurement, five explicit display units, shared Cartesian point input and accepted-coordinate reporting. The contract's Model_Units display default remains open.

### Inputs

Enter or pick two points without requiring geometry selection. The shared Rust parser supports CPlane `x,y` / `x,y,z`, world `wX,Y,Z`, relative CPlane `r`, `R` or `@`, and relative world `wr`. Explicit coordinate units and the command-start document input scale are supported; relative input uses the last accepted world point. Native picks, available snaps and F4 origin arrive in world millimeters and bypass typed-coordinate transforms.

### Parameters and defaults

`Unit=mm`, `cm`, `m`, `in` and `ft` select the result display unit only. Each command currently starts in `mm`; changing this option does not change coordinate interpretation or document units. The specified `Model_Units` choice/default is not implemented, so the original default requirement remains unsatisfied. Coordinate components must be finite and within ±`1e9` mm. Coincident points are valid and return zero.

### Output

Rust computes Euclidean 3D distance. The final result and both accepted world coordinates appear in command/report history; a completed owned input report is published in the main-window `om9MeasurementReport` string property. No document object, annotation, geometry edit or Undo entry is created.

### Preview / commit / cancel

The first accepted point is retained through rejected coordinate or display-unit input. Only a valid second point publishes the completed result/report. The CPlane remains latched from command start; document owner/UID and exact unit context are rechecked before acceptance and reporting. Document switch/close, stale context, Esc, command replacement or workbench cancellation clears pending state. A failed attempt does not replace the previous completed report.

### History / dependency model

The coordinate report owns accepted input text, resolved points, units, source kind, frame revision and logical view slot. It is session/UI reporting, not persisted document History or an associative link to measured geometry. Snap object identity is explicitly unavailable; no replay dependency is inferred from the source kind.

### Error / invalid-input behavior

Malformed, ambiguous, non-finite and overflowing coordinates are rejected by the shared Rust parser. Invalid `Unit=` values retain the accepted first point and current supported unit. Final matching-module fixtures verify unit separation, frame latch, same-name document replacement and unchanged model/Undo state. Model_Units and broader grammar remain open.

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
- [ ] Automated tests use feature ID `OM9-MEASURE-002`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-MEASURE-002` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
