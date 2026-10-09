---
id: OM9-TOOLS-038
name: Surface Inset
command: gvInset
domain: 09-tools
module: Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-038 — Surface Inset

Alias tương thích: `gvInset`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvInset chọn surface, đặt parameters trước pick boundary kín trên surface. Angle độ cho taper walls, Length cho độ dài walls; Tolerance 0.01–0.1 (0.02 là ví dụ, không default), Splits tăng điểm tham chiếu tránh xoắn, Surface Sweep/Loft chọn phương pháp; Direction pick hai điểm định hướng pitch. Đây là inset surface, không mặc định solid kín.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Surface` | `surface` | 1 | 1 |
| `Boundary` | `curve` | 1 | 1 |
| `DirectionPoints` | `point` | 0 | 2 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Angle` | `number` | Góc taper walls, độ. | Chưa xác định; không tự gán giá trị. |
| `Length` | `number` | Độ dài walls. | Chưa xác định; không tự gán giá trị. |
| `Tolerance` | `number` | Sai số trong 0.01–0.1; không suy 0.02 default. | Chưa xác định; không tự gán giá trị. |
| `Splits` | `number` | Số điểm tham chiếu để hạn chế twist. Chính sách mẫu OpenMatrix9 yêu cầu số nguyên; chưa xác nhận quy tắc nội bộ khác. | Chưa xác định; không tự gán giá trị. |
| `Surface` | `choice` | Sweep hoặc Loft. | Chưa xác định; không tự gán giá trị. |
| `Direction` | `reference` | Hai điểm định hướng pitch. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-038` và các vai trò Surface, Boundary, Direction Points; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tạo sections walls theo boundary và direction, chọn sweep/loft, chia theo Splits và kiểm tra sai lệch tolerance.
2. C++/Qt: đăng ký và điều phối native command `gvInset`; chuyển kế hoạch Surface Inset sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-038`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-038",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Boundary", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "DirectionPoints", kind: InputKind::Point, min: 0, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Splits", kind: ParameterKind::Number, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Boundary ngoài mặt bị từ chối; Angle/Direction đổi taper, tăng Splits giúp xử lý đoạn xoắn; Tolerance=0.02 không bị ép làm mặc định.
- OM9-TOOLS-038: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-038`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-038` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-038` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-038` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-038` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-038` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-038` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-038` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-038` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
