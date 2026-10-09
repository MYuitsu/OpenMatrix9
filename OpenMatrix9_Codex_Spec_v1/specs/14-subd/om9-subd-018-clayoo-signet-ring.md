---
id: OM9-SUBD-018
name: Clayoo Signet Ring
command: ClaySignetRing
domain: 14-subd
module: SubD
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SUBD-018 — Clayoo Signet Ring

Alias tương thích: `ClaySignetRing`. Nhóm: `14-subd`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

ClaySignetRing Region/Size hoặc Diameter inside; Seal Width/Depth và Corner Width/Depth mặt đầu, TopWidth vùng dưới seal, TopThickness seal, Shoulder Width/Thickness, Middle Width ở 3 giờ/MiddleThickness, BottomWidth/Thickness. Okay/Enter/Cancel; values minh họa không defaults.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Region` | `choice` | Hệ cỡ nhẫn được chọn; phụ thuộc bảng chuyển đổi được kiểm chứng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Size` | `number` | Cỡ nhẫn theo Region; không phải kích thước chiều dài trực tiếp. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Diameter` | `number` | Đường kính trong theo đơn vị chiều dài tài liệu; thay nhánh Region/Size. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Seal Width` | `number` | Kích thước mặt seal hoặc phần góc tương ứng, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Seal Depth` | `number` | Kích thước mặt seal hoặc phần góc tương ứng, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Corner Width` | `number` | Kích thước mặt seal hoặc phần góc tương ứng, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Corner Depth` | `number` | Kích thước mặt seal hoặc phần góc tương ứng, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Top Width` | `number` | Bề rộng phần dưới seal, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Top Thickness` | `number` | Chiều dày seal, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Shoulder Width` | `number` | Kích thước vai nhẫn, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Shoulder Thickness` | `number` | Kích thước vai nhẫn, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Middle Width` | `number` | Kích thước vùng giữa thân ở khoảng 3 giờ, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Middle Thickness` | `number` | Kích thước vùng giữa thân ở khoảng 3 giờ, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bottom Width` | `number` | Kích thước đáy thân nhẫn, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bottom Thickness` | `number` | Kích thước đáy thân nhẫn, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SUBD-018` và các vai trò Document; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Sinh signet cage với z ones seal/shoulder/mid/bottom, signed frame units không fall back TSpline Signet.
2. C++/Qt: đăng ký và điều phối native command `ClaySignetRing`; chuyển kế hoạch Clayoo Signet Ring sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SUBD-018`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-018",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Region", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seal Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seal Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shoulder Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shoulder Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Seal Depth đổi Y extent, Shoulder Thickness đổi đúng z one, Diameter giữ size finger; Clayoo object có cage editable.
- OM9-SUBD-018: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SUBD-018`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SUBD-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SUBD-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SUBD-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SUBD-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SUBD-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-SUBD-018` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
