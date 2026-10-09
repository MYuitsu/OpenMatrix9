---
id: OM9-ART-005
name: Effects Menu
command: null
domain: 07-matrix-art
module: Matrix Art
kind: menu
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-ART-005 — Effects Menu

Alias tương thích: `Chưa có alias command`. Nhóm: `07-matrix-art`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Effects sửa dữ liệu ảnh nên thay đổi relief từ sáng/tối. Adjust Contrast có Low End nâng vùng tối về xám, High End hạ vùng sáng về xám, Both Ends phối hợp; Blur Amount Low/Medium/High làm mượt ảnh toàn cục. Select Area/Deselect Area vẽ mask, Select All/Reverse Selection/Clear Selection quản lý miền. Diameter, Strength và Fall Off điều khiển kích thước, tốc độ và chuyển tiếp của brush. Brightness có Brightness/Contrast; Blur có Blur size/Blur Amount; Bitmap Invert đảo cao/thấp trong mask. Brightness và Blur không dùng đồng thời. Apply Effect chốt thay đổi, giữ mask cho hiệu ứng tiếp nếu muốn; phải Clear Selection trước vùng mới. Undo/Redo theo từng lần Apply.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Bitmap` | `image` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Diameter` | `number` | Đường kính brush chọn miền. | Chưa xác định; không tự gán giá trị. |
| `Strength` | `number` | Tốc độ tác động brush. | Chưa xác định; không tự gán giá trị. |
| `Fall Off` | `number` | Mức chuyển tiếp từ tâm tới biên brush. | Chưa xác định; không tự gán giá trị. |
| `Adjust Contrast` | `choice` | Low End, High End hoặc Both Ends. | Chưa xác định; không tự gán giá trị. |
| `Blur Amount` | `choice` | Low, Medium hoặc High cho làm mượt toàn ảnh. | Chưa xác định; không tự gán giá trị. |
| `Brightness` | `number` | Mức đổi sáng trong mask. | Chưa xác định; không tự gán giá trị. |
| `Contrast` | `number` | Mức đổi tương phản trong mask. | Chưa xác định; không tự gán giá trị. |
| `Blur size` | `number` | Kích cỡ blur trong mask; khác preset blur toàn ảnh. | Chưa xác định; không tự gán giá trị. |
| `Bitmap Invert` | `boolean` | Đảo sáng/tối trong mask. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-ART-005` và các vai trò Bitmap; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Lưu ảnh gốc, stack hiệu ứng và mask riêng; brush sửa mask trước, Apply tính ảnh mới có thể undo; tái dựng relief từ ảnh sau hiệu ứng.
2. C++/Qt: đăng ký và điều phối native command `OM9-ART-005`; chuyển kế hoạch Effects Menu sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-ART-005`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-005",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Bitmap", kind: InputKind::Image, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Strength", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fall Off", kind: ParameterKind::Number, required: false },
        Parameter { name: "Adjust Contrast", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Blur Amount", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Brightness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Contrast", kind: ParameterKind::Number, required: false },
        Parameter { name: "Blur size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bitmap Invert", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Brightness nâng đúng vùng chọn, Blur giảm cạnh răng cưa, Invert đảo mức sáng; Apply rồi Clear Selection phải ngăn vùng cũ nhận thay đổi tiếp.
- OM9-ART-005: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-ART-005`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-ART-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-ART-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-ART-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-ART-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-ART-005` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
