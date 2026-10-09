---
id: OM9-RENDER-021
name: Matrix Movie Maker
command: null
domain: 13-render
module: Render Menu
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-RENDER-021 — Matrix Movie Maker

Alias tương thích: `Chưa có alias command`. Nhóm: `13-render`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Movie Maker ghépBMP/JPG/PNG thành MP4; chọn file đầu numbered series hỏi Import All, otherwise chọn list. Move Up/Down, Delete Selected/All chỉ bỏ entries window, Select All; Reverse đảo list, Add Single Frame Repeat, Dissolve green/red cho overlay transition (20 frames gợiý). FPSdefault 30, Quality 1 best 10 worst, Movie Size Width/Height. Watermark Use/Pick, Lower Left/Right, Size Small Medium Large, Opacity default 50 nhưng 0 solid/100 transparent. Fade In/Out dùng Pic hoặc Color; Frames count box mô tả seconds nên phải xác minh label/unit trước implementation.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Frames` | `image` | 1 | Không đặt trong mẫu |
| `Destination` | `path` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `FPS` | `number` | Số frames mỗi giây của movie. | 30 |
| `Quality` | `number` | Mức chất lượng movie: 1 tốt nhất, 10 thấp nhất; không đảo ý nghĩa. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Movie Width` | `number` | Chiều rộng movie, pixels. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Movie Height` | `number` | Chiều cao movie, pixels. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Repeat` | `number` | Số lần lặp của Add Single Frame; count nguyên theo policy OM9. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Dissolve Frames` | `number` | Số frames transition; 20 là gợi ý, không mặc định. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Watermark Use` | `boolean` | Bật watermark khi xuất movie. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Watermark Image` | `reference` | Asset watermark được Pick; cần resolver ảnh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Watermark Position` | `choice` | Lower Left hoặc Lower Right. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Watermark Size` | `choice` | Small, Medium hoặc Large; adapter định nghĩa mức kích thước. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Opacity` | `number` | Thang watermark: 0 solid/100 transparent. | 50 |
| `Fade In Mode` | `choice` | Pic hoặc Color. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Fade In Pic` | `reference` | Ảnh fade-in khi chọn Pic. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Fade In Color` | `text` | Màu fade-in khi chọn Color. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Fade In Frames` | `number` | Control ghi Frames nhưng mô tả seconds; units chưa thống nhất, không thực hiện trước khi adapter chốt. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Fade Out Mode` | `choice` | Pic hoặc Color. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Fade Out Pic` | `reference` | Ảnh fade-out khi chọn Pic. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Fade Out Color` | `text` | Màu fade-out khi chọn Color. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Fade Out Frames` | `number` | Control ghi Frames nhưng mô tả seconds; phải chốt units trước encoder. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-RENDER-021` và các vai trò Frames, Destination; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Frame play list có order/repeat/dissolve metadata, decode/composite water marks rồi encode; không xóa files khi Delete entries; opacityconverter 1-v/100.
2. C++/Qt: đăng ký và điều phối native command `OM9-RENDER-021`; chuyển kế hoạch Matrix Movie Maker sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Giữ thao tác ở phạm vi trạng thái hoặc đầu ra đã chọn. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-RENDER-021`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-021",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "Frames", kind: InputKind::Image, min: 1, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "FPS", kind: ParameterKind::Number, required: false },
        Parameter { name: "Quality", kind: ParameterKind::Number, required: false },
        Parameter { name: "Movie Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Movie Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Repeat", kind: ParameterKind::Number, required: false },
        Parameter { name: "Dissolve Frames", kind: ParameterKind::Number, required: false },
        Parameter { name: "Watermark Use", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Watermark Image", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Watermark Position", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Watermark Size", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fade In Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Fade In Pic", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Fade In Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Fade In Frames", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fade Out Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Fade Out Pic", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Fade Out Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Fade Out Frames", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Reverse đúng order, Delete không xóa disk, FPS 30 timeline đúng duration; Opacity 0 solid 100 invisible; fade unit không bị coi frames âm thầm.
- OM9-RENDER-021: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra chuyển chế độ hoặc hủy đầu ra không làm thay đổi hình học.

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
- [ ] Automated tests use feature ID `OM9-RENDER-021`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-RENDER-021` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-RENDER-021` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-RENDER-021` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-RENDER-021` hoặc ghi lý do không áp dụng.
- [ ] [OM9-RENDER](../../ENGINEERING_CONTRACTS.md#om9-render): kiểm chứng cho `OM9-RENDER-021` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-RENDER-021` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
