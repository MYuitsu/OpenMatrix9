---
id: OM9-CURVE-007
name: Arc
command: Arc
domain: 02-curve
module: Curve Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-CURVE-007 — Arc

Alias tương thích: `Arc`. Nhóm: `02-curve`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-007` — Arc.**

Nhánh chính chọn tâm, điểm đầu rồi điểm cuối hoặc góc. Tilted bổ sung hướng mặt phẳng lệch CPlane; Length nhận chiều dài có dấu hoặc hai điểm đo. Deformable xuất NURBS theo Degree/PointCount; muốn bậc p cần đủ p+1 control point. StartPoint chọn đầu, cuối và điểm đi qua; Direction đổi sang đầu–tiếp tuyến–cuối; ThroughPoint đổi thứ tự đầu–đi qua–cuối; Center lấy tâm. Tangent nhận hai curve với Radius tùy chọn, curve thứ ba hoặc Enter để tạo circle; chọn nghiệm và chiều bằng con trỏ. Point bỏ ràng buộc tiếp xúc, FromFirstPoint khóa vị trí đầu, Radius chỉ ở giai đoạn curve thứ hai. Extension chọn curve rồi đầu cuối của cung mới; Center chọn tâm trong mặt vuông góc đầu curve. Cung Extension là object riêng; muốn nối dùng Extend. Hướng pick cần CPlane/Ortho/Shift nhất quán.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `picked_points` | `point` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Tilted` | `boolean` | Đổi mặt phẳng cung | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Length` | `number` | Chiều dài cung có dấu theo đơn vị document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Angle` | `number` | Góc quét, độ tại UI | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Deformable` | `boolean` | Xấp xỉ NURBS | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Degree` | `number` | Bậc xấp xỉ | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `PointCount` | `number` | Số control point | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `StartPoint` | `boolean` | Nhánh chọn đầu cung | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Direction` | `reference` | Tiếp tuyến đầu | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `ThroughPoint` | `reference` | Điểm đi qua | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Center` | `reference` | Tâm cung | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Tangent` | `boolean` | Nhánh tiếp xúc curve | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Point` | `reference` | Điểm không bị ràng buộc tangent | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `FromFirstPoint` | `boolean` | Khóa vị trí đầu | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Radius` | `number` | Bán kính theo đơn vị document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Extension` | `boolean` | Cung kéo dài riêng | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-007` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Tách solver tâm/góc, ba điểm, tiếp tuyến và extension; lưu hướng cung và phạm vi tham số, không tự Join nhánh Extension.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-007` tới command native `Arc` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Tilted", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Deformable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "StartPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "ThroughPoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Point", kind: ParameterKind::Reference, required: false },
        Parameter { name: "FromFirstPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extension", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Cung tâm bán kính 5 và góc 90 độ có chiều dài 5π/2; Extension không thay object gốc; bậc vượt số điểm phải có chẩn đoán.
- Cancel ở giữa phiên không thay object/layer; lỗi solver không commit output một phần. Undo/Redo phục hồi toàn bộ thao tác; save/reload bảo toàn geometry, placement và links theo hợp đồng.
- Kiểm tra NaN/Infinity bị từ chối và UI cùng CMD cho cùng kết quả với cùng input; các options chưa có mặc định yêu cầu xác nhận giá trị đang hiển thị.

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
- [ ] Automated tests use feature ID `OM9-CURVE-007`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CURVE-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CURVE-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CURVE-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CURVE-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CURVE-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-CURVE-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-CURVE-007` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
