---
id: OM9-TOOLS-002
name: Profile Placer
command: gvProfilePlacer
domain: 09-tools
module: Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-002 — Profile Placer

Alias tương thích: `gvProfilePlacer`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvProfilePlacer đặt profile kín làm section sweep trên rail. Profile có handle là preview, Enter chốt; mở lệnh khác trước Enter có thể hủy. Edge Profile hover cập nhật; Width/Height, Tilt/Twist/Roll, X/Y/Z Offset và Position On Curve (nhập vị trí độ theo workflow rail) cùng điều khiển handle. Add tạo bản trùng vị trí để chuyển; Add Mirror nối qua midpoint rail, đổi vị trí/shape đồng bộ; Mirror Rotate làm X Offset/Roll/Twist/Tilt ngược phía mirror; Break Mirror gỡ liên kết. Select Rail có Keep Location Yes giữ đặt đúng, No định hướng lại. Orient Type mặc định Y Axis, Origin hướng seam về F4, Flat vuông góc rail; Second Rail điều khiển height. Edit sửa Sweep Edit Points vàng dưới/tím trên, từng bộ mirror và chỉ Sweep Multi dùng điểm này. Profile Editor cần một curve kín đặt baseline: Left/Right, Mirror/Blend, Auto Scale tối đa 10 × 10 mm, Set on Baseline, Set Seam Point ở giữa đáy, Paste Object from Clipboard, Name, Save Profile, Return Profile; Cancel về profile ban đầu.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Rail` | `curve` | 1 | 1 |
| `Profile` | `curve` | 0 | 1 |
| `SecondRail` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Edge Profile` | `reference` | Profile kín từ thư viện/editor. | Chưa xác định; không tự gán giá trị. |
| `Width` | `number` | Bề rộng section. | Chưa xác định; không tự gán giá trị. |
| `Height` | `number` | Độ cao section. | Chưa xác định; không tự gán giá trị. |
| `Tilt` | `number` | Góc tilt profile. | Chưa xác định; không tự gán giá trị. |
| `Twist` | `number` | Góc twist profile. | Chưa xác định; không tự gán giá trị. |
| `Roll` | `number` | Góc roll profile. | Chưa xác định; không tự gán giá trị. |
| `X Offset` | `number` | Dịch profile theo X local. | Chưa xác định; không tự gán giá trị. |
| `Y Offset` | `number` | Dịch profile theo Y local. | Chưa xác định; không tự gán giá trị. |
| `Z Offset` | `number` | Dịch profile theo Z local. | Chưa xác định; không tự gán giá trị. |
| `Position On Curve` | `number` | Vị trí độ theo workflow rail; không tự coi mm. | Chưa xác định; không tự gán giá trị. |
| `Mirror Rotate` | `boolean` | Đảo dấu offsets/angles được nêu ở phía mirror. | Chưa xác định; không tự gán giá trị. |
| `Keep Location` | `boolean` | Yes giữ placement khi đổi rail, No định hướng lại. | Chưa xác định; không tự gán giá trị. |
| `Orient Type` | `choice` | Y Axis, Origin hoặc Flat. | Y Axis |
| `Second Rail` | `reference` | Rail thứ hai điều khiển height. | Chưa xác định; không tự gán giá trị. |
| `Name` | `text` | Tên profile trong editor. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-002` và các vai trò Rail, Profile, Second Rail; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tạo frame rail, profile với seam/origin và linkage mirror; editor có snapshot riêng và dữ liệu sweep points; không áp dữ liệu này vào các sweep khác.
2. C++/Qt: đăng ký và điều phối native command `gvProfilePlacer`; chuyển kế hoạch Profile Placer sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-002`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Rail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
        Role { name: "SecondRail", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tilt", kind: ParameterKind::Number, required: false },
        Parameter { name: "Twist", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "X Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Y Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Position On Curve", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mirror Rotate", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Keep Location", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Orient Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Second Rail", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Add không tự dịch bản sao; Enter giữ profile còn Cancel bỏ preview; Second Rail đổi height; Set Seam Point đúng đưa handle về hướng mong muốn; Auto Scale giữ kích thước tối đa 10 mm.
- OM9-TOOLS-002: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-002`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-002` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
