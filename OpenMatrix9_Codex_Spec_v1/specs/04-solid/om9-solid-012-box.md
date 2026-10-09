---
id: OM9-SOLID-012
name: Box
command: Box
domain: 04-solid
module: Solid Tools
kind: command
implementation_status: partially_implemented
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SOLID-012 — Box

Alias tương thích: `Box`. Nhóm: `04-solid`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Box dựng solid từ rectangle base và cao. Chọn corner/length, width Enter dùng length, height Enter dùng width. Diagonal dùng hai góc đối diện không hỏi side length; Cube trong Diagonal dùng góc base/top xác định cao và orientation.

### Tùy chọn và tương tác

- `Mode`: Corners/Diagonal/3Point/Vertical/Center
- `Length/Width/Height`: kích thước box
- `3Point`: hai đầu cạnh rồi width
- `Vertical`: base vuông góc C-Plane
- `Center`: base quanh tâm
- `Cube`: chọn diagonal 3D xác định khối lập phương

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `definition_points` | `point` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Corners/Diagonal/3Point/Vertical/Center | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Length` | `number` | kích thước box Giá trị độc lập của Length. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Width` | `number` | kích thước box Giá trị độc lập của Width. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Height` | `number` | kích thước box Giá trị độc lập của Height. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `3Point` | `reference` | hai đầu cạnh rồi width | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Vertical` | `boolean` | base vuông góc C-Plane | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Center` | `reference` | base quanh tâm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Cube` | `reference` | chọn diagonal 3D xác định khối lập phương | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SOLID-012 và phiên chọn có thứ tự, kiểm tra vai trò definition_points; chuẩn hóa tùy chọn riêng của Box.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Box trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
4. Preview nếu cần dùng dữ liệu tạm; xác nhận ghi kết quả và dependency trong một transaction. Hủy/lỗi không tạo output dở dang; Undo/Redo và save/reload giữ contract.
5. Native C++ đăng ký command/menu và dispatch sang Rust/native geometry. Python đăng ký workbench và hỗ trợ fixtures/macro kiểm chứng. Capability chưa có hoặc chưa kiểm chứng phải giữ disabled với lý do cụ thể.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "3Point", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Cube", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- BBox và volume bằng kích thước; Center đối xứng; Vertical phụ thuộc C-Plane; Enter truyền width/height đúng.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Status: **partially_implemented / validated_supported_slice**, verified 2026-10-09. All listed construction modes have native adapters within the explicit host scope below.

### Supported construction

- Stable menu ID `OM9_SolidBoxCornertoCornerHeight`; CMD `Box`, `_Box`, stable ID. Right-click the Box icon starts 3Point.
- `Mode=Corners` (default): two base corners then height, or first corner then typed Length/Width/Height. `3Point`/`3p`: two endpoints of the first edge, then signed width and height; edge projects onto the captured CPlane.
- `Diagonal`: opposite base corner then height; numeric side length is rejected. `Cube` within Diagonal (also a mode shortcut): two opposite 3D corners define equal edges; a minimal rotation aligns the captured frame's body diagonal with the picked diagonal. The exact half-turn has a deterministic axis. Cube preview reaches both corners.
- `Vertical`: first edge in the captured CPlane, width along its frozen normal, height perpendicular to that vertical base. Changing viewport after the first point does not change the captured normal. Mouse width projects into the perpendicular base plane.
- `Center`: first point is base center; typed Length/Width are full dimensions, or picked corner doubles the two center-to-corner extents. Height starts at the base plane.
- `Length=`, `Width=`, `Height=` apply at their respective steps. Enter at Width uses Length; Enter at Height uses Width. Typed Length must be positive; signed width/height and negative picked extents normalize the origin and positive dimensions while preserving a right-handed frame.
- Mouse height uses the closest point between mouse ray and the frozen base-normal axis. A parallel ray requires typed Height or a side/perspective view.

