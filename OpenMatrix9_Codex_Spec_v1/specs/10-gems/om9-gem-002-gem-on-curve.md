---
id: OM9-GEM-002
name: Gem on Curve
command: gvGemOnCrv
domain: 10-gems
module: Placing Gems
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-002 — Gem on Curve

Alias tương thích: `gvGemOnCrv`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvGemOnCrv preview đá dọc rail; Enter chốt và F6 Edit Gem on Curve mở lại group để sửa. Start/End Position là vị trí dọc rail theo phần trăm trong workflow (overlay có thể không ghi %); Spacing mm, Z Offset, Roll độ, Start Size và End Size khi Taper Gems. Mirror đặt end handle ở giữa, start điều khiển hai phía. Culet hướng F4/Down/Object (point/curve/surface); Flip đảo; Gem Tops on đặt table sát rail, off girdle. Taper Spacing tăng khe đá lớn. Spacing giữ minimum và khoảng bằng nhau, Size Gems thay size để giữ spacing/start/end, Fixed giữ size/spacing và nới endpoints. Object Type RoundGem mặc định, SelectGem/Object từ F4, Sphere hoặc Browser; SnaptoCorners đặt ở góc, ShowProngs mở preview và chuyển Prong Builder. Styles giữ cấu hình; không suy carat từ over lays.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Curve` | `curve` | 1 | 1 |
| `Template` | `gem` | 0 | 1 |
| `Aim` | `object` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Start Position` | `number` | Vị trí đầu tính dọc đường dẫn Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Position` | `number` | Vị trí cuối tính dọc đường dẫn Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Spacing` | `number` | Khe hở giữa các viên Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Z Offset` | `number` | Độ lệch theo Z của khung đặt Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Roll` | `number` | Xoay viên trong khung đặt Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Start Size` | `number` | Kích thước đầu khi Taper Gems bật Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Size` | `number` | Kích thước cuối khi Taper Gems bật Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Gems` | `boolean` | Nội suy kích thước từ đầu tới cuối | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper Spacing` | `boolean` | Thay đổi khe hở dọc đường dẫn | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Mirror` | `boolean` | Đồng bộ hai phía từ tay nắm giữa | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Culet` | `choice` | Ngắm culet theo F4, Down hoặc Object | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Object` | `reference` | Điểm, curve hoặc surface dùng làm đích hướng culet | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Flip` | `boolean` | Đảo hướng đặt | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Gem Tops` | `boolean` | Bật dùng table làm mốc; tắt dùng girdle | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Spacing Method` | `choice` | Spacing phân phối khe hở; Size Gems đổi kích thước để giữ khe hở và hai đầu; Fixed giữ kích thước/khe hở và cho phép lệch điểm cuối. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Object Type` | `choice` | RoundGem, SelectGem, Object, Sphere hoặc Browser; Object sử dụng vị trí F4. | RoundGem |
| `SelectGem` | `reference` | Viên mẫu dùng trong chế độ SelectGem | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Browser` | `reference` | Đối tượng thư viện được chọn | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `SnaptoCorners` | `boolean` | Bám góc của đường dẫn | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `ShowProngs` | `boolean` | Hiện gợi ý chấu; mở Prong Builder để tạo chấu | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Styles` | `reference` | Tham chiếu bộ cấu hình builder; codec và cách lưu cần được adapter xác nhận. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-002` và các vai trò Curve, Template, Aim; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Packing theo độ dài rail và mode li nh hoạt, frame đá theo aim, anchor table/girdle; lưu group builder params để F6 sửa được.
2. C++/Qt: đăng ký và điều phối native command `gvGemOnCrv`; chuyển kế hoạch Gem on Curve sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-002`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Template", kind: InputKind::Gem, min: 0, max: Some(1) },
        Role { name: "Aim", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Start Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper Gems", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Culet", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Object", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gem Tops", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Spacing Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Object Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "SelectGem", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Browser", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SnaptoCorners", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ShowProngs", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Fixed giữ size/spacing dù endpoints lệch; Size Gems đổi cỡ chứ không spacing; Enter rồi F6 phải khôi phục params và có group editable.
- OM9-GEM-002: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-002`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-002` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
