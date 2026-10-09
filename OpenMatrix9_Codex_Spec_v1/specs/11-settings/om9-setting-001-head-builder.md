---
id: OM9-SETTING-001
name: Head Builder
command: gvHeadBuilder
domain: 11-settings
module: Setting Menu
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SETTING-001 — Head Builder

Alias tương thích: `gvHeadBuilder`. Nhóm: `11-settings`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvHeadBuilder tạo basket/half-bezel head quanh gem; layouts phụ thuộc shape, chỉ tên Prongs 4 Diagonals đã nhận diện cho round, không đặt tên các pictograms khác. Prong Profile và Rail Profile chọn thư viện. Half-bezel toggle thay profile shape và không có lựa chọn prong profile; angle handle đổi circumference ở level 1/2, position ở level 3/4. Level 1 Overall: Prong Size, Head Drop đổi base giữ top, Nudge, Dome, Rail Drop upper rails lower rail cố định ở bottom, Height Above Girdle trước dome, Head Angle X taper, Top Angle. Level 2 P/R: prong X/Y size, Nudge, Angle; rails X width/Y height, X Offset diameter, Drop; half-bezel top/bottom groups cùng circumference. Level 3 top/base prongs riêng để blend; X/Y size top/base, Nudge top overlap/base inward, Angle cùng group; từng rail; half-bezel top/base position. Level 4 độc lập từng top/base prong và rail. Rail Count 0–4, Shear Rails mặc định On biến section vuông thành parallelogram; Styles. 2 Rails thấy trong giao diện là ví dụ không default.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Gem` | `gem` | 1 | 1 |
| `ProngProfile` | `curve` | 0 | 1 |
| `RailProfile` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Prong hoặc Half-bezel; Half-bezel không dùng Prong Profile/Rail Profile | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Profile` | `reference` | Profile mặt cắt chấu trong chế độ Prong | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Rail Profile` | `reference` | Profile rail trong chế độ Prong | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Control Level` | `choice` | Level 1, 2, 3 hoặc 4; cấp cao tách điều khiển prong/rail và phần top/base | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Layout` | `choice` | Bố trí chấu theo biểu tượng; chỉ dùng danh sách layout đã có adapter xác nhận | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Size` | `number` | Kích thước chấu ở Level 1 Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Head Drop` | `number` | Dịch head theo chiều cao Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Nudge` | `number` | Độ chấu đè lên gem; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Dome` | `number` | Mức bo đầu chấu; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Rail Drop` | `number` | Vị trí rail theo chiều cao Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Height Above Girdle` | `number` | Chiều cao chấu trên girdle Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Head Angle X` | `number` | Góc nghiêng head quanh X Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Top Angle` | `number` | Góc phần đỉnh chấu Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong X Size` | `number` | Cỡ X của chấu ở cấp tách trục Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Y Size` | `number` | Cỡ Y của chấu ở cấp tách trục Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Angle` | `number` | Góc chấu đang chỉnh Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Rail X Size` | `number` | Bề rộng X của rail Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Rail Y Size` | `number` | Chiều cao Y của rail Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Rail X Offset` | `number` | Độ dịch rail vào/ra; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong X Size Top` | `number` | Cỡ X phần top ở Level 3/4 Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Y Size Top` | `number` | Cỡ Y phần top ở Level 3/4 Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong X Size Base` | `number` | Cỡ X phần base ở Level 3/4 Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Y Size Base` | `number` | Cỡ Y phần base ở Level 3/4 Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Nudge Top` | `number` | Độ chấu đè lên gem tại top; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Nudge Base` | `number` | Độ chấu đè lên gem tại base; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Rail Count` | `number` | Số rail theo các lựa chọn 0…4; adapter kiểm tra số nguyên Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Shear Rails` | `boolean` | Shear rail theo hình head | On |
| `Styles` | `reference` | Tham chiếu bộ cấu hình builder; codec và cách lưu cần được adapter xác nhận. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Half Bezel Shape Toggle` | `number` | Điều chỉnh cung/chu vi half-bezel ở Level 1/2; Level 3/4 tác động vị trí cung; không phải toggle boolean Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SETTING-001` và các vai trò Gem, Prong Profile, Rail Profile; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Hierarchical parameter scope overall→component→top/base→individual, resolve effective prong/rail dimensions rồi sweep/loft; half-bezel là generator riêng, layout phải theo gem shape.
2. C++/Qt: đăng ký và điều phối native command `gvHeadBuilder`; chuyển kế hoạch Head Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SETTING-001`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gem", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "ProngProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "RailProfile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Prong Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Rail Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Control Level", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Prong Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Prong Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Head Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Dome", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height Above Girdle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Head Angle X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong X Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Y Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail X Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail Y Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail X Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong X Size Top", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Y Size Top", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong X Size Base", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Y Size Base", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Nudge Top", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prong Nudge Base", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rail Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shear Rails", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Half Bezel Shape Toggle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Level 4 sửa một prong không đổi others; Head Drop giữ top, Shear Rails off giữ góc section; Rail Count 0 không rails,4 bốn rails; half-bezel không nhận custom Prong Profile.
- OM9-SETTING-001: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SETTING-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SETTING-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SETTING-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SETTING-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SETTING-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SETTING-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SETTING-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SETTING-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
