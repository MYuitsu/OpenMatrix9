---
id: OM9-INFO-022
name: Relocate Gumball
command: RelocateGumball
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

# OM9-INFO-022 — Relocate Gumball

Alias tương thích: `RelocateGumball`. Nhóm: `01-core`. Trạng thái: `implemented` trong approved OM9 scope.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-INFO-022` — Relocate Gumball.**

RelocateGumball đổi origin và hướng của frame Gumball dùng chung trong cả bốn
viewport. Chỉ di chuyển/xoay widget, không dịch geometry hoặc tạo Undo geometry.
CMD và pointer dùng parser/coordinate context đã capture; input không có prefix
không mặc nhiên là world point.

### Relocate trong bốn viewport — revision 2026-10-11

1. Gumball phải On và selection có capability hợp lệ. `RelocateGumball`,
   alias `GumballRelocate` hoặc menu ring của bất kỳ ô nào vào cùng workflow.
   Prompts trên Command: origin mới → point xác định hướng X → Y point hoặc
   Enter. Origin/point theo document units và captured CPlane; prefix `w`
   chọn world theo coordinate parser hiện hữu.
2. Sau origin và X point, Enter xác lập Y/Z từ captured CPlane khi đủ hướng.
   Y point tùy chọn giải hướng còn lại, đặc biệt X song song CPlane normal.
   Đây là lựa chọn OM9 đã có, được đối chiếu thêm Rhino 5; ảnh tĩnh không chứng
   minh thuật toán frame hoặc thứ tự pick.
3. Ctrl có từ mouse-down trên handle bắt đầu widget-only gesture. Sau đó thả
   Ctrl và drag vẫn Relocate widget; Ctrl numeric Move/Rotate/Scale chỉ chỉnh
   origin/basis/độ dài điều khiển. Không đổi kích thước geometry. Ctrl thêm
   sau motion trên Move arrow thuộc Extrude của `OM9-INFO-020`, không Relocate.
4. Origin và basis preview được hiển thị đồng thời trong cả bốn view. Camera
   nơi thao tác sinh ray/pick, nhưng world frame và preview là state chung.
   Numeric editor chỉ ở ô nguồn; không tạo một editor hoặc pivot riêng mỗi ô.
5. Osnap dùng native candidates có capability và state hợp lệ; khi không có
   candidate, Rust giao ray với captured CPlane. Ray song song, hướng zero,
   đồng tuyến hoặc NaN/Infinity bị reject, giữ phase/frame trước và báo lý do;
   không chọn điểm chiếu tùy ý để có vẻ thao tác đã thành công.
6. Khi chấp nhận origin/basis mới, tính lại Scale anchors, dotted leaders,
   plane handles và hit targets ở mọi ô theo `OM9-INFO-020`. Đổi focus khi
   idle/maximize/restore giữ relocated frame; source stamp đổi, Reset hoặc
   explicit Alignment áp dụng chính sách invalidation chung `OM9-INFO-021`.
7. Esc, foreign command, navigation/view change trong gesture hoặc ô nguồn
   đóng hủy toàn bộ phiên và trả frame trước. Ô không sở hữu gesture đóng
   chỉ dọn copy overlay của ô đó; document/workbench close dọn cả collection.
8. Single-owner cache vẫn process-local theo identity/incarnation/revision.
   Multi-selection/group/control-point selection không tự nhận pivot cache
   của một owner. Không tuyên bố pivot bền qua cold FCStd khi chưa có evidence.

Nguồn: bản trích Matrix 8 Book 1 PDF3/trang in37; Ctrl timing và optional Y
đối chiếu [Rhino 5 Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/gumball.htm).
Ảnh bốn viewport quy định kết quả hiển thị chung, không xác lập defaults parser.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `selected_objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Origin` | `reference` | Point của pivot trong captured coordinate context | Input/pick rõ, document units; `w` chọn world; không tự thay bằng centroid khi input lỗi. |
| `XDirection` | `reference` | X point tương đối origin để xác định hướng | Hữu hạn, khác origin; cùng coordinate context; Enter/Y point theo workflow trên. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-INFO-022`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust state origin/direction pick, Qt preview frame, native gizmo store lưu transform widget riêng với object.
3. Tích hợp `RelocateGumball` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-022",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Origin", kind: ParameterKind::Reference, required: false },
        Parameter { name: "XDirection", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Relocate origin cách 5 không move object; xoay X frame chỉ đổi handles; cancel trả pivot cũ.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

2026-10-12: `implemented` / `validated_supported_slice`; `approved_scope_complete=true`.
Approved OM9 core scope completed against the exact30 native suite: native primitive dimensions/expressions, standard FreeCAD Link/Scale with native OM9 Compound-derived helpers preserving admitted definitions/dependencies/subelements/Body trees and presentation across Undo/Redo/FCStd, bounded native input settling before fresh gestures after Undo with unchanged history and callback cancellation, actual Mesh Ctrl+Shift component picking, typed document Camera/Light physical frames, shared four-view Relocate and process-local cache. Native host limits and unsupported graph/frame cases remain explicit. Clayoo/SmartTargets, Rhino internal roll, full original-host parity, full Render and live host camera widgets are excluded. Tests run locally (30 processes / 955 checks); at the user's request, GitHub Actions only builds/packages. Its exact-commit Action is authoritative for build status and records tests_executed false. See the local-tests/CI-build-only policy report.

Development-host checkpoint plugin SHA256 `559e0dca08e3535174879dd81c598d4a894e70e43989d22dba8b6d77a8db6b20`; host SHA256 `98eef899354a8aa84cfa2934e6e1a3c8558c7e6fff3b89c347c06731f06018a8`.
[Final native evidence](../../../docs/validation/2026-10-12-gumball-core-completion.md).

### Approved core closure — 2026-10-12, final native gates passed

The same origin/X/optional-Y and Ctrl-start view-state workflow must operate on
the expanded owner and typed document-frame selection without changing native
definitions, dimensions, dependencies, Placement or Undo. Cancel restores the
shared frame. Typed Camera/Light object FCStd restoration is distinct from
widget persistence: the bounded256 single-owner pivot cache remains deliberately
process-local. Reset, explicit Alignment and source-stamp invalidation clear the
override; groups/multiple owners do not inherit another owner's cached pivot.
Final-source native view-only and cleanup gates passed. Rhino internal
roll, complete snap parity and cold persistence of the widget are outside the
approved scope.

[Validated baseline](../../../docs/validation/2026-10-11-gumball-four-view-completion.md).
[Candidate scope and applicability](../../../docs/validation/2026-10-12-gumball-core-release-candidate.md).
[Inputs/units/output/lifecycle and limits](../../../docs/features/OM9-INFO-022.md).
Source design sections and compiled request-planner examples above remain unchanged.

## Acceptance checklist

Checked native items refer to the dated 2026-10-11 baseline. New approved-scope items are validated by the final native cohort.

- [x] Relocate bằng Command và Ctrl-start ở mỗi ô đồng bộ origin/basis/Scale anchors sang bốn ô.
- [x] Default coordinate input và `w` world cho kết quả đúng; đổi camera không đổi parser giữa phiên.
- [x] Ctrl-start/thả Ctrl và late-Ctrl Extrude được replay riêng; widget-only không đổi geometry/Undo.
- [x] Relocate numeric/pointer/Osnap, optional Y và invalid axes giữ đúng preview/phase ở tất cả ô.
- [x] Idle focus/maximize/restore giữ pivot; Reset/Alignment recapture đồng bộ; cancel/close không để stale targets.

- [x] Hợp đồng và điều kiện hình học được kiểm chứng trong supported native slice; adapter gaps ghi riêng.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] Alignment/Relocate are view state; no dependency mutation or new associative History graph (native unchanged geometry/count/Undo evidence).
- [x] No document geometry/count/Undo writes for view-state changes; successful geometry uses OM9-INFO-020 transaction.
- [x] N/A for cold widget-cache persistence: bounded256 single-owner cache is process-local by approved design; native object FCStd persistence is tested separately.
- [x] Recheck view-only behavior and cleanup on final-source expanded owner/typed frame fixtures. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
- [x] Automated tests use feature ID `OM9-INFO-022`.

## Hợp đồng kỹ thuật chung

Applicability is explicit below. Dated baseline and passing final native gates are
mapped in the [candidate report](../../../docs/validation/2026-10-12-gumball-core-release-candidate.md).

- [x] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): Menu/CMD, numeric input, cancel and modifier timing: dated 2026-10-11 four-view native evidence; final-source replay passed.
- [x] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): Applicable as read-only source identity/permission/stamp checks. No geometry output; final expanded-owner/typed-frame unchanged-properties gates passed. Final evidence: [exact30 native cohort](../../../docs/validation/2026-10-12-gumball-core-completion.md).
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

## Multi-selection regression — 2026-10-12

- [x] Whole-object Shift/window selection, common frame/extents and MSR from all four views.
- [x] One Undo/Redo batch, transient multi-selection Relocate and retained component guards.

[Final local31/1099 evidence](../../../docs/validation/2026-10-12-gumball-multi-selection.md); new regression143 checks, Rust848 tests; CI build/package only.
