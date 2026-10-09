---
id: OM9-TRANSFORM-039
name: MoveUVN
command: MoveUVN
domain: 05-transform
module: Transform Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TRANSFORM-039 — MoveUVN

Alias tương thích: `MoveUVN`. Nhóm: `05-transform`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

MoveUVN dịch surface control points được chọn bằng slider U/V/N; Shift chọn nhiều. Scale chỉnh độ nhạy mm. Có smoothing làm đều lưới và symmetry hai phương cho sửa đồng thời hai phía.

### Tùy chọn và tương tác

- `U/V/N`: mức dịch theo phương surface
- `Scale`: hệ số slider mm
- `UV Move Mode`: Along Tangent plane xấp xỉ tangent; Along Control Polygon theo chord; Along Extensions theo đường kéo dài chord
- `U-Symmetry/V-Symmetry`: bật/tắt; tác động đối xứng points đã chọn
- `Smoothing`: làm đều lưới trước/sau phối hợp

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `control_points` | `selection` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `U` | `number` | mức dịch theo phương surface Giá trị độc lập của U. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `V` | `number` | mức dịch theo phương surface Giá trị độc lập của V. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `N` | `number` | mức dịch theo phương surface Giá trị độc lập của N. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Scale` | `number` | hệ số slider mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `UV Move Mode` | `choice` | Along Tangent plane xấp xỉ tangent; Along Control Polygon theo chord; Along Extensions theo đường kéo dài chord | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `U-Symmetry` | `boolean` | bật/tắt; tác động đối xứng points đã chọn Giá trị độc lập của U-Symmetry. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `V-Symmetry` | `boolean` | bật/tắt; tác động đối xứng points đã chọn Giá trị độc lập của V-Symmetry. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Smoothing` | `reference` | làm đều lưới trước/sau phối hợp | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TRANSFORM-039 và phiên chọn có thứ tự, kiểm tra vai trò control_points; chuẩn hóa tùy chọn riêng của MoveUVN.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính MoveUVN trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TRANSFORM-039",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "control_points", kind: InputKind::Selection, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "U", kind: ParameterKind::Number, required: false },
        Parameter { name: "V", kind: ParameterKind::Number, required: false },
        Parameter { name: "N", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale", kind: ParameterKind::Number, required: false },
        Parameter { name: "UV Move Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "U-Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "V-Symmetry", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Smoothing", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- N slider đi theo normal, U/V modes tạo vector khác; symmetry cần partner chọn và giữ tương quan.
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
- [ ] Automated tests use feature ID `OM9-TRANSFORM-039`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TRANSFORM-039` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TRANSFORM-039` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TRANSFORM-039` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TRANSFORM-039` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TRANSFORM-039` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TRANSFORM-039` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
