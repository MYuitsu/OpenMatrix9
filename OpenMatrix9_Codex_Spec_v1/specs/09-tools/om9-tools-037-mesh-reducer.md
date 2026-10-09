---
id: OM9-TOOLS-037
name: Mesh Reducer
command: null
domain: 09-tools
module: Tools
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-037 — Mesh Reducer

Alias tương thích: `Chưa có alias command`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Mesh Reducer chọn phần trăm kích thước ban đầu bằng slider, hiển thị Input/Output polygon count rồi Reduce, CMD options, Enter chờ mesh mới. Gợi ý giảm từng bước nhỏ 3–10% để giữ dáng, không coi là giới hạn cứng. Delete xóa đầu vào khi bật; Select/Undo. Weld mặc định No, Yes gộp vertices đồng vị trí theo WeldAngle và làm normal trung bình, có thể hợp sub meshes không explode lại. SplitDisjointMeshes xử lý face rời theo chế độ đã mô tả; cần kiểm chứng ý nghĩa tách/tái nối. ModelUnits mặc định Millimeters.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Mesh` | `mesh` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Percentage` | `number` | Mục tiêu kích thước mesh so đầu vào. | Chưa xác định; không tự gán giá trị. |
| `Delete` | `boolean` | Xóa mesh input sau thành công. | Chưa xác định; không tự gán giá trị. |
| `Weld` | `boolean` | Gộp vertices đồng vị trí và average normals theo WeldAngle. | No |
| `WeldAngle` | `number` | Ngưỡng góc weld khi Weld Yes. | Chưa xác định; không tự gán giá trị. |
| `SplitDisjointMeshes` | `boolean` | Xử lý face rời; cơ chế tách/tái nối cần xác minh. | Chưa xác định; không tự gán giá trị. |
| `ModelUnits` | `choice` | Đơn vị model cho reducer. | Millimeters |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-037` và các vai trò Mesh; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Đơn giản hóa mesh theo target count và constraints biên, weld tùy angle riêng; báo counts thực tế, commit rồi mới Delete.
2. C++/Qt: đăng ký và điều phối native command `OM9-TOOLS-037`; chuyển kế hoạch Mesh Reducer sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-037`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-037",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Mesh", kind: InputKind::Mesh, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Percentage", kind: ParameterKind::Number, required: false },
        Parameter { name: "Delete", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Weld", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "WeldAngle", kind: ParameterKind::Number, required: false },
        Parameter { name: "SplitDisjointMeshes", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ModelUnits", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Weld No giữ crease, Yes theo angle làm mượt; giảm giữ silhouette trong tolerance công khai, Input/Output counts chính xác, Undo trả mesh gốc.
- OM9-TOOLS-037: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-037`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-037` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-037` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-037` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-037` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-037` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-037` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-037` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-037` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
