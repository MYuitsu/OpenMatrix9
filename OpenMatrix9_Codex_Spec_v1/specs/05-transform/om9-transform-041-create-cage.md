---
id: OM9-TRANSFORM-041
name: Create Cage
command: Cage
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

# OM9-TRANSFORM-041 — Create Cage

Alias tương thích: `Cage`. Nhóm: `05-transform`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Cage tạo control cage dạng box từ rectangle base/height hoặc BoundingBox objects; sau đó CageEdit mới capture objects. Cage ít điểm điều khiển có thể biến dạng model nhiều điểm mượt. Không tự capture khi chỉ tạo cage.

### Tùy chọn và tương tác

- `BoundingBox`: dùng bbox targets
- `CoordinateSystem`: CPlane/World/3Point
- `Base Mode`: Corners/Diagonal/Cube
- `XPointCount/YPointCount/ZPointCount`: counts 3D
- `XDegree/YDegree/ZDegree`: bậc 3D

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `definition_points` | `point` | 0 | Không đặt trong mẫu |
| `targets_for_bbox` | `object` | 0 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `BoundingBox` | `reference` | dùng bbox targets | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `CoordinateSystem` | `choice` | CPlane/World/3Point | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Base Mode` | `choice` | Corners/Diagonal/Cube | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `XPointCount` | `number` | counts 3D Giá trị độc lập của XPointCount. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `YPointCount` | `number` | counts 3D Giá trị độc lập của YPointCount. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `ZPointCount` | `number` | counts 3D Giá trị độc lập của ZPointCount. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `XDegree` | `number` | bậc 3D Giá trị độc lập của XDegree. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `YDegree` | `number` | bậc 3D Giá trị độc lập của YDegree. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `ZDegree` | `number` | bậc 3D Giá trị độc lập của ZDegree. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TRANSFORM-041 và phiên chọn có thứ tự, kiểm tra vai trò definition_points, targets_for_bbox; chuẩn hóa tùy chọn riêng của Create Cage.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Create Cage trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TRANSFORM-041",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 0, max: None },
        Role { name: "targets_for_bbox", kind: InputKind::Object, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "BoundingBox", kind: ParameterKind::Reference, required: false },
        Parameter { name: "CoordinateSystem", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Base Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "XPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "YPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZPointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "XDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "YDegree", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZDegree", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Cage có grid counts đúng và bbox đúng hệ tọa độ; tạo cage không đổi target, capture rồi mới ảnh hưởng.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Status:** partially_implemented; validated_supported_slice.

### Sources, defaults and output

Native menu `OM9_TransformCageEditingCreateCage`, CMD `Cage` and `OpenMatrix9Gui.createCage(documentName, lo3, hi3, counts3, degrees3)` share one native service. Whole native Part/Mesh source preselection or typed object names determine the union of their world bounding boxes. Only World + BoundingBox is supported; all X/Y/Z extents must be finite and positive. Missing thickness rejects without silent padding.

The nonmodal panel explicitly displays OM9 defaults U/V/WCount=2 and U/V/WDegree=1. Counts are finite integers 2..128, degree 1..16, each count exceeds its degree, and the product is at most 1,000,000. These are current OM9 defaults, not reconstructed original defaults. OK creates one independent `OpenMatrix9Gui::CageControl` with editable ControlPoints and exact feature ID 041. Original sources remain unchanged and no captive binding is created.

Rust owns phase/order/options/finite validation and independent geometry/frame snapshots; C++ is required for FreeCAD/Qt selection, exact topology/world-box inspection and typed native API calls. Cancel/Esc, document close/switch and workbench deactivation create no document preview/output objects. Stale inputs reject before OK; the native service owns its Undo transaction and persistent parameters. CPlane/3Point, rectangle/base-height/corner construction and 1D/2D controls remain unsupported. See [Cage command contract](../../../docs/features/cage-command-native-contract.md).

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
- [ ] Automated tests use feature ID `OM9-TRANSFORM-041`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TRANSFORM-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TRANSFORM-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TRANSFORM-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TRANSFORM-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TRANSFORM-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TRANSFORM-041` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
