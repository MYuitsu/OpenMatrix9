---
id: OM9-SOLID-025
name: Array Hole
command: ArrayHole
domain: 04-solid
module: Solid Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SOLID-025 — Array Hole

Alias tương thích: `ArrayHole`. Nhóm: `04-solid`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

ArrayHole sao hole trên surface theo hai hướng A/B. Nhập count A/B, base point rồi direction và spacing từng hướng; chỉnh preview trước commit.

### Tùy chọn và tương tác

- `ADirection`: hướng hàng đầu
- `ANumber/BNumber`: số hole mỗi hướng
- `ASpacing/BSpacing`: khoảng tâm hole
- `Rectangular`: Yes/No; Yes B vuông A, No cho skew
- `UseASpacing`: bật/tắt; BSpacing bằng ASpacing

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `hole` | `selection` | 1 | 1 |
| `base` | `point` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `ADirection` | `reference` | hướng hàng đầu | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `ANumber` | `number` | số hole mỗi hướng Giá trị độc lập của ANumber. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `BNumber` | `number` | số hole mỗi hướng Giá trị độc lập của BNumber. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `ASpacing` | `number` | khoảng tâm hole Giá trị độc lập của ASpacing. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `BSpacing` | `number` | khoảng tâm hole Giá trị độc lập của BSpacing. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Rectangular` | `boolean` | Yes/No; Yes B vuông A, No cho skew | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `UseASpacing` | `boolean` | bật/tắt; BSpacing bằng ASpacing | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SOLID-025 và phiên chọn có thứ tự, kiểm tra vai trò hole, base; chuẩn hóa tùy chọn riêng của Array Hole.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Array Hole trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
4. Preview nếu cần dùng dữ liệu tạm; xác nhận ghi kết quả và dependency trong một transaction. Hủy/lỗi không tạo output dở dang; Undo/Redo và save/reload giữ contract.
5. Native C++ đăng ký command/menu và dispatch sang Rust/native geometry. Python đăng ký workbench và hỗ trợ fixtures/macro kiểm chứng. Capability chưa có hoặc chưa kiểm chứng phải giữ disabled với lý do cụ thể.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-025",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "hole", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "base", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "ADirection", kind: ParameterKind::Reference, required: false },
        Parameter { name: "ANumber", kind: ParameterKind::Number, required: false },
        Parameter { name: "BNumber", kind: ParameterKind::Number, required: false },
        Parameter { name: "ASpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "BSpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rectangular", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "UseASpacing", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Count A×B hole theo policy gồm hole gốc đã chốt; Rectangular góc 90°, No lattice skew; spacing đo từ center.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

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
- [ ] Automated tests use feature ID `OM9-SOLID-025`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SOLID-025` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SOLID-025` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SOLID-025` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SOLID-025` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SOLID-025` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SOLID-025` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SOLID-025` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SOLID-025` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
