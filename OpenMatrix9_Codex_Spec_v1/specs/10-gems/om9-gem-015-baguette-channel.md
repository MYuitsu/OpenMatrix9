---
id: OM9-GEM-015
name: Baguette Channel
command: gvTaperBagChannel
domain: 10-gems
module: Placing Gems
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-015 — Baguette Channel

Alias tương thích: `gvTaperBagChannel`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvTaperBagChannel bố trí baguettes giữa hai curves; marker Start và lines hướng tables. Lists Size, Aspect (% taper tới 100% straight), Banking độ cho pair nonplanar thêm/xóa từng entry; Spacing mm. Flip đảo table so normal, Middle căn đầu từ giữa stone thay edge, Straight bỏ taper/aspect, Change Start đổi vị trí bắt đầu; Apply tính kết quả.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Curves` | `curve` | 2 | 2 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Size List` | `reference` | Danh sách kích thước theo thứ tự; tham chiếu dữ liệu danh sách, không parse thành một số | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Size` | `number` | Kích thước mục đang chỉnh trong danh sách Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Aspect` | `number` | Tỷ lệ hai đầu baguette; 100% tạo dạng thẳng Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Banking` | `number` | Góc banking cho cặp curve không phẳng Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Spacing` | `number` | Khe hở giữa các baguette Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Flip` | `boolean` | Đảo định hướng bố trí | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Middle` | `boolean` | Canh giữa bố trí | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Straight` | `boolean` | Vô hiệu taper/aspect để dùng baguette thẳng | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-015` và các vai trò Curves; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tạo mapping hai rails và lựa chọn size/aspect/banking từ allowed lists, packing từng stone theo frame; chưa có solver tối ưu cụ thể nên dùng policy có báo sai lệch.
2. C++/Qt: đăng ký và điều phối native command `gvTaperBagChannel`; chuyển kế hoạch Baguette Channel sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-015`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-015",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Size List", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Aspect", kind: ParameterKind::Number, required: false },
        Parameter { name: "Banking", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Middle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Straight", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Straight tạo stone không taper, Aspect=100% straight; Middle dịch start theo half stone; Flip đảo hướng table, lists dùng đúng entry hợp lệ.
- OM9-GEM-015: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-015`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-015` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-015` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-015` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-015` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-015` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-015` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-015` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
