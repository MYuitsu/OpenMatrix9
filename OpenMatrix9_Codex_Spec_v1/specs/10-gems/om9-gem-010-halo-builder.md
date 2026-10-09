---
id: OM9-GEM-010
name: Halo Builder
command: gvHaloBuilder
domain: 10-gems
module: Placing Gems
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-010 — Halo Builder

Alias tương thích: `gvHaloBuilder`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvHaloBuilder dựng halo quanh stone. Profile section từ library; Spacing From Center là khe halo–center, Gems Height Offset Z, Spacing tối thiểu đá nhỏ, Angle đổi đá và side halo, Rotation xoay stones, Placement đổi Z halo. Fix Position chỉ round chuyển fixed gem 12→3 giờ. Profile Width/Height; Inner/Outer Channel Wall Thickness và Channel Spacing tác động size đá/prongs; Channel Wall Height và Channel Blend (lớn mềm). Prongs Height Offset trên table, Dome Amount, Taper % base so với top (âm base nhỏ), Edge Nudge vị trí trong channel, Gem Nudge phần chồng đá có thể về không prongs, Prongs Vertical theo culet. Override chọn shape halo khác stone; Match Shape hợp halo symmetric với stone dài/rộng; Even đổi vị trí bắt đầu; Prong Meshed tạo mesh thay NURBS; Load styles, Reset.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `CenterGem` | `gem` | 1 | 1 |
| `Profile` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Profile` | `reference` | Profile mặt cắt halo | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Spacing From Center` | `number` | Khoảng cách halo tới viên trung tâm Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Gems Height Offset` | `number` | Đặt viên phụ theo Z Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Spacing` | `number` | Khe hở nhỏ nhất giữa viên phụ Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Angle` | `number` | Góc cạnh halo và các viên phụ Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Rotation` | `number` | Góc xoay các viên phụ Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Placement` | `number` | Dịch halo theo Z Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Fix Position` | `number` | Điều khiển vị trí viên cố định từ hướng 12 giờ sang 3 giờ, chỉ với viên trung tâm tròn; ánh xạ độ dài sang vị trí cần xác nhận Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile Width` | `number` | Bề rộng halo, đồng thời ảnh hưởng kích thước đá/chấu Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile Height` | `number` | Chiều cao halo Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Inner Channel Wall Thickness` | `number` | Dày thành gần viên trung tâm Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Outer Channel Wall Thickness` | `number` | Dày thành phía ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Channel Spacing` | `number` | Khoảng đá tới thành channel Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Channel Wall Height` | `number` | Chiều cao thành channel Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Channel Blend` | `number` | Mức bo thành channel; lớn tạo chuyển tiếp mềm hơn Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prongs- Height Offset` | `number` | Độ cao chấu nhô qua table Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prongs-Dome Amount` | `number` | Mức làm tròn đầu chấu; ví dụ 0 và 100 là hai trạng thái minh họa, đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prongs-Taper` | `number` | Thay đổi đường kính đáy so với đỉnh; âm đáy nhỏ, dương đáy lớn Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Edge Nudge` | `number` | Vị trí chấu trong channel Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Gem Nudge` | `number` | Mức chấu đè lên viên phụ Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prongs Vertical` | `number` | Điều chỉnh vị trí đáy chấu so với culet; đây là kích thước, không phải toggle Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Override` | `boolean` | Cho phép chọn hình halo khác hình viên trung tâm | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Gem Shape` | `choice` | Hình halo thay thế khi Override bật | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Match Shape` | `boolean` | Co halo đối xứng theo tỷ lệ dài/rộng của viên trung tâm | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Even` | `boolean` | Đổi pha bắt đầu của các viên phụ | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Meshed` | `boolean` | Bật tạo chấu mesh; tắt tạo dạng NURBS | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Styles` | `reference` | Tham chiếu bộ cấu hình builder; codec và cách lưu cần được adapter xác nhận. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-010` và các vai trò Center Gem, Profile; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tính halo rail từ center/override và offset, giải channel free width trước packing accent gems và prongs; phụ thuộc size phải recompute đồng thời.
2. C++/Qt: đăng ký và điều phối native command `gvHaloBuilder`; chuyển kế hoạch Halo Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-010`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-010",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "CenterGem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Spacing From Center", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gems Height Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fix Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Channel Wall Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Channel Wall Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Channel Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Channel Wall Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Channel Blend", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs- Height Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs-Dome Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs-Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs Vertical", kind: ParameterKind::Number, required: false },
        Parameter { name: "Override", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gem Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Match Shape", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Even", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Prong Meshed", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Tăng wall thickness giảm chỗ đá và đổi size/prongs; Prong Meshed đổi kiểu đầu ra, Fix Position chỉ round; Override off halo giữ shape stone.
- OM9-GEM-010: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-010`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-010` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
