---
id: OM9-CURVE-005
name: Circle
command: Circle
domain: 02-curve
module: Curve Tools
kind: command
implementation_status: partially_implemented
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-CURVE-005 — Circle

Alias tương thích: `Circle`. Nhóm: `02-curve`. Trạng thái: `partially_implemented` — phần hỗ trợ đã kiểm chứng ngày 2026-10-09.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-005` — Circle.**

Dựng circle bằng tâm và radius; Radius/Diameter chuyển ý nghĩa giá trị. Orientation dùng hướng pháp tuyến mặt circle; Vertical dựng vuông góc CPlane. Circumference suy radius từ chu vi, Area từ diện tích. 2Point lấy hai đầu đường kính và là nhánh chuột phải. 3Point lấy ba điểm trên circle; sau hai điểm có thể nhập Radius và hướng. Tangent nhận curve; Point nới điều kiện tới điểm tự do, FromFirstPoint khóa vị trí tiếp xúc đầu, Radius giới hạn nghiệm tại curve thứ hai. AroundCurve đặt tâm trên curve và mặt circle vuông góc tiếp tuyến. FitPoints cần ít nhất ba điểm, có thể lấy point object, control point hoặc mesh vertex. Deformable xuất xấp xỉ NURBS theo Degree/PointCount thay cho conic chính xác; những giá trị minh họa không là mặc định.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `picked_points` | `point` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Radius` | `number` | Bán kính theo đơn vị document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Diameter` | `number` | Đường kính theo đơn vị document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Circumference` | `number` | Chu vi theo đơn vị document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Area` | `number` | Diện tích theo đơn vị document bình phương | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Orientation` | `reference` | Pháp tuyến mặt circle | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Deformable` | `boolean` | Xuất xấp xỉ NURBS | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Degree` | `number` | Bậc xấp xỉ | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `PointCount` | `number` | Số control point | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Vertical` | `boolean` | Vuông góc CPlane | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `2Point` | `boolean` | Hai đầu đường kính | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `3Point` | `boolean` | Ba điểm trên circle | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Tangent` | `boolean` | Solver tiếp xúc | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Point` | `reference` | Điểm tự do | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `FromFirstPoint` | `boolean` | Khóa điểm đầu | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `AroundCurve` | `reference` | Curve quy định hướng | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `FitPoints` | `boolean` | Fit điểm đã chọn | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-005` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Ưu tiên conic chính xác cho các nhánh tâm/ba điểm; triển khai least-squares fit và bộ giải tiếp xúc riêng, báo sai số cho Deformable.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-005` tới command native `Circle` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-005",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circumference", kind: ParameterKind::Number, required: false },
        Parameter { name: "Area", kind: ParameterKind::Number, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Deformable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "2Point", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "3Point", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Point", kind: ParameterKind::Reference, required: false },
        Parameter { name: "FromFirstPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AroundCurve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "FitPoints", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Đường kính 8 cho radius 4; ba điểm thẳng hàng không tạo circle; AroundCurve có pháp tuyến song song tiếp tuyến tại tâm.
- Cancel ở giữa phiên không thay object/layer; lỗi solver không commit output một phần. Undo/Redo phục hồi toàn bộ thao tác; save/reload bảo toàn geometry, placement và links theo hợp đồng.
- Kiểm tra NaN/Infinity bị từ chối và UI cùng CMD cho cùng kết quả với cùng input; các options chưa có mặc định yêu cầu xác nhận giá trị đang hiển thị.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

### Source evidence

Read `OpenMatrix9_Codex_Spec_v1/matrix8_book_1.pdf`, PDF pages 138–140
(printed 128–130), from Circle through its continuation before Ellipse.
The source covers center/Radius/Diameter, Orientation, Circumference, Area,
Vertical, 2Point, 3Point, Tangent, AroundCurve, FitPoints and Deformable.
The engineering contract and Rust example in spec_v1 remain the full target.
This implementation does not make the remaining branches complete.

### Supported interaction

Menu, sidebar, F6 and CMD `Circle` enter the same native capability gate.
An editable document and a 3D view are required. Center mode and Radius are
the displayed OM9 defaults; the source does not establish numeric defaults.

- Center: pick/enter center, then a radius point or positive scalar.
- `Radius`, `Diameter`, `Circumference`, `Area` select the measure. A named
  scalar such as `Diameter=8` or `Area=50.26548245743669` commits after center.
  Radius is respectively value, value/2, value/(2π), or sqrt(value/π).
- Length scalars accept bare mm, `mm`, `cm`, `in`; area accepts bare mm²,
  `mm2`/`mm^2`, `cm2`/`cm^2`, `in2`/`in^2`. Model coordinates remain mm.
- `2Point` before the first point uses exact 3D diameter endpoints.
- `3Point` before the first point constructs the exact spatial circumcircle.
  After two points, `Radius=5` or `Radius` then a value/location requests a
  center-side direction. The radius must be at least half the chord length.
- `Orientation` after center requests a normal-direction point, then size.
  The direction point is not a circumference point.
- `Vertical` before center constructs a plane perpendicular to CPlane.
- `AroundCurve`: select a bounded native edge by mouse or `Curve=Object.EdgeN`,
  then pick its center on the edge or enter `OnCurve=fraction` (0..1), then
  Radius/Diameter or another supported size. Its tangent is the circle normal.
- `FitPoints`: append typed points, or `Selection` imports selected native
  vertices, curve poles, a BSpline surface's poles, mesh vertices or point-cloud
  points. Enter fits and commits. Enter with no accumulated points imports the
  current selection and fits it. Input batches are limited to 1024 points.
- `Tangent`: pick native bounded curve edges, or use
  `Curve=Object.EdgeN@worldX,worldY,worldZ`. Three curve/free-point constraints
  solve a circle; `Radius=value` needs two constraints. `Point` makes the next
  input a free point. `FromFirstPoint=Yes` locks the first curve contact to its
  picked point. Equal-scoring branches require `Solution=n`; Undo permits
  reselecting a constraint. Hover shows an unpickable white contact cue.
  Curves may be noncoplanar; `Vertical=Yes` requires the solved Circle plane to be
  perpendicular to CPlane, and `Vertical=No` removes that restriction.
- All Circle branches accept `History=Yes/No` (OM9 default No), including
  Deformable. `Point=Object.VertexN` supplies a live point; after `Orientation`,
  `Direction=Object.VertexN` supplies a live direction endpoint. Typed/mouse
  coordinates without explicit source references remain constants. AroundCurve
  records its edge/fraction; FitPoints Selection records current native samples;
  Tangent records curve/point inputs and options. Global Record gates creation.
- `Layer=1..32`, `Layer=Gem 01`, or `Layer=None` selects the output layer. Sidebar
  arrows, swatches, locks and visibility buttons use the same persistent state.
  The selected layer owns normal, History and Deformable Circle output.
- `Deformable=Yes` outputs a periodic nonrational B-spline approximation;
  `Degree=1..11`, `PointCount=Degree+1..256`. OM9 defaults are Degree 3 and
  PointCount 8, displayed in the prompt; these are not Matrix defaults.
- Session `Undo` removes the last construction point, or backs out/reopens
  the Orientation step. Esc/Cancel clears the transient session.

### OM9 host decisions and tolerances

These choices specify this host slice; they are not inferred Matrix defaults.
CPlane axes latch at the first construction point. Typed two-axis coordinates
use that CPlane, three-axis coordinates are world XYZ, and existing relative
coordinate parsing is retained. F4 refreshes the current viewport frame before
picking its CPlane origin, including a viewport switch before the first pick.

Center point sizing projects the displacement into the Circle plane. Diameter
and Circumference point picks measure that projected length and convert the
selected measure to radius. An Area location uses the projected radius under
the cursor; numeric Area remains a square-unit measure. This mapping is an OM9
decision for the source's "show area value" interaction.
Analytic hover displays Radius/Diameter/Circumference/Area in mm/mm² without
mutating the session or document; an invalid hover restores the session prompt.
Circle does not apply global Ortho or Shift to sizing points.

For a 3D diameter, choose the normal closest to latched CPlane Z and perpendicular
to its direction, falling back to projected CPlane Y when parallel. Three
points determine their own plane; collinearity is rejected at normalized cross
magnitude ≤1e-10. The cross product is normalized before direction checks so
small valid circles remain supported.

3Point Radius uses the chord midpoint and a perpendicular projection of the
direction point to choose the center side and circle plane. A picked radius
length is measured from the second circumference point. At Radius=half-chord,
the center is its midpoint, and the direction still defines the normal.

FitPoints performs a normalized algebraic least-squares circle fit after a
covariance best-plane solve. Coincident/collinear or ill-conditioned sets reject;
noisy/nonplanar input is projected into that plane and the maximum sampled
spatial residual is persisted as `CircleFitDeviation` in mm. It does not claim
the original Matrix fit algorithm. Selected native shapes are read in world
placement; BSpline surfaces currently import their whole pole grid, not an
individual edit-mode surface-control-point selection. Imported points are
snapshots by default; `History=Yes` associates explicit native Selection sources
and reads their current samples on recompute.

Tangent solves bounded native lines/circles and planar NURBS in any common plane.
Rust infers a plane from complete exact NURBS pole/endpoint evidence, retaining
the latched CPlane frame when geometrically valid. Preferred-plane agreement
uses min(1e-6 mm, span*1e-10), preserving tiny conic orientation. Plane inference
normalizes vectors and checks all evidence within 1e-6 mm. Collinear constraints
use the plane normal closest to CPlane Z, falling back to CPlane Y. Native OCCT
GCC constructs candidates; contact position, perpendicularity, radius, finite
edge bounds and FromFirstPoint are verified. Picks rank branches, while tied
scores require explicit selection. Both planar and spatial results receive
exact 3D native contact/direction checks; near-planar projection cannot hide an
out-of-plane tangent. Noncoplanar constraints use Safe Rust bounded damped least
squares with exact native position/unit-tangent callbacks. The solver searches
at most 14 initial guesses, 64 iterations each and 24000 curve evaluations,
clamps parameters to each bounded edge, and checks accepted contacts again.
Plane/radius residual tolerance is 1e-6 mm (radius scaled by max(1,R)); tangent
angular dot products must be <=1e-8. Callback failures and an unsuccessful
bounded search reject without creating an object; this is not proof of no root.
Tangent Vertical checks |normal·CPlaneZ| <= 1e-8 before commit; a failed constraint
leaves the same references active for `Vertical=No`, Undo or Cancel recovery.
General NURBS use bounded multistart for unconstrained radius; exhaustive root
enumeration is not promised. Tangent Radius location measures from the last
constraint; before the first constraint its radius must be entered numerically.
References are checked for geometry/placement changes before further use and
commit; Undo and Cancel remain available when a reference becomes stale.

Deformable rebuilds 1024 normalized unit-circle samples into the exact requested
periodic Degree/PointCount, then places/scales the poles in world coordinates.
This normalization preserves tiny circles and translated centers. All poles
must also obey ±1e9 mm bounds. `CircleApproxDeviation` records the maximum
radial/plane error at 1024 sampled parameters; this is a sampled estimate,
not a certified supremum. Low-degree/coarse approximations may have large error.

Vertical point sizing chooses the plane containing CPlane Z and the horizontal
center-to-point direction. A scalar with no direction uses CPlane X/Z, normal Y.
Mouse radius picking for Orientation intersects the session plane; Vertical
direction picking uses the latched CPlane. Snapped spatial points are passed
to the same Rust validation and plane projection.

Coordinates and full analytic extents must remain finite within ±1e9 mm;
radius is 1e-7 through 1e9 mm and distinct direction/point distance is ≥1e-7 mm.
No current document tolerance is claimed. Invalid data/options leave the
session active without advancing points or changing a failed named size.

### Native output and lifecycle

Safe Rust owns the construction session, numeric validation, plane/circle fit,
periodic approximation and deviation data. C++ is retained for FreeCAD/Qt
selection, OCCT bounded-curve/GCC solving and native transactions. References
remain GUI-owned snapshots with revision checks; Rust retains no native pointer.
Test macros use the required FreeCAD Python API. This is a native API exception,
not a claim that the FFI or OCCT dependencies are memory-safe.

Rust produces an analytic center/normal/radius plan. The typed C++ adapter calls
`Part.makeCircle` and wraps its edge in a closed `Part.Wire`, then creates one
root `Part::Feature` with `OM9FeatureId=OM9-CURVE-005` in one transaction.
Normal output has one exact `Part.Circle` edge, not a polygon. Deformable output
has one closed periodic `Part.BSplineCurve` edge with unit weights, requested
degree/count, and the same feature ID. `CircleDeformable`, `CircleFitDeviation`
and `CircleApproxDeviation` survive save/reload.
No source object is replaced. Default output is a snapshot. `History=Yes`
creates native `OpenMatrix9Gui::CircleHistory` with a versioned owned recipe,
construction points, source roles and links. Numeric size is a fixed converted
radius; picked size is recomputed from current points. AroundCurve stores an
EdgeN link and normalized parameter fraction (not arclength); Tangent stores
normalized source picks and re-solves exact contacts. Fit Selection rereads whole
source samples, including supported topology growth. Source shape/placement and
parent placement/group changes recompute the Circle. Deformable replay constructs
a private periodic spline without mutating the live preview spline. Original
v1 analytic AroundCurve files remain supported.
RCORE-09 policy revision (2026-10-09): global Update pauses/resumes existing
records independently of Record. Record No suppresses new records and does not
freeze existing recorded Circle outputs. This updates the OM9 implementation
policy; the original source contract above is retained.
Lock blocks child edits; child shape/placement edits detach, with Undo restoring
links. Explicit Recorded=false also detaches. Missing/invalid/empty sources clear
stale geometry with a recompute error; valid source recovery and deletion Undo
restore it. Undo/Redo refresh parent notifications and FCStd cold restore loads
the native type without activating the workbench. Source links are immutable
while recorded; detach before replacing them. Shape failures do not change any
source. Automatic topology correspondence beyond the stored EdgeN is unaccepted.
Without an active layer the output remains a root feature with OM9 green wire
color. Explicit layers are tagged root `App::DocumentObjectGroup` objects with
per-document active selection and native color/visibility/lock properties.
Locked layers reject before opening the Circle transaction. Group membership,
style and output creation share one Undo step. Hidden layers create hidden
Circles; color changes update tagged OM9 members. Selection/styles persist in
FCStd; imported 3DM layer metadata is not reassigned. These are OM9 host policies,
not a claim of full Rhino layer inheritance or imported layer mapping.

Radius/direction hover preview samples 128 segments, with a repeated closing point, under an
unpickable Coin node. It creates no document object and does not change the
session. Commit/Cancel/document switch/document deletion/workbench exit remove
the transient preview. Reference/solver errors leave the session active for
Undo/options/Cancel recovery; native output adapter failure aborts its transaction and ends
the session with a visible error. Success history is recorded only on commit.
Native Undo/Redo and FCStd persistence retain exact geometry and feature ID.

### Remaining scope

Certified/exhaustive NURBS solver roots, individual surface edit-control-point
selection, automatic source topology correspondence, full Rhino layer
inheritance/import mapping and exhaustive
Matrix compatibility have not been accepted. The original sidebar right-click
now starts 2Point; menu/CMD `Circle` then `2Point` remains available.

### Validation

See [base evidence](../../../docs/validation/2026-10-09-circle.md) and
[advanced evidence](../../../docs/validation/2026-10-09-circle-advanced.md): advanced native
geometry/references, selection placement, constrained and ambiguous tangent
solutions, periodic approximation, Qt mouse/contact cue, right-click,
recovery, Undo/Redo and FCStd metadata round trip.
The later [spatial/History evidence](../../../docs/validation/2026-10-09-circle-spatial-history.md)
records arbitrarily oriented common planes, Tangent Vertical and analytic
AroundCurve dependencies, including parent Undo/Redo and cold restore.
The [remaining-branches validation](../../../docs/validation/2026-10-09-circle-complete-branches.md)
records noncoplanar NURBS contacts, all-mode History replay and layer integration.

## Acceptance checklist

Dấu kiểm chỉ áp dụng cho phần Circle cơ bản và nâng cao đã kiểm chứng nêu trên. History cho các nhánh Circle và layer hiện hành đã kiểm chứng; mặc định vẫn là snapshot. Nghiệm thu đầy đủ feature và các hợp đồng
kỹ thuật chung chưa đủ bằng chứng vẫn để trống.

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] History/dependencies tested if applicable.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-CURVE-005`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CURVE-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CURVE-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CURVE-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CURVE-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CURVE-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-CURVE-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-CURVE-005` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
