---
id: OM9-FILE-009
name: Print
command: null
domain: 01-core
module: File
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-FILE-009 — Print

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-FILE-009` — Print.**

Print xuất viewport/layout tới printer hoặc file. Destination chọn Printer/Size/Portrait-Landscape/Properties/Copies; Print to File cần backend .prn hoặc format hỗ trợ. Output Type vector/raster, Output Color Print Color/Display Color/Black and White. View/Scale chọn Viewport/Extents/Window, Window cho pick rectangle, Set sửa grips/center. Multiple Layouts nhận pages/ranges, Print All Layouts; PDF hợp nhất, image file tạo suffix index; thiếu layout thì các tùy chọn disabled. Scale ánh xạ On paper/In model theo units. Margins/position dùng inch/cm/mm/pixel, top/left/right/bottom + width/height hoặc centered, match viewport aspect hoặc printable area. Position Centered vô hiệu Offset From corners/X/Y. Linetypes match definition hoặc viewport, Line width Scale by/Default/Hairline/No Print; non-scaling point/arrow/text-dot sizes. Visibility điều khiển background color/bitmap/wallpaper/lights/clipping planes/selected only/locked/grid/axes/margins. Notes và Filename ở None/Top/Bottom; printer calibration Scale X/Y. Không lấy display pixels như tỷ lệ model khi chưa có unit/calibration.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Printer` | `reference` | Printer backend | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Size` | `choice` | Paper size | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Orientation` | `choice` | Portrait hoặc Landscape | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Copies` | `number` | Số bản | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Print to File` | `boolean` | Ghi file | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Output Type` | `choice` | Vector hoặc Raster | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Output Color` | `choice` | Print Color, Display Color hoặc Black and White | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `View` | `choice` | Viewport, Extents hoặc Window | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Multiple Layouts` | `text` | Pages/ranges | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Print All Layouts` | `boolean` | Mọi layout | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Scale` | `number` | Tỷ lệ paper/model | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `MarginUnits` | `choice` | Inches, centimeters, millimeters hoặc pixels | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Margins` | `text` | Bốn lề và geometry vùng in | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Position` | `choice` | Centered hoặc Offset | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Scale X` | `number` | Hiệu chỉnh printer X | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Scale Y` | `number` | Hiệu chỉnh printer Y | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Visibility` | `text` | Các lớp hiển thị được chọn | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Notes` | `choice` | None, Top hoặc Bottom | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Filename` | `choice` | None, Top hoặc Bottom | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `LineWidthScale` | `number` | Hệ số line widths | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-FILE-009`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust print scene snapshot + page/layout/scale/style schema; Qt print/PDF/image backend, vector HLR adapter khi hỗ trợ; validate printable bounds trước output.
3. Tích hợp `Print` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-FILE-009",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Printer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Size", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Copies", kind: ParameterKind::Number, required: false },
        Parameter { name: "Print to File", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Output Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Output Color", kind: ParameterKind::Choice, required: false },
        Parameter { name: "View", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Multiple Layouts", kind: ParameterKind::Text, required: false },
        Parameter { name: "Print All Layouts", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "MarginUnits", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Margins", kind: ParameterKind::Text, required: false },
        Parameter { name: "Position", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Scale X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Visibility", kind: ParameterKind::Text, required: false },
        Parameter { name: "Notes", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Filename", kind: ParameterKind::Choice, required: false },
        Parameter { name: "LineWidthScale", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Window print chỉ vùng chọn; Extents bỏ vùng trống viewport; layout image xuất numbered files; No Print bỏ line; Scale 1:1 có units đúng; cancel không gửi job.
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
- [ ] Automated tests use feature ID `OM9-FILE-009`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-FILE-009` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