### Output, lifecycle and dependency

- Rust owns ordered input state, units, finite bounds and pure construction math. C++/Qt resolves native references and builds one document-root `Part::Feature` using typed Part/OpenCascade calls in one transaction. No CMD text is evaluated as Python.
- Output must be a valid closed BRep with one solid and positive finite volume; metadata `OM9FeatureId`, `OM9Command`, `SolidParameters` records mode, normalized dimensions/frame and curve reference names. Solid purple is RGB 166,104,209.
- Standalone snapshot geometry: editing a source after commit does not regenerate the result. Source BRep/global placement is revalidated before commit; changed/deleted inputs create no partial output. Curves/points/meshes are preserved. Associative History, active layers, Builder/Styles and full shared foundation contracts remain unverified.
- Final-step hover preview is an unpickable Coin scene node; FitPoints also previews the candidate fit with the hovered point. Tangent resolves after its final constraint; it has no live preview of all branches. Commit creates one object; input Undo revisits accepted states, native Undo/Redo revisits the output.
- Cancel/Esc (including unsubmitted CMD text), document switch/close and workbench deactivation discard input and preview. Invalid numbers, singular geometry and stale references leave no document output. Native kernel failure aborts its transaction.
- Shared menu/CMD and Qt mouse paths use the same controller; successful commands alone enter command history. Native FCStd save/reload preserves geometry, mode and metadata.

### Units and host decisions

- mm by default; typed dimensions also accept mm/cm/in. Coordinates/final corners are bounded by +/-1e9 mm; dimensions are finite, nonzero, at least 1e-7 mm and at most 1e9 mm. Sphere radius is positive.
- Absolute `x,y[,z]` and relative `rX,Y[,Z]` use the current CPlane. A construction frame freezes when the first defining point is accepted. Mouse uses shared snaps or the relevant construction plane; a parallel ray requires a different view or typed input. Curve `@x,y,z` pick seeds are explicit world coordinates.
- Host thresholds and finite multi-start solving are OpenMatrix9 policies, not inferred Matrix defaults. Full Grid/Ortho/snaps compatibility is not claimed.

### Validation

- Exact feature fixtures: `rust/tests/solid_commands.rs`, `solid_session.rs`, `solid_options.rs`; native `tests/solid_commands_smoke.FCMacro` and `solid_options_smoke.FCMacro` exercise the actual shared CMD, menus and Qt mouse events.
- Native geometry, analytic volumes, source preservation, degenerate inputs, selected-fit placements, branch choice, scene-only preview, input Undo, output Undo/Redo and FCStd reload have evidence.
- Independent review corrections cover frozen Vertical orientation, solid-edge references, seam-crossing periodic paths, near-antiparallel Cube diagonals and elevated AroundCurve mouse centers.
- Fresh report paths, check counts, matching OCCT 8.0.1/module provenance and global-check limits: [validation record](../../../docs/validation/2026-10-09-solid-box-sphere-options.md). Earlier core evidence remains in the [2026-10-06 record](../../../docs/validation/2026-10-06-solid-box-sphere.md).

## Acceptance checklist

- [x] Hành vi của supported slice đã được đối chiếu.
- [x] Supported inputs and selection state documented.
- [x] Supported parameters/defaults/units documented; host decisions identified.
- [x] Output geometry documented and checked natively.
- [x] Scene-only preview/commit/cancel behavior tested.
- [x] Success history/no-input-dependency model checked; associative History remains TODO_EVIDENCE.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-SOLID-012`.
- [x] Listed construction modes have native adapters and fixtures within the documented scope.
- [ ] Full Matrix solver/History/layer compatibility verified.
- [ ] Full applicable Rhino foundation acceptance completed.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SOLID-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SOLID-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SOLID-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SOLID-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SOLID-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SOLID-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SOLID-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SOLID-012` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, giới hạn solver và kết quả native theo exact feature ID.
