---
id: OM9-CURVE-006
name: Ellipse
command: Ellipse
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

# OM9-CURVE-006 — Ellipse

Alias tương thích: `Ellipse`. Nhóm: `02-curve`. Trạng thái: `partially_implemented` — phần hỗ trợ đã kiểm chứng ngày 2026-10-09.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-006` — Ellipse.**

Nhập tâm rồi hai đầu bán trục để dựng ellipse kín. Diameter nhận hai đầu trục thứ nhất rồi đầu trục thứ hai. Corner lấy hai góc bounding rectangle. Vertical đặt mặt ellipse vuông góc CPlane. FromFoci dùng hai tiêu điểm và một điểm trên ellipse; MarkFoci bổ sung point object tại tiêu điểm. AroundCurve lấy vị trí tâm trên curve và dựng hai trục ở mặt vuông góc tiếp tuyến. Deformable chọn xấp xỉ NURBS, Degree và PointCount điều khiển biểu diễn; không mặc định các số minh họa.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `picked_points` | `point` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Deformable` | `boolean` | Xấp xỉ NURBS | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Degree` | `number` | Bậc xấp xỉ | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `PointCount` | `number` | Số control point | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Vertical` | `boolean` | Plane vuông góc CPlane | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Corner` | `boolean` | Từ bounding rectangle | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Diameter` | `boolean` | Từ đầu trục | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `FromFoci` | `boolean` | Từ hai tiêu điểm và điểm trên ellipse | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `MarkFoci` | `boolean` | Tạo hai point tiêu điểm | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `AroundCurve` | `reference` | Curve quy định plane | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-006` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Tạo ellipse trong frame trực chuẩn; với FromFoci tính tổng khoảng cách và kiểm tra tính khả thi trước khi xuất conic/BSpline.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-006` tới command native `Ellipse` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-006",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Deformable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Corner", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "FromFoci", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MarkFoci", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "AroundCurve", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Tâm và hai trục tạo ellipse đúng kích thước; MarkFoci thêm hai point đúng vị trí; điểm có tổng khoảng cách không lớn hơn khoảng cách hai tiêu điểm bị từ chối.
- Cancel ở giữa phiên không thay object/layer; lỗi solver không commit output một phần. Undo/Redo phục hồi toàn bộ thao tác; save/reload bảo toàn geometry, placement và links theo hợp đồng.
- Kiểm tra NaN/Infinity bị từ chối và UI cùng CMD cho cùng kết quả với cùng input; các options chưa có mặc định yêu cầu xác nhận giá trị đang hiển thị.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

### Source evidence

Matrix 8 Book 1 PDF pages 140–141 (printed 130–131), resolved through the private reference registry. Source defines center/two semiaxis endpoints, right-click Diameter, Corner, Vertical, FromFoci/MarkFoci, AroundCurve and Deformable with Degree/PointCount. It gives no numeric radius defaults or History requirement. The original engineering contract and Rust request-planner example remain authoritative and unchanged.

### Supported behavior and host choices

Rust owns all portable construction/session/validation. C++ only adapts Qt events, exact OCCT/Part geometry, native edge references and document/layer transactions.

- Center: center, first semiaxis endpoint, second semiaxis endpoint. First axis projects into the latched CPlane; the second size is the perpendicular component. Three-dimensional centers retain their elevation.
- Diameter: first axis start/end then second semiaxis endpoint. Both first-axis vector and center use the same plane projection. Numeric first-axis entry means full diameter.
- Corner: opposite enclosing rectangle corners aligned with CPlane X/Y. Vertical: first axis in CPlane, second along CPlane normal.
- FromFoci: two distinct foci and an off-axis curve point define the spatial ellipse plane. MarkFoci creates two separate native vertex objects at the exact analytic foci.
- AroundCurve: a bounded native `Object.EdgeN`, center picked on that edge or `OnCurve=0..1`; ellipse normal follows its exact tangent. World picks and CPlane typed coordinates remain distinct. Changed/deleted references reject without advancing and permit Undo/reselect.
- Deformable: Rust fits a periodic unit-circle spline then applies the ellipse frame/scales. Degree and pole count are preserved. Persisted `EllipseApproxDeviation` is the maximum Euclidean difference at 1024 corresponding parameter samples, not a certified continuous Hausdorff bound.

