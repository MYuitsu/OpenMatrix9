---
id: OM9-SURFACE-018
name: Revolve
command: Revolve
domain: 03-surface
module: Surface Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SURFACE-018 — Revolve

Alias tương thích: `Revolve`. Nhóm: `03-surface`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Quay profile quanh axis hai điểm; Enter ở điểm axis cuối dùng C-Plane Z. Chọn start angle và góc quay theo độ trong 0–360. Tạo surface hoặc polysurface.

### Tùy chọn và tương tác

- `StartAngle`: góc bắt đầu độ
- `RevolutionAngle`: góc quay độ
- `DeleteInput`: bật/tắt; xóa profile sau thành công
- `Deformable`: Yes/No; No revolve rational chính xác knot quadrant, Yes rebuild phương vòng bậc ba non-rational
- `PointCount`: số điểm phương vòng khi Deformable Yes
- `FullCircle`: quay 360 và nhớ 360 cho lần chạy sau
- `AskForStartAngle`: Yes/No; No bắt đầu tại profile gốc, Yes hỏi góc
- `SplitAtTangents`: bật/tắt; tách surface tangent

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `profiles` | `curve` | 1 | Không đặt trong mẫu |
| `axis_points` | `point` | 2 | 2 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `StartAngle` | `number` | góc bắt đầu độ | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `RevolutionAngle` | `number` | góc quay độ | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `DeleteInput` | `boolean` | bật/tắt; xóa profile sau thành công | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Deformable` | `boolean` | Yes/No; No revolve rational chính xác knot quadrant, Yes rebuild phương vòng bậc ba non-rational | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `PointCount` | `number` | số điểm phương vòng khi Deformable Yes | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `FullCircle` | `reference` | quay 360 và nhớ 360 cho lần chạy sau | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `AskForStartAngle` | `boolean` | Yes/No; No bắt đầu tại profile gốc, Yes hỏi góc | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `SplitAtTangents` | `boolean` | bật/tắt; tách surface tangent | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SURFACE-018 và phiên chọn có thứ tự, kiểm tra vai trò profiles, axis_points; chuẩn hóa tùy chọn riêng của Revolve.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Revolve trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-SURFACE-018",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "profiles", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "axis_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "StartAngle", kind: ParameterKind::Number, required: false },
        Parameter { name: "RevolutionAngle", kind: ParameterKind::Number, required: false },
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Deformable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "FullCircle", kind: ParameterKind::Reference, required: false },
        Parameter { name: "AskForStartAngle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- FullCircle giữ góc 360 lần kế; Deformable khác cấu trúc rational nhưng sai lệch được đo; axis suy biến không tạo output.
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
- [ ] Automated tests use feature ID `OM9-SURFACE-018`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SURFACE-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SURFACE-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SURFACE-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SURFACE-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SURFACE-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SURFACE-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SURFACE-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-SURFACE-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SURFACE-018` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
