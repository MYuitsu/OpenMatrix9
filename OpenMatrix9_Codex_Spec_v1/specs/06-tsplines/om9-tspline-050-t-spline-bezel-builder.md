---
id: OM9-TSPLINE-050
name: T-Spline Bezel Builder
command: gvtsBezelBuilder
domain: 06-tsplines
module: T-Splines
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TSPLINE-050 — T-Spline Bezel Builder

Alias tương thích: `gvtsBezelBuilder`. Nhóm: `06-tsplines`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Builder bezel control model quanh gem hiện hữu. Gem từ input box hoặc preselected F6; preview động, icon/slider/value và VCH cùng một parameter. Yellow vertical chỉnh height, yellow horizontal top/base thickness, light blue dome, blue vertical seat height, blue diagonal seat length, white rotation các angle, yellow cube placement. Persist gem reference và options để edit lại; chỉ geometry backend riêng được triển khai độc lập.

### Tùy chọn và tương tác

- `Crease Top`: bật/tắt; mặc định Off; top phẳng thay dome
- `Crease Seat`: bật/tắt; mặc định On; seat phẳng
- `Crease Bottom`: bật/tắt; mặc định Off; base phẳng
- `Faces`: Few/Many; density, không mặc định số face chưa chứng minh
- `Seat Height`: vị trí/cao độ seat theo mm, phải phân biệt nhãn Seat Depth
- `Seat Length`: chiều dài seat xuống pavilion mm
- `Seat Angle`: góc seat độ
- `Bezel Angle`: góc thành bezel độ
- `Placement`: vị trí dọc Z mm
- `Bezel Height`: chiều cao top tới base mm
- `Top Thickness`: thickness tại girdle mm
- `Base Thickness`: thickness base mm
- `Dome Height`: phần tròn nhô top mm

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `gem` | `gem` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Crease Top` | `boolean` | bật/tắt; mặc định Off; top phẳng thay dome | Off |
| `Crease Seat` | `boolean` | bật/tắt; mặc định On; seat phẳng | On |
| `Crease Bottom` | `boolean` | bật/tắt; mặc định Off; base phẳng | Off |
| `Faces` | `choice` | Few/Many; density, không mặc định số face chưa chứng minh | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Seat Height` | `number` | vị trí/cao độ seat theo mm, phải phân biệt nhãn Seat Depth | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Seat Length` | `number` | chiều dài seat xuống pavilion mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Seat Angle` | `number` | góc seat độ | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Bezel Angle` | `number` | góc thành bezel độ | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Placement` | `number` | vị trí dọc Z mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Bezel Height` | `number` | chiều cao top tới base mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Top Thickness` | `number` | thickness tại girdle mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Base Thickness` | `number` | thickness base mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Dome Height` | `number` | phần tròn nhô top mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TSPLINE-050 và phiên chọn có thứ tự, kiểm tra vai trò gem; chuẩn hóa tùy chọn riêng của T-Spline Bezel Builder.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính T-Spline Bezel Builder trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TSPLINE-050",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "gem", kind: InputKind::Gem, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Crease Top", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Seat", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Crease Bottom", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Faces", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Seat Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Seat Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Placement", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Dome Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Round/oval gem có bezel quanh girdle đúng transform, đổi seat/angle chỉnh riêng tương ứng; crease defaults và density Few/Many không mất khi save/edit.
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
- [ ] Automated tests use feature ID `OM9-TSPLINE-050`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TSPLINE-050` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TSPLINE-050` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TSPLINE-050` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TSPLINE-050` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TSPLINE-050` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TSPLINE-050` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
