---
id: OM9-TSPLINE-043
name: T-Spline Sphere
command: tsSphere
domain: 06-tsplines
module: T-Splines
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TSPLINE-043 — T-Spline Sphere

Alias tương thích: `tsSphere`. Nhóm: `06-tsplines`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

tsSphere là sphere xấp xỉ bậc ba, không phải analytic sphere chính xác. Center+size, Orientation theo plane normal. 2Point dùng hai endpoint đường kính; 3Point fit qua ba điểm trên vòng, hoặc sau hai điểm nhập Radius và hướng; Tangent fit theo curve chạm, Point cho một điểm không tangent; FitPoints least-fit từ tối thiểu ba point/control point/mesh vertex. AroundCurve dùng plane vuông góc tangent path; Vertical dùng plane vuông góc C-Plane. 4Point vòng ba điểm rồi điểm thứ tư định sphere. Topology VerticalFaces/AroundFaces và symmetry tách khỏi kích thước.

### Tùy chọn và tương tác

- `Size Mode`: Radius mặc định; Diameter; Circumference; Area, Area cần chốt base-circle hay surface area theo primitive trước support
- `Radius`: bán kính base
- `Diameter`: đường kính base
- `Circumference`: chu vi circle base
- `Area`: diện tích với định nghĩa tường minh
- `Orientation`: plane normal từ center và điểm hướng
- `Circle Mode`: Center/2Point/3Point/Tangent/AroundCurve/4Point/FitPoints
- `VerticalFaces/AroundFaces`: mật độ dọc/vòng, cần count mapping rõ
- `OutputType`: Smooth mặc định cho primitive; Box dùng cage/control mesh nhanh hơn
- `AxialSymmetry`: Yes/No; Yes mở XSymmetry/YSymmetry/ZSymmetry riêng mỗi axis
- `XSymmetry`: Yes/No; ràng buộc axis X
- `YSymmetry`: Yes/No; ràng buộc axis Y
- `ZSymmetry`: Yes/No; ràng buộc axis Z
- `Symmetry`: Yes/No; mở axial/radial controls
- `Radial Symmetry`: Yes/No; mirror quanh center
- `SymmetrySegments`: số sector radial
- `FacesPerSegment`: số faces mỗi sector

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `definition_points` | `point` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Size Mode` | `choice` | Radius mặc định; Diameter; Circumference; Area, Area cần chốt base-circle hay surface area theo primitive trước support | Radius |
| `Radius` | `number` | bán kính base | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Diameter` | `number` | đường kính base | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Circumference` | `number` | chu vi circle base | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Area` | `number` | diện tích với định nghĩa tường minh | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Orientation` | `choice` | plane normal từ center và điểm hướng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Circle Mode` | `choice` | Center/2Point/3Point/Tangent/AroundCurve/4Point/FitPoints | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `VerticalFaces` | `number` | mật độ dọc/vòng, cần count mapping rõ Giá trị độc lập của VerticalFaces. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `AroundFaces` | `number` | mật độ dọc/vòng, cần count mapping rõ Giá trị độc lập của AroundFaces. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `OutputType` | `choice` | Smooth mặc định cho primitive; Box dùng cage/control mesh nhanh hơn | Smooth |
| `AxialSymmetry` | `boolean` | Yes/No; Yes mở XSymmetry/YSymmetry/ZSymmetry riêng mỗi axis | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `XSymmetry` | `boolean` | Yes/No; ràng buộc axis X | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `YSymmetry` | `boolean` | Yes/No; ràng buộc axis Y | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `ZSymmetry` | `boolean` | Yes/No; ràng buộc axis Z | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Symmetry` | `boolean` | Yes/No; mở axial/radial controls | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Radial Symmetry` | `boolean` | Yes/No; mirror quanh center | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `SymmetrySegments` | `number` | số sector radial | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `FacesPerSegment` | `number` | số faces mỗi sector | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TSPLINE-043 và phiên chọn có thứ tự, kiểm tra vai trò definition_points; chuẩn hóa tùy chọn riêng của T-Spline Sphere.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính T-Spline Sphere trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TSPLINE-043",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "definition_points", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Size Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Circumference", kind: ParameterKind::Number, required: false },
        Parameter { name: "Area", kind: ParameterKind::Number, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Circle Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "VerticalFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "AroundFaces", kind: ParameterKind::Number, required: false },
        Parameter { name: "OutputType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AxialSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "XSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "YSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ZSymmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Radial Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SymmetrySegments", kind: ParameterKind::Number, required: false },
        Parameter { name: "FacesPerSegment", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Sphere limit đo radial deviation và báo approximate; circle modes giữ constraint center/radius; radial faces mỗi segment đúng.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

> This section is intentionally separate theo hợp đồng feature-derived material. Fill it when implementing this feature. Các hành vi chưa xác định phải được ghi rõ.

### Inputs

- TODO: normalize supported object types and required selection state theo hợp đồng feature.

### Parameters and defaults

- TODO: verify exact semantics, units, ranges, and defaults theo hợp đồng feature.

### Output

- TODO: define resulting OpenMatrix9 object(s), geometry type, and ownership/history relationships.

### Preview / commit / cancel

- TODO: define interactive lifecycle only where supported by source.

### History / dependency model

- TODO: verify whether this feature records/uses History and define the OpenMatrix9 dependency graph.

### Error / invalid-input behavior

- TODO: define explicit errors and no-op/cancel cases.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-TSPLINE-043`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TSPLINE-043` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TSPLINE-043` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TSPLINE-043` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TSPLINE-043` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TSPLINE-043` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TSPLINE-043` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
