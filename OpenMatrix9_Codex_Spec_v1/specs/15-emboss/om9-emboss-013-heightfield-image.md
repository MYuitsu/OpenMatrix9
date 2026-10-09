---
id: OM9-EMBOSS-013
name: Heightfield Image
command: null
domain: 15-emboss
module: Emboss
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-EMBOSS-013 — Heightfield Image

Alias tương thích: `Chưa có alias command`. Nhóm: `15-emboss`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Heightfield Image tạo ảnh độ cao từ geometry cho Texture/Stamp. Resolution X/Y là dimensions ảnh, Constrain Proportions giữ tỉ lệ khi đổi một chiều; bounding box đỏ có handles để chọn miền. Side Up/Down lấy cao/thấp, Preview xem trước. Check mark mở Name: Location chọn thư viện Textures/Stamps, Name đặt tài nguyên. Các dimensions minh họa không phải default.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Geometry` | `object` | 1 | Không đặt trong mẫu |
| `Destination` | `path` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Resolution X` | `number` | Chiều rộng ảnh đầu ra, pixel. | Chưa xác định; không tự gán giá trị. |
| `Resolution Y` | `number` | Chiều cao ảnh đầu ra, pixel. | Chưa xác định; không tự gán giá trị. |
| `Constrain Proportions` | `boolean` | Giữ aspect khi thay một chiều. | Chưa xác định; không tự gán giá trị. |
| `Side` | `choice` | Up cao nhất hoặc Down thấp nhất. | Chưa xác định; không tự gán giá trị. |
| `Location` | `choice` | Thư viện Textures hoặc Stamps. | Chưa xác định; không tự gán giá trị. |
| `Name` | `text` | Tên tài nguyên heightfield. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-EMBOSS-013` và các vai trò Geometry, Destination; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Rasterize height theo frame và miền box, lấy extremum theo Side, lưu ảnh/metadata đơn vị và scale vào thư viện OpenMatrix9; không dùng đường dẫn thư viện thương mại.
2. C++/Qt: đăng ký và điều phối native command `OM9-EMBOSS-013`; chuyển kế hoạch Heightfield Image sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Giữ thao tác ở phạm vi trạng thái hoặc đầu ra đã chọn. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-EMBOSS-013`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-013",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Geometry", kind: InputKind::Object, min: 1, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Resolution X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Resolution Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Constrain Proportions", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Side", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Location", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Constrain Proportions giữ aspect khi đổi X hoặc Y; Up/Down khác cho geometry nhiều lớp; tên trùng xử lý rõ trước ghi, preview không ghi file.
- OM9-EMBOSS-013: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra chuyển chế độ hoặc hủy đầu ra không làm thay đổi hình học.

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
- [ ] Automated tests use feature ID `OM9-EMBOSS-013`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-EMBOSS-013` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-EMBOSS-013` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-EMBOSS-013` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-EMBOSS-013` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-EMBOSS-013` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
