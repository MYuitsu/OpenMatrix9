---
id: OM9-SURFACE-016
name: Surface Extrude All
command: null
domain: 03-surface
module: Surface Tools
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SURFACE-016 — Surface Extrude All

Alias tương thích: `Chưa có alias command`. Nhóm: `03-surface`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Extrude cùng loại curve hoặc surface, không trộn nhóm. Mode Straight mặc định, Tapered, To Point, Along Curve. Curve phẳng có thể cap; curve không phẳng không được hứa tạo solid. Along Curve chọn gần đầu path; SubCurve dùng khoảng cách giữa hai điểm nhưng bắt đầu từ đầu path.

### Tùy chọn và tương tác

- `Mode`: Straight/Tapered/To Point/Along Curve
- `Distance`: chiều dài extrusion
- `Direction`: hai điểm định hướng
- `BothSides`: bật/tắt; hai phía mỗi phía bằng độ dài nhập nên tổng gấp đôi
- `Solid`: Yes/No; cap và join khi hình cho phép
- `DeleteInput`: Yes/No; xóa đầu vào sau thành công
- `ToBoundary`: surface dừng extrusion
- `SplitAtTangents`: bật/tắt; tách tangent segments
- `SetBasePoint`: điểm gốc đo distance
- `DraftAngle`: góc taper
- `Corners`: Sharp mặc định G0; Round G1; Smooth G2
- `FlipAngle`: đổi phía taper
- `SubCurve`: cặp vị trí trên path định chiều dài

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `profiles` | `object` | 1 | Không đặt trong mẫu |
| `path` | `curve` | 0 | 1 |
| `tip` | `point` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Straight/Tapered/To Point/Along Curve | Straight |
| `Distance` | `number` | chiều dài extrusion | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Direction` | `reference` | hai điểm định hướng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `BothSides` | `boolean` | bật/tắt; hai phía mỗi phía bằng độ dài nhập nên tổng gấp đôi | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Solid` | `boolean` | Yes/No; cap và join khi hình cho phép | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `DeleteInput` | `boolean` | Yes/No; xóa đầu vào sau thành công | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `ToBoundary` | `reference` | surface dừng extrusion | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `SplitAtTangents` | `boolean` | bật/tắt; tách tangent segments | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `SetBasePoint` | `reference` | điểm gốc đo distance | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `DraftAngle` | `number` | góc taper | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Corners` | `choice` | Sharp mặc định G0; Round G1; Smooth G2 | Sharp |
| `FlipAngle` | `reference` | đổi phía taper | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `SubCurve` | `boolean` | cặp vị trí trên path định chiều dài | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SURFACE-016 và phiên chọn có thứ tự, kiểm tra vai trò profiles, path, tip; chuẩn hóa tùy chọn riêng của Surface Extrude All.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Surface Extrude All trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-SURFACE-016",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Object, min: 1, max: None },
        Role { name: "path", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "tip", kind: InputKind::Point, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Solid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ToBoundary", kind: ParameterKind::Reference, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SetBasePoint", kind: ParameterKind::Reference, required: false },
        Parameter { name: "DraftAngle", kind: ParameterKind::Number, required: false },
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

- Along Curve SubCurve dùng độ dài giữa điểm nhưng khởi từ đầu path; BothSides gấp đôi tổng dài; nonplanar không cap giả.
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
- [ ] Automated tests use feature ID `OM9-SURFACE-016`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SURFACE-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SURFACE-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SURFACE-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SURFACE-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SURFACE-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SURFACE-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SURFACE-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-SURFACE-016` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SURFACE-016` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
