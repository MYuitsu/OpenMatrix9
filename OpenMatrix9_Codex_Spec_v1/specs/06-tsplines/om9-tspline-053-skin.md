---
id: OM9-TSPLINE-053
name: Skin
command: tsSkin
domain: 06-tsplines
module: T-Splines
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TSPLINE-053 — Skin

Alias tương thích: `tsSkin`. Nhóm: `06-tsplines`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

tsSkin fit control surface qua curve network đã định hình; curve đơn giản, spacing đều, ưu tiên quad regions. Curve gần nhau tính giao theo tolerance; vùng trống cần thêm curve. Face Layout chỉnh chọn face/edge trước fit, Add Creases mark edges, Mark T-Points hiển thị topology. Spans thêm điểm giữa đều mỗi edge, Shift giảm.

### Tùy chọn và tương tác

- `Use File Tolerance`: bật/tắt; mặc định On; dùng tolerance document hoặc custom
- `Tolerance Value`: sai số giao curve nếu File Tolerance off
- `Display`: phiên hiển thị giao và chỉnh tolerance rồi Enter trở lại dialog
- `MaxAutoFace`: số cạnh tối đa tự tạo face; mặc định 4; 0 tắt auto
- `MaxManualFace`: số cạnh tối đa trong lựa chọn manual
- `Face Layout`: danh sách faces/edges cần bật/tắt
- `Add Creases`: danh sách edges crease/uncrease
- `Mark T-Points`: bật/tắt; highlight topology forks
- `Spans`: mặc định 1 điểm tại giao; tăng thêm điểm đều dọc edge
- `Chord Length`: bật/tắt; cải thiện fit faces có segment dài/ngắn lệch
- `Preview`: xem mặt trước OK/Cancel

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `curve_network` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Use File Tolerance` | `boolean` | bật/tắt; mặc định On; dùng tolerance document hoặc custom | On |
| `Tolerance Value` | `choice` | sai số giao curve nếu File Tolerance off | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Display` | `reference` | phiên hiển thị giao và chỉnh tolerance rồi Enter trở lại dialog | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `MaxAutoFace` | `number` | số cạnh tối đa tự tạo face; mặc định 4; 0 tắt auto | 4 |
| `MaxManualFace` | `number` | số cạnh tối đa trong lựa chọn manual | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Face Layout` | `reference` | danh sách faces/edges cần bật/tắt | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Add Creases` | `reference` | danh sách edges crease/uncrease | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Mark T-Points` | `boolean` | bật/tắt; highlight topology forks | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Spans` | `number` | mặc định 1 điểm tại giao; tăng thêm điểm đều dọc edge | 1 |
| `Chord Length` | `boolean` | bật/tắt; cải thiện fit faces có segment dài/ngắn lệch | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Preview` | `boolean` | xem mặt trước OK/Cancel | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TSPLINE-053 và phiên chọn có thứ tự, kiểm tra vai trò curve_network; chuẩn hóa tùy chọn riêng của Skin.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Skin trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TSPLINE-053",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "curve_network", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Use File Tolerance", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tolerance Value", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Display", kind: ParameterKind::Reference, required: false },
        Parameter { name: "MaxAutoFace", kind: ParameterKind::Number, required: false },
        Parameter { name: "MaxManualFace", kind: ParameterKind::Number, required: false },
        Parameter { name: "Face Layout", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Add Creases", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Mark T-Points", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Spans", kind: ParameterKind::Number, required: false },
        Parameter { name: "Chord Length", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Network quad dựng đúng patch, MaxAutoFace 4 để N-gon hole; Spans tăng fit/density; custom tolerance bật đúng khi file off.
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
- [ ] Automated tests use feature ID `OM9-TSPLINE-053`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TSPLINE-053` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TSPLINE-053` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TSPLINE-053` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TSPLINE-053` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TSPLINE-053` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TSPLINE-053` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
