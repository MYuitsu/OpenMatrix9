---
id: OM9-TOOLS-016
name: Image Trace
command: gvImageTrace
domain: 09-tools
module: Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-016 — Image Trace

Alias tương thích: `gvImageTrace`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvImageTrace nhận PNG/JPG/GIF/JPEG/BMP/TIFF, Enter kết thúc. Threshold phân tối/sáng, Despeckle bỏ contours nhỏ, Corner Soften làm mềm góc. Blur bật mới có Blur Radius (nhãn Blue Radius cần kiểm tra UI), Outline bật mới có Outline Radius và tạo hai contours quanh outline dày. Preprocess Method None/Grayscale/Quantize, Grayscale mặc định; Prescale 1X mặc định 100%, 2X 200%, 3X 300%. Invert đảo pixel; Group Output nhóm curves; Reset cấu hình. Ảnh tương phản tốt, ít màu dễ trace hơn.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Image` | `image` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Threshold` | `number` | Ngưỡng phân pixel sáng/tối. | Chưa xác định; không tự gán giá trị. |
| `Despeckle` | `number` | Mức loại contours nhỏ. | Chưa xác định; không tự gán giá trị. |
| `Corner Soften` | `number` | Mức làm mềm góc contours. | Chưa xác định; không tự gán giá trị. |
| `Blur` | `boolean` | Bật tiền xử lý blur để dùng Blur Radius. | Chưa xác định; không tự gán giá trị. |
| `Blur Radius` | `number` | Bán kính blur; chỉ khi Blur bật. | Chưa xác định; không tự gán giá trị. |
| `Outline` | `boolean` | Tạo hai biên cho nét dày. | Chưa xác định; không tự gán giá trị. |
| `Outline Radius` | `number` | Bán kính outline khi Outline bật. | Chưa xác định; không tự gán giá trị. |
| `Preprocess Method` | `choice` | None, Grayscale hoặc Quantize. | Grayscale |
| `Prescale` | `choice` | 1X 100%, 2X 200%, 3X 300%. | 1X |
| `Invert` | `boolean` | Đảo mức sáng trước trace. | Chưa xác định; không tự gán giá trị. |
| `Group Output` | `boolean` | Nhóm curves đầu ra. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-016` và các vai trò Image; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Decode ảnh, preprocess theo mode, prescale và invert trước contour extraction; bỏ contour theo despeckle rồi soften; outline là nhánh riêng và xuất group tùy chọn.
2. C++/Qt: đăng ký và điều phối native command `gvImageTrace`; chuyển kế hoạch Image Trace sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-016`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-016",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Image", kind: InputKind::Image, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Threshold", kind: ParameterKind::Number, required: false },
        Parameter { name: "Despeckle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corner Soften", kind: ParameterKind::Number, required: false },
        Parameter { name: "Blur", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Blur Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outline", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Outline Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preprocess Method", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Prescale", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Invert", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Group Output", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Grayscale/1X là defaults; Outline cho hai biên ở nét dày; Despeckle tăng bỏ contour lớn hơn; Enter chốt nhóm còn Cancel không tạo.
- OM9-TOOLS-016: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-016`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-016` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
