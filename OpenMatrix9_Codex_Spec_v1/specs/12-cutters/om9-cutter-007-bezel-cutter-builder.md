---
id: OM9-CUTTER-007
name: Bezel Cutter Builder
command: gvBezelCutter
domain: 12-cutters
module: Creating Cutters
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-CUTTER-007 — Bezel Cutter Builder

Alias tương thích: `gvBezelCutter`. Nhóm: `12-cutters`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvBezelCutter tạo scallop cutters quanh gem/bezel; Edge Profiles hover preview và editor/library. X/Y Inside Size là width/height gần stone, Outside Size xa stone mm; Cutter Length mm, Angle độ, Z Offset lên/xuống, Y Offset vào/ra, X Offset chuyển vị trí quanh/xa stone, Number of Cutters. Position Odd giữ metal trên/dưới stone, Even offset khỏi hai vị trí đó. Enter tạo cutters rồi Boolean Difference riêng; Boolean interactive chọn Surface thấy cut preview và Enter chốt; Styles/Reset.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `GemOrBezel` | `object` | 1 | 1 |
| `Target` | `solid` | 0 | 1 |
| `Profile` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Edge Profile` | `reference` | Profile đường cắt | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `X Inside Size` | `number` | Kích thước X phía trong Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Y Inside Size` | `number` | Kích thước Y phía trong Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `X Outside Size` | `number` | Kích thước X phía ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Y Outside Size` | `number` | Kích thước Y phía ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Cutter Length` | `number` | Chiều dài cutter Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Angle` | `number` | Góc cutter Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Z Offset` | `number` | Dịch Z Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Y Offset` | `number` | Dịch Y Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `X Offset` | `number` | Dịch X Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Number Of Cutters` | `number` | Số cutter; adapter kiểm tra số nguyên Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Position` | `choice` | Odd hoặc Even cho pha bố trí | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Boolean` | `boolean` | Hiện preview cắt trên surface được chọn | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Surface` | `reference` | Surface/khối tiếp nhận phép cắt | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Styles` | `reference` | Tham chiếu bộ cấu hình builder; codec và cách lưu cần được adapter xác nhận. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-CUTTER-007` và các vai trò Gem Or Bezel, Target, Profile; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Dựng array cutters theo gem boundary và parity, loft section inside/outside; interactive boolean là nhánh target required, không tự cắt khi off.
2. C++/Qt: đăng ký và điều phối native command `gvBezelCutter`; chuyển kế hoạch Bezel Cutter Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-CUTTER-007`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "GemOrBezel", kind: InputKind::Object, min: 1, max: Some(1) },
        Role { name: "Target", kind: InputKind::Solid, min: 0, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "X Inside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Inside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Outside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Outside Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cutter Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Number Of Cutters", kind: ParameterKind::Number, required: false },
        Parameter { name: "Position", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Odd/Even thay metal còn lại đúng vị trí; count đúng, inside/outside dimensions riêng; off Enter chỉ cutters, on target valid mới cut.
- OM9-CUTTER-007: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-CUTTER-007`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CUTTER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CUTTER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CUTTER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CUTTER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CUTTER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-CUTTER-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-CUTTER-007` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
