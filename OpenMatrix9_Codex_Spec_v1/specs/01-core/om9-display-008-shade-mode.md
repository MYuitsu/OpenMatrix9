---
id: OM9-DISPLAY-008
name: Shade Mode
command: null
domain: 01-core
module: Display
kind: mode_or_view
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-DISPLAY-008 — Shade Mode

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-DISPLAY-008` — Shade Mode.**

Shade Mode lưu chế độ đã chọn và toggle với Wireframe. Wireframe vẽ edges/isocurves/mesh edges; Shaded dùng màu layer, Rendered dùng material. Ghosted bán trong suốt, X-Ray ít trong hơn; Technical nền trắng/outline đậm; Artistic/Pen mô phỏng sketch; Art Color bỏ curves/grid và Art Color Wires giữ; Chalk kiểu bảng phấn. Detect All phối diagnostics; Detect Backface xanh outward/đỏ inward, Detect Naked Edges highlight open edge, Detect Smart Flow đỏ cho metadata phù hợp, Detect UV Normals dùng đỏ outward/xanh inward nên không dùng cùng bảng màu Backface. Floorplan blueprint, Ice xám trong không curves; Legacy sơ đồ; Machine màu đặc không wires; Matrix Blue/White sơ đồ theo theme; Pastel màu layer dịu; Plastic/Shiny Plastic/Working Shade bề mặt gloss với backface tương ứng đỏ/xám; Presentation materials không curves/grid; Simple Shade xám không curves; Tech White thêm isocurves/edges; Vivid có edges/grid/curves; Wire Render giữ isocurves, Working Render bỏ isocurves; TsShiny là shade cho capability riêng. Preset chưa có renderer giữ visible disabled, không sao chép tài sản thương mại.

### Chi tiết danh sách mode, năm nút và phạm vi tác động

Dropdown lấy display modes đang có từ host, loại Wireframe khỏi danh sách
chọn và lọc một số mode nội bộ/T-Splines. Wireframe là nhánh toggle riêng.
Các tên preset trong hợp đồng là vocabulary mục tiêu; runtime phải resolve
stable ID/backend và báo mode unavailable khi host không cung cấp.

Năm nút khởi tạo Plastic, Shaded, Tech Shade, Ghosted và X-Ray. Preference
có thể thay từng mục khi resolve được tên trong danh sách hiện có. Chọn một
mode mới ngoài năm mục dịch danh sách và thêm cuối; chọn lại tên đã có giữ
thứ tự. Vì vậy policy gốc chưa phải LRU tổng quát. Click thường áp active
view, right-click áp mọi view mà host liệt kê; toggle off chuyển Wireframe.
UI phải hiển thị scope và đồng bộ theo active view.

Trước đổi view, helper còn xử lý material cho layer chuẩn đã tồn tại: bỏ qua
layer thiếu, tạo material khi chưa có, bổ sung environment texture tương ứng
nếu thiếu, đổi tên material theo layer definition và đặt texture decal mode,
rồi cập nhật material/layer link. Material đang tồn tại có thể bị sửa; khi
material index dùng chung cần kiểm ảnh hưởng tới object/layer khác. Nhánh này
có thể thay dữ liệu material dù không thay shape; không thấy nó reset visibility, lock, current
layer hay màu layer hiện có. Các kết quả ModifyMaterial/ModifyLayer trong
helper chưa được kiểm chứng khi chạy.

OM9 tách lựa chọn display mode với policy khởi tạo material. Nếu hỗ trợ
material initialization, phải có option/scope, resource validation, transaction
và nghiệm thu bảo toàn material tùy biến; mẫu ViewState hiện tại chỉ bao phủ
đường đổi viewport. Geometry, selection và History giữ nguyên khi chỉ đổi
viewport. Shader và texture của preset cần adapter/backend riêng.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_viewport` | `viewport` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Display preset được khai báo | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Enabled` | `boolean` | Toggle preset/Wireframe | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-DISPLAY-008`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust mode catalog + render/overlay flags và capability; Qt viewport shader/view-provider áp theme/diagnostics; lưu style trước để toggle Wireframe quay lại.
3. Tích hợp `Shade Mode` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-DISPLAY-008",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_viewport", kind: InputKind::Viewport, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Toggle hai lần khôi phục mode; Backface và UV Normals dùng đúng bảng riêng; Art Color hide grid/curves; Part shape không đổi.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Supported slice dưới đây là hợp đồng để triển khai; trạng thái native giữ nguyên
và các ca runtime vẫn cần nghiệm thu.

### Inputs

Active viewport hoặc explicit all-view scope, display-mode registry và năm slot preference; material initialization là action riêng nếu được hỗ trợ.

### Parameters and defaults

Five-slot seeds Plastic/Shaded/Tech Shade/Ghosted/X-Ray; chọn mode mới dịch/thêm cuối, tên đã có giữ thứ tự. Toggle off chọn Wireframe. Khả dụng phụ thuộc host/backend.

### Output

Viewport display state. Material helper gốc còn có thể bổ sung material/texture/layer links; nhánh này ngoài phạm vi mẫu ViewState.

### Preview / commit / cancel

Click thường active, right-click all views. Đổi viewport giữ shape; nếu có material action thì validate resources, transaction và rollback riêng.

### History / dependency model

Display preference không replay geometry; material links cần persistence đúng scope khi được thay.

### Error / invalid-input behavior

Mode/resource không tồn tại báo unavailable; không reset visibility/lock/current layer hoặc ghi đè material tùy biến chỉ từ tên helper.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-DISPLAY-008`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-DISPLAY-008` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
