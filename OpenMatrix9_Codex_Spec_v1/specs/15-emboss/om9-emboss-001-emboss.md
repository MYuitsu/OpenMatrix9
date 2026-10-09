---
id: OM9-EMBOSS-001
name: Emboss
command: ClayEmboss
domain: 15-emboss
module: Emboss
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-EMBOSS-001 — Emboss

Alias tương thích: `ClayEmboss`. Nhóm: `15-emboss`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

ClayEmboss quản lý relief qua danh sách operation có thứ tự. New Project chọn Mesh mở, Thickness kín có chiều dày, Symmetry đối xứng Y có cap, Core kín với cap phẳng, Surface NURBS hoặc Image TIFF 16 bit. Delete Base cắt theo miền ngoài cùng và không dùng với Symmetry, Surface, Image. Cap Distance dành cho Thickness/Core/Symmetry. Width/Height đặt nền; Workbench Position mặc định Center Origin trên World XY hoặc Top Left/Right, Bottom Left/Right. Resolution điều khiển mật độ điểm, không coi số minh họa là mặc định. Display Material, Show Bounding Box và Assistant hỗ trợ xem. Dấu cộng thêm operation; refresh tính preview; check mark sang Decimator rồi check mark chốt. Kéo đổi thứ tự làm đổi relief. Không chọn geometry thì operation mặc định By Profile; có geometry thì Base On Geometry. Mỗi curve operation cần ít nhất một phần nằm trong nền. Có bật/tắt, xóa, duplicate và Show/Hide Gumball từng operation. Show/Hide Relief, Project Settings, Save Heightfield Image và Save Project là các hành động riêng; mở lại hỏi dự án mới hay hiện có. Place Image cần xác minh chi tiết.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Document` | `document` | 1 | 1 |
| `Operations` | `object` | 0 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Project Type` | `choice` | Mesh, Thickness, Symmetry, Core, Surface hoặc Image. | Chưa xác định; không tự gán giá trị. |
| `Delete Base` | `boolean` | Trim theo miền ngoài cùng; khóa ở Symmetry/Surface/Image. | Chưa xác định; không tự gán giá trị. |
| `Cap Distance` | `number` | Khoảng cap cho Thickness/Core/Symmetry. | Chưa xác định; không tự gán giá trị. |
| `Width` | `number` | Kích thước nền theo X. | Chưa xác định; không tự gán giá trị. |
| `Height` | `number` | Kích thước nền theo Y. | Chưa xác định; không tự gán giá trị. |
| `Workbench Position` | `choice` | Center Origin, Top Left/Right hoặc Bottom Left/Right. | Center Origin |
| `Resolution` | `number` | Mật độ points relief. Chính sách mẫu OpenMatrix9 yêu cầu số nguyên; chưa xác nhận quy tắc nội bộ khác. | Chưa xác định; không tự gán giá trị. |
| `Display Material` | `reference` | Identity material hiển thị cho relief; không phải cờ bật/tắt. | Chưa xác định; không tự gán giá trị. |
| `Show Bounding Box` | `boolean` | Hiển thị miền nền. | Chưa xác định; không tự gán giá trị. |
| `Assistant` | `boolean` | Hiện hướng dẫn thao tác. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-EMBOSS-001` và các vai trò Document, Operations; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Rust lưu project type, kích thước, frame nền và ordered operations với trạng thái enable; đánh giá stack thành trường cao, rồi adapter tạo mesh/NURBS/TIFF theo capability. Không dùng cùng một kiểu cap cho mọi project.
2. C++/Qt: đăng ký và điều phối native command `ClayEmboss`; chuyển kế hoạch Emboss sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-EMBOSS-001`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-EMBOSS-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "Operations", kind: InputKind::Object, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Project Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Delete Base", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Cap Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Workbench Position", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Resolution", kind: ParameterKind::Number, required: false },
        Parameter { name: "Display Material", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Show Bounding Box", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Assistant", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Đổi thứ tự hai operation không giao hoán phải đổi kết quả; Delete Base bị khóa đúng mode; Image xuất 16 bit và Surface không bị coi là mesh. Cancel trước Decimator giữ tài liệu.
- OM9-EMBOSS-001: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-EMBOSS-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-EMBOSS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-EMBOSS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-EMBOSS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-EMBOSS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-EMBOSS-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
