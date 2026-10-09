---
id: OM9-SOLID-014
name: Sphere
command: Sphere
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

# OM9-SOLID-014 — Sphere

Alias tương thích: `Sphere`. Nhóm: `04-solid`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Sphere solid từ center và radius. 2Point dùng hai endpoint đường kính; 3Point fit qua ba điểm trên vòng, hoặc sau hai điểm nhập Radius và hướng; Tangent fit theo curve chạm, Point cho một điểm không tangent; FitPoints least-fit từ tối thiểu ba point/control point/mesh vertex. AroundCurve dùng plane vuông góc tangent path; Vertical dùng plane vuông góc C-Plane. 4Point dùng ba điểm/vòng hoặc hai điểm+radius rồi điểm thứ tư xác định sphere qua vòng.

### Tùy chọn và tương tác

- `Mode`: Center/2Point/3Point/Tangent/AroundCurve/4Point/FitPoints
- `Radius`: bán kính sphere
- `Diameter`: đường kính khi chọn mode tương ứng

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `definition_points` | `point` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Center/2Point/3Point/Tangent/AroundCurve/4Point/FitPoints | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Radius` | `number` | bán kính sphere | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Diameter` | `number` | đường kính khi chọn mode tương ứng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SOLID-014 và phiên chọn có thứ tự, kiểm tra vai trò definition_points; chuẩn hóa tùy chọn riêng của Sphere.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Sphere trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-SOLID-014",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Sphere volume 4πr³/3; 4Point đi qua điểm chỉ định; fit suy biến báo lỗi.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Status: **partially_implemented / validated_supported_slice**, verified 2026-10-09. All listed construction modes have native adapters within the explicit host scope below.

### Supported construction

- Stable menu ID `OM9_SolidSphereCenterRadius`; CMD `Sphere`, `_Sphere`, stable ID. Right-click the Sphere icon starts 2Point. `Mode=` or a mode name is accepted before the first point.
- `Center` (default): center and positive Radius or a radius point measured by world distance. `Diameter=value` halves the entered value. `Vertical` uses a perpendicular mouse radius plane.
- `2Point`: endpoints of a diameter; midpoint is center. A typed Diameter at the second step extends along captured CPlane X.
- `3Point`: three non-collinear world points define a circumcircle; that circle's center/radius defines the sphere. After two points, `Radius=value` or `Radius` then a value plus an off-chord direction chooses the circle plane and side. Radius must be at least half the chord.
- `4Point`: three points (or the two-point Radius construction above) define a circle, then a fourth point outside its plane defines the unique circumsphere. Coplanar fourth points are rejected.
- `FitPoints`: enter/pick points, then Enter; `Selection` appends the current selection, or Enter on an empty session imports it. Supports Part vertices/VertexN, B-spline control poles, mesh vertices and point clouds, including local/group placement. Entire imported batches are atomic and Undo removes a batch. Limit 3..1024 points. Normalized algebraic seed plus damped Gauss-Newton minimizes radial errors. Coplanar data fits a circle and uses its center as the sphere center; collinear/singular data is rejected. This coplanar fallback is an explicit host policy.
- `AroundCurve`: select `Path=Object.EdgeN` (whole object only if it has one edge), then mouse-pick center on that same native edge or `OnCurve=0..1` (parameter fraction, not arc-length fraction), then Radius. Uses exact bounded native geometry/tangent; periodic seam parameters are normalized into the edge interval. Mouse center uses the curve's native world pick even when it is above the CPlane. Radius mouse plane is normal to the path tangent. Ambiguous nearest branches require a fraction; cusps/degenerate tangents are rejected.
- `Tangent`: select three `Curve=Object.EdgeN@worldX,worldY,worldZ` constraints; `Point` makes the next point a passing-point constraint. After two constraints, `Radius=value` or `Radius` then value solves fixed-radius branches. Supported exact planar Lines/Circles and native NURBS converted to 2D with weights/knots preserved; solid EdgeN and nested placement work. The construction plane is parallel to the CPlane captured at the first constraint. Noncoplanar constraints are rejected.
- Tangent uses matching OCCT circle-tangency solvers, validates bounded contacts/radius/tangency and ranks solutions by distance from picked contacts. Ties require `Solution=1..N`; no arbitrary branch is committed. General NURBS use nine bounded initial seeds, so exhaustive discovery of all branches is not claimed. Nonplanar 3D sphere tangency, links and associative updates remain unsupported.

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
- [x] Automated tests use feature ID `OM9-SOLID-014`.
- [x] Listed construction modes have native adapters and fixtures within the documented scope.
- [ ] Full Matrix solver/History/layer compatibility verified.
- [ ] Full applicable Rhino foundation acceptance completed.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SOLID-014` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SOLID-014` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SOLID-014` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SOLID-014` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SOLID-014` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SOLID-014` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SOLID-014` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SOLID-014` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, giới hạn solver và kết quả native theo exact feature ID.
