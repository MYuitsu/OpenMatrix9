---
id: OM9-ART-001
name: Matrix Art
command: null
domain: 07-matrix-art
module: Matrix Art
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-ART-001 — Matrix Art

Alias tương thích: `Chưa có alias command`. Nhóm: `07-matrix-art`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Dựng relief dạng 2½D, mặt sau phẳng, từ đường kín và tùy chọn ảnh tạo độ cao. Colors View biểu diễn cao/thấp bằng màu; tắt thì mỗi miền dùng màu đường. Ghosted View làm trong suốt; Preview View giảm độ phân giải xem trước nhưng không đổi độ phân giải đầu ra. Resolution có mặc định 10; Update Resolution mới áp dụng thay đổi. Capped mặc định bật, đóng lưng mesh; Drop Depth tính bằng mm phía dưới mặt phẳng Looking Down và chỉ dùng khi Capped bật. Create Mesh tạo mesh và tắt preview; có thể bật lại để chỉnh trong cùng phiên, đóng builder xóa cấu hình. Convert to Bitmap xuất ảnh độ cao.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Curves` | `curve` | 1 | Không đặt trong mẫu |
| `Bitmap` | `image` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Resolution` | `number` | Mật độ mesh đầu ra; Update Resolution mới áp dụng. | 10 |
| `Capped` | `boolean` | Đóng lưng mesh. | Bật |
| `Drop Depth` | `number` | Độ sâu lưng dưới Looking Down, mm; chỉ khi Capped bật. | Chưa xác định; không tự gán giá trị. |
| `Colors View` | `boolean` | Hiện màu theo mức cao thay vì màu đường. | Chưa xác định; không tự gán giá trị. |
| `Ghosted View` | `boolean` | Xem relief trong suốt. | Chưa xác định; không tự gán giá trị. |
| `Preview View` | `boolean` | Giảm mật độ preview mà giữ mật độ đầu ra. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-ART-001` và các vai trò Curves, Bitmap; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tách trường độ cao, mật độ preview và mật độ mesh đầu ra; tạo mặt sau và thành biên khi Capped, áp dụng Drop Depth trước kiểm tra kín.
2. C++/Qt: đăng ký và điều phối native command `OM9-ART-001`; chuyển kế hoạch Matrix Art sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-ART-001`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "Bitmap", kind: InputKind::Image, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Resolution", kind: ParameterKind::Number, required: false },
        Parameter { name: "Capped", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Drop Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Colors View", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Ghosted View", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview View", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- So sánh Resolution 10 và 20: đầu ra 20 chi tiết hơn; preview thấp vẫn tạo mesh theo cấu hình cuối. Drop Depth với Capped tắt phải báo điều kiện chưa đáp ứng.
- OM9-ART-001: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-ART-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-ART-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-ART-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-ART-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-ART-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-ART-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
