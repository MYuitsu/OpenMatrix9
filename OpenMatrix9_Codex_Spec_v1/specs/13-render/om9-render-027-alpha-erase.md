---
id: OM9-RENDER-027
name: Alpha Erase
command: gvAlphaErase
domain: 13-render
module: Render Menu
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-RENDER-027 — Alpha Erase

Alias tương thích: `gvAlphaErase`. Nhóm: `13-render`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvAlphaErase chỉ PNG có Alpha, Files To Fix+/−. Remove bỏ transparency và phục hồi original background nếu image giữ RGB under alpha; Flatten composited selected color. Save Location rồi Run/Finished. Không suy background đã lost có thể phục hồi.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `PNGs` | `image` | 1 | Không đặt trong mẫu |
| `Destination` | `path` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Remove bỏ alpha và dùng RGB còn lưu dưới alpha; Flatten composite với màu chọn. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Color` | `text` | Màu nền khi Flatten; không dùng để phục hồi dữ liệu background đã mất. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Save Location` | `text` | Folder output của ảnh đã xử lý. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Files To Fix` | `reference` | Danh sách PNG có Alpha đã chọn; không gồm ảnh không có alpha. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-RENDER-027` và các vai trò PNGs, Destination; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Decode RGB A, Remove đặt alpha opaque giữ RGB, Flatten alpha composite color, save new files không overwrite input mặc định.
2. C++/Qt: đăng ký và điều phối native command `gvAlphaErase`; chuyển kế hoạch Alpha Erase sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Giữ thao tác ở phạm vi trạng thái hoặc đầu ra đã chọn. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-RENDER-027`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-027",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "PNGs", kind: InputKind::Image, min: 1, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Save Location", kind: ParameterKind::Text, required: false },
        Parameter { name: "Files To Fix", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Remove giữ RGB nền transparent, Flatten đúng selected color; PNG không alpha báo không cần, non PNG reject; original files giữ nguyên.
- OM9-RENDER-027: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra chuyển chế độ hoặc hủy đầu ra không làm thay đổi hình học.

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
- [ ] Automated tests use feature ID `OM9-RENDER-027`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-RENDER-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-RENDER-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-RENDER-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-RENDER-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-RENDER](../../ENGINEERING_CONTRACTS.md#om9-render): kiểm chứng cho `OM9-RENDER-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-RENDER-027` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
