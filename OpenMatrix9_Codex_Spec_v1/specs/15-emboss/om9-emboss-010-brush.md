---
id: OM9-EMBOSS-010
name: Brush
command: null
domain: 15-emboss
module: Emboss
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-EMBOSS-010 — Brush

Alias tương thích: `Chưa có alias command`. Nhóm: `15-emboss`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Brush vẽ chi tiết thủ công bằng kéo chuột trái; nút xanh kết thúc phiên brush theo workflow. Height/Width định cỡ nét. Accumulate On cộng thêm mỗi lượt, Off giới hạn theo Height. Styles Add/Subtract/Highest Union/Lowest Union/Absolute kết hợp operation với stack; Change Color hiển thị. Không suy Width thành pixel vì đơn vị chưa xác lập.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Relief` | `mesh` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Styles` | `choice` | Add, Subtract, Highest Union, Lowest Union hoặc Absolute; phép kết hợp độ cao theo thứ tự operation. | Chưa xác định; không tự gán giá trị. |
| `Change Color` | `text` | Màu hiển thị operation, không thay hình học. | Chưa xác định; không tự gán giá trị. |
| `Height` | `number` | Biên độ brush. | Chưa xác định; không tự gán giá trị. |
| `Width` | `number` | Footprint brush; unit chưa xác lập. | Chưa xác định; không tự gán giá trị. |
| `Accumulate` | `boolean` | On cộng mỗi lượt; Off giới hạn theo Height. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-EMBOSS-010` và các vai trò Relief; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Lưu stroke heightfield độc lập nền, định nghĩa kernel tự tác giả và giới hạn Height cho Accumulate Off; gesture preview rồi commit stroke.
2. C++/Qt: đăng ký và điều phối native command `OM9-EMBOSS-010`; chuyển kế hoạch Brush sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-EMBOSS-010`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-010",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Relief", kind: InputKind::Mesh, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Styles", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Change Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Accumulate", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Hai lượt cùng vị trí tăng độ cao khi On, Off không vượt giới hạn Height theo policy; Width đổi footprint; hủy không giữ stroke chưa commit.
- OM9-EMBOSS-010: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-EMBOSS-010`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-EMBOSS-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-EMBOSS-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-EMBOSS-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-EMBOSS-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-EMBOSS-010` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
