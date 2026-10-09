---
id: OM9-MATRIXTOOLS-004
name: Text On Curve
command: ClayEmbossTextOnCurve
domain: 16-matrix-tools
module: Matrix Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MATRIXTOOLS-004 — Text On Curve

Alias tương thích: `ClayEmbossTextOnCurve`. Nhóm: `16-matrix-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Text On Curve bố trí chữ 2D hoặc 3D theo curve. Gán curve, nhập text, chọn font hệ thống và chỉnh font/placement; dấu check hoặc Enter tạo geometry, X hủy. Font có trong máy chưa đủ để đảm bảo hình chữ thích hợp chế tạo: kiểm tra contour hở, tự giao, đảo orientation và glyph thiếu.

Alignment gồm Left/Center/Right/Justify; Justify phân phối chữ dọc khoảng đã chọn. Orientation Top/Center/Bottom chọn đường tham chiếu tương đối chiều cao glyph. Bold và Italic thay outline/style; Mirror Left/Right hoặc Top/Bottom lật từng ký tự. TextHeight là chiều cao chữ hoa, TextSpacing là khoảng giữa ký tự, Angle xoay chữ tương đối curve. Hai handle đầu/cuối xác định khoảng curve dùng để bố trí. Extrusion tạo khối nếu khác 0 theo hợp đồng; khi BothSides bật và extrusion dương, khoảng đùn được áp dụng về cả hai phía. Các kích thước hiển thị trong ví dụ không xác lập default.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `guide_curve` | `curve` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Text` | `text` | Chuỗi cần dựng, adapter kiểm tra nội dung/glyph. | Chưa xác định; không tự điền. |
| `Font` | `reference` | Font identity phải resolve trên host. | Chưa xác định; không tự điền. |
| `Alignment` | `choice` | Left/Center/Right/Justify. | Chưa xác định; không tự điền. |
| `Orientation` | `choice` | Top/Center/Bottom. | Chưa xác định; không tự điền. |
| `Bold` | `boolean` | Font style đậm. | Chưa xác định; không tự điền. |
| `Italic` | `boolean` | Font style nghiêng. | Chưa xác định; không tự điền. |
| `MirrorLeftRight` | `boolean` | Lật glyph trái/phải. | Chưa xác định; không tự điền. |
| `MirrorTopBottom` | `boolean` | Lật glyph trên/dưới. | Chưa xác định; không tự điền. |
| `TextHeight` | `number` | Chiều cao chữ hoa theo units chiều dài. | Chưa xác định; không tự điền. |
| `TextSpacing` | `number` | Khoảng giữa ký tự theo units chiều dài. | Chưa xác định; không tự điền. |
| `Extrusion` | `number` | Chiều đùn; giá trị 0 cho nhánh chữ 2D theo policy. | Chưa xác định; không tự điền. |
| `Angle` | `number` | Góc tương đối curve (độ). | Chưa xác định; không tự điền. |
| `BothSides` | `boolean` | Đùn distance về hai phía khi distance dương. | Chưa xác định; không tự điền. |
| `Start` | `reference` | Vị trí đầu trên curve, phải resolve và nằm trong miền. | Chưa xác định; không tự điền. |
| `End` | `reference` | Vị trí cuối trên curve, phải resolve và có khoảng hợp lệ. | Chưa xác định; không tự điền. |

## Hướng dẫn triển khai

1. Rust giữ chuỗi/font/style và khoảng guide; validator miền chốt units, độ cao dương và branch extrusion/both sides.
2. C++ qua adapter font tạo glyph outlines, kiểm tra wires/holes, đo advance và phân phối trên độ dài guide theo alignment. Dùng frame theo tangent để đặt glyph với orientation/mirror/angle; không chia curve theo số ký tự một cách tùy tiện.
3. Tạo faces/solids đúng nhánh; kiểm tra font fallback, hình hở và va chạm glyph. Preview ngoài document, commit geometry/metadata trong một transaction.
4. Đăng ký command/menu/CMD trong C++ native; Rust giữ request/state. Python đăng ký workbench và fixture. Giữ nhánh thiếu adapter disabled/unsupported.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-004",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "guide_curve", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Text", kind: ParameterKind::Text, required: true },
        Parameter { name: "Font", kind: ParameterKind::Reference, required: true },
        Parameter { name: "Alignment", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Bold", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Italic", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MirrorLeftRight", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "MirrorTopBottom", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "TextHeight", kind: ParameterKind::Number, required: false },
        Parameter { name: "TextSpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extrusion", kind: ParameterKind::Number, required: false },
        Parameter { name: "Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Start", kind: ParameterKind::Reference, required: false },
        Parameter { name: "End", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Left/Center/Right/Justify có vị trí đúng trên cùng guide; start/end thay miền và không đảo text âm thầm.
- Glyph có hole, Unicode thiếu font, curve ngắn và chữ tự giao cho chẩn đoán rõ.
- Extrusion0 cho 2D; BothSides với distance dương có độ đùn đúng hai phía; Cancel/Undo/Redo/save-reload giữ outlines và font metadata.

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
- [ ] Automated tests use feature ID `OM9-MATRIXTOOLS-004`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MATRIXTOOLS-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MATRIXTOOLS-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MATRIXTOOLS-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MATRIXTOOLS-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MATRIXTOOLS-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-MATRIXTOOLS-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MATRIXTOOLS-004` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
