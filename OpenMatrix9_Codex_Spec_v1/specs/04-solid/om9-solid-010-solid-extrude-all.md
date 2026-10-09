---
id: OM9-SOLID-010
name: Solid Extrude All
command: ExtrudeCrv
domain: 04-solid
module: Solid Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SOLID-010 — Solid Extrude All

Alias tương thích: `ExtrudeCrv`. Nhóm: `04-solid`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Solid Extrude All nhận toàn curve hoặc toàn surface, cho phép cùng nhóm planar/nonplanar. Straight tạo thành thẳng; Tapered tạo draft; ToPoint hội tụ tại điểm; AlongCurve đi theo path. Capping vẫn cần closed planar phù hợp; không tự suy ra solid từ tên menu. SubCurve trên AlongCurve nêu đoạn path giữa điểm đầu/cuối; khác biệt điểm bắt đầu giữa các biến thể extrusion cần chốt trong test.

### Tùy chọn và tương tác

- `Direction`: hai điểm định vector
- `BothSides`: bật/tắt; tổng dài gấp đôi distance nhập
- `Solid`: Yes/No; cap khi profile kín phẳng
- `ToBoundary`: mặt dừng
- `DeleteInput`: Yes/No; xóa input sau thành công
- `SplitAtTangents`: bật/tắt; tạo face tương ứng tangent subcurve, không tác dụng khi backend UseExtrusions
- `SetBasePoint`: điểm gốc đo distance
- `Distance`: độ extrusion
- `Mode`: Straight/Tapered/ToPoint/AlongCurve
- `Draft Angle`: góc taper
- `Corners`: Sharp cạnh thẳng; Round góc arc; Smooth góc blend
- `FlipAngle`: đảo dấu draft
- `SubCurve`: Yes/No; chọn đoạn path bằng hai điểm

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `inputs` | `object` | 1 | Không đặt trong mẫu |
| `path` | `curve` | 0 | 1 |
| `tip` | `point` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Direction` | `reference` | hai điểm định vector | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `BothSides` | `boolean` | bật/tắt; tổng dài gấp đôi distance nhập | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Solid` | `boolean` | Yes/No; cap khi profile kín phẳng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `ToBoundary` | `reference` | mặt dừng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `DeleteInput` | `boolean` | Yes/No; xóa input sau thành công | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `SplitAtTangents` | `boolean` | bật/tắt; tạo face tương ứng tangent subcurve, không tác dụng khi backend UseExtrusions | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `SetBasePoint` | `reference` | điểm gốc đo distance | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Distance` | `number` | độ extrusion | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Mode` | `choice` | Straight/Tapered/ToPoint/AlongCurve | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Draft Angle` | `number` | góc taper | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Corners` | `choice` | Sharp cạnh thẳng; Round góc arc; Smooth góc blend | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `FlipAngle` | `reference` | đảo dấu draft | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `SubCurve` | `boolean` | Yes/No; chọn đoạn path bằng hai điểm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SOLID-010 và phiên chọn có thứ tự, kiểm tra vai trò inputs, path, tip; chuẩn hóa tùy chọn riêng của Solid Extrude All.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Solid Extrude All trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-SOLID-010",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "inputs", kind: InputKind::Object, min: 1, max: None },
        Role { name: "path", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "tip", kind: InputKind::Point, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ToBoundary", kind: ParameterKind::Reference, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SetBasePoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Draft Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Corners", kind: ParameterKind::Choice, required: false },
        Parameter { name: "FlipAngle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SubCurve", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Kiểm tra riêng bốn mode và input curve/surface; SubCurve fixture xác định chính xác miền path; unsupported mode disabled.
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
- [ ] Automated tests use feature ID `OM9-SOLID-010`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SOLID-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SOLID-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SOLID-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SOLID-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SOLID-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SOLID-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SOLID-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SOLID-010` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SOLID-010` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
