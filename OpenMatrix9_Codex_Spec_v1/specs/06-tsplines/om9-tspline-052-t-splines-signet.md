---
id: OM9-TSPLINE-052
name: T-Splines Signet
command: gvtsSignet
domain: 06-tsplines
module: T-Splines
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TSPLINE-052 — T-Splines Signet

Alias tương thích: `gvtsSignet`. Nhóm: `06-tsplines`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Signet builder nhận ring rail/size, nếu thiếu +RingRail tạo rail theo size rồi Start wireframe preview. Slider/VCH/command line cùng dữ liệu. Dimensions tham chiếu ring rail và Looking Down axes. Output editable control model với options để recreate; không dùng tọa độ template private làm sample.

### Tùy chọn và tương tác

- `Ring Size`: kích thước rail khi tạo mới
- `Shank Thickness`: độ dày tại 3/9 giờ mm
- `Top Thickness`: khoảng finger rail tới top mm
- `Top Width`: kích thước top X mm
- `Top Depth`: top Y mm, alias Top Length cần chuẩn hóa
- `Top Width Corner`: offset góc top X
- `Top Depth Corner`: offset góc top Y
- `Middle Thickness`: dày phía trên band theo finger rail
- `Bottom Thickness`: dày đáy band
- `Top Shank Thickness`: dày vùng 2/10 giờ

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `ring_rail` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Ring Size` | `number` | kích thước rail khi tạo mới | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Shank Thickness` | `number` | độ dày tại 3/9 giờ mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Top Thickness` | `number` | khoảng finger rail tới top mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Top Width` | `number` | kích thước top X mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Top Depth` | `number` | top Y mm, alias Top Length cần chuẩn hóa | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Top Width Corner` | `number` | offset góc top X | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Top Depth Corner` | `number` | offset góc top Y | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Middle Thickness` | `number` | dày phía trên band theo finger rail | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Bottom Thickness` | `number` | dày đáy band | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Top Shank Thickness` | `number` | dày vùng 2/10 giờ | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TSPLINE-052 và phiên chọn có thứ tự, kiểm tra vai trò ring_rail; chuẩn hóa tùy chọn riêng của T-Splines Signet.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính T-Splines Signet trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TSPLINE-052",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "ring_rail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Ring Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shank Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Width Corner", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Depth Corner", kind: ParameterKind::Number, required: false },
        Parameter { name: "Middle Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Shank Thickness", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Đổi size giữ inner rail, đổi Top Width/Depth độc lập XY; local thickness 3/9,2/10,đáy đo đúng vị trí; Start preview chưa ghi.
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
- [ ] Automated tests use feature ID `OM9-TSPLINE-052`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TSPLINE-052` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TSPLINE-052` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TSPLINE-052` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TSPLINE-052` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TSPLINE-052` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TSPLINE-052` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
