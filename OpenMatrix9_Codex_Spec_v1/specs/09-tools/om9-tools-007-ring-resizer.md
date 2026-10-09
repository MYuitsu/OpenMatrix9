---
id: OM9-TOOLS-007
name: Ring Resizer
command: null
domain: 09-tools
module: Tools
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-007 — Ring Resizer

Alias tương thích: `Chưa có alias command`. Nhóm: `09-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Ring Resizer đổi kích thước shank và trả mesh, không có History. Nhập metal và rail cỡ gốc, loại gems; Primary radius từ circle qua girdle đá chính nhìn Through Finger; pick Left/Right angle từ F4 qua vùng đầu để bảo vệ head. Chọn nhiều size với Control. Symmetrical và Add Rings mặc định bật; tắt Symmetrical cho hai góc khác nhau. Save to File tạo từng .3 dm tên/location đã chọn; STL Options điều khiển mesh. Cần đầu vào kín/hợp lệ; size gốc từ rail.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Metal` | `solid` | 1 | Không đặt trong mẫu |
| `RingRail` | `curve` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Primary radius` | `number` | Bán kính circle qua girdle đá chính. | Chưa xác định; không tự gán giá trị. |
| `Left angle` | `number` | Biên góc bảo vệ head phía trái từ F4. | Chưa xác định; không tự gán giá trị. |
| `Right angle` | `number` | Biên góc bảo vệ head phía phải từ F4. | Chưa xác định; không tự gán giá trị. |
| `Sizes` | `reference` | Các ring sizes đích đã chọn. | Chưa xác định; không tự gán giá trị. |
| `Symmetrical` | `boolean` | Liên kết hai góc bảo vệ. | Bật |
| `Add Rings` | `boolean` | Thêm mesh outputs vào tài liệu. | Bật |
| `Save to File` | `boolean` | Xuất mỗi size qua adapter tương thích. | Chưa xác định; không tự gán giá trị. |
| `STL Options` | `reference` | Cấu hình mesh export. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-007` và các vai trò Metal, RingRail; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tách miền shank biến dạng khỏi vùng head theo radius/góc, tạo mesh độc lập mỗi size; xuất file chỉ sau validate và không ghi History giả.
2. C++/Qt: đăng ký và điều phối native command `OM9-TOOLS-007`; chuyển kế hoạch Ring Resizer sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-007`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-007",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Metal", kind: InputKind::Solid, min: 1, max: None },
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Primary radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Left angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Right angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Sizes", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Symmetrical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Rings", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Save to File", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "STL Options", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Gems không được biến dạng; Add Rings tạo bản mesh riêng, tắt không thêm; thay size giữ vùng head đã khóa và chiều trong đúng mục tiêu.
- OM9-TOOLS-007: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-TOOLS-007`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-007` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
