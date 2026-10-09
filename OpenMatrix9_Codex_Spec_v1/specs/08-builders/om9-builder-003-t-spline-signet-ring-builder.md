---
id: OM9-BUILDER-003
name: T-Spline Signet Ring Builder
command: gvtsSignet
domain: 08-builders
module: Builders
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-BUILDER-003 — T-Spline Signet Ring Builder

Alias tương thích: `gvtsSignet`. Nhóm: `08-builders`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvtsSignet tạo signet kiểu T-Spline, Start cho preview wireframe sau chọn Ring Rail hoặc +RingRail. Shank Thickness điều chỉnh 3/9 giờ; Top Thickness từ rail đến đỉnh; Top Width theo X, Top Depth theo Y tính mm; Top Width Corner/Top Depth Corner dịch góc đỉnh theo X/Y; Middle Thickness ở thân trên sát rail; Bottom Thickness ở đáy; Top Shank Thickness ở 2/10 giờ. Kéo handle và CMD đồng bộ. Cấu trúc T-Spline phải tách khỏi nhẫn signet NURBS và Clayoo.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `RingRail` | `curve` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Shank Thickness` | `number` | Độ dày thân ở 3/9 giờ. | Chưa xác định; không tự gán giá trị. |
| `Top Thickness` | `number` | Khoảng từ rail đến đỉnh. | Chưa xác định; không tự gán giá trị. |
| `Top Width` | `number` | Kích thước top theo X, mm. | Chưa xác định; không tự gán giá trị. |
| `Top Depth` | `number` | Kích thước top theo Y, mm. | Chưa xác định; không tự gán giá trị. |
| `Top Width Corner` | `number` | Dịch góc đỉnh theo X. | Chưa xác định; không tự gán giá trị. |
| `Top Depth Corner` | `number` | Dịch góc đỉnh theo Y. | Chưa xác định; không tự gán giá trị. |
| `Middle Thickness` | `number` | Độ dày thân trên sát rail. | Chưa xác định; không tự gán giá trị. |
| `Bottom Thickness` | `number` | Độ dày đáy. | Chưa xác định; không tự gán giá trị. |
| `Top Shank Thickness` | `number` | Độ dày vùng 2/10 giờ. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-BUILDER-003` và các vai trò RingRail; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Sinh cage của nhẫn với các vùng điều khiển độc lập; nếu chưa có adapter T-Spline hợp lệ thì chỉ cung cấp mô hình kế hoạch, không thay bằng Clayoo âm thầm.
2. C++/Qt: đăng ký và điều phối native command `gvtsSignet`; chuyển kế hoạch T-Spline Signet Ring Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-BUILDER-003`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Shank Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width Corner", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Depth Corner", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Shank Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Tăng Top Width phải đổi X, Top Depth phải đổi Y; đổi Top Shank Thickness chỉ tác động vùng 2/10 giờ theo thiết kế cage đã kiểm chứng.
- OM9-BUILDER-003: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

> This section is intentionally separate theo hợp đồng feature-derived material. Fill it when implementing this feature. Các hành vi chưa xác định phải được ghi rõ.

### Inputs

- TODO: normalize supported object types and required selection state theo hợp đồng feature.

### Parameters and defaults

- TODO: verify exact semantics, units, ranges, and defaults theo hợp đồng feature.

### Output

- TODO: define resulting OpenMatrix9 object(s), geometry type, and ownership/history relationships.

### Preview / commit / cancel

- TODO: define interactive lifecycle only where supported by source.

### History / dependency model

- TODO: verify whether this feature records/uses History and define the OpenMatrix9 dependency graph.

### Error / invalid-input behavior

- TODO: define explicit errors and no-op/cancel cases.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-BUILDER-003`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDER-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDER-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDER-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDER-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDER-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-BUILDER-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-BUILDER-003` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
