---
id: OM9-ART-006
name: Sculpt Menu
command: null
domain: 07-matrix-art
module: Matrix Art
kind: menu
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-ART-006 — Sculpt Menu

Alias tương thích: `Chưa có alias command`. Nhóm: `07-matrix-art`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Sculpt sửa lớp độ cao hình học và giữ texture bitmap. Deposit nâng, Carve hạ; Diameter, Strength, Height, Fall Off điều khiển nét kéo. Smooth, Smudge, Fade và Erase chỉ xử lý dấu sculpt sau Deposit/Carve, không làm mượt texture ảnh; tác động từ Smooth nhẹ tới Fade mạnh, Erase khôi phục miền trước sculpt. Undo/Redo từng thay đổi; Reset xóa toàn bộ sculpt nhưng giữ mô hình nền trước sculpt.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Relief` | `mesh` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Diameter` | `number` | Đường kính brush sculpt. | Chưa xác định; không tự gán giá trị. |
| `Strength` | `number` | Cường độ Deposit/Carve. | Chưa xác định; không tự gán giá trị. |
| `Height` | `number` | Độ cao tác động sculpt. | Chưa xác định; không tự gán giá trị. |
| `Fall Off` | `number` | Chuyển tiếp tác động ở biên brush. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-ART-006` và các vai trò Relief; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Giữ trường sculpt tách khỏi bitmap/effects; thao tác brush lên displacement, Erase trả displacement về 0 trong mask và Reset xóa toàn stack sculpt.
2. C++/Qt: đăng ký và điều phối native command `OM9-ART-006`; chuyển kế hoạch Sculpt Menu sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-ART-006`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-006",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Relief", kind: InputKind::Mesh, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Strength", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fall Off", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Texture phải giữ nguyên khi Deposit; Erase chỉ bỏ dấu sculpt, Reset khôi phục nền relief, còn muốn làm mượt texture phải qua Effects.
- OM9-ART-006: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-ART-006`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-ART-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-ART-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-ART-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-ART-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-ART-006` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
