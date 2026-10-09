---
id: OM9-BUILDER-005
name: Award Ring Builder
command: null
domain: 08-builders
module: Builders
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-BUILDER-005 — Award Ring Builder

Alias tương thích: `Chưa có alias command`. Nhóm: `08-builders`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Award Ring Builder bổ sung mặt đầu cho nhẫn từ Signet Builder qua Set Signet. Bước 1: Outer Rail/Inner Rail, Outer X/Y Size mm, Outer Z Height tới đầu bevel, Z Offset từ đáy cap với zero tại F4, Inner X/Y Size và Inner Z Height từ mặt trên; Update rồi Next. Bước 2: Profiles sweep Base Profile giữa rails hoặc Starburst tạo peaks/valleys với Starburst Count, Inner Height, Outer Height. Bezel tùy chọn ở cả hai mode: Bezel Shape, Width, Height; Flutes chỉ khi Bezel bật với Shape, Count, Z Angle, X Angle, Z Offset, Height, Width. Bước 3: Text Profile, Text Field, Font/Font Style hoặc Artwork đường kín phẳng Looking Down. Orientation gồm Normal Bottom, Mirror Bottom, Mirror Top, Normal Top; Mirror dùng tạo dấu in. Ý nghĩa vị trí của Normal Top còn cần xác minh vì mô tả vị trí không nhất quán. Mirror Angle gộp góc bắt đầu/kết thúc thành Text Angle Mirrored; tắt thì Text Starting Angle và Text Ending Angle độc lập. Text Top/Bottom/Left/Right Offset căn theo rails/góc; Text Height âm tạo chữ chìm; Update mới tính kết quả, Undo từng bước.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Signet` | `solid` | 1 | 1 |
| `Artwork` | `curve` | 0 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Outer Rail` | `reference` | Rail ngoài của mặt đầu. | Chưa xác định; không tự gán giá trị. |
| `Inner Rail` | `reference` | Rail trong của mặt đầu. | Chưa xác định; không tự gán giá trị. |
| `Outer X Size` | `number` | Kích thước rail ngoài theo X, mm. | Chưa xác định; không tự gán giá trị. |
| `Outer Y Size` | `number` | Kích thước rail ngoài theo Y, mm. | Chưa xác định; không tự gán giá trị. |
| `Outer Z Height` | `number` | Độ cao tới đầu bevel. | Chưa xác định; không tự gán giá trị. |
| `Z Offset` | `number` | Độ cao đáy cap; zero ở F4. | Chưa xác định; không tự gán giá trị. |
| `Inner X Size` | `number` | Kích thước rail trong theo X. | Chưa xác định; không tự gán giá trị. |
| `Inner Y Size` | `number` | Kích thước rail trong theo Y. | Chưa xác định; không tự gán giá trị. |
| `Inner Z Height` | `number` | Khoảng từ mặt trên tới rail trong. | Chưa xác định; không tự gán giá trị. |
| `Base Profile` | `reference` | Mặt cắt sweep giữa rails ở mode Profiles. | Chưa xác định; không tự gán giá trị. |
| `Mode` | `choice` | Profiles hoặc Starburst. | Chưa xác định; không tự gán giá trị. |
| `Starburst Count` | `number` | Số peaks/valleys. Chính sách mẫu OpenMatrix9 yêu cầu số nguyên; chưa xác nhận quy tắc nội bộ khác. | Chưa xác định; không tự gán giá trị. |
| `Inner Height` | `number` | Độ cao đầu trong của starburst. | Chưa xác định; không tự gán giá trị. |
| `Outer Height` | `number` | Độ cao đầu ngoài của starburst. | Chưa xác định; không tự gán giá trị. |
| `Bezel` | `boolean` | Bật phần bezel tùy chọn. | Chưa xác định; không tự gán giá trị. |
| `Bezel Shape` | `choice` | Shape bezel trong catalog tự tác giả. | Chưa xác định; không tự gán giá trị. |
| `Bezel Width` | `number` | Bề rộng bezel. | Chưa xác định; không tự gán giá trị. |
| `Bezel Height` | `number` | Độ cao bezel. | Chưa xác định; không tự gán giá trị. |
| `Flutes` | `boolean` | Bật flutes; chỉ khi Bezel bật. | Chưa xác định; không tự gán giá trị. |
| `Flute Shape` | `choice` | Shape flute theo catalog. | Chưa xác định; không tự gán giá trị. |
| `Flute Count` | `number` | Số flutes. Chính sách mẫu OpenMatrix9 yêu cầu số nguyên; chưa xác nhận quy tắc nội bộ khác. | Chưa xác định; không tự gán giá trị. |
| `Flute Z Angle` | `number` | Góc flute theo Z. | Chưa xác định; không tự gán giá trị. |
| `Flute X Angle` | `number` | Góc flute theo X. | Chưa xác định; không tự gán giá trị. |
| `Flute Z Offset` | `number` | Offset flute theo Z. | Chưa xác định; không tự gán giá trị. |
| `Flute Height` | `number` | Độ cao flute. | Chưa xác định; không tự gán giá trị. |
| `Flute Width` | `number` | Bề rộng flute. | Chưa xác định; không tự gán giá trị. |
| `Text Profile` | `reference` | Mặt cắt chữ trên cap. | Chưa xác định; không tự gán giá trị. |
| `Text Field` | `text` | Nội dung chữ. | Chưa xác định; không tự gán giá trị. |
| `Font` | `text` | Font hệ thống. | Chưa xác định; không tự gán giá trị. |
| `Font Style` | `choice` | Font style hỗ trợ. | Chưa xác định; không tự gán giá trị. |
| `Artwork` | `reference` | Đường kín phẳng Looking Down. | Chưa xác định; không tự gán giá trị. |
| `Orientation` | `choice` | Normal Bottom, Mirror Bottom, Mirror Top, Normal Top; Normal Top cần xác minh vị trí. | Chưa xác định; không tự gán giá trị. |
| `Mirror Angle` | `boolean` | Liên kết góc bắt đầu/kết thúc chữ. | Chưa xác định; không tự gán giá trị. |
| `Text Angle Mirrored` | `number` | Góc chữ chung khi Mirror Angle bật. | Chưa xác định; không tự gán giá trị. |
| `Text Starting Angle` | `number` | Góc đầu khi không mirror. | Chưa xác định; không tự gán giá trị. |
| `Text Ending Angle` | `number` | Góc cuối khi không mirror. | Chưa xác định; không tự gán giá trị. |
| `Text Top Offset` | `number` | Căn phía trên so rails. | Chưa xác định; không tự gán giá trị. |
| `Text Bottom Offset` | `number` | Căn phía dưới so rails. | Chưa xác định; không tự gán giá trị. |
| `Text Left Offset` | `number` | Căn phía trái/góc. | Chưa xác định; không tự gán giá trị. |
| `Text Right Offset` | `number` | Căn phía phải/góc. | Chưa xác định; không tự gán giá trị. |
| `Text Height` | `number` | Độ cao chữ; âm cho chìm. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-BUILDER-005` và các vai trò Signet, Artwork; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Máy trạng thái ba bước và hai mode bước 2; tái dựng cap riêng, bezel/flutes điều kiện, rồi flow chữ/artwork theo miền rail và hướng đọc.
2. C++/Qt: đăng ký và điều phối native command `OM9-BUILDER-005`; chuyển kế hoạch Award Ring Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-BUILDER-005`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-005",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Signet", kind: InputKind::Solid, min: 1, max: Some(1) },
        Role { name: "Artwork", kind: InputKind::Curve, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Outer Rail", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Inner Rail", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Outer X Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Y Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Z Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner X Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Y Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Z Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Base Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Starburst Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Inner Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Outer Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bezel Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Bezel Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bezel Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flutes", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flute Shape", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Flute Count", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute Z Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute X Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute Z Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flute Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Text Field", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Artwork", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mirror Angle", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Text Angle Mirrored", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Starting Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Ending Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Top Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Bottom Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Left Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Right Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Text Height", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Flutes không hoạt động khi Bezel tắt; Starburst Count đổi số peaks; Text Height âm tạo chìm; Mirror phải đọc đúng sau in dấu; chưa nghiệm thu Normal Top khi thiếu quyết định rõ.
- OM9-BUILDER-005: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-BUILDER-005`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-BUILDER-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-BUILDER-005` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
