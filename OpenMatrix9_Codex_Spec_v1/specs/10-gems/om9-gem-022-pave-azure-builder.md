---
id: OM9-GEM-022
name: Pavé Azure Builder
command: null
domain: 10-gems
module: Placing Gems
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-022 — Pavé Azure Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Pavé Azure Builder chỉ cho layout Hex sau chỉnh gems. Nhập Top nơi đặt gems, Bottom là mặt lưng thoát holes, rồi Gems tạo previews. Hexagonal Toggle đổi circle sang hex azures; Azure Depth điều chỉnh middle circle để định cỡ/góc cut, Azure Size là lower exit profile, Through Hole là middle profile, Seat Angle định seat curve; Set Azures tạo azures, không tự quy thành boolean nếu chưa xác minh.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Top` | `surface` | 1 | 1 |
| `Bottom` | `surface` | 1 | 1 |
| `Gems` | `gem` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Top` | `reference` | Surface mang gem | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bottom` | `reference` | Surface thoát lỗ azure | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Gems` | `reference` | Bố trí gem hex được dùng tạo azure | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Hexagonal Toggle` | `boolean` | Bật dùng tiết diện hex; tắt dùng tiết diện tròn | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Azure Depth` | `number` | Vị trí vòng giữa điều khiển chiều sâu/hướng azure; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Azure Size` | `number` | Kích thước đường thoát trên surface đáy; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Through Hole` | `number` | Kích thước vòng giữa của lỗ xuyên; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Seat Angle` | `number` | Góc seat của azure Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-022` và các vai trò Top, Bottom, Gems; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Giải trục stone và giao Top/Bottom, loft cutter sections circle/hex với seat và through-hole; tạo cutter objects riêng.
2. C++/Qt: đăng ký và điều phối native command `OM9-GEM-022`; chuyển kế hoạch Pavé Azure Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-022`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-022",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Top", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Bottom", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Top", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bottom", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gems", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Hexagonal Toggle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Azure Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Azure Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Through Hole", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Circle/hex toggle đổi profile đầu ra; Azure Size đổi exit footprint trên Bottom, Through Hole không đổi stone size; non hex layouts báo unsupported.
- OM9-GEM-022: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-022`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-022` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
