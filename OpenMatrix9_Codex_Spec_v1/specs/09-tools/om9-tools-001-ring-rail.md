---
id: OM9-TOOLS-001
name: Ring Rail
command: gvRingRail
domain: 09-tools
module: Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-001 — Ring Rail

Alias tương thích: `gvRingRail`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvRingRail chọn kích thước theo Region Type, USA mặc định và bước ¼ size. Custom Ring Size chọn Millimeters/Inches, Diameter/Circumference, Set Region rồi nhập số. Khi đã có rail: Yes thay rail, No thêm rail thứ hai, Cancel không thêm. Advanced chọn rail không tròn từ Custom library; Edit mở editor; Add existing rail to the library nhận curve quanh F4. Không được suy bảng size quốc tế nếu chưa có dữ liệu kiểm chứng.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Document` | `document` | 1 | 1 |
| `CustomRail` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Region Type` | `choice` | Bảng ring-size theo region. | USA |
| `Custom Ring Size Units` | `choice` | Millimeters hoặc Inches. | Chưa xác định; không tự gán giá trị. |
| `Custom Ring Size Measure` | `choice` | Diameter hoặc Circumference. | Chưa xác định; không tự gán giá trị. |
| `Custom Ring Size` | `number` | Kích thước theo units và measure đã chọn. | Chưa xác định; không tự gán giá trị. |
| `Custom Rail` | `reference` | Rail không tròn từ thư viện. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-001` và các vai trò Document, Custom Rail; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Dùng bảng region và chuyển đơn vị theo mode; tạo rail tròn từ diameter hoặc circumference, hoặc scale custom rail bằng chính sách riêng; xử lý lựa chọn thay thế trong transaction.
2. C++/Qt: đăng ký và điều phối native command `gvRingRail`; chuyển kế hoạch Ring Rail sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-001`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "CustomRail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Region Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Custom Ring Size Units", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Custom Ring Size Measure", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Custom Ring Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Custom Rail", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Chu vi và đường kính tương ứng phải cùng circle; Yes/No/Cancel có đúng số đối tượng; custom rail không bị ép thành circle.
- OM9-TOOLS-001: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
