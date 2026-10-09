---
id: OM9-BUILDER-007
name: Freeform Knot Builder
command: null
domain: 08-builders
module: Builders
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-BUILDER-007 — Freeform Knot Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `08-builders`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Freeform Knot nhận Curves To chồng/giao nhau rồi split tại giao; ba nhóm Input Curves phân loại crossover, loop X và loop Y. Apply sinh knot, kết quả cần preview vì cách kết nối phức tạp. Không có handle viewport; Profile Width/Height, Profile Spacing (nhỏ cho kiểm soát nhiều hơn), Profile Height at Knot và Center Height at Knot điều khiển hình và khe giao. Nurbs Profile chọn profile mượt, Keep Curve giữ đường vào, Select Knot chọn kết quả cuối và Undo hoàn tác.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Curves` | `curve` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Input Curves` | `reference` | Ba nhóm segments crossover, loop X và loop Y sau split. | Chưa xác định; không tự gán giá trị. |
| `Profile Width` | `number` | Bề rộng profile. | Chưa xác định; không tự gán giá trị. |
| `Profile Height` | `number` | Độ cao profile. | Chưa xác định; không tự gán giá trị. |
| `Profile Spacing` | `number` | Khoảng giữa các sections; nhỏ cho nhiều sections. | Chưa xác định; không tự gán giá trị. |
| `Profile Height at Knot` | `number` | Độ cao profile tại giao knot. | Chưa xác định; không tự gán giá trị. |
| `Center Height at Knot` | `number` | Độ cao tâm tại giao knot. | Chưa xác định; không tự gán giá trị. |
| `Nurbs Profile` | `boolean` | Dùng profile mượt. | Chưa xác định; không tự gán giá trị. |
| `Keep Curve` | `boolean` | Giữ input curves sau commit. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-BUILDER-007` và các vai trò Curves; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Dựng graph sau split, lưu phân loại mỗi segment và giải tuyến over/under; sweep section biến đổi ở giao, chỉ xóa curves sau commit khi Keep Curve tắt.
2. C++/Qt: đăng ký và điều phối native command `OM9-BUILDER-007`; chuyển kế hoạch Freeform Knot Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-BUILDER-007`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Input Curves", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Height at Knot", kind: ParameterKind::Number, required: false },
        Parameter { name: "Center Height at Knot", kind: ParameterKind::Number, required: false },
        Parameter { name: "Nurbs Profile", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Keep Curve", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Hai đường giao phải được split; đưa segment vào nhóm X/Y phải thay hướng loop; Keep Curve bật giữ đường gốc, Cancel giữ nguyên tất cả.
- OM9-BUILDER-007: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-BUILDER-007`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-BUILDER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-BUILDER-007` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
