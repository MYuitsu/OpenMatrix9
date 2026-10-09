---
id: OM9-VIEW-001
name: Picture Frame
command: PictureFrame
domain: 01-core
module: View
kind: command
implementation_status: partially_implemented
implementation_validation_status: validated_supported_slice
implementation_verified_at: 2026-10-09
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-VIEW-001 — Picture Frame

Alias tương thích: `PictureFrame`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

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

**Supported slice:** decoded image placement as a native `Image::ImagePlane`, with aspect-preserving dimensions, transient preview and owned coordinate/unit provenance. Explicit embedding/external-link and alpha-display modes remain open.

### Inputs

Select a readable image file, then enter/pick a first corner and either a width-reference point or scalar width. Existing geometry selection is not required. The shared Rust parser supports CPlane/world/relative Cartesian points, explicit coordinate/length units and the captured document input scale. A scalar is accepted only in the width phase and remains a tagged length in provenance. Typed `Length=` is not registered as an option; width is entered directly. Native picks/snaps/F4 are world-coordinate inputs.

### Parameters and defaults

Each session starts with `Vertical=No`, `SelfIllumination=Yes` and `Autoname=Yes`; these options accept case-insensitive Yes/No or 1/0. Vertical uses CPlane Z for image height. SelfIllumination selects the native No shading/Shading display mode. Autoname changes the visible Label to the source filename stem; the internal object Name remains a unique PictureFrame name, so the full filename-as-object-name contract is not completed. Width and derived height must be in `[1e-7, 1e9]` mm, with finite resulting corners within ±`1e9` mm. Scalar width follows captured CPlane X; a reference point determines width/direction in the captured plane. Mouse Shift/F8 uses the existing Ortho path. The prompt does not yet present all current option values.

`EmbedBitmap` mode selection, external-path missing status, and `AlphaTrasnparency` Object-color/Transparent selection are not implemented. Native image storage or an RGBA texture alone does not establish those option contracts. Preserve the exact compatibility spelling `AlphaTrasnparency`.

### Output

Creation produces one document-owned `Image::ImagePlane` with image file, X/Y sizes and centered Placement. It is a textured reference object, not traced geometry or a Part BRep. The same creation transaction stores `OM9CoordinateReport` and `OM9UnitContext`. Native placement and size properties remain editable after creation. Separate-process reload after removal of the external fixture verifies actual native image storage; it does not establish an EmbedBitmap Yes/No control.

### Preview / commit / cancel

The document owner, units and CPlane are captured before the image dialog and checked again after it. Placement uses that command-start frame. Hover preview is transient and does not append accepted inputs or consume Once; rejected width/reference values retain the accepted corner and options. The final candidate report is validated before mutation. A rejected transaction start fails before adding an image; successful creation places the image and provenance in one transaction. Cancel clears preview and pending inputs without creating an object. Document switch/close, workbench cancellation or stale units invalidate placement.

### History / dependency model

Native Undo/Redo governs image creation and its provenance together. The persisted report is an owned record of the accepted point/point or point/length inputs and their captured context. It does not establish associative History links to snapped objects or a feature replay graph. Undo/Redo, same-process restore and separate-process reload passed against the final tested module.

### Error / invalid-input behavior

Unreadable images, malformed/non-finite/overflowing input, degenerate width, unsupported dimensions and out-of-range corners fail without accepting the candidate. Native mutation/recompute failures abort an opened transaction and do not record command success. The final native replay exercises locked-transaction rejection and retry. Forced post-add property/recompute faults remain unexercised. Full EmbedBitmap/external missing, alpha-mode, option-prompt and object-name semantics remain outside this validated slice.

Validation: [shared point-consumer report](../../../docs/validation/2026-10-09-rhino-core-point-consumers.md) and its hash-bound evidence. Status applies only to the supported slice above.

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
