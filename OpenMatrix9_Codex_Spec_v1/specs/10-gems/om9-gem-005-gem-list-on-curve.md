---
id: OM9-GEM-005
name: Gem List on Curve
command: gvGemListOnCrv
domain: 10-gems
module: Placing Gems
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-005 — Gem List on Curve

Alias tương thích: `gvGemListOnCrv`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvGemListOnCrv lấy danh sách đá có sẵn, Gem List giữ thứ tự và cuộn; lên/xuống đổi order, X xóa entry. Placement Girdle/Table/Culet, Orientation Y Axis/F4/Down/Curve/Point/Surface; Direction đảo thứ tự layout, Flip đảo culet, Pull cho Surface; Taper Diameter/Spacing/Y Offset/Z Offset, Copy duplicate hàng và Reset. Builder chỉ dùng settings phiên, không suy các size list thành template một cỡ.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Curve` | `curve` | 1 | 1 |
| `GemList` | `gem` | 1 | Không đặt trong mẫu |
| `Aim` | `object` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Gem List` | `reference` | Danh sách viên theo thứ tự; reorder/remove thay đổi đúng mục trong danh sách | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Position` | `number` | Vị trí bắt đầu dọc curve Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Spacing` | `number` | Khoảng giữa các viên Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Start Y Offset` | `number` | Dịch viên theo Y tại đầu; đơn vị khoảng cách ô nhập chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Y Offset` | `number` | Dịch viên theo Y tại cuối khi Taper Y Offset bật; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Start Z Offset` | `number` | Dịch đứng ở đầu; đơn vị khoảng cách ô nhập chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Z Offset` | `number` | Dịch đứng ở cuối khi Taper Z Offset bật; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Tilt` | `number` | Xoay viên theo Z Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Roll` | `number` | Xoay viên theo khung bố trí Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Start Diameter` | `number` | Kích thước đầu khi Taper Diameter bật; đơn vị ô nhập chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Diameter` | `number` | Kích thước cuối khi Taper Diameter bật; đơn vị ô nhập chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Placement` | `choice` | Lấy Girdle, Table hoặc Culet làm mốc đặt viên đá. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Orientation` | `choice` | Hướng culet: Y Axis, F4, Down, Curve, Point hoặc Surface; Pull chỉ có nghĩa khi chọn Surface. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Direction` | `choice` | Chiều duyệt danh sách trên curve | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Flip` | `boolean` | Đảo hướng đặt | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Pull` | `boolean` | Ép tới surface khi Orientation=Surface | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Diameter` | `boolean` | Biến thiên kích thước các viên theo điều khiển đã xác nhận | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Spacing` | `boolean` | Biến thiên khe hở | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Y Offset` | `boolean` | Biến thiên lệch Y | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Z Offset` | `boolean` | Biến thiên lệch Z | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Copy` | `boolean` | Giữ đối tượng đầu vào khi sao chép | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-005` và các vai trò Curve, Gem List, Aim; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Duyệt entries theo order/direction và giữ shape riêng mỗi entry, ánh xạ anchor/frame rồi packing theo geometry metadata.
2. C++/Qt: đăng ký và điều phối native command `gvGemListOnCrv`; chuyển kế hoạch Gem List on Curve sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-005`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-005",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "GemList", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Aim", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Gem List", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tilt", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pull", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Diameter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Y Offset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Z Offset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Danh sách Round/A/B phải giữ shapes và order, xóa entry chỉ bỏ đá đó; Direction đảo layout còn Flip chỉ đảo hướng culet.
- OM9-GEM-005: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-005`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-005` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
