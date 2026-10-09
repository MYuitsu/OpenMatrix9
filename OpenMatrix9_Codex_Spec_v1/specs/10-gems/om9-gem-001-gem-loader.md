---
id: OM9-GEM-001
name: Gem Loader
command: gvLoader
domain: 10-gems
module: Placing Gems
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-001 — Gem Loader

Alias tương thích: `gvLoader`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvLoader đặt hoặc thay đá: chọn Cut Type, Gem Shape và Gem Size (mm diameter round hoặc length/width), nhiều tab cho cỡ khác. Diamond mặc định; Simple giảm độ phức tạp, Gem, Cabochon và Custom có catalog riêng. Keep Original Size giữ kích thước đo, không phải carat khi đổi shape/size. Đá mới ở F4, chọn đá tồn tại thì thay tại chỗ. Custom Gem Sizes nhập Length, Width, Depth, Percentage rồi mũi tên; On Surface mở builder Gem on Surface. Các cỡ Round thấy trong minh họa gồm 1.0–2.2 từng 0.1, 2.4, 2.5, 2.7–3.0 từng 0.1, 3.2, 3.3, 3.4 nhưng không phải toàn catalog hay miền liên tục; 1.40 D/0.01 ct là ví dụ không phải công thức carat.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Document` | `document` | 1 | 1 |
| `ExistingGem` | `gem` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Cut Type` | `choice` | Chọn Diamond, Simple, Gem, Cabochon hoặc Custom theo thư viện. | Diamond |
| `Gem Shape` | `choice` | Chọn hình viên đá từ nhóm cut hiện hành | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Gem Size` | `number` | Đường kính viên tròn hoặc kích thước dài/rộng của hình khác Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Keep Original Size` | `boolean` | Giữ kích thước đo của viên khi đổi mẫu; không giữ carat | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Length` | `number` | Chiều dài custom gem Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Width` | `number` | Chiều rộng custom gem Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Depth` | `number` | Chiều sâu custom gem Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Percentage` | `number` | Điều chỉnh tỷ lệ của custom gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `On Surface` | `boolean` | Chuyển sang workflow đặt gem trên surface | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-001` và các vai trò Document, Existing Gem; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Đọc catalog hình học tự tạo/cấp phép theo cut/shape/size và metadata; áp Keep Original Size trên dimensions và giữ transform khi thay đá.
2. C++/Qt: đăng ký và điều phối native command `gvLoader`; chuyển kế hoạch Gem Loader sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-001`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "ExistingGem", kind: InputKind::Gem, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Cut Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Gem Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Gem Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Keep Original Size", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Percentage", kind: ParameterKind::Number, required: false },
        Parameter { name: "On Surface", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Đổi shape khi Keep Original Size giữ measurement, không giữ carat; đá mới tại F4 còn replace giữ transform/material policy rõ; tab không bị coi là toàn miền size.
- OM9-GEM-001: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
