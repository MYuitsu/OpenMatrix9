---
id: OM9-SETTING-007
name: Prong Builder
command: gvProngAdder
domain: 11-settings
module: Setting Menu
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SETTING-007 — Prong Builder

Alias tương thích: `gvProngAdder`. Nhóm: `11-settings`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvProngAdder Gem Flow mặc định với 2 Shared Prongs cho line; Flow layouts Ends Only,2 Prongs,2 Prongs 90,2 Prongs 45,4 Prongs,1 Shared Prongs,2 Shared Prongs,1 Shared 1 Side,1 Shared 2 Side,1 Left 2 Right. Có thể nhập gems lần nữa để thêm layout khác. Gem North đặt quanh North axis:2 Prongs 0/30/60/90/120/150 Degrees;3 Prongs 0/180 Degrees;4 Prongs 0 (mặc định North)/45/Neg 45/Close;6 Prongs 0/90 Degrees. Góc pair 180 và các shared/side sites phải theo từng layout, không dùng uniform circle cho mọi mode. Large/Small Diameter, Large/Small Height tối thiểu 0.1 mm từ girdle, Drop dưới girdle, Nudge default 10%, Fillet 0 flat/100 hemisphere, Taper 0 side thẳng dương base lớn âm base nhỏ, Rotate Z. End Prong Layout thường 2 có 1/0; a symmetric có Left/Right. Styles, Reset, Enter set.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Gems` | `gem` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Prong Mode` | `choice` | Gem Flow theo hàng hoặc Gem North theo từng viên | Gem Flow |
| `Flow Layout` | `choice` | Ends Only, 2 Prongs, 2 Prongs 90, 2 Prongs 45, 4 Prongs, 1 Shared Prongs, 2 Shared Prongs, 1 Shared 1 Side, 1 Shared 2 Side hoặc 1 Left 2 Right | 2 Shared Prongs |
| `North Layout` | `choice` | 2 Prongs với góc 0/30/60/90/120/150; 3 Prongs 0/180; 4 Prongs 0/45/Neg45/Close; 6 Prongs 0/90 | 4 Prongs 0 Degrees |
| `Large Prong Diameter` | `number` | Đường kính chấu lớn Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Small Prong Diameter` | `number` | Đường kính chấu nhỏ Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Large Prong Height` | `number` | Chiều cao chấu lớn; mốc tối thiểu được mô tả 0.1 mm Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Small Prong Height` | `number` | Chiều cao chấu nhỏ; mốc tối thiểu được mô tả 0.1 mm Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Drop` | `number` | Dịch chấu theo chiều cao Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Nudge` | `number` | Mức chấu đè lên gem Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | 10% |
| `Fillet` | `number` | Bo đầu chấu: 0% phẳng, 100% bán cầu Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Taper` | `number` | 0 thẳng; dương đáy lớn hơn, âm đáy nhỏ hơn Đơn vị phần trăm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Rotate Z` | `number` | Xoay profile chấu quanh Z Đơn vị độ. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Prong Layout` | `choice` | Chọn 0/1/2 chấu cuối hoặc Left/Right theo layout hiện hành | Nhiều layout dùng 2; chưa xác lập mặc định chung cho mọi layout |
| `Styles` | `reference` | Tham chiếu bộ cấu hình builder; codec và cách lưu cần được adapter xác nhận. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SETTING-007` và các vai trò Gems; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Lưu mode North/Flow và explicit site graph mỗi layout, tính shared sites từ neighbors rồi size taper theo row; preserve end layout.
2. C++/Qt: đăng ký và điều phối native command `gvProngAdder`; chuyển kế hoạch Prong Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SETTING-007`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Prong Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Flow Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "North Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Large Prong Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Small Prong Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Large Prong Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Small Prong Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Drop", kind: ParameterKind::Number, required: false },
        Parameter { name: "Nudge", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fillet", kind: ParameterKind::Number, required: false },
        Parameter { name: "Taper", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Prong Layout", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Default Flow 2 Shared tạo hai shared giữa cặp, North default 4 Prongs 0; heights<0.1 bị báo, Nudge 10% overlap; a symmetric end layout dùng đúng left/right.
- OM9-SETTING-007: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SETTING-007`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SETTING-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SETTING-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SETTING-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SETTING-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SETTING-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SETTING-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SETTING-007` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
