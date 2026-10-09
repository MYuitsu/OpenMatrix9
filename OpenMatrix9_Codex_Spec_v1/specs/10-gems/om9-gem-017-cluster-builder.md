---
id: OM9-GEM-017
name: Cluster Builder
command: null
domain: 10-gems
module: Placing Gems
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-017 — Cluster Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Cluster Builder ba stages dùng độc lập hoặc liên tiếp. Edge Gems nhận center gem/construction curve, XY Offset mm, Z Offset dương xuống/âm lên, Degree, Edge Gem Size và Number of Gems hoặc Use Maximum Gems; Add Gems tạo accent/profile, Select/Undo riêng, Next; Start Top/Right/Bottom/Left ở 12/3/6/9 giờ. Under Bezel dùng Edge Profile, Percentage of Height và Diameter đều mặc định 50%, Distance from Girdle mm; Add Bezels, Select/Undo, Back/Next. Prong Work Base Curve nơi outer prongs blend tới, Bottom Angle, Nudge %, Inside/Outside Height/Width mm; layouts 1 Top 1 Bottom, 2 Top 1 Bottom, 1 Top 2 Bottom, 2 Top 2 Bottom. Use Base Curve, Dome Prongs, Add Inner và Add Outer đều mặc định On. Add Prongs/Back; stage Prong Work không có Undo/Select trong workflow gốc, có thể chuyển layer trước; transaction Undo trong OpenMatrix9 là quyết định bổ sung.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Center` | `gem` | 1 | 1 |
| `BaseCurve` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Stage` | `choice` | Edge Gems, Under Bezel hoặc Prong Work; giữ riêng bộ điều khiển từng giai đoạn | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `XY Offset` | `number` | Độ lệch đá viền trên XY Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Z Offset` | `number` | Lệch đá viền theo Z; dương đi xuống, âm đi lên Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Degree` | `number` | Góc đặt đá viền, không phải bậc NURBS Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Edge Gem Size` | `number` | Kích thước đá viền Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Number of Gems` | `number` | Số đá viền khi không dùng tối đa Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Use Maximum Gems` | `boolean` | Tự dùng số viên tối đa | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Start` | `choice` | Top, Right, Bottom hoặc Left tương ứng hướng 12, 3, 6, 9 giờ | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Edge Profile` | `reference` | Profile under bezel | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Percentage of Height` | `number` | Chiều cao under bezel theo chiều cao viên Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | 50% |
| `Diameter` | `number` | Đường kính under bezel theo kích thước viên Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | 50% |
| `Distance from Girdle` | `number` | Khoảng under bezel tới girdle Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Base Curve` | `reference` | Curve đế của chấu | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bottom Angle` | `number` | Góc đáy chấu Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Nudge` | `number` | Mức đè chấu lên viên Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Inside Prong Height` | `number` | Chiều cao chấu trong Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Inside Prong Width` | `number` | Bề rộng chấu trong Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Outside Prong Height` | `number` | Chiều cao chấu ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Outside Prong Width` | `number` | Bề rộng chấu ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Layout` | `choice` | 1 Top 1 Bottom, 2 Top 1 Bottom, 1 Top 2 Bottom hoặc 2 Top 2 Bottom | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Use Base Curve` | `boolean` | Dùng base curve cho chấu | On |
| `Dome Prongs` | `boolean` | Bo đầu chấu | On |
| `Add Inner` | `boolean` | Tạo chấu phía trong | On |
| `Add Outer` | `boolean` | Tạo chấu phía ngoài | On |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-017` và các vai trò Center, Base Curve; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Máy trạng thái ba stages, records accent/under bezel/prong độc lập, preserve ZOffset signed convention và graph tương ứng center; khai báo Undo bổ sung khác UI gốc.
2. C++/Qt: đăng ký và điều phối native command `OM9-GEM-017`; chuyển kế hoạch Cluster Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-017`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-017",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Center", kind: InputKind::Gem, min: 1, max: Some(1) },
        Role { name: "BaseCurve", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Stage", kind: ParameterKind::Choice, required: false },
        Parameter { name: "XY Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Gem Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Number of Gems", kind: ParameterKind::Number, required: false },
        Parameter { name: "Use Maximum Gems", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Start", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Percentage of Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Distance from Girdle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Bottom Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inside Prong Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inside Prong Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outside Prong Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outside Prong Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Use Base Curve", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Dome Prongs", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Inner", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Outer", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Height/Diameter 50% ở Under Bezel; Add Inner off chỉ outer, Use Base Curve off không blend tới base; ZOffset dương xuống, Next giữ accent outputs.
- OM9-GEM-017: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-017`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-017` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
