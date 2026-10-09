---
id: OM9-SUBD-016
name: Clayoo Bezel
command: ClayBezel
domain: 14-subd
module: SubD
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SUBD-016 — Clayoo Bezel

Alias tương thích: `ClayBezel`. Nhóm: `14-subd`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

ClayBezel tạo Clayoo bezel có Dome Height, TopThickness gần girdle, Seat Height depth, Seat Length xuống pavilion, Seat Angle, Global Height, Bezel Angle ngoài, BottomThickness. Beige handle vừa Seat Length/Seat Angle, cần gesture constraint riêng; Okay validate. Khác gvtsBezel T-Spline.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Gem` | `gem` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Dome Height` | `number` | Độ cao dome trên bezel, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Top Thickness` | `number` | Chiều dày thành trên sát girdle, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Seat Height` | `number` | Độ sâu seat theo điều khiển này; mốc cần xác minh, không nhầm Global Height. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Seat Length` | `number` | Chiều seat đi xuống pavilion, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Seat Angle` | `number` | Góc seat so với pavilion, độ; handle kết hợp với Seat Length. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Global Height` | `number` | Chiều cao tổng bezel, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bezel Angle` | `number` | Góc thành ngoài bezel, độ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bottom Thickness` | `number` | Chiều dày thành đáy, đơn vị chiều dài tài liệu. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SUBD-016` và các vai trò Gem; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Cage bezel theo gem frame và seat geometry, typed handle 2 D điều khiển length/angle; adapter Clayoo riêng.
2. C++/Qt: đăng ký và điều phối native command `ClayBezel`; chuyển kế hoạch Clayoo Bezel sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SUBD-016`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-016",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Dome Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Global Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Seat Length/Angle cùng handle nhưng parameters riêng, Dome Height đổi roundedlip, Okay valid cage và gem giữ nguyên.
- OM9-SUBD-016: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SUBD-016`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SUBD-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SUBD-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SUBD-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SUBD-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SUBD-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-SUBD-016` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
