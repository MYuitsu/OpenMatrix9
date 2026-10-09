---
id: OM9-TOOLS-027
name: Blend Supports
command: gvRPSupport
domain: 09-tools
module: Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-027 — Blend Supports

Alias tương thích: `gvRPSupport`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvRPSupport tạo blend supports: Quad Mirror qua X/Y, Shared Start giữ điểm đầu, kết hợp hai chế độ; Large Shared Start dùng đầu lớn. Mirror X/Y và Chain Supports: On lấy endpoint trước làm start mới, Off giữ start. ArmLength trong 0.25–3, BlendTension trong 0.1–3; StartDiameter/EndDiameter định cỡ đầu. BlendContinuity nhãn Continuity mặc định mô tả G 1, Position mô tả G 0, Tangency mô tả G 2; phải giữ mapping nhãn này và kiểm chứng hình, không sửa theo suy đoán.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Points` | `point` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mirror X` | `boolean` | Mirror support theo X. | Chưa xác định; không tự gán giá trị. |
| `Mirror Y` | `boolean` | Mirror support theo Y. | Chưa xác định; không tự gán giá trị. |
| `Chain Supports` | `boolean` | On dùng endpoint trước làm start mới. | Chưa xác định; không tự gán giá trị. |
| `ArmLength` | `number` | Chiều dài arm trong 0.25–3 theo mô tả. | Chưa xác định; không tự gán giá trị. |
| `BlendTension` | `number` | Độ căng blend trong 0.1–3. | Chưa xác định; không tự gán giá trị. |
| `StartDiameter` | `number` | Đường kính đầu start. | Chưa xác định; không tự gán giá trị. |
| `EndDiameter` | `number` | Đường kính đầu end. | Chưa xác định; không tự gán giá trị. |
| `BlendContinuity` | `choice` | Continuity G1, Position G0, Tangency G2 theo nhãn cần fixture. | Continuity |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-027` và các vai trò Points; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tạo hai đoạn tip và curve blend giữa chúng, sweep đường kính biến đổi; dùng enum nhãn/mức liên tục được nêu, lập mirror và chain graph.
2. C++/Qt: đăng ký và điều phối native command `gvRPSupport`; chuyển kế hoạch Blend Supports sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-027`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-027",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Mirror X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Chain Supports", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "ArmLength", kind: ParameterKind::Number, required: false },
        Parameter { name: "BlendTension", kind: ParameterKind::Number, required: false },
        Parameter { name: "StartDiameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "EndDiameter", kind: ParameterKind::Number, required: false },
        Parameter { name: "BlendContinuity", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Chain On nối liên tiếp còn Off cùng anchor; ArmLength/Tension ngoài giới hạn bị báo; kiểm chứng tiếp tuyến/curvature của từng nhãn continuity.
- OM9-TOOLS-027: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-027`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-027` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