Displayed OM9 defaults (not attributed to Matrix): Center, Deformable=false, Degree=3, PointCount=8, MarkFoci=false. Degree 1–11, pole count 4–256 and strictly greater than degree. Positive numeric lengths support the shared input-unit context and explicit unit suffixes. Minimum semiaxis 1e-7 mm; all coordinates and the complete analytic extent stay within +/-1e9 mm. Equal semiaxes produce an exact circle; unequal semiaxes remain an analytic ellipse.

### Native outputs and lifecycle

Exact and deformable outputs are valid closed single-edge `Part::Feature` wires, tagged `OM9FeatureId=OM9-CURVE-006`, `OM9Role=Ellipse`. MarkFoci adds `Focus1` and `Focus2` vertex features with the same feature ID. No document object is created during preview. Invalid input preserves the current picks/options; Undo removes the last pick (or selected path), Cancel clears the session.

Ellipse plus both foci share one transaction and current OM9 layer. Layer lock is checked before creating the transaction; membership/style/visibility follow the existing layer adapter. Creation Undo/Redo and FCStd preserve geometry, identity and membership. Native references are released on completion, cancellation and restart.

### Limits

Outputs are snapshots; Ellipse History is explicitly unsupported. Coincident foci and collinear FromFoci points are rejected because they cannot determine the requested plane. This scope does not accept every Rhino layer/import mapping, automatic topology correspondence, all object-editing/CV operations or the complete original engineering contract.

### Validation

Native acceptance is recorded separately in [the validation report](../../../docs/validation/2026-10-09-ellipse.md); Rust unit/session checks alone do not count as native proof.

## Acceptance checklist

- [x] Hợp đồng feature và điều kiện hình học được đối chiếu Matrix8 Book1 PDF140–141 trước triển khai.
- [x] Inputs, lựa chọn curve và các nhánh dựng được ghi rõ.
- [x] Defaults hiển thị, units, Degree/PointCount và tolerance là quyết định OM9, không suy từ hình minh họa.
- [x] Native exact ellipse/circle wire, periodic spline và hai point tiêu điểm được kiểm chứng.
- [x] Preview không tạo object; Cancel, input lỗi, reference thay đổi, Undo/reselect được kiểm chứng.
- [x] History/dependencies: snapshot; nguồn không yêu cầu History, option History bị từ chối rõ ràng.
- [x] Undo/Redo phục hồi đồng thời ellipse, hai foci và layer; đổi màu layer gồm PointColor có Undo/Redo.
- [x] FCStd reload bảo toàn geometry, feature/role IDs và layer membership.
- [x] Automated native tests có feature ID `OM9-CURVE-006`: 75/75 checks; Rust session 14/14.

## Hợp đồng kỹ thuật chung

Giữ đánh giá theo phạm vi: kiểm tra thực tế dưới đây không xác nhận toàn bộ hợp đồng Matrix/Rhino.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): menu/sidebar/CMD và right-click đã kiểm chứng; toàn bộ completion/aliases/F6 theo hợp đồng chung chưa nghiệm thu.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): output type/metadata/transaction/layer/FCStd đã kiểm chứng; đầy đủ Rhino/import semantics chưa nghiệm thu.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): AroundCurve native edge và stale recovery đã kiểm chứng; automatic topology correspondence chưa hỗ trợ.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): mouse axes, edge/center, transient preview đã kiểm chứng; toàn bộ snap/special picking modes chưa nghiệm thu.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): latched CPlane, spatial points, shared unit context và explicit lengths có kiểm thử; mọi dạng coordinate grammar chưa nghiệm thu.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): creation Undo/Redo và layer color Undo/Redo đã kiểm chứng; mọi thao tác edit chưa nghiệm thu.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): MarkFoci tạo hai vertex object; Deformable đúng degree/poles, không nghiệm thu toàn bộ CV editing.

- [x] Supported slice, tolerance, source/default separation và giới hạn native được ghi riêng; [bằng chứng](../../../docs/validation/2026-10-09-ellipse.md).
