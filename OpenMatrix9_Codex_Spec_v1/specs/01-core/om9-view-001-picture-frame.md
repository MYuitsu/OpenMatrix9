---
id: OM9-VIEW-001
name: Picture Frame
command: PictureFrame
domain: 01-core
module: View
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-VIEW-001 — Picture Frame

Alias tương thích: `PictureFrame`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-VIEW-001` — Picture Frame.**

PictureFrame chọn image file rồi hai điểm/góc và length, giữ aspect ratio; Shift dùng Ortho. Plane có thể move/scale và transparency qua material properties. Vertical theo normal CPlane; SelfIllumination giữ intensity không theo lights/shadows; EmbedBitmap lưu bytes image trong document hoặc external path có missing status; Autoname lấy filename cho object name; AlphaTrasnparency giữ identifier tương thích và chọn hiển thị màu object xuyên alpha hoặc thật trong suốt. Đây là textured plane tham chiếu, không phải bitmap tracing tự tạo geometry.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `image` | `image` | 1 | 1 |
| `picked_points` | `point` | 2 | 2 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Vertical` | `boolean` | Plane vuông góc CPlane | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `SelfIllumination` | `boolean` | Không chịu lighting | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `EmbedBitmap` | `boolean` | Lưu image bytes | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Autoname` | `boolean` | Tên từ filename | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `AlphaTrasnparency` | `choice` | Object color hoặc Transparent | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Length` | `number` | Chiều dài theo document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-VIEW-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Native Part/scene reference plane với dimensions/image metadata, Qt texture provider; Rust asset policy kiểm tra file/image dimensions và embedding.
3. Tích hợp `PictureFrame` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-VIEW-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "image", kind: InputKind::Image, min: 1, max: Some(1) },
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SelfIllumination", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "EmbedBitmap", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Autoname", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AlphaTrasnparency", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Image 2:1 tạo plane 10×5; EmbedBitmap mở lại khi file gốc bị dời; External missing hiện lỗi; Vertical plane đúng CPlane.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

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
- [ ] Automated tests use feature ID `OM9-VIEW-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICTURE](../../ENGINEERING_CONTRACTS.md#om9-picture): kiểm chứng cho `OM9-VIEW-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
