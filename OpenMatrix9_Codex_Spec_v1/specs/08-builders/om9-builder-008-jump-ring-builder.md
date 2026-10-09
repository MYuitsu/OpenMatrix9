---
id: OM9-BUILDER-008
name: Jump Ring Builder
command: null
domain: 08-builders
module: Builders
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-BUILDER-008 — Jump Ring Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `08-builders`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Jump Ring Builder sweep Shape Profile theo Rail Profile thư viện. Flat rails đồng phẳng creation plane, Curved rails dao động Z; chỉ chỉnh Flat Rail Profiles. Make Ring tạo kết quả, không có handle viewport. Rail Profile Height theo X, Rail Profile Width theo Y; Shape Profile Depth là chiều dày, Shape Profile Width là bề rộng; Shape Outside/Shape Inside xác định rail size là đường kính ngoài hay trong. Undo hoàn tác.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `RailProfile` | `curve` | 1 | 1 |
| `ShapeProfile` | `curve` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Rail Profile` | `reference` | Rail thư viện; Flat sửa được, Curved giữ biến thiên Z. | Chưa xác định; không tự gán giá trị. |
| `Shape Profile` | `reference` | Mặt cắt sweep của jump ring. | Chưa xác định; không tự gán giá trị. |
| `Rail Profile Height` | `number` | Kích thước rail theo X. | Chưa xác định; không tự gán giá trị. |
| `Rail Profile Width` | `number` | Kích thước rail theo Y. | Chưa xác định; không tự gán giá trị. |
| `Shape Profile Depth` | `number` | Chiều dày section. | Chưa xác định; không tự gán giá trị. |
| `Shape Profile Width` | `number` | Bề rộng section. | Chưa xác định; không tự gán giá trị. |
| `Shape Outside/Shape Inside` | `choice` | Rail size đo theo diameter ngoài hay trong. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-BUILDER-008` và các vai trò Rail Profile, Shape Profile; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Đặt hai profile vào cùng khung, offset rail theo quy ước đường kính trong/ngoài và sweep section; giữ riêng chỉnh rail phẳng và chọn rail cong.
2. C++/Qt: đăng ký và điều phối native command `OM9-BUILDER-008`; chuyển kế hoạch Jump Ring Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-BUILDER-008`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "RailProfile", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "ShapeProfile", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Rail Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Shape Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Rail Profile Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shape Profile Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shape Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shape Outside/Shape Inside", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Cùng rail size và section phải có đường kính ngoài khác nhau giữa Shape Inside/Outside; rail cong giữ biến thiên Z, sửa bị chặn ở loại không hỗ trợ.
- OM9-BUILDER-008: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-BUILDER-008`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-BUILDER-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-BUILDER-008` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
