---
id: OM9-ART-003
name: Picture Menu
command: null
domain: 07-matrix-art
module: Matrix Art
kind: menu
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-ART-003 — Picture Menu

Alias tương thích: `Chưa có alias command`. Nhóm: `07-matrix-art`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Picture Menu nạp hoặc Remove Bitmap cho Current Layer; dùng Picture Frame và Duplicate Border/Rectangle để lấy miền đúng tỉ lệ, đặt miền ảnh riêng ở layer thấp. Fill Shape phủ ảnh lên bounding extents nên có thể méo nếu miền khác tỉ lệ; Maintain Aspect giữ tỉ lệ khi kéo Size Width/Size Height. Bitmap Height là độ relief cộng vào Surface Height, sáng cao/tối thấp; Size Height là chiều ảnh 2 D. Offset X/Y dịch ảnh; Rotate xoay quanh góc dưới trái, không phải tâm. Tile Bitmap lặp ảnh, Invert Bitmap đổi sáng/tối và chuyển Bitmap Height sang âm. Trim to Bitmap bỏ pixel đen tuyệt đối (0,0,0), phối hợp Surface Height để bỏ nền; chiều dày đầu ra do Bitmap Height và Add Depth. Build có thể đưa texture lên lớp phía trên; dùng Cut hoặc Grid để tách. Muốn bỏ nền bằng miền, đặt đường bao ảnh ở layer cao rồi Trim nó theo miền giữ ở layer thấp.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Bitmap` | `image` | 1 | 1 |
| `Region` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Bitmap Height` | `number` | Biên độ relief cộng vào Surface Height. | Chưa xác định; không tự gán giá trị. |
| `Size Width` | `number` | Kích thước ảnh trên nền theo X. | Chưa xác định; không tự gán giá trị. |
| `Size Height` | `number` | Kích thước ảnh trên nền theo Y, khác Bitmap Height. | Chưa xác định; không tự gán giá trị. |
| `Offset X` | `number` | Dịch vị trí ảnh theo X. | Chưa xác định; không tự gán giá trị. |
| `Offset Y` | `number` | Dịch vị trí ảnh theo Y. | Chưa xác định; không tự gán giá trị. |
| `Rotate` | `number` | Góc xoay ảnh quanh góc dưới trái. | Chưa xác định; không tự gán giá trị. |
| `Maintain Aspect` | `boolean` | Giữ tỉ lệ khi chỉnh kích thước ảnh. | Chưa xác định; không tự gán giá trị. |
| `Tile Bitmap` | `boolean` | Lặp ảnh trong miền. | Chưa xác định; không tự gán giá trị. |
| `Invert Bitmap` | `boolean` | Đảo sáng/tối và dấu Bitmap Height. | Chưa xác định; không tự gán giá trị. |
| `Trim to Bitmap` | `boolean` | Bỏ nền pixel đen tuyệt đối. | Chưa xác định; không tự gán giá trị. |
| `Add Depth` | `number` | Chiều dày nền phối hợp Bitmap Height. | Chưa xác định; không tự gán giá trị. |
| `Fill Shape` | `boolean` | Phủ ảnh lên extents miền; có thể đổi aspect. | Chưa xác định; không tự gán giá trị. |
| `Current Layer` | `reference` | Identity layer đang nhận bitmap và Surface Height. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-ART-003` và các vai trò Bitmap, Region; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Dùng transform ảnh với gốc dưới trái và phép lấy mẫu trường cao; giữ mask nền đen độc lập với ánh xạ màu và quan hệ layer.
2. C++/Qt: đăng ký và điều phối native command `OM9-ART-003`; chuyển kế hoạch Picture Menu sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-ART-003`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Bitmap", kind: InputKind::Image, min: 1, max: Some(1) },
        Role { name: "Region", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Bitmap Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Offset X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Offset Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maintain Aspect", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tile Bitmap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Invert Bitmap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Trim to Bitmap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fill Shape", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Current Layer", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Ảnh bất đối xứng xoay phải quanh góc dưới trái; Maintain Aspect và Fill Shape có hành vi khác nhau; chỉ đen tuyệt đối bị Trim to Bitmap, texture không truyền qua quan hệ Cut.
- OM9-ART-003: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-ART-003`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-ART-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-ART-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-ART-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-ART-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-ART-003` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
