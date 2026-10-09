---
id: OM9-GEM-023
name: Pavé Prong Builder
command: null
domain: 10-gems
module: Placing Gems
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-023 — Pavé Prong Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Pavé Prong Builder tạo center chạm 3 gems, shared chạm 2 và unshared sát edge chạm 1. Center Prong Minimum/Maximum mm, Shared/Unshared Prong Size mm, Height từ girdle mm, Drop cho base xuyên surface, Taper % top/base dưới 100 top nhỏ, trên 100 top lớn; Fillet 0 flat,100 hemisphere; Scale % đồng loạt từ cỡ riêng. Set Prongs/Reset; prongs không tham gia Match Attributes, Styles hoặc History, sửa phải xóa và chạy lại.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Surface` | `surface` | 1 | 1 |
| `Gems` | `gem` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Center Prong Minimum` | `number` | Cỡ chấu trung tâm nhỏ nhất Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Center Prong Maximum` | `number` | Cỡ chấu trung tâm lớn nhất Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Shared Prong Size` | `number` | Cỡ chấu dùng chung Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Unshared Prong Size` | `number` | Cỡ chấu không dùng chung Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Height` | `number` | Chiều cao tính từ girdle Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Drop` | `number` | Dịch chấu xuống Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper` | `number` | Tỷ lệ đỉnh so với đáy; dưới 100% đỉnh nhỏ hơn đáy Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Fillet` | `number` | Bo đầu chấu: 0% phẳng, 100% bán cầu Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Scale` | `number` | Tỷ lệ kích thước toàn bộ chấu Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-023` và các vai trò Surface, Gems; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tính adjacency stones và phân sites 1/2/3 neighbors, tạo prongs theo local surface normals và dimensions; không đăng ký style/history compatibility giả.
2. C++/Qt: đăng ký và điều phối native command `OM9-GEM-023`; chuyển kế hoạch Pavé Prong Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-023`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-023",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Center Prong Minimum", kind: ParameterKind::Number, required: false },
        Parameter { name: "Center Prong Maximum", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shared Prong Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Unshared Prong Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fillet", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Centers chạm đúng 3 stones, Shared đúng 2; Fillet 0/100 đạt flat/hemisphere; Match Attributes phải không liệt kê pavé prong style.
- OM9-GEM-023: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-023`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-023` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
