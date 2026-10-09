---
id: OM9-RENDER-022
name: Render Editor
command: gvRenderEditor
domain: 13-render
module: Render Menu
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-RENDER-022 — Render Editor

Alias tương thích: `gvRenderEditor`. Nhóm: `13-render`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvRenderEditor tự render khi open sau materials, groundplane/style đã áp; Original Render base trên layout zoom scroll. File Save JPG/BMP/PNG, Send to Layout all visible layer picture frames, Save/Send CurrentLayer, Revert về original layers, Slide show≤15 images MP4, Contact Sheet. Edit Undo, Copy all visible, Copy Selected Layer, Paste thêm new layer, Insert image. Layers Show/Hide(curves thường hidden), Lock, Design materials metal/gem không hide, Background default và Alternate Background một entry; Move Up/Down, Text font/style/size box, Add from Viewport shade/orientation active, Reset Layer, Delete Layer Yes/No. Effects per capability: Color Hue/Saturation/Illumination, Opacity, None/Grayscale/Sepia/Invert; graphics Location Left/Top/Width/Height/Rotation, Flip/Reverse/FlipReverse, SkewX/Y, TileX/Y; gem/metal không move/tile. Blur/Sharpen/None, Bevel None In Out, Shadow graphics Only On/Color/Opacity/OffsetX/Y/Blur, minimize background/frame khi áp. Wire Composite Pick Mode+render, Angle vertical default, Blend; Advanced 2 shade modes, Position/Amount/Opacity và Combine Normal Blend Hard Light Soft Light Lighten Darken Luminize Modulate Multiply Overlay. Backgrounds/Graphics/Frames/Themes library/custom image, stock alpha PNG, phải dùng asset tự tạo; layer st a ts/templates chưa đủ cần xác minh.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Document` | `document` | 1 | 1 |
| `Images` | `image` | 0 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Save Format` | `choice` | JPG, BMP hoặc PNG khi encoder hỗ trợ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Current Layer` | `reference` | Layer đang được chọn để edit/save/send; không là geometry selection. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Layer/Visible` | `boolean` | Hiện hoặc ẩn layer; design material layers metal/gem không hỗ trợ hide theo contract. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Layer/Lock` | `boolean` | Khóa layer đối với thao tác chỉnh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Text` | `text` | Nội dung text layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Font` | `reference` | Font hệ thống resolve theo identity; thiếu glyph phải báo. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Font Style` | `choice` | Style font trong adapter host. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Font Size` | `number` | Kích thước chữ trong layout; unit của Size box cần xác minh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Add from Viewport/Shade Mode` | `choice` | Chế độ shade của active viewport khi thêm layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Add from Viewport/Orientation` | `choice` | Hướng view được chụp vào layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Color/Hue` | `number` | Hiệu chỉnh hue của layer có effect capability. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Color/Saturation` | `number` | Hiệu chỉnh saturation. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Color/Illumination` | `number` | Hiệu chỉnh độ sáng layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Opacity` | `number` | Độ đục layer; thang 0/100 chưa xác lập trong editor này, không mượn thang watermark Movie Maker. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Color/Mode` | `choice` | None, Grayscale, Sepia hoặc Invert. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Location/Left` | `number` | Vị trí ngang graphics layer trong layout; unit cần xác minh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Location/Top` | `number` | Vị trí dọc graphics layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Location/Width` | `number` | Bề rộng graphics layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Location/Height` | `number` | Chiều cao graphics layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Location/Rotation` | `number` | Góc xoay graphics layer, độ; gem/metal layer không di chuyển/tile theo workflow. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Flip` | `choice` | Flip, Reverse hoặc Flip Reverse cho graphics layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Skew X` | `number` | Độ skew theo X; unit/thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Skew Y` | `number` | Độ skew theo Y; unit/thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Tile X` | `number` | Số/tỷ lệ lặp graphics layer theo X; không cho gem/metal. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Tile Y` | `number` | Số/tỷ lệ lặp graphics layer theo Y; không cho gem/metal. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Filter` | `choice` | Blur, Sharpen hoặc None cho layer có capability. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Bevel` | `choice` | None, In hoặc Out cho layer có capability. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Shadow/On` | `boolean` | Bật shadow riêng cho graphics layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Shadow/Color` | `text` | Màu shadow của graphics layer. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Shadow/Opacity` | `number` | Độ đục shadow; thang cần xác minh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Shadow/Offset X` | `number` | Dịch shadow theo X trong layout; units cần host xác định. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Shadow/Offset Y` | `number` | Dịch shadow theo Y trong layout. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Shadow/Blur` | `number` | Mức blur shadow; thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Wire Composite/Mode` | `reference` | Shade mode và render pair dùng composite; cần đồng bộ camera/extent. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Wire Composite/Angle` | `number` | Góc overlay wire theo control; nhánh vertical là khởi tạo mô tả nhưng numeric default chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Wire Composite/Blend` | `number` | Mức hòa wire/render; thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Advanced/Modes` | `reference` | Hai shade modes dùng composite nâng cao; registry kiểm tra capability. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Advanced/Position` | `number` | Vị trí composite theo control; representation/units chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Advanced/Amount` | `number` | Mức tác động composite; thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Advanced/Opacity` | `number` | Độ đục composite; thang chưa xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Advanced/Combine` | `choice` | Normal, Blend, Hard Light, Soft Light, Lighten, Darken, Luminize, Modulate, Multiply hoặc Overlay. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Background` | `reference` | Asset background/custom image có license phù hợp; một Alternate Background entry theo workflow. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Graphic` | `reference` | Asset graphics overlay. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Frame` | `reference` | Asset frame overlay. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Theme` | `reference` | Cấu hình phối hợp background/graphics/frame; không lấy library thương mại làm asset đã có. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-RENDER-022` và các vai trò Document, Images; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Layer com pos it or với immutable Original Render, typed capabilities per layer, stack effects/Undo, snapshot cameras; Send To Layout transaction riêng và Save file output riêng.
2. C++/Qt: đăng ký và điều phối native command `gvRenderEditor`; chuyển kế hoạch Render Editor sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-RENDER-022`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-022",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "Images", kind: InputKind::Image, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Save Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Current Layer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Layer/Visible", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Layer/Lock", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Text", kind: ParameterKind::Text, required: false },
        Parameter { name: "Font", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Font Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Font Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Add from Viewport/Shade Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Add from Viewport/Orientation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Color/Hue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color/Saturation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color/Illumination", kind: ParameterKind::Number, required: false },
        Parameter { name: "Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Color/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Location/Left", kind: ParameterKind::Number, required: false },
        Parameter { name: "Location/Top", kind: ParameterKind::Number, required: false },
        Parameter { name: "Location/Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Location/Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Location/Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Skew X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Skew Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tile X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tile Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Filter", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Bevel", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Shadow/On", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Shadow/Color", kind: ParameterKind::Text, required: false },
        Parameter { name: "Shadow/Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shadow/Offset X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shadow/Offset Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Shadow/Blur", kind: ParameterKind::Number, required: false },
        Parameter { name: "Wire Composite/Mode", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Wire Composite/Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Wire Composite/Blend", kind: ParameterKind::Number, required: false },
        Parameter { name: "Advanced/Modes", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Advanced/Position", kind: ParameterKind::Number, required: false },
        Parameter { name: "Advanced/Amount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Advanced/Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Advanced/Combine", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Background", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Graphic", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Frame", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Theme", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Gem layer không move/tile, Design không hide, Revert loại added layers khỏi visible và restore original; Copy/Paste giữ visible layers, Wire Composite pixel math có fixtures, slide show 16 báo limit.
- OM9-RENDER-022: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-RENDER-022`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-RENDER-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-RENDER-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-RENDER-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-RENDER-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-RENDER](../../ENGINEERING_CONTRACTS.md#om9-render): kiểm chứng cho `OM9-RENDER-022` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-RENDER-022` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
