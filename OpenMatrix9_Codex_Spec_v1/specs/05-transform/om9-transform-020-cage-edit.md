---
id: OM9-TRANSFORM-020
name: Cage Edit
command: CageEdit
domain: 05-transform
module: Transform Tools
kind: command
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
implementation_validation_record: ../../../docs/validation/2026-10-09-reusable-builder-cage.md
---

# OM9-TRANSFORM-020 — Cage Edit

Alias tương thích: `CageEdit`. Nhóm: `05-transform`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

CageEdit capture objects bằng cage/curve/surface có sẵn hoặc tạo BoundingBox, Line, Rectangle, Box. Cage một/hai/ba chiều biến dạng mượt, không tách polysurface seam. History dependency luôn có bất kể setting toàn cục. Cho deformation tổng thể hoặc vùng local với falloff.

### Tùy chọn và tương tác

- `Control Object`: Existing/BoundingBox/Line/Rectangle/Box; có thể dùng chính object
- `CoordinateSystem`: CPlane/World/3Point cho bounding box
- `Degree/PointCount`: cấu trúc line; count phải đủ degree+1 ở host
- `UDegree/VDegree/UPointCount/VPointCount`: cấu trúc rectangle, 3Point/Vertical/Center chọn plane
- `XPointCount/YPointCount/ZPointCount`: cấu trúc cage 3D
- `XDegree/YDegree/ZDegree`: bậc cage 3D
- `Deformation`: Accurate chậm/refit dày; Fast ít điểm/giảm chính xác
- `PreserveStructure`: Yes/No; Yes giữ control-point structure có thể giảm độ chính xác, No refit thêm điểm để phù hợp biến dạng
- `Region`: Global ảnh hưởng toàn không gian kể cả ngoài cage; Local tới falloff; Other box/sphere/cylinder giới hạn vùng
- `Falloff Distance`: chuyển tiếp ngoài vùng

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `captives` | `object` | 1 | Không đặt trong mẫu |
| `control_object` | `object` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Control Object` | `choice` | Existing/BoundingBox/Line/Rectangle/Box; có thể dùng chính object | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `CoordinateSystem` | `choice` | CPlane/World/3Point cho bounding box | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Degree` | `number` | cấu trúc line; count phải đủ degree+1 ở host Giá trị độc lập của Degree. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `PointCount` | `number` | cấu trúc line; count phải đủ degree+1 ở host Giá trị độc lập của PointCount. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `UDegree` | `number` | cấu trúc rectangle, 3Point/Vertical/Center chọn plane Giá trị độc lập của UDegree. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `VDegree` | `number` | cấu trúc rectangle, 3Point/Vertical/Center chọn plane Giá trị độc lập của VDegree. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `UPointCount` | `number` | cấu trúc rectangle, 3Point/Vertical/Center chọn plane Giá trị độc lập của UPointCount. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `VPointCount` | `number` | cấu trúc rectangle, 3Point/Vertical/Center chọn plane Giá trị độc lập của VPointCount. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `XPointCount` | `number` | cấu trúc cage 3D Giá trị độc lập của XPointCount. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `YPointCount` | `number` | cấu trúc cage 3D Giá trị độc lập của YPointCount. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `ZPointCount` | `number` | cấu trúc cage 3D Giá trị độc lập của ZPointCount. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `XDegree` | `number` | bậc cage 3D Giá trị độc lập của XDegree. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `YDegree` | `number` | bậc cage 3D Giá trị độc lập của YDegree. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `ZDegree` | `number` | bậc cage 3D Giá trị độc lập của ZDegree. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Deformation` | `choice` | Accurate chậm/refit dày; Fast ít điểm/giảm chính xác | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `PreserveStructure` | `boolean` | Yes/No; Yes giữ control-point structure có thể giảm độ chính xác, No refit thêm điểm để phù hợp biến dạng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Region` | `choice` | Global ảnh hưởng toàn không gian kể cả ngoài cage; Local tới falloff; Other box/sphere/cylinder giới hạn vùng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Falloff Distance` | `number` | chuyển tiếp ngoài vùng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TRANSFORM-020 và phiên chọn có thứ tự, kiểm tra vai trò captives, control_object; chuẩn hóa tùy chọn riêng của Cage Edit.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Cage Edit trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TRANSFORM-020",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "captives", kind: InputKind::Object, min: 1, max: None },
        Role { name: "control_object", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Control Object", kind: ParameterKind::Choice, required: false },
        Parameter { name: "CoordinateSystem", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Degree", kind: ParameterKind::Number, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "UDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "VDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "UPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "VPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "XPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "YPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "XDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "YDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "Deformation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "PreserveStructure", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Region", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Falloff Distance", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Dịch cage control giữ shell seams; Local không đổi điểm ngoài falloff; dependency cập nhật dù History off; ReleaseFromCage tách đúng captive.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Status:** partially_implemented; validated_supported_slice.

### Captives, control and parameters

Native menu `OM9_TransformCageEditingCageEdit`, CMD `CageEdit` and `OpenMatrix9Gui.captureCage(control, captives, region="Global", falloff=0)` share the native service. Select whole captives, Enter, then choose one existing native 3D CageControl by name/preselection/list. Roles remain separate; the control cannot capture itself. Global/Local and finite nonnegative falloff in mm are supported. Local uses the cage world bounding box at capture. Bindings store current captured geometry/parameters and update independently of global History Record/Update.

Triangular native Mesh and supported single-edge/rectangular-face NURBS geometry permit nonlinear deformation. General BRep permits only certified global affine cage transforms; unsupported topology/region/refit combinations reject. Explicit Restore of an existing retained 3DM control routes `restore3dmCage` through verified archive/hash/namespace/UUID and current inverse checks, preserving precise API errors before partial mutation.

The nonmodal controller creates no document preview objects. Rust owns identity/phases/options/owned snapshots; C++ supplies exact native topology/global-frame inspection, Qt widgets and typed native API calls. Stale input/control/ancestor placement rejects before OK. Cancel/Esc, close, document switch and workbench deactivation discard the uncommitted session. Native bindings use FreeCAD transactions/properties for Undo/Redo/FCStd. 1D/2D line/surface controls, automatically constructed control modes, Accurate/Fast, PreserveStructure switches, general BRep refit and Other sphere/cylinder regions remain unsupported. See [Cage command contract](../../../docs/features/cage-command-native-contract.md).

[Matching native validation](../../../docs/validation/2026-10-09-reusable-builder-cage.md) records the accepted API/command scope and limits. Original design requirements above remain broader than this implemented slice.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-TRANSFORM-020`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TRANSFORM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TRANSFORM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TRANSFORM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TRANSFORM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TRANSFORM-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TRANSFORM-020` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
