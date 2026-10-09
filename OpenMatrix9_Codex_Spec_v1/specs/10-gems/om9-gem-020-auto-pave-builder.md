---
id: OM9-GEM-020
name: Auto Pavé Builder
command: null
domain: 10-gems
module: Placing Gems
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-020 — Auto Pavé Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Auto Pavé Builder cần Surface và Start, Run preview rồi Done set gems/exit; Flip hướng. Start chọn đúng một Base Direction (point và xoay hex), Seed Gems, Curve hoặc Edges; chuẩn bị seeds/curves trước vào builder. Edges dùng surface edges hoặc Edge Curve, min/max edge size nằm trong overall min/max; Lock Start bảo vệ stones đầu khỏi correction. Edge Curve giới hạn miền, Attractor hướng layout nhẹ theo curve; Auto Attractor chỉ khi Start Curve. Layout Hex mặc định, Hexr tính outside-in, Square hai hàng trực giao, Tri, HexPent nhóm năm hexagons, All Directions random; Incremental, Slow, Edges bổ sung strategy. Sizing Base Size/Min/Max, Taper cao tăng biến thiên; Basic ít đổi, Normal thiên grow, Grow ép grow, Shrink giảm đá ngoài. Spacing Tight/Loose và Min; Pre-Sizing Equalize uniform/Perturb random; Post-Sizing Equalize/Perturb interactions, Perturb Grow lớn đá thay thêm đá. Styles Save/Load giữ settings nhưng refs cần nhập lại; Results Weight/Count/Time; Display hiện graph geodesic và links interactions thay gems trong lúc Run.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Surface` | `surface` | 1 | 1 |
| `Start` | `point` | 0 | 1 |
| `SeedGems` | `gem` | 0 | Không đặt trong mẫu |
| `Curves` | `curve` | 0 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Start` | `choice` | Base Direction, Seed Gems, Curve hoặc Edges | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Surface` | `reference` | Surface tiếp nhận bố trí | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Seed Gems` | `reference` | Các viên khởi tạo khi Start=Seed Gems | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Curve` | `reference` | Curve khởi tạo khi Start=Curve | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Lock Start` | `boolean` | Khóa điều kiện khởi tạo | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Edge Curve` | `reference` | Curve biên | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Edge Minimum` | `number` | Kích thước nhỏ nhất của gem biên khi Start=Edges; phải nằm trong min/max toàn bố trí. Đơn vị ô nhập chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Edge Maximum` | `number` | Kích thước lớn nhất của gem biên khi Start=Edges; phải nằm trong min/max toàn bố trí. Đơn vị ô nhập chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Attractor` | `reference` | Đối tượng định hướng mật độ | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Auto Attractor` | `boolean` | Chỉ có ý nghĩa khi Start=Curve | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Layout` | `choice` | Hex, Hexr, Square, Tri, HexPent hoặc All Directions | Hex |
| `Incremental` | `boolean` | Chiến lược tăng dần bố trí | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Slow` | `boolean` | Chiến lược tính chậm để tinh chỉnh | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Edges` | `boolean` | Chiến lược xử lý biên | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Base Size` | `number` | Kích thước cơ sở Đơn vị ô nhập chưa xác lập; mẫu đề xuất dùng đơn vị chiều dài document sau khi adapter xác nhận. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Min Size` | `number` | Kích thước nhỏ nhất Đơn vị ô nhập chưa xác lập; mẫu đề xuất dùng đơn vị chiều dài document sau khi adapter xác nhận. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Max Size` | `number` | Kích thước lớn nhất Đơn vị ô nhập chưa xác lập; mẫu đề xuất dùng đơn vị chiều dài document sau khi adapter xác nhận. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper` | `number` | Mức biến thiên kích thước; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Sizing Mode` | `choice` | Basic ít đổi kích thước; Normal có xu hướng lớn; Grow ép tăng; Shrink Outside thu nhỏ các viên ngoài | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Tight or Loose` | `number` | Mức spacing trên thang Tight tới Loose; đây là slider, không phải lựa chọn hai giá trị; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Min Spacing` | `number` | Giới hạn khoảng nhỏ nhất giữa gem; đơn vị ô nhập chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Pre-Sizing Equalize` | `number` | Slider trước bố trí làm kích thước đều hơn; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Post-Sizing Equalize` | `number` | Slider sau bố trí làm kích thước/khe hở đều hơn; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Display` | `boolean` | Hiện graph và quan hệ giữa viên trong quá trình Run thay vì preview viên | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Styles` | `reference` | Tham chiếu bộ cấu hình builder; codec và cách lưu cần được adapter xác nhận. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Pre-Sizing Perturb` | `number` | Slider trước bố trí tăng biến thiên/ngẫu nhiên; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Post-Sizing Perturb` | `number` | Slider sau bố trí tăng ngẫu nhiên/kết hợp giữa viên; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Post-Sizing Perturb Grow` | `number` | Mức ưu tiên tăng viên để lấp khe thay vì thêm viên mới; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Flip` | `boolean` | Đảo phía surface để table hướng lên | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-020` và các vai trò Surface, Start, Seed Gems, Curves; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tạo solver packing trên mặt với seed/edge locks, hai phases correction riêng và random seed tái lập; không sao chép solver chưa được kiểm chứng, báo violation/coverage trước Done.
2. C++/Qt: đăng ký và điều phối native command `OM9-GEM-020`; chuyển kế hoạch Auto Pavé Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-020`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-020",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "Start", kind: InputKind::Point, min: 0, max: Some(1) },
        Role { name: "SeedGems", kind: InputKind::Gem, min: 0, max: None },
        Role { name: "Curves", kind: InputKind::Curve, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Start", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Seed Gems", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Lock Start", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Edge Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Edge Minimum", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Maximum", kind: ParameterKind::Number, required: false },
        Parameter { name: "Attractor", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Auto Attractor", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Incremental", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Slow", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Edges", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Base Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Min Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Max Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sizing Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Tight or Loose", kind: ParameterKind::Number, required: false },
        Parameter { name: "Min Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pre-Sizing Equalize", kind: ParameterKind::Number, required: false },
        Parameter { name: "Post-Sizing Equalize", kind: ParameterKind::Number, required: false },
        Parameter { name: "Display", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Pre-Sizing Perturb", kind: ParameterKind::Number, required: false },
        Parameter { name: "Post-Sizing Perturb", kind: ParameterKind::Number, required: false },
        Parameter { name: "Post-Sizing Perturb Grow", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Start modes loại trừ nhau, Lock Start không đổi seeds qua sizing; Min/Max và Min Spacing được giữ hoặc lỗi rõ; random cùng seed tái lập; Done mới tạo stones.
- OM9-GEM-020: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-020`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-020` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
