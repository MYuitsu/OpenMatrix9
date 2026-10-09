---
id: OM9-BUILDER-002
name: Signet Builder
command: gvSignet
domain: 08-builders
module: Builders
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-BUILDER-002 — Signet Builder

Alias tương thích: `gvSignet`. Nhóm: `08-builders`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvSignet Start tạo preview nhẫn signet với +RingRail khi cần, truy cập F6 khi chọn rail. SignetType gồm Cross-Section và 2 Shape. Top Profile quyết định mặt đầu hòa vào thân; Edge Profile định dạng nhìn Side; hover thư viện cập nhật preview. Top Length theo X và Top Width theo Y, mm; Top Thickness từ Finger Rail tới mặt đầu; Shank Thickness ở vị trí 3/9 giờ; Bottom Width là kích thước đáy trước cắt. Cut Bottom mặc định bật, thay đáy phẳng bằng profile; chỉ khi bật mới có Bottom Width Scale, Bottom Length Scale, Bottom Cut Angle đối xứng hai phía. 2 Shape thêm Middle Profile dưới Top Profile và Middle Width nhìn Side. Reset và Styles quản lý cấu hình; các default khác chưa rõ.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `RingRail` | `curve` | 1 | 1 |
| `TopProfile` | `curve` | 0 | 1 |
| `EdgeProfile` | `curve` | 0 | 1 |
| `MiddleProfile` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `SignetType` | `choice` | Cross-Section hoặc 2 Shape. | Chưa xác định; không tự gán giá trị. |
| `Top Profile` | `reference` | Mặt đầu và thân signet. | Chưa xác định; không tự gán giá trị. |
| `Edge Profile` | `reference` | Profile nhìn Side. | Chưa xác định; không tự gán giá trị. |
| `Middle Profile` | `reference` | Profile dưới top; chỉ 2 Shape. | Chưa xác định; không tự gán giá trị. |
| `Top Length` | `number` | Kích thước top theo X, mm. | Chưa xác định; không tự gán giá trị. |
| `Top Width` | `number` | Kích thước top theo Y, mm. | Chưa xác định; không tự gán giá trị. |
| `Top Thickness` | `number` | Khoảng Finger Rail tới mặt đầu. | Chưa xác định; không tự gán giá trị. |
| `Shank Thickness` | `number` | Chiều dày vị trí 3/9 giờ. | Chưa xác định; không tự gán giá trị. |
| `Bottom Width` | `number` | Kích thước đáy trước cắt. | Chưa xác định; không tự gán giá trị. |
| `Cut Bottom` | `boolean` | Thay đáy phẳng bằng profile. | Bật |
| `Bottom Width Scale` | `number` | Scale bề rộng đáy khi Cut Bottom bật. | Chưa xác định; không tự gán giá trị. |
| `Bottom Length Scale` | `number` | Scale chiều dài đáy khi Cut Bottom bật. | Chưa xác định; không tự gán giá trị. |
| `Bottom Cut Angle` | `number` | Góc cắt đối xứng hai phía khi Cut Bottom bật. | Chưa xác định; không tự gán giá trị. |
| `Middle Width` | `number` | Bề rộng middle nhìn Side, chỉ 2 Shape. | Chưa xác định; không tự gán giá trị. |
| `Styles` | `reference` | Preset tham số signet. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-BUILDER-002` và các vai trò RingRail, Top Profile, Edge Profile, Middle Profile; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tạo nhánh trạng thái Cross-Section/2 Shape và dependency profile; dựng thân nhẫn và thay đoạn đáy theo Cut Bottom, không cho dùng bộ tham số đáy bị vô hiệu.
2. C++/Qt: đăng ký và điều phối native command `gvSignet`; chuyển kế hoạch Signet Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-BUILDER-002`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "TopProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "EdgeProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "MiddleProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "SignetType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Top Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Middle Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Top Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shank Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cut Bottom", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bottom Width Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Length Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Cut Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Cut Bottom off phải giữ đáy theo Top Profile; bật mới nhận scale/góc cắt; 2 Shape thay Middle Profile phải đổi vùng chuyển tiếp.
- OM9-BUILDER-002: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-BUILDER-002`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDER-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDER-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDER-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDER-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDER-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-BUILDER-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-BUILDER-002` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
