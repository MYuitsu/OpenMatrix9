---
id: OM9-SUBD-031
name: Clayoo Sweep 1
command: ClaySweep1
domain: 14-subd
module: SubD
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SUBD-031 — Clayoo Sweep 1

Alias tương thích: `ClaySweep1`. Nhóm: `14-subd`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

ClaySweep1 rail rồi profiles, Segments và Segments Along theo hai hướng subdivision, Flip Rail; Align Seams chỉ closed profiles, all seams/directions nên match, Curve Direction white arrows. Okay/Enter/Cancel.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Rail` | `curve` | 1 | 1 |
| `Profiles` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Segments` | `number` | Số phân đoạn/faces chạy vuông góc rail curve; không nhầm với Segments Along. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Segments Along` | `number` | Số phân đoạn/faces chạy vuông góc input profiles, tương ứng hướng dọc rail. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Flip Rail` | `boolean` | Đảo direction rail trước sweep. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Align Seams` | `reference` | Correspondence seam của closed profiles; chỉ áp khi profile kín. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Curve Direction` | `reference` | Dữ liệu direction của từng profile theo white arrows; không một boolean dùng cho toàn bộ profiles. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SUBD-031` và các vai trò Rail, Profiles; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Resample profiles và transport frames rails, tạo quad loft mesh cage with explicit seam/correspondence.
2. C++/Qt: đăng ký và điều phối native command `ClaySweep1`; chuyển kế hoạch Clayoo Sweep1 sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SUBD-031`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-031",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Rail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Profiles", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Segments", kind: ParameterKind::Number, required: false },
        Parameter { name: "Segments Along", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip Rail", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align Seams", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Curve Direction", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Mismatched directions báo/twist preview, align seams giảm twist; open profiles không seam handle, count đúng từng hướng.
- OM9-SUBD-031: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SUBD-031`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SUBD-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SUBD-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SUBD-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SUBD-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SUBD-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-SUBD-031` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
