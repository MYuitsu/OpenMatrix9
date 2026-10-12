---
id: OM9-INFO-021
name: Gumball Alignment
command: GumballAlignment
domain: 01-core
module: Info & Settings
kind: command
implementation_status: implemented
approved_scope_complete: true
stock_ci_status: build_only_action_authoritative
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-INFO-021 — Gumball Alignment

Alias tương thích: `GumballAlignment`. Nhóm: `01-core`. Trạng thái: `implemented` trong approved OM9 scope.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-INFO-021` — Gumball Alignment.**

GumballAlignment chọn CPlane, Object hoặc World cho **một frame Gumball dùng
chung** trong document. Mọi overlay thuộc `OM9-INFO-020` hiển thị frame đó từ
camera của mình; lệnh alignment không xoay shape hoặc tạo Undo geometry.

### Đồng bộ hệ trục trong bốn viewport — revision 2026-10-11

- `GumballAlignment World`: axes theo world XYZ ở mọi ô. Đổi focus hoặc orbit
  camera không đổi frame/pivot. Đây là mode dùng cho fixture đối chiếu ảnh
  Top/Front/Right/Perspective; màu theo trục, không theo chiều màn hình.
- `GumballAlignment Object`: axes theo native frame của selection có adapter
  xác định được, gồm curve/extrusion/planar subobject và các owner được hỗ trợ.
  Một frame thế giới được capture rồi project ra mọi camera; không lấy frame
  object riêng theo từng view. Không có frame duy nhất thì báo unresolved,
  giữ frame/alignment trước; không chọn World ngầm.
- `GumballAlignment CPlane`: idle frame theo CPlane của viewport đang active.
  Khi đổi active view/CPlane, recapture một frame và cập nhật tất cả overlays
  cùng lúc; không để bốn ô có bốn hướng khác nhau. Basis đã capture phải được
  giữ nguyên suốt gesture; đổi view/CPlane trong gesture hủy preview đồng bộ.
- Default OM9 CPlane cho profile mới và preference đã lưu vẫn được giữ theo
  quyết định trước. World là cách đối chiếu ảnh, không phải bằng chứng ảnh
  xác lập default startup của Rhino/Matrix.
- Relocated frame override alignment tự động tới khi Reset, explicit Alignment
  hoặc source stamp không còn hợp lệ; đổi focus khi idle không xóa pivot đã
  relocate. Sau Reset/Alignment, Scale extents và hit targets phải được tính
  lại cho mọi view, không dùng geometry screen-space cũ.
- Menu trên bất kỳ ô nào và Command dùng cùng handler/checked state. Chọn
  Alignment không tự bật Gumball nếu đang Off. Khi On, thay basis không tạo
  object, thay placement, thêm transaction hoặc nhân bản bộ gizmo state.

Ảnh dùng làm chuẩn màu/bốn view; bản trích Matrix 8 Book 1 PDF3/trang in37
mô tả CPlane/Object/World. [Rhino 5 Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/gumball.htm)
đối chiếu active CPlane và reset alignment. Quy tắc một frame chia sẻ và
update đồng bộ là hợp đồng OM9 để đạt ảnh, không phải chứng minh code nội bộ
Rhino. Selection groups/multiple owners/control points phải có chính sách
cache rõ; không tự thừa hưởng single-owner relocated frame.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `selected_objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Alignment` | `choice` | CPlane, Object hoặc World | Default OM9 CPlane ở profile mới; giữ preference đã lưu; World cho fixture ảnh; Object cần frame native xác định được. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-INFO-021`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust lưu alignment policy, C++ lấy frame object/subelement hoặc CPlane và Qt dựng gizmo axes.
3. Tích hợp `GumballAlignment` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-021",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Alignment", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Object xoay 30 độ làm axes theo object; World giữ XYZ; đổi alignment không đổi placement/shape.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

2026-10-12: `implemented` / `validated_supported_slice`; `approved_scope_complete=true`.
Approved OM9 core scope completed against the exact30 native suite: native primitive dimensions/expressions, standard FreeCAD Link/Scale with native OM9 Compound-derived helpers preserving admitted definitions/dependencies/subelements/Body trees and presentation across Undo/Redo/FCStd, bounded native input settling before fresh gestures after Undo with unchanged history and callback cancellation, actual Mesh Ctrl+Shift component picking, typed document Camera/Light physical frames, shared four-view Relocate and process-local cache. Native host limits and unsupported graph/frame cases remain explicit. Clayoo/SmartTargets, Rhino internal roll, full original-host parity, full Render and live host camera widgets are excluded. Tests run locally (30 processes / 955 checks); at the user's request, GitHub Actions only builds/packages. Its exact-commit Action is authoritative for build status and records tests_executed false. See the local-tests/CI-build-only policy report.

Development-host checkpoint plugin SHA256 `559e0dca08e3535174879dd81c598d4a894e70e43989d22dba8b6d77a8db6b20`; host SHA256 `98eef899354a8aa84cfa2934e6e1a3c8558c7e6fff3b89c347c06731f06018a8`.
[Final native evidence](../../../docs/validation/2026-10-12-gumball-core-completion.md).

