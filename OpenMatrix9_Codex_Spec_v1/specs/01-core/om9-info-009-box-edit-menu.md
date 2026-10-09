---
id: OM9-INFO-009
name: Box Edit Menu
command: BoxEdit
domain: 01-core
module: Info & Settings
kind: menu
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-INFO-009 — Box Edit Menu

Alias tương thích: `BoxEdit`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-INFO-009` — Box Edit Menu.**

BoxEdit hiển thị số objects selected, chỉnh Size, Scale, Position, Rotation theo X/Y/Z và Increment spinner từng nhóm. Size/Position là chiều dài document; Scale là hệ số không đơn vị, Rotation là góc độ tại UI — đây là chuẩn hóa OM9 để tránh mô tả đơn vị mơ hồ. Pivot chọn Min/Cen/Max độc lập trên mỗi trục; Cen là khởi tạo được mô tả. World/current CPlane quy định frame. Transform objects individually dùng bbox/pivot mỗi object; mặc định mô tả dùng một bbox chung. Show Bounding Box chỉ overlay. Copy Objects sửa copies; Select Copied Objects chỉ khả dụng khi Copy bật. Apply commit và tính lại bbox; Reset bỏ thay đổi chưa chấp nhận, cần giữ scope không hoàn tác mọi Apply trước tùy tiện.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `selected_objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `SizeX` | `number` | Kích thước X theo unit document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `SizeY` | `number` | Kích thước Y theo unit document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `SizeZ` | `number` | Kích thước Z theo unit document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `ScaleX` | `number` | Hệ số X không đơn vị | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `ScaleY` | `number` | Hệ số Y không đơn vị | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `ScaleZ` | `number` | Hệ số Z không đơn vị | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `PositionX` | `number` | Tọa độ X trong frame chọn | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `PositionY` | `number` | Tọa độ Y trong frame chọn | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `PositionZ` | `number` | Tọa độ Z trong frame chọn | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `RotationX` | `number` | Góc X, độ tại UI | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `RotationY` | `number` | Góc Y, độ tại UI | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `RotationZ` | `number` | Góc Z, độ tại UI | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Increment` | `number` | Bước spinner theo đơn vị nhóm control | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `PivotX` | `choice` | Min, Cen hoặc Max trên X | Cen — khởi tạo pivot được mô tả. |
| `PivotY` | `choice` | Min, Cen hoặc Max trên Y | Cen — khởi tạo pivot được mô tả. |
| `PivotZ` | `choice` | Min, Cen hoặc Max trên Z | Cen — khởi tạo pivot được mô tả. |
| `CoordinateSpace` | `choice` | World hoặc Current C-Plane | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Transform objects individually` | `boolean` | Bbox/pivot mỗi object | No — workflow mô tả bbox/pivot chung cho selection. |
| `Show Bounding Box` | `boolean` | Hiện overlay | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Copy Objects` | `boolean` | Transform copies | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Select Copied Objects` | `boolean` | Chọn copies, phụ thuộc Copy Objects | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Action` | `choice` | Apply hoặc Reset | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-INFO-009`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Tính affine transform trong frame chọn từ bbox snapshot, native adapter cập nhật placement hoặc transformed shape và kiểm tra nonuniform scaling.
3. Tích hợp `BoxEdit` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "SizeX", kind: ParameterKind::Number, required: false },
        Parameter { name: "SizeY", kind: ParameterKind::Number, required: false },
        Parameter { name: "SizeZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleX", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleY", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "PositionX", kind: ParameterKind::Number, required: false },
        Parameter { name: "PositionY", kind: ParameterKind::Number, required: false },
        Parameter { name: "PositionZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotationX", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotationY", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotationZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "Increment", kind: ParameterKind::Number, required: false },
        Parameter { name: "PivotX", kind: ParameterKind::Choice, required: false },
        Parameter { name: "PivotY", kind: ParameterKind::Choice, required: false },
        Parameter { name: "PivotZ", kind: ParameterKind::Choice, required: false },
        Parameter { name: "CoordinateSpace", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Transform objects individually", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Show Bounding Box", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copy Objects", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Select Copied Objects", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Scale=2 gấp kích thước theo frame; pivot Min giữ điểm min; Individual hai objects giữ tâm riêng; Reset preview không undo Apply trước.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

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
- [ ] Automated tests use feature ID `OM9-INFO-009`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-INFO-009` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
