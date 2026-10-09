---
id: OM9-TRANSFORM-032
name: Stretch
command: Stretch
domain: 05-transform
module: Transform Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TRANSFORM-032 — Stretch

Alias tương thích: `Stretch`. Nhóm: `05-transform`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Stretch scale một vùng object một chiều dọc axis, rồi factor/target point hoặc overall Length. Chọn axis bằng endpoint trực tiếp; BothSides kéo hai phía gấp đôi độ dài. Normal lấy normal surface; Angled dùng base line và Pivot Angle; Vertical theo C-Plane; FourPoint dùng cặp hướng và cặp chiều dài; Bisector theo angle ba điểm; Perpendicular/Tangent theo curve. Point cho điểm gần curve, PointOnCurve khóa điểm pick, 2Curves lấy ràng buộc giữa hai curve; trường hợp nhãn Tangent/2Curves có nghĩa không nhất quán cần TODO_EVIDENCE. Extension dùng line kéo từ đầu curve.

### Tùy chọn và tương tác

- `Copy`: Yes/No; Yes giữ bản gốc và tạo bản mới, cursor dấu cộng
- `Rigid`: Yes/No; Yes giữ shape từng object và chỉ biến đổi frame, No biến dạng object
- `Stretch Factor`: tỷ lệ vùng stretch
- `Length`: chiều dài axis mới
- `Axis Mode`: Normal/Angled/Vertical/FourPoint/Bisector/Perpendicular/Tangent/Extension
- `Pivot Angle`: góc axis trong Angled

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `objects` | `object` | 1 | Không đặt trong mẫu |
| `axis_points` | `point` | 2 | 2 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Copy` | `boolean` | Yes/No; Yes giữ bản gốc và tạo bản mới, cursor dấu cộng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Rigid` | `boolean` | Yes/No; Yes giữ shape từng object và chỉ biến đổi frame, No biến dạng object | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Stretch Factor` | `number` | tỷ lệ vùng stretch | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Length` | `number` | chiều dài axis mới | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Axis Mode` | `choice` | Normal/Angled/Vertical/FourPoint/Bisector/Perpendicular/Tangent/Extension | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Pivot Angle` | `number` | góc axis trong Angled | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TRANSFORM-032 và phiên chọn có thứ tự, kiểm tra vai trò objects, axis_points; chuẩn hóa tùy chọn riêng của Stretch.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Stretch trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TRANSFORM-032",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "axis_points", kind: InputKind::Point, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "Copy", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rigid", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Stretch Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Axis Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Pivot Angle", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Vùng axis kéo dài theo factor, vùng ngoài theo policy nối đã chốt; rigid nhóm giữ từng shape; axis mode frame đúng.
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
- [ ] Automated tests use feature ID `OM9-TRANSFORM-032`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TRANSFORM-032` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TRANSFORM-032` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TRANSFORM-032` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TRANSFORM-032` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TRANSFORM-032` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TRANSFORM-032` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
