---
id: OM9-INFO-020
name: Gumball
command: Gumball
domain: 01-core
module: Info & Settings
kind: command
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-INFO-020 — Gumball

Alias tương thích: `Gumball`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-INFO-020` — Gumball.**

Gumball toggle hiển thị move/scale/rotate widgets khi object selected. X đỏ, Y xanh lá, Z xanh dương theo chỉnh giao diện Rhino 5 được user duyệt ngày 2026-10-11; arrows move một axis, plane handle move hai axes, origin free move; arcs rotate, outline squares scale một axis ở phía đối diện arrows. Menu ball cạnh tâm mở Relocate/Reset và CPlane/Object/World; numeric editor gọn, giữ trong viewport. Numeric edit nhận distance/góc/scale factor. Alt trong drag toggle Copy, Alt khi click numeric làm copy. Trong drag nhập số+Enter ràng buộc distance hoặc bước góc. Ctrl khi nhả arrow drag của planar curve/surface tạo extrude, có thể nhập distance trước; không coi Ctrl drag mọi object là extrude. Giữ đúng pivot/frame và capability, hủy trả preview.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `selected_objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Enabled` | `boolean` | Hiện gizmo | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Operation` | `choice` | Move, Rotate, Scale hoặc Extrude | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Axis` | `choice` | X, Y, Z, Plane hoặc Free | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Distance` | `number` | Dịch/extrude theo unit document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Angle` | `number` | Góc, độ tại UI | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Scale Factor` | `number` | Hệ số không đơn vị | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Copy` | `boolean` | Giữ originals | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-INFO-020`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust gizmo operation state và constrained transform/extrude plan; Qt hit-test handles, native adapter placement/shape/extrude, copy giữ gốc.
3. Tích hợp `Gumball` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Axis", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Alt copy không di chuyển gốc; scale X không đổi Y/Z; Ctrl planar curve tạo extrude đúng distance; Esc drag giữ object ban đầu.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Ngày 2026-10-11: `partially_implemented` / `validated_supported_slice`.
13 native handles; Move/Rotate Placement, plain BRep/Mesh/Points affine Scale, planar Extrude, Alt Copy, held Ctrl and native Qt input. Parametric/Link Scale and external-document Link remain unsupported; failed posttransaction callback rollback loses old Redo in the shared host.

Rhino-style visual revision adds a separate native menu target (14 scene paths
including the unchanged 13 Rust operation handles), camera-facing outline scale
squares and compact Qt numeric editor. Rendering and popup ownership use native
Coin/Qt; operation math/state/publication remain Rust-owned or native adapters
from the core slice. [Visual evidence](../../../docs/validation/2026-10-11-gumball-rhino-visual.md).

Source evidence: Matrix 8 Book 1 PDF45–47 / printed35–37, continuation inspected
until History PDF48. Source Y cyan/blue; approved OM9 Y green is a design choice.
[Inputs, parameters/defaults/units, outputs, lifecycle, dependencies, errors and tolerances](../../../docs/features/OM9-INFO-020.md).
[Native/Rust verification and limits](../../../docs/validation/2026-10-11-gumball-core.md).
No new associative History graph; existing native owner dependencies retained.
Geometry/report metadata persists in FCStd; widget cache is session view state.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [x] Undo/redo tested for successful gestures; failed posttransaction callback loses old Redo (host limitation).
- [x] Save/reload persistence tested for native geometry/report/layer/Link/retained 3DM.
- [x] Automated tests use feature ID `OM9-INFO-020`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [x] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): native supported slice; evidence/limitations in feature record.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-INFO-020` hoặc ghi lý do không áp dụng.
- [x] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): native supported slice; evidence/limitations in feature record.
- [x] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): native supported slice; evidence/limitations in feature record.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-INFO-020` hoặc ghi lý do không áp dụng.
- [x] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): native supported slice; evidence/limitations in feature record.
- [x] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): native supported slice; evidence/limitations in feature record.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-INFO-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-INFO-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-INFO-020` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
