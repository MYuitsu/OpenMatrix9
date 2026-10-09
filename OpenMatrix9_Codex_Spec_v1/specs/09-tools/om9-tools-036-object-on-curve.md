---
id: OM9-TOOLS-036
name: Object on Curve
command: null
domain: 09-tools
module: Tools
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-036 — Object on Curve

Alias tương thích: `Chưa có alias command`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Object on Curve lấy asset Browser mặc định hoặc Object tự tạo quanh F4; Object Type còn Round Gem, Select a Gem từ stone tại F4, Sphere (Milgrain). Start/End Position giới hạn cung; Spacing mm, Z Offset theo normal, Roll, Start Size; End Size chỉ khi Taper Gems. Mirror đưa end về giữa, start điều khiển hai đầu. Hướng F4 tới origin, Down thẳng xuống, Object tới point/curve/surface chọn; Flip đảo. Gem Tops on đặt table sát curve, off đặt girdle; tương tự định hướng object. Taper Gems đổi size dọc tuyến, Taper Spacing làm spacing theo size. Spacing cho tối thiểu và thêm khi còn chỗ; Gem Size đặt sát; Fixed chỉ thêm khi đủ khoảng chính xác. Styles lưu/nạp, Enter chốt.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Curve` | `curve` | 1 | 1 |
| `Object` | `object` | 0 | Không đặt trong mẫu |
| `Aim` | `object` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Object Type` | `choice` | Browser, Object, Round Gem, Select a Gem hoặc Sphere. | Browser |
| `Start Position` | `number` | Đầu khoảng bố trí trên rail. | Chưa xác định; không tự gán giá trị. |
| `End Position` | `number` | Cuối khoảng bố trí trên rail. | Chưa xác định; không tự gán giá trị. |
| `Spacing` | `number` | Khe tối thiểu giữa objects, mm. | Chưa xác định; không tự gán giá trị. |
| `Z Offset` | `number` | Offset theo normal. | Chưa xác định; không tự gán giá trị. |
| `Roll` | `number` | Góc roll object. | Chưa xác định; không tự gán giá trị. |
| `Start Size` | `number` | Cỡ object ở đầu. | Chưa xác định; không tự gán giá trị. |
| `End Size` | `number` | Cỡ object ở cuối khi Taper Gems bật. | Chưa xác định; không tự gán giá trị. |
| `Mirror` | `boolean` | Đặt end ở giữa và start điều khiển hai đầu. | Chưa xác định; không tự gán giá trị. |
| `Orientation` | `choice` | F4, Down hoặc Object. | Chưa xác định; không tự gán giá trị. |
| `Flip` | `boolean` | Đảo hướng đặt. | Chưa xác định; không tự gán giá trị. |
| `Gem Tops` | `boolean` | Table sát rail khi On, girdle khi Off. | Chưa xác định; không tự gán giá trị. |
| `Taper Gems` | `boolean` | Thay size dọc rail. | Chưa xác định; không tự gán giá trị. |
| `Taper Spacing` | `boolean` | Điều chỉnh khe theo size. | Chưa xác định; không tự gán giá trị. |
| `Packing Mode` | `choice` | Spacing, Gem Size hoặc Fixed. | Chưa xác định; không tự gán giá trị. |
| `Styles` | `reference` | Preset builder. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-036` và các vai trò Curve, Object, Aim; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Dùng frame theo rail/aim và anchor table/girdle; packing theo ba mode và size taper, bảo toàn asset metadata; xử lý self-overlap như báo cáo riêng.
2. C++/Qt: đăng ký và điều phối native command `OM9-TOOLS-036`; chuyển kế hoạch Object on Curve sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-036`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-036",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curve", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Object", kind: InputKind::Object, min: 0, max: None },
        Role { name: "Aim", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Object Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Start Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Roll", kind: ParameterKind::Number, required: false },
        Parameter { name: "Start Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Gem Tops", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Gems", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Taper Spacing", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Packing Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Fixed giữ spacing chính xác, Gem Size chạm không khe; table/girdle khác offset; Object không tại F4 cần lỗi/chuẩn hóa rõ, Mirror đối xứng layout.
- OM9-TOOLS-036: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-036`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-036` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-036` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-036` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-036` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-036` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-036` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-036` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-036` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
