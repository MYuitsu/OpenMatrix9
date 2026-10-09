---
id: OM9-MATRIXTOOLS-006
name: User Libraries
command: null
domain: 16-matrix-tools
module: Matrix Tools
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MATRIXTOOLS-006 — User Libraries

Alias tương thích: `Chưa có alias command`. Nhóm: `16-matrix-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

User Libraries tổ chức design/element người dùng. Các thư mục con của library là category. Browser có Size Slider thay kích thước thumbnail và Gem Filter lọc theo metadata đá; chọn model rồi Import hoặc Open. Import thêm model vào document hiện tại; Open chuyển sang design được chọn và phải xử lý document đang sửa theo cơ chế host. Lệnh tương thích gvSaveCurrentModelLibraryFormat đưa design vào thư viện.

Import cần giữ units/world placement và remap identity/link để tránh trùng object. Open không được đóng document cũ trước khi việc đọc/kiểm tra tệp mới thành công và quy trình bảo vệ thay đổi chưa lưu được thực hiện. Path traversal, file hỏng, metadata thiếu và asset ngoài library cần policy rõ. Không diễn giải Gem Filter thiếu metadata thành số đá bằng 0.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `library_location` | `path` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Action` | `choice` | Browse/Import/Open/SaveModel; policy OM9 tách luồng. | Chưa xác định; không tự điền. |
| `Model` | `reference` | Identity model trong library. | Chưa xác định; không tự điền. |
| `Category` | `text` | Subfolder category. | Chưa xác định; không tự điền. |
| `ThumbnailSize` | `number` | Kích thước hiển thị thumbnail, không scale model. | Chưa xác định; không tự điền. |
| `GemFilter` | `text` | Điều kiện lọc gem metadata, grammar cần chốt. | Chưa xác định; không tự điền. |

## Hướng dẫn triển khai

1. Rust phân biệt browser state và action; giữ identity/category/filter cùng request, kiểm tra path nằm trong library scope.
2. C++/Qt đọc thumbnail và metadata không tải geometry toàn bộ cho browse. Import decode model/units trước transaction, remap links và commit nguyên tử vào document hiện tại.
3. Open qua host document lifecycle, bảo vệ unsaved changes, validate model mới trước thay document. SaveModel dùng serialize/atomic file replacement; lỗi/hủy giữ dữ liệu cũ.
4. Đăng ký command/menu/CMD trong C++ native; Rust giữ request/state. Python đăng ký workbench và fixture. Giữ nhánh thiếu adapter disabled/unsupported.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "library_location", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: true },
        Parameter { name: "Model", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Category", kind: ParameterKind::Text, required: false },
        Parameter { name: "ThumbnailSize", kind: ParameterKind::Number, required: false },
        Parameter { name: "GemFilter", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Thêm category folder xuất hiện trong browser; slider chỉ đổi thumbnail, filter xử lý metadata thiếu riêng.
- Import cùng model hai lần không trùng IDs/links và giữ placement; lỗi mid-import rollback.
- Open khi document dirty cho đúng quy trình host; file hỏng/Cancel giữ document cũ; SaveModel đọc lại đầy đủ.

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
- [ ] Automated tests use feature ID `OM9-MATRIXTOOLS-006`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MATRIXTOOLS-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MATRIXTOOLS-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MATRIXTOOLS-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MATRIXTOOLS-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MATRIXTOOLS-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-MATRIXTOOLS-006` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MATRIXTOOLS-006` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
