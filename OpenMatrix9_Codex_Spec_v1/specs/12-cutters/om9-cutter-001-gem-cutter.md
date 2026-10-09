---
id: OM9-CUTTER-001
name: Gem Cutter
command: gvAzureCutter
domain: 12-cutters
module: Creating Cutters
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-CUTTER-001 — Gem Cutter

Alias tương thích: `gvAzureCutter`. Nhóm: `12-cutters`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvAzureCutter ba modes Gem mặc định, Azure, Advanced; dimensions dựa %stone trừ ZOffset cần xác minh unit theo điều khiển. Gem: Lower Taper từ lower seat tới bottom, Lower Depth girdle→bottom, Lower Seat Size diameter/position pavilion, Girdle Scale, Girdle Width, ZOffset. Azure thêm Edge Profile,1 Rotation/1 Y Scale lower profile,2 Rotation/2 Y Scale lower seat, Lower Seat Depth. Advanced thêm 3 Rotation/3 Y Scale girdle,4 Rotation/4 Y Scale table, Upper Seat Size/Height,5 Rotation/5 Y Scale top, Upper Taper/Height. Rotation độ, Y Scale %, Seat/Girdle thickness/Depth % tương đối stone. Interactive Boolean Off mặc định, On preview all, One preview một; Surface required khi On/One, Bottom Dir pick point hướng bottom, Rebuild curves nếu boolean fail; F6 tự mở Style Library, Styles/Reset. Lower Depth âm xuất hiện như ví dụ chỉnh, không suy default/range.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Gems` | `gem` | 1 | Không đặt trong mẫu |
| `Target` | `solid` | 0 | 1 |
| `Direction` | `point` | 0 | 1 |
| `Profile` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Gem, Azure hoặc Advanced | Gem |
| `Lower Taper` | `number` | Độ taper phần dưới theo kích thước gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Lower Depth` | `number` | Chiều sâu phần dưới theo kích thước gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Lower Seat Size` | `number` | Kích thước seat dưới theo gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Girdle Scale` | `number` | Tỷ lệ phần girdle theo gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Girdle Width` | `number` | Bề rộng vùng girdle theo gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Z Offset` | `number` | Dịch Z của cutter; đơn vị chưa xác lập, không suy từ những điều khiển phần trăm khác Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Edge Profile` | `reference` | Profile biên trong Azure | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `1 Rotation` | `number` | Góc profile thứ nhất trong Azure/Advanced Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `1 Y Scale` | `number` | Tỷ lệ Y profile thứ nhất Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `2 Rotation` | `number` | Góc profile thứ hai Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `2 Y Scale` | `number` | Tỷ lệ Y profile thứ hai Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Lower Seat Depth` | `number` | Độ sâu seat dưới theo gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `3 Rotation` | `number` | Góc profile thứ ba trong Advanced Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `3 Y Scale` | `number` | Tỷ lệ Y profile thứ ba Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `4 Rotation` | `number` | Góc profile thứ tư trong Advanced Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `4 Y Scale` | `number` | Tỷ lệ Y profile thứ tư Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `5 Rotation` | `number` | Góc profile thứ năm trong Advanced Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `5 Y Scale` | `number` | Tỷ lệ Y profile thứ năm Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Upper Seat Size` | `number` | Kích thước seat trên theo gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Upper Seat Height` | `number` | Cao seat trên theo gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Upper Taper` | `number` | Taper trên theo gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Upper Height` | `number` | Chiều cao phần trên theo gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Interactive Boolean` | `choice` | Off, On hoặc One; đây là trạng thái ba giá trị, không phải boolean | Off |
| `Surface` | `reference` | Đối tượng cần cắt khi Interactive Boolean=On/One | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bottom Dir` | `reference` | Điểm hướng đáy cutter | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Styles` | `reference` | Tham chiếu bộ cấu hình builder; codec và cách lưu cần được adapter xác nhận. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-CUTTER-001` và các vai trò Gems, Target, Direction, Profile; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Loft năm section theo mode và gem frame, profile choice/taper riêng; interactive boolean bounded targets và preview one; cho phép signed depth theo policy đã kiểm chứng.
2. C++/Qt: đăng ký và điều phối native command `gvAzureCutter`; chuyển kế hoạch Gem Cutter sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-CUTTER-001`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CUTTER-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Target", kind: InputKind::Solid, min: 0, max: Some(1) },
        Role { name: "Direction", kind: InputKind::Point, min: 0, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Lower Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lower Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lower Seat Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Girdle Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Girdle Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "1 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "1 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "2 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "2 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lower Seat Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "3 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "3 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "4 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "4 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "5 Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "5 Y Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Upper Seat Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Upper Seat Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Upper Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Upper Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Interactive Boolean", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bottom Dir", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Gem mode không nhận advanced params, One chỉ cắt một preview, Off chỉ cutter; Lower Depth âm không bị từ chối vì gán range tùy tiện, Rebuild không xóa stone.
- OM9-CUTTER-001: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-CUTTER-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CUTTER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CUTTER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CUTTER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CUTTER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CUTTER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-CUTTER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-CUTTER-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
