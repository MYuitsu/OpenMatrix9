---
id: OM9-SOLID-008
name: Text
command: TextObject
domain: 04-solid
module: Solid Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SOLID-008 — Text

Alias tương thích: `TextObject`. Nhóm: `04-solid`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

TextObject dùng font TrueType cài trên máy tạo outline curve, planar face hoặc solid theo nội dung Unicode. Đặt output tại điểm viewport sau dialog. Glyph có self-intersection hoặc font thiếu không được hứa tạo solid hợp lệ; có thể cần sửa outline rồi extrude.

### Tùy chọn và tương tác

- `Text to Create`: chuỗi nhập, hỗ trợ cut/copy/paste
- `Font Name`: tên font đã cài
- `Bold/Italic`: bật/tắt style
- `Create`: Curves/Surfaces/Solids
- `Group Objects`: bật/tắt; group output
- `Allow single-stroke fonts`: bật/tắt; cho engraving stroke mở, tắt dùng outline kín
- `Height`: cao model units, thường mm theo Y
- `Solid Thickness`: extrusion Z cho solid
- `Lower case as small caps`: bật/tắt; chữ thường thành small caps
- `Small Caps Size`: phần trăm kích thước chữ thường
- `Add spacing`: khoảng cách thêm giữa ký tự

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `placement` | `point` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Text to Create` | `text` | chuỗi nhập, hỗ trợ cut/copy/paste | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Font Name` | `text` | tên font đã cài | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Bold` | `boolean` | bật/tắt style Giá trị độc lập của Bold. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Italic` | `boolean` | bật/tắt style Giá trị độc lập của Italic. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Create` | `choice` | Curves/Surfaces/Solids | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Group Objects` | `boolean` | bật/tắt; group output | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Allow single-stroke fonts` | `boolean` | bật/tắt; cho engraving stroke mở, tắt dùng outline kín | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Height` | `number` | cao model units, thường mm theo Y | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Solid Thickness` | `number` | extrusion Z cho solid | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Lower case as small caps` | `boolean` | bật/tắt; chữ thường thành small caps | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Small Caps Size` | `number` | phần trăm kích thước chữ thường | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Add spacing` | `number` | khoảng cách thêm giữa ký tự | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SOLID-008 và phiên chọn có thứ tự, kiểm tra vai trò placement; chuẩn hóa tùy chọn riêng của Text.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Text trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
4. Preview nếu cần dùng dữ liệu tạm; xác nhận ghi kết quả và dependency trong một transaction. Hủy/lỗi không tạo output dở dang; Undo/Redo và save/reload giữ contract.
5. Native C++ đăng ký command/menu và dispatch sang Rust/native geometry. Python đăng ký workbench và hỗ trợ fixtures/macro kiểm chứng. Capability chưa có hoặc chưa kiểm chứng phải giữ disabled với lý do cụ thể.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "placement", kind: InputKind::Point, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text to Create", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Bold", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Italic", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Create", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Group Objects", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Allow single-stroke fonts", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Solid Thickness", kind: ParameterKind::Number, required: false },
        Parameter { name: "Lower case as small caps", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Small Caps Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Add spacing", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Chuỗi có dấu và glyph có lỗ: outline/face/solid giữ lỗ, Height và Z thickness đúng; missing font báo rõ.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

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
- [ ] Automated tests use feature ID `OM9-SOLID-008`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SOLID-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SOLID-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SOLID-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SOLID-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SOLID-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SOLID-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SOLID-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SOLID-008` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
