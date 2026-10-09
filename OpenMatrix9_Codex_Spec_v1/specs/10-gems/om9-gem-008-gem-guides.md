---
id: OM9-GEM-008
name: Gem Guides
command: gvGemGemGuides
domain: 10-gems
module: Placing Gems
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-GEM-008 — Gem Guides

Alias tương thích: `gvGemGemGuides`. Nhóm: `10-gems`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvGemGemGuides bật/tắt riêng Culet arrow, North arrow (hướng tới gem tiếp trong prong arrangement), Information X/Y/Z/Weight. Bezel Profile quanh girdle, Prong Points, Horizontal/Vertical Prong Lines tại vị trí prong thông dụng; Apply tạo curves/points, nhóm prong guides. Mode Offset dùng mm hoặc Scale dùng %; line lengths mm. View Gem chỉ khi một stone, View Reset về views mặc định, Reset cấu hình; icon bật giữ đến khi tắt, highlight vàng.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Gems` | `gem` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Culet` | `boolean` | Hiện mũi tên hướng culet | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `North` | `boolean` | Hiện mũi tên hướng north | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Information X` | `boolean` | Hiện thông tin kích thước X | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Information Y` | `boolean` | Hiện thông tin kích thước Y | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Information Z` | `boolean` | Hiện thông tin kích thước Z | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Information Weight` | `boolean` | Hiện thông tin trọng lượng | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bezel Profile` | `boolean` | Tạo/hiện profile bao viên | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prong Points` | `boolean` | Tạo/hiện điểm đặt chấu | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Horizontal Prong Lines` | `boolean` | Tạo/hiện đường chấu ngang | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Vertical Prong Lines` | `boolean` | Tạo/hiện đường chấu dọc | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Mode` | `choice` | Offset dùng mm; Scale dùng phần trăm | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile: Offset Amount` | `number` | Khoảng offset mm khi Mode=Offset; tỷ lệ phần trăm khi Mode=Scale; adapter phải xét Mode Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prongs: Horizontal Length` | `number` | Chiều dài đường chấu ngang Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Prongs: Vertical Length` | `number` | Chiều dài đường chấu dọc Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-008` và các vai trò Gems; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tách overlay information khỏi geometry Apply; lấy prong sites theo shape catalog đã xác minh, offset/scale curve và nhóm guides đã commit.
2. C++/Qt: đăng ký và điều phối native command `gvGemGemGuides`; chuyển kế hoạch Gem Guides sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-008`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-008",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Culet", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "North", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Information X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Information Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Information Z", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Information Weight", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bezel Profile", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Prong Points", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Horizontal Prong Lines", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Vertical Prong Lines", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Profile: Offset Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs: Horizontal Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Prongs: Vertical Length", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Bật Culet không tạo curve; Apply Prong Points ra group; single stone mới View Gem, nhiều stones disabled; Offset mm khác Scale %.
- OM9-GEM-008: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-GEM-008`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-008` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
