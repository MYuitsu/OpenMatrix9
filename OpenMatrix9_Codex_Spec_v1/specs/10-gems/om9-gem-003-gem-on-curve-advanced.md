---
id: OM9-GEM-003
name: Gem on Curve Advanced
command: gvGemOnCrvAdvanced
domain: 10-gems
module: Placing Gems
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-003 — Gem on Curve Advanced

Alias tương thích: `gvGemOnCrvAdvanced`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvGemOnCrvAdvanced đặt đá với Start/End Position, Start/End Diameter (end khi Taper Diameter), Spacing mm, Start/End Y Offset và Z Offset độc lập khi Taper tương ứng, Tilt quanh Z và Roll. Gem Shape chọn loader. Placement Girdle/Table/Culet; Orientation Y Axis mặc định, F4, Down, Curve, Point, Surface; Pull chỉ dùng Surface. Spacing/Size Gems/Fixed tương ứng li nh hoạt khoảng, size, endpoints. Close trên curve kín nối đầu/cuối; Mirror đặt end ở giữa, Flip đảo culet; Taper Spacing, Copy tạo hàng duplicate để offset, Reset. Enter chốt, không mở lại chỉnh group như bản thường; chỉ nhớ settings trong cùng phiên. Thiết kế phải xử lý Close và Mirror như hai layout khác để tránh đầu/cuối mâu thuẫn.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Curve` | `curve` | 1 | 1 |
| `Aim` | `object` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Start Position` | `number` | Vị trí đầu dọc curve Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Position` | `number` | Vị trí cuối dọc curve Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Start Diameter` | `number` | Đường kính đầu khi Taper Diameter bật Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Diameter` | `number` | Đường kính cuối khi Taper Diameter bật Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Spacing` | `number` | Khe hở giữa các viên Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Start Y Offset` | `number` | Lệch Y đầu Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Y Offset` | `number` | Lệch Y cuối khi Taper Y Offset bật Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Start Z Offset` | `number` | Lệch Z đầu Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Z Offset` | `number` | Lệch Z cuối khi Taper Z Offset bật Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Tilt` | `number` | Xoay theo Z của khung đặt Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Roll` | `number` | Xoay viên theo khung đặt Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Gem Shape` | `choice` | Hình viên chọn qua Gem Loader | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Placement` | `choice` | Lấy Girdle, Table hoặc Culet làm mốc đặt viên đá. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Orientation` | `choice` | Hướng culet: Y Axis, F4, Down, Curve, Point hoặc Surface; Pull chỉ có nghĩa khi chọn Surface. | Y Axis |
| `Spacing Method` | `choice` | Spacing phân phối khe hở; Size Gems đổi kích thước để giữ khe hở và hai đầu; Fixed giữ kích thước/khe hở và cho phép lệch điểm cuối. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Pull` | `boolean` | Ép bố trí tới surface khi Orientation=Surface | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Close` | `boolean` | Bố trí theo vòng khép | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Mirror` | `boolean` | Đồng bộ điều khiển hai đầu | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Flip` | `boolean` | Đảo hướng đặt | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Diameter` | `boolean` | Nội suy đường kính đầu/cuối | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Spacing` | `boolean` | Biến thiên khe hở dọc curve | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Y Offset` | `boolean` | Nội suy lệch Y | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Z Offset` | `boolean` | Nội suy lệch Z | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Copy` | `boolean` | Giữ bản gốc theo chế độ sao chép | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-003` và các vai trò Curve, Aim; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Giữ trạng thái phiên và rows copies riêng, packing với diameter/offset taper, en force target required theo Orientation; không ghi editable History giả sau commit.
2. C++/Qt: đăng ký và điều phối native command `gvGemOnCrvAdvanced`; chuyển kế hoạch Gem on Curve Advanced sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-003`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Aim", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Start Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tilt", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Spacing Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Pull", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
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

- Taper Y off không dùng End Y; Surface mới nhận Pull; Copy giữ hàng cũ và offset hàng mới; group sau Enter không quả ng cáo Edit bản thường.
- OM9-GEM-003: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-003`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-003` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
