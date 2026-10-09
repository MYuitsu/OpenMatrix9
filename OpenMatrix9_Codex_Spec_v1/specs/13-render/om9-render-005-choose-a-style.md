---
id: OM9-RENDER-005
name: Choose a Style
command: null
domain: 13-render
module: Render Menu
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-RENDER-005 — Choose a Style

Alias tương thích: `Chưa có alias command`. Nhóm: `13-render`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Choose a Style đổi environment, lighting và biến thể materials gán trên metal/gems. Styles Classic, Color Dome, Cut Out White, Plastic Render, Satin Blur, Toon Outline đánh Fast; Falloff, Fifth Ave, Lights Camera Action, Sunroom, White Room đánh Slow; đây là tốc độ tương đối không SLA. Style không chỉ là background.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Style` | `reference` | Identity cấu hình environment/lighting/material variants; tên Classic/Color Dome/... là catalog cần assets được phép dùng, không chỉ background. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-RENDER-005` và các vai trò Document; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Áp preset scene có mapping materials theo identity, không thay geometry; dùng assets tự tạo/cấp phép và capability adapter.
2. C++/Qt: đăng ký và điều phối native command `OM9-RENDER-005`; chuyển kế hoạch Choose a Style sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Giữ thao tác ở phạm vi trạng thái hoặc đầu ra đã chọn. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-RENDER-005`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-005",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Đổi style cập nhật cả scene/lights/material variants; Fast/Slow không cam kết số giây; original material identity vẫn truy được.
- OM9-RENDER-005: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra chuyển chế độ hoặc hủy đầu ra không làm thay đổi hình học.

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
- [ ] Automated tests use feature ID `OM9-RENDER-005`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-RENDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-RENDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-RENDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-RENDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-RENDER](../../ENGINEERING_CONTRACTS.md#om9-render): kiểm chứng cho `OM9-RENDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-RENDER-005` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
