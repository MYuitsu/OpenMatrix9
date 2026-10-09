---
id: OM9-BUILDER-006
name: Knot Builder
command: null
domain: 08-builders
module: Builders
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-BUILDER-006 — Knot Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `08-builders`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Knot Builder tạo knot Celtic bằng lưới: trái đặt thanh xanh cho giao chéo, phải đặt loop không chéo, ô trắng không knot; preview cửa sổ riêng, Apply mới tạo trong viewport. Không có handle viewport. Knot Size X/Y mm; Knot Height là cao profile; Cap Height ảnh hưởng bevel/chamfer; Knot Width; Corner Scale khoảng góc trong/ngoài; Cross Over Knot Height và Cross Over Cap Height điều chỉnh profile giao; Cross Over Pass Height là khe trên/dưới; Corner Angle độ. Nurbs Profile làm mượt hơn polyline; Center Knot căn F4; Select chọn knot cuối, Undo, Reset, Save/Open pattern.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Pattern` | `selection` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Knot Size X` | `number` | Kích thước lưới knot theo X, mm. | Chưa xác định; không tự gán giá trị. |
| `Knot Size Y` | `number` | Kích thước lưới knot theo Y, mm. | Chưa xác định; không tự gán giá trị. |
| `Knot Height` | `number` | Độ cao profile knot. | Chưa xác định; không tự gán giá trị. |
| `Cap Height` | `number` | Độ cao bevel/chamfer của cap. | Chưa xác định; không tự gán giá trị. |
| `Knot Width` | `number` | Bề rộng profile knot. | Chưa xác định; không tự gán giá trị. |
| `Corner Scale` | `number` | Khoảng góc trong/ngoài knot. | Chưa xác định; không tự gán giá trị. |
| `Cross Over Knot Height` | `number` | Độ cao profile tại crossover. | Chưa xác định; không tự gán giá trị. |
| `Cross Over Cap Height` | `number` | Độ cao cap tại crossover. | Chưa xác định; không tự gán giá trị. |
| `Cross Over Pass Height` | `number` | Khe trên/dưới tại crossover. | Chưa xác định; không tự gán giá trị. |
| `Corner Angle` | `number` | Góc corner, độ. | Chưa xác định; không tự gán giá trị. |
| `Nurbs Profile` | `boolean` | Profile mượt thay polyline. | Chưa xác định; không tự gán giá trị. |
| `Center Knot` | `boolean` | Căn knot về F4. | Chưa xác định; không tự gán giá trị. |
| `Pattern` | `reference` | Lưới cells crossing/loop do người dùng tạo. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-BUILDER-006` và các vai trò Pattern; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Chuyển lưới thành graph tuyến over/under, sweep profile có cao và khe riêng ở giao; serialize pattern thay vì ảnh preview.
2. C++/Qt: đăng ký và điều phối native command `OM9-BUILDER-006`; chuyển kế hoạch Knot Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-BUILDER-006`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-006",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Pattern", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Knot Size X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Knot Size Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Knot Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Knot Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cross Over Knot Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cross Over Cap Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cross Over Pass Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Nurbs Profile", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Center Knot", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pattern", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Đổi ô từ crossing sang loop phải đổi graph; Cross Over Pass Height đổi khe giao, Center Knot đưa tâm về F4; Save/Open giữ toàn lưới.
- OM9-BUILDER-006: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-BUILDER-006`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDER-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDER-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDER-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDER-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDER-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-BUILDER-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-BUILDER-006` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
