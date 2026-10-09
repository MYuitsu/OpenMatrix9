---
id: OM9-TOOLS-026
name: Support Ring
command: gvRPSupportRing
domain: 09-tools
module: Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-026 — Support Ring

Alias tương thích: `gvRPSupportRing`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvRPSupportRing pick anchor trên sprue rồi nhiều tips trên model, Enter sang Edit và Enter lần nữa chốt. Mirror X/Y tạo children có History; parent xanh, children xám, chỉnh parent để cập nhật mirror. Handles chỉnh tip position, End Tip Diameter, Arc Amount, Sprue Height, Top Sprue Diameter, Sprue Start X/Y Width (không dưới 0.7 mm), Sprue Mid Height. AddNewSupports nhận anchor mới hoặc SnapToSprueTip dùng tip cũ; RemoveSupports chọn để xóa; Reset bỏ mọi support. Extender bật thêm bộ handles ở base để chỉnh thêm vị trí/kích thước.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Anchor` | `point` | 1 | 1 |
| `Tips` | `point` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mirror X` | `boolean` | Tạo child mirror theo X với History. | Chưa xác định; không tự gán giá trị. |
| `Mirror Y` | `boolean` | Tạo child mirror theo Y với History. | Chưa xác định; không tự gán giá trị. |
| `End Tip Diameter` | `number` | Đường kính đầu tip. | Chưa xác định; không tự gán giá trị. |
| `Arc Amount` | `number` | Độ cong support. | Chưa xác định; không tự gán giá trị. |
| `Sprue Height` | `number` | Độ cao sprue. | Chưa xác định; không tự gán giá trị. |
| `Top Sprue Diameter` | `number` | Đường kính đầu sprue. | Chưa xác định; không tự gán giá trị. |
| `Sprue Start X Width` | `number` | Bề rộng chân X; không dưới 0.7 mm. | Chưa xác định; không tự gán giá trị. |
| `Sprue Start Y Width` | `number` | Bề rộng chân Y; không dưới 0.7 mm. | Chưa xác định; không tự gán giá trị. |
| `Sprue Mid Height` | `number` | Độ cao vị trí giữa sprue. | Chưa xác định; không tự gán giá trị. |
| `Extender` | `boolean` | Thêm bộ handles tại base. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-026` và các vai trò Anchor, Tips; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tách trunk anchor, nhánh cong và parent/children mirror trong graph; hai pha placement/edit; giới hạn 0.7 mm áp đúng width chân sprue.
2. C++/Qt: đăng ký và điều phối native command `gvRPSupportRing`; chuyển kế hoạch Support Ring sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-026`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-026",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Anchor", kind: InputKind::Point, min: 1, max: Some(1) },
        Role { name: "Tips", kind: InputKind::Point, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Mirror X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "End Tip Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Arc Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sprue Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Sprue Diameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sprue Start X Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sprue Start Y Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sprue Mid Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extender", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Move parent phải cập nhật mirrored children; width dưới 0.7 mm bị từ chối; Remove chỉ bỏ nhánh chọn, Reset bỏ toàn bộ support preview.
- OM9-TOOLS-026: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-026`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-026` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-026` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-026` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-026` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-026` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-026` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-026` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-026` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
