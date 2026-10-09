---
id: OM9-TOOLS-041
name: Boolean Builder
command: null
domain: 09-tools
module: Tools
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-041 — Boolean Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Boolean Builder dùng solids kín, kiểm tra từng input. Difference lấy Cutter khỏi Object; Keep Cutter giữ cutter hoặc xóa sau thành công. Union hợp object giao; Intersection lấy miền chung; mỗi hộp có thể nhiều objects. Do Boolean chạy, Undo Boolean và Reset. Boolean làm mất History theo workflow này. One by One xử lý cutters riêng và giữ cutters fail; Force Boolean chỉ khi One by One bật, thử dịch/scale nhẹ. Auto Scale Object 1/2 mặc định factor 1.001 khi bật; Auto Move nhận tọa độ dịch để tránh coplanar. Khuyên chỉ dùng hỗ trợ trên một phía vì dịch khỏi F4 ảnh hưởng lệnh khác. F6 cutter là shortcut Difference với prompt object, không toàn builder.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Object1` | `solid` | 1 | Không đặt trong mẫu |
| `Object2` | `solid` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `OnebyOne` | `boolean` | Thử boolean từng đối tượng riêng. | Chưa xác định; không tự gán giá trị. |
| `ForceBoolean` | `boolean` | Dùng nhánh boolean cưỡng chế theo workflow; lỗi vẫn cần rollback. | Chưa xác định; không tự gán giá trị. |
| `Operation` | `choice` | Difference, Union hoặc Intersection. | Chưa xác định; không tự gán giá trị. |
| `Keep Cutter` | `boolean` | Giữ cutters sau Difference thành công. | Chưa xác định; không tự gán giá trị. |
| `Auto Scale Object 1` | `boolean` | Cho scale nhẹ Object1 theo factor hỗ trợ. | Chưa xác định; không tự gán giá trị. |
| `Auto Scale Object 2` | `boolean` | Cho scale nhẹ Object2 theo factor hỗ trợ. | Chưa xác định; không tự gán giá trị. |
| `Auto Scale Factor` | `number` | Factor hỗ trợ khi auto-scale bật. | 1.001 |
| `Auto Move` | `reference` | Tọa độ dịch hỗ trợ tránh coplanar; chỉ dùng explicit. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-041` và các vai trò Object 1, Object 2; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Lập kế hoạch boolean với journal riêng từng cutter; không che failed booleans bằng xóa input; scale/move là biến đổi được ghi trong preview, chưa biết perturb ation Force nên cần policy rõ.
2. C++/Qt: đăng ký và điều phối native command `OM9-TOOLS-041`; chuyển kế hoạch Boolean Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-041`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-041",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Object1", kind: InputKind::Solid, min: 1, max: None },
        Role { name: "Object2", kind: InputKind::Solid, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "OnebyOne", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ForceBoolean", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Keep Cutter", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Auto Scale Object 1", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Auto Scale Object 2", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Auto Scale Factor", kind: ParameterKind::Number, required: false },
        Parameter { name: "Auto Move", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Cutters thành công được bỏ theo Keep Cutter, fail phải còn; Force bị vô hiệu khi One by One off; Union/Intersection volume đúng và Undo phục hồi inputs/history snapshot theo policy.
- OM9-TOOLS-041: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-041`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-041` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-041` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
