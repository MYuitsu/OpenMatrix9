---
id: OM9-TSPLINE-056
name: Match
command: tsMatch
domain: 06-tsplines
module: T-Splines
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TSPLINE-056 — Match

Alias tương thích: `tsMatch`. Nhóm: `06-tsplines`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

tsMatch từ border loop T-Spline tới edges/curve T-Spline hoặc NURBS, hai surface vẫn là objects riêng. Alignment connectors cặp vị trí có thể Add/Delete; Flip direction bỏ twist. G1 dùng hàng điểm kế, G2 thêm hàng thứ ba; refinement thêm points theo tolerance.

### Tùy chọn và tương tác

- `Alignment Add/Delete`: cặp tương ứng hoặc connector cần bỏ
- `AlignmentType`: ArcLength giảm khoảng cách vật lý; Parametric theo vị trí tham số tương tự
- `FlipAlignment Direction`: đảo hướng mapping
- `Continuity`: G0 vị trí; G1 tangent; G2 curvature
- `Tangent Scale`: hệ số ảnh hưởng tangent, mặc định 1; chỉ G1/G2
- `UseFalloff`: Yes/No; No ảnh hưởng tối thiểu, Yes tới khoảng đã nhập
- `Falloff Distance`: miền ảnh hưởng
- `Use Refinement`: Yes/No; thêm points để đạt tolerance
- `Preview`: bật/tắt; mặt mới

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `source_edges` | `selection` | 1 | Không đặt trong mẫu |
| `target` | `object` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Alignment Add/Delete` | `reference` | cặp tương ứng hoặc connector cần bỏ | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `AlignmentType` | `choice` | ArcLength giảm khoảng cách vật lý; Parametric theo vị trí tham số tương tự | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `FlipAlignment Direction` | `reference` | đảo hướng mapping | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Continuity` | `choice` | G0 vị trí; G1 tangent; G2 curvature | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Tangent Scale` | `number` | hệ số ảnh hưởng tangent, mặc định 1; chỉ G1/G2 | 1 |
| `UseFalloff` | `boolean` | Yes/No; No ảnh hưởng tối thiểu, Yes tới khoảng đã nhập | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Falloff Distance` | `number` | miền ảnh hưởng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Use Refinement` | `boolean` | Yes/No; thêm points để đạt tolerance | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Preview` | `boolean` | bật/tắt; mặt mới | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TSPLINE-056 và phiên chọn có thứ tự, kiểm tra vai trò source_edges, target; chuẩn hóa tùy chọn riêng của Match.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Match trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TSPLINE-056",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "source_edges", kind: InputKind::Selection, min: 1, max: None },
        Role { name: "target", kind: InputKind::Object, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Alignment Add/Delete", kind: ParameterKind::Reference, required: false },
        Parameter { name: "AlignmentType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "FlipAlignment Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Tangent Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "UseFalloff", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Falloff Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Use Refinement", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- G0/G1/G2 đo continuity riêng; connector đảo direction bỏ twist; hai objects không tự join; falloff ngoài miền giữ geometry.
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
- [ ] Automated tests use feature ID `OM9-TSPLINE-056`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TSPLINE-056` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TSPLINE-056` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TSPLINE-056` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TSPLINE-056` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TSPLINE-056` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TSPLINE-056` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
