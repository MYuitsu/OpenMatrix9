---
id: OM9-SUBD-035
name: Clayoo Revolve
command: ClayRevolve
domain: 14-subd
module: SubD
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SUBD-035 — Clayoo Revolve

Alias tương thích: `ClayRevolve`. Nhóm: `14-subd`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

ClayRevolve input curve quanh Axis X/Y/Z hoặc Custom, Angle to Fill, Segments around/Segments Along profile. Cap Start/End Polygonal/Ngon/None, Okay/Enter/Cancel. Axis phù hợp profile plane, chưa có góc mặc định.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Curve` | `curve` | 1 | 1 |
| `AxisPoints` | `point` | 0 | 2 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Axis` | `choice` | X, Y, Z hoặc Custom trong frame hiện hành. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Custom Axis` | `reference` | Axis tùy chọn có origin/direction; chỉ khi Custom. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Angle to Fill` | `number` | Góc phủ revolve, độ; chưa xác lập mặc định. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Segments` | `number` | Số phân đoạn quanh trục revolve. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Segments Along` | `number` | Số phân đoạn dọc profile. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Cap Start` | `choice` | Polygonal, Ngon hoặc None cho đầu revolve. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Cap End` | `choice` | Polygonal, Ngon hoặc None cho cuối revolve. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SUBD-035` và các vai trò Curve, Axis Points; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Sample profile và rotate samples around axis, build cage/connect seam when circle complete, caps restricted valid loops.
2. C++/Qt: đăng ký và điều phối native command `ClayRevolve`; chuyển kế hoạch Clayoo Revolve sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SUBD-035`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-035",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "AxisPoints", kind: InputKind::Point, min: 0, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Axis", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Custom Axis", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Angle to Fill", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Along", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap Start", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Cap End", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Custom axis đúng picked line, partial angle open bounds, closed full revolve weld seam theo policy; cap selection đúng.
- OM9-SUBD-035: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SUBD-035`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SUBD-035` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SUBD-035` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SUBD-035` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SUBD-035` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SUBD-035` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-SUBD-035` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
