---
id: OM9-SUBD-017
name: Clayoo Ring
command: ClayRing
domain: 14-subd
module: SubD
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SUBD-017 — Clayoo Ring

Alias tương thích: `ClayRing`. Nhóm: `14-subd`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

ClayRing Region/Size hoặc Diameter inside, Segments quanh circumference, Outer/Inner/Left(back)/Right(front)Segments phân faces từng side. Top/Middle/BottomWidth và Height điều khiển band shape; Okay/Enter tạo, Cancel bỏ. Illustrated counts/dimensions không defaults.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Region` | `choice` | Hệ bảng cỡ nhẫn; dùng bảng đã kiểm chứng, không suy đổi size quốc tế. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Size` | `number` | Cỡ nhẫn theo Region hiện hành; giá trị không phải mm. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Diameter` | `number` | Đường kính trong theo đơn vị chiều dài tài liệu; nhánh thay cách chọn Region/Size. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Segments` | `number` | Số phân đoạn cage tại scope được đặt tên: Segments quanh chu vi, Outer/Inner ngoài/trong, Left phía back và Right phía front; topology adapter kiểm tra số nguyên. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Outer Segments` | `number` | Số phân đoạn cage tại scope được đặt tên: Segments quanh chu vi, Outer/Inner ngoài/trong, Left phía back và Right phía front; topology adapter kiểm tra số nguyên. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Inner Segments` | `number` | Số phân đoạn cage tại scope được đặt tên: Segments quanh chu vi, Outer/Inner ngoài/trong, Left phía back và Right phía front; topology adapter kiểm tra số nguyên. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Left Segments` | `number` | Số phân đoạn cage tại scope được đặt tên: Segments quanh chu vi, Outer/Inner ngoài/trong, Left phía back và Right phía front; topology adapter kiểm tra số nguyên. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Right Segments` | `number` | Số phân đoạn cage tại scope được đặt tên: Segments quanh chu vi, Outer/Inner ngoài/trong, Left phía back và Right phía front; topology adapter kiểm tra số nguyên. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Top Width` | `number` | Bề rộng hoặc chiều cao vùng đỉnh band theo tên control, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Top Height` | `number` | Bề rộng hoặc chiều cao vùng đỉnh band theo tên control, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Middle Width` | `number` | Bề rộng hoặc chiều cao vùng bên/giữa band theo tên control, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Middle Height` | `number` | Bề rộng hoặc chiều cao vùng bên/giữa band theo tên control, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bottom Width` | `number` | Bề rộng hoặc chiều cao đáy band theo tên control, đơn vị chiều dài tài liệu; không lấy dimensions minh họa làm mặc định. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bottom Height` | `number` | Bề rộng hoặc chiều cao đáy band theo tên control, đơn vị chiều dài tài liệu; không lấy dimensions minh họa làm mặc định. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SUBD-017` và các vai trò Document; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Sinh closed ring cage có regions và per-side segment ation, ring sizes verified table hoặc explicit Diameter.
2. C++/Qt: đăng ký và điều phối native command `ClayRing`; chuyển kế hoạch Clayoo Ring sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SUBD-017`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-017",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Region", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Left Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Right Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Diameter đúng inside, kích thước Top/Middle/Bottom độc lập, Increase Outer Segments không đổi Inner count; Cancel giữ document.
- OM9-SUBD-017: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SUBD-017`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SUBD-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SUBD-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SUBD-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SUBD-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SUBD-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-SUBD-017` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
