---
id: OM9-BUILDER-004
name: Raised Band Builder
command: gvBuilderRaisedBand
domain: 08-builders
module: Builders
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-BUILDER-004 — Raised Band Builder

Alias tương thích: `gvBuilderRaisedBand`. Nhóm: `08-builders`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvBuilderRaisedBand tạo chữ hoặc đường kín đồng phẳng nổi/chìm trên band. Chọn Shank Profile cho mặt trong/thành, Text Profile cho mặt tiếp nhận; Text Field, Font và Font Style theo font hệ thống hoặc Text Curves tự vẽ. Update Text phải trước Update Ring; không có handle viewport, chỉnh trong cửa sổ builder rồi Update. Raised Band Width, Raised Band Height từ finger rail, Text Edge Offset và Text Raised Height đều mm; height âm cho chữ chìm. Pick Surface cho chọn mặt riêng; Center Text căn trong UV Curves; Show Guide hiện UV để đặt artwork; Undo hoàn tác.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `RingRail` | `curve` | 1 | 1 |
| `Artwork` | `curve` | 0 | Không đặt trong mẫu |
| `Target` | `surface` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Shank Profile` | `reference` | Mặt cắt phía trong và thành band. | Chưa xác định; không tự gán giá trị. |
| `Text Profile` | `reference` | Profile mặt tiếp nhận text. | Chưa xác định; không tự gán giá trị. |
| `Text Field` | `text` | Nội dung chữ. | Chưa xác định; không tự gán giá trị. |
| `Font` | `text` | Tên font hệ thống. | Chưa xác định; không tự gán giá trị. |
| `Font Style` | `choice` | Style font mà font hệ thống hỗ trợ. | Chưa xác định; không tự gán giá trị. |
| `Text Curves` | `reference` | Artwork đường kín đồng phẳng thay text. | Chưa xác định; không tự gán giá trị. |
| `Raised Band Width` | `number` | Bề rộng band, mm. | Chưa xác định; không tự gán giá trị. |
| `Raised Band Height` | `number` | Độ cao từ Finger Rail, mm. | Chưa xác định; không tự gán giá trị. |
| `Text Edge Offset` | `number` | Khoảng chữ tới biên, mm. | Chưa xác định; không tự gán giá trị. |
| `Text Raised Height` | `number` | Chiều cao chữ, mm; âm cho chìm. | Chưa xác định; không tự gán giá trị. |
| `Pick Surface` | `reference` | Mặt riêng để đặt chữ. | Chưa xác định; không tự gán giá trị. |
| `Center Text` | `boolean` | Căn chữ trong UV Curves. | Chưa xác định; không tự gán giá trị. |
| `Show Guide` | `boolean` | Hiện guide UV để đặt artwork. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-BUILDER-004` và các vai trò RingRail, Artwork, Target; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tạo outline chữ, kiểm tra đường kín, đặt trong miền UV rồi flow lên mặt và extrude theo chiều cao có dấu; hai cờ dirty riêng cho chữ và band.
2. C++/Qt: đăng ký và điều phối native command `gvBuilderRaisedBand`; chuyển kế hoạch Raised Band Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-BUILDER-004`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-004",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Artwork", kind: InputKind::Curve, min: 0, max: None },
        Role { name: "Target", kind: InputKind::Surface, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Shank Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Text Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Text Field", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Text Curves", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Raised Band Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Raised Band Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Edge Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Raised Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pick Surface", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Center Text", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Show Guide", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Update Ring khi chữ chưa cập nhật phải thể hiện trạng thái cần Update Text; height âm cho khắc chìm, Show Guide không sinh hình cuối.
- OM9-BUILDER-004: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-BUILDER-004`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDER-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDER-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDER-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDER-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDER-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-BUILDER-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-BUILDER-004` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
