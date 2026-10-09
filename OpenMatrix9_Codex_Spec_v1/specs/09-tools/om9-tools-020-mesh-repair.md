---
id: OM9-TOOLS-020
name: Mesh Repair
command: gvMeshRepair
domain: 09-tools
module: Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-020 — Mesh Repair

Alias tương thích: `gvMeshRepair`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvMeshRepair tạo một mesh kín để in, sửa biên mở/vertices chồng và các lỗi; chọn rồi Enter, Object Properties và Check Mesh cho báo cáo. Extended Repair tăng thời gian/khả năng xử lý và báo progress; DeleteInputObjects bật xóa NURBS sau tạo mesh; Mesh Tolerance nhỏ bám hình hơn nhưng nhiều polygons. Simple Planes, Optimize Milgrain, Milgrain Grid Count giảm số polygons/file. Chưa có bảo đảm mọi mesh lỗi đều sửa thành công.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Extended Repair` | `boolean` | Chạy repair mở rộng với progress. | Chưa xác định; không tự gán giá trị. |
| `DeleteInputObjects` | `boolean` | Xóa đầu vào sau khi mesh đạt yêu cầu. | Chưa xác định; không tự gán giá trị. |
| `Mesh Tolerance` | `number` | Sai số tessellation; nhỏ hơn cho nhiều polygons. | Chưa xác định; không tự gán giá trị. |
| `Simple Planes` | `boolean` | Giảm tessellation trên mặt phẳng. | Chưa xác định; không tự gán giá trị. |
| `Optimize Milgrain` | `boolean` | Tối ưu tessellation milgrain. | Chưa xác định; không tự gán giá trị. |
| `Milgrain Grid Count` | `number` | Mật độ lưới milgrain. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-020` và các vai trò Objects; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tessellate theo tolerance đã chọn rồi repair với báo cáo lỗi còn lại; chỉ commit/delete đầu vào khi mesh hợp lệ đạt mục tiêu, hỗ trợ hủy tác vụ dài.
2. C++/Qt: đăng ký và điều phối native command `gvMeshRepair`; chuyển kế hoạch Mesh Repair sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-020`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Extended Repair", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "DeleteInputObjects", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mesh Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Simple Planes", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Optimize Milgrain", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Milgrain Grid Count", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Mesh có biên mở được sửa hoặc báo lỗi rõ; DeleteInputObjects chỉ xóa sau thành công; Extended Repair báo tiến độ và cancel không mất NURBS.
- OM9-TOOLS-020: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-020`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-020` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
