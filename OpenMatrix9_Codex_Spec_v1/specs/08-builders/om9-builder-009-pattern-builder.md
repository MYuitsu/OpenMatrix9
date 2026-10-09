---
id: OM9-BUILDER-009
name: Pattern Builder
command: null
domain: 08-builders
module: Builders
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-BUILDER-009 — Pattern Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `08-builders`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Pattern Builder Preview đặt hai tam giác trắng ở Looking Down để định hướng hai nhóm hình; nhập nội dung từng tam giác vào hai hộp, chọn Pattern 1–40. Pattern xây tile vuông từ hình trong tam giác, mỗi số có bố trí khác cần xác minh riêng. X Repeat/Y Repeat là số lần theo trục; X Scale/Y Scale là scale riêng hai trục.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `PatternA` | `object` | 1 | Không đặt trong mẫu |
| `PatternB` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Pattern` | `choice` | Index 1–40; mỗi mapping cần kiểm chứng riêng. | Chưa xác định; không tự gán giá trị. |
| `X Repeat` | `number` | Số tile theo X; policy mẫu yêu cầu integer. | Chưa xác định; không tự gán giá trị. |
| `Y Repeat` | `number` | Số tile theo Y; policy mẫu yêu cầu integer. | Chưa xác định; không tự gán giá trị. |
| `X Scale` | `number` | Scale riêng trục X. | Chưa xác định; không tự gán giá trị. |
| `Y Scale` | `number` | Scale riêng trục Y. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-BUILDER-009` và các vai trò Pattern A, Pattern B; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Lưu nhóm A/B và index pattern; adapter tile biến đổi đối tượng nguyên bản, lặp bằng ma trận X/Y; không tự thay pattern chưa xác minh bằng random.
2. C++/Qt: đăng ký và điều phối native command `OM9-BUILDER-009`; chuyển kế hoạch Pattern Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-BUILDER-009`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-009",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "PatternA", kind: InputKind::Object, min: 1, max: None },
        Role { name: "PatternB", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Pattern", kind: ParameterKind::Choice, required: false },
        Parameter { name: "X Repeat", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Repeat", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Scale", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Lặp X=2 Y=3 phải ra sáu tile cho một pattern xác minh; scale X không đổi Y; kiểm tra mỗi index có mapping xác định trước khi enable.
- OM9-BUILDER-009: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-BUILDER-009`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDER-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDER-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDER-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDER-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDER-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-BUILDER-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-BUILDER-009` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
