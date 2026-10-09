---
id: OM9-SETTING-002
name: Bezel Builder
command: gvBezelBuilder
domain: 11-settings
module: Setting Menu
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SETTING-002 — Bezel Builder

Alias tương thích: `gvBezelBuilder`. Nhóm: `11-settings`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvBezelBuilder có Bezel mặc định, Under Bezel và Gallery Rail. Bezel Edge Profile quyết side: Height từ bottom chamfer tới base, Top/Base Thickness, Dome Height, Bezel Angle, Seat Depth/Length/Angle theo pavilion, Placement Z, Scale tổng và Chamfer X/Z top edge. Under Bezel có Height tổng, Top/Base Thickness, Dome, Angle, Seat Depth/Length/Angle, Placement, Scale nhưng không các chamfer controls trên. Gallery Rail dùng Bezel Profile, Height, Top Thickness, Angle, Placement, Scale. Boolean mặc định On cắt phần intersect Finger Rail; Styles, Reset.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Gem` | `gem` | 1 | 1 |
| `Profile` | `curve` | 0 | 1 |
| `RingRail` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Bezel, Under Bezel hoặc Gallery Rail | Bezel |
| `Edge Profile` | `reference` | Profile đường biên theo gem | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bezel Profile` | `reference` | Profile mặt cắt bezel | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bezel Height` | `number` | Chiều cao bezel; nghĩa mốc khác nhau theo Mode Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Top Thickness` | `number` | Dày tại girdle hoặc dày gallery rail theo Mode Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Base Thickness` | `number` | Dày ở đáy bezel Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Dome Height` | `number` | Chiều cao bo mép trên Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bezel Angle` | `number` | Góc thành bezel Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Seat Depth` | `number` | Độ sâu viên nằm trong bezel Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Seat Length` | `number` | Chiều dài seat theo pavilion Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Seat Angle` | `number` | Góc seat Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bezel Placement` | `number` | Dịch bezel theo Z Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bezel Scale` | `number` | Tỷ lệ toàn bezel Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bezel Chamfer X` | `number` | Dịch cạnh trên chamfer theo X; chỉ Bezel mode Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bezel Chamfer Z` | `number` | Dịch cạnh trên chamfer theo Z; chỉ Bezel mode Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Boolean` | `boolean` | Cắt phần giao với finger rail trong workflow | On |
| `Styles` | `reference` | Tham chiếu bộ cấu hình builder; codec và cách lưu cần được adapter xác nhận. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SETTING-002` và các vai trò Gem, Profile, RingRail; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Mode-disc rim in a ted params và shape generator tương ứng; boolean clipping theo rail chỉ khi reference hợp lệ, preview riêng trước commit.
2. C++/Qt: đăng ký và điều phối native command `gvBezelBuilder`; chuyển kế hoạch Bezel Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SETTING-002`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "RingRail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bezel Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bezel Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Dome Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Placement", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Chamfer X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Chamfer Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Boolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Chuyển Gallery Rail không dùng seat/chamfer của Bezel; Boolean off giữ kim loại qua rail, on cắt; Top/BaseThickness đổi đúng vùng.
- OM9-SETTING-002: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SETTING-002`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SETTING-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SETTING-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SETTING-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SETTING-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SETTING-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SETTING-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SETTING-002` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