### Approved core closure — 2026-10-12, final native gates passed

Object alignment adds real Ctrl+Shift pointer selection of Mesh vertex, edge and
planar facet identities in the source viewport, including an initially empty
selection. The shared frame must use the native picked component and its world
coordinates; hidden/locked/stale, degenerate, nonplanar and ambiguous inputs
must reject without an implicit World frame. This is component selection and
frame derivation, not a mesh vertex deformation or Clayoo/SmartTargets command.

Typed document Camera/Light objects supply their physical origin and native
orientation. Camera FocalDistance/Height/HeightAngle/AspectRatio/Perspective and
Light SizeU/SizeV/Reach/Power remain typed source properties through FCStd.
Selecting or aligning them must leave Placement, properties, count and Undo
unchanged. Warm/cold native object and actual pointer gates passed;
this does not claim renderer output or the host's live camera-widget parity.

[Validated baseline](../../../docs/validation/2026-10-11-gumball-four-view-completion.md).
[Candidate scope and applicability](../../../docs/validation/2026-10-12-gumball-core-release-candidate.md).
[Inputs/units/output/lifecycle and limits](../../../docs/features/OM9-INFO-021.md).
Source design sections and compiled request-planner examples above remain unchanged.

## Acceptance checklist

Checked native items refer to the dated 2026-10-11 baseline. New approved-scope items are validated by the final native cohort.

- [x] World/Object dùng cùng frame trong bốn ô; idle focus/camera không đổi pivot hoặc geometry.
- [x] CPlane của active view cập nhật tất cả overlays; không recapture giữa gesture.
- [x] Menu/CMD Alignment và Reset đồng bộ basis, Scale extents, colors, pick và checked state ở mọi ô.
- [x] Relocated override giữ qua focus và maximize/restore; explicit Alignment reset đúng selection.
- [x] Native planar oriented Mesh Segment Object frame, rotated Placement và ambiguous/degenerate/nonplanar/cache rejection có evidence riêng.
- [x] Candidate Mesh vertex/edge/planar facet Object frames from actual Ctrl+Shift pointer events in all four views, including empty selection. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] Typed document Camera/Light physical Object frames and native property/view-provider warm/cold checks. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] Mọi case trên giữ object count, placement/shape và Undo; cancel trả frame cũ ở cả bốn ô.

- [x] Hợp đồng và điều kiện hình học được kiểm chứng trong supported native slice; adapter gaps ghi riêng.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] Alignment/Relocate are view state; no dependency mutation or new associative History graph (native unchanged geometry/count/Undo evidence).
- [x] No document geometry/count/Undo writes for view-state changes; successful geometry uses OM9-INFO-020 transaction.
- [x] N/A for cold widget-cache persistence: bounded256 single-owner cache is process-local by approved design; native object FCStd persistence is tested separately.
- [x] Recheck view-only behavior and cleanup on final-source expanded owner/typed frame fixtures. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] Automated tests use feature ID `OM9-INFO-021`.

## Hợp đồng kỹ thuật chung

Applicability is explicit below. Dated baseline and passing final native gates are
mapped in the [candidate report](../../../docs/validation/2026-10-12-gumball-core-release-candidate.md).

- [x] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): Menu/CMD, numeric input, cancel and modifier timing: dated 2026-10-11 four-view native evidence; final-source replay passed.
- [x] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): Applicable: native identity/topology, typed owner properties and FCStd; baseline evidence exists, new primitive/associative/Camera-Light gates passed. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): Same-document editable ownership, canonical roots and layer permissions: dated ownership/review evidence. New Mesh Ctrl+Shift and dependency graph checks passed.
- [x] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): Shared frame, first-click picks, focus/maximize and source/passive-view cleanup: dated four-view/lifecycle evidence; new-owner replay passed.
- [x] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): Unpickable preview, colors/logical pixels and DPI: dated visual/four-view evidence. New typed Camera/Light view-provider checks passed.
- [x] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): Native End/Mid/Point and source-view rays: dated Relocate pointer evidence. New Mesh Ctrl+Shift identities passed; no full snap-combination claim.
- [x] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): Document units, captured CPlane, w-world, world placement and invalid axes: dated core/four-view evidence; new physical-source frame checks passed.
- [x] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): Read-only permission/hierarchy lookup applies (dated ownership evidence). N/A for layer/group/Link/Body mutation: Alignment/Relocate writes only widget view state.
- [x] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): N/A for document annotations: Gumball creates no text/dimension annotation. Numeric editors are transient viewport UI, with dated DPI evidence.
- [x] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): N/A for a measurement/diagnostic command: no independent analysis output is created. Fixture geometry/volume checks validate transforms only.
- [x] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): N/A for geometry transforms in this command; frame changes are view state. Geometry transforms belong to OM9-INFO-020.
- [x] Remaining engineering contracts have explicit N/A reasons in the candidate report; Camera/Light names do not certify a renderer.
