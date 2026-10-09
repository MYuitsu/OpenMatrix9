---
id: OM9-SETTING-010
name: Bead on Surface
command: gvOrientOnSurface
domain: 11-settings
module: Setting Menu
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SETTING-010 — Bead on Surface

Alias tương thích: `gvOrientOnSurface`. Nhóm: `11-settings`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvOrientOnSurface Bead mặc định đặt spheres; Object nhận object tại F4, CMD prompt khi đổi mode. Scale, Vertical Offset, Object Angle chỉ Object mode. Copy mặc định On đặt nhiều, off chỉ một rồi restart; Start dùng surface cũ hoặc hỏi mới, Flip đảo, Curve ràng buộc object lên cả target surface/curve, Reset; Enter hoàn.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Surface` | `surface` | 1 | 1 |
| `Object` | `object` | 0 | 1 |
| `Curve` | `curve` | 0 | 1 |
| `Locations` | `point` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Bead hoặc Object | Bead |
| `Scale` | `number` | Kích thước bead/đối tượng; phím Q/E thay đổi 10%, nhưng đơn vị ô nhập chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Vertical Offset` | `number` | Khoảng bead tới surface; 0 để tâm sphere nằm trên surface Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Object Angle` | `number` | Góc đối tượng; chỉ có trong Object mode Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Copy` | `boolean` | Tiếp tục đặt nhiều bead/đối tượng | On |
| `Flip` | `boolean` | Đảo phía normal | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Curve` | `reference` | Curve ràng buộc bố trí với surface | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SETTING-010` và các vai trò Surface, Object, Curve, Locations; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Mode Bead/Object có template origin contract, frame normal/tangent và projection curve; giữ last params phiên.
2. C++/Qt: đăng ký và điều phối native command `gvOrientOnSurface`; chuyển kế hoạch Bead on Surface sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SETTING-010`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-010",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Object", kind: InputKind::Object, min: 0, max: Some(1) },
        Role { name: "Curve", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "Locations", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertical Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Object Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Curve", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Copy On nhiều click tạo nhiều beads, off một; Object không tại F4 cần xử lý rõ, Object Angle không đổi sphere silhouette; Curve giữ points trên curve/mặt.
- OM9-SETTING-010: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SETTING-010`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SETTING-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SETTING-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SETTING-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SETTING-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SETTING-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SETTING-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SETTING-010` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
