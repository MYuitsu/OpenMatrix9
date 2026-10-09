---
id: OM9-RENDER-020
name: Animation Builder Advanced
command: gvAnimationBuilderAdvanced
domain: 13-render
module: Render Menu
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-RENDER-020 — Animation Builder Advanced

Alias tương thích: `gvAnimationBuilderAdvanced`. Nhóm: `13-render`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvAnimationBuilderAdvanced có New Animation xóa animation data sau Yes/No, Save as Template và Pieces components. Properties Image Type Viewport/VRay Render, Size, Viewport 4 views, Total Frames; Save Name/Format/Save To; Preview Play/Scrub, Record On xuất frames với Extra Display Modes cùng count rồi Movie Maker tạo MP4. Cameras new/from active với path/target point hoặc curve, Start/End Frame, EaseIn/Out/In Out, Bounce/Elastic; Lens Sets/Tilts tối đa 3 sets/frame/size/ease. Objects Move path center hoặc Pivot Plane, Orient bottom theo curve,3 sets/frames/ease/Repeat None Loop Loop Reverse; Rotate X/Y/Z degrees/pivot tương tự; Scale disabled 100%, enabled X/Y/Z percent 0–100. Copies default Mirror hoặc Array, Hide Original, frames, pivot; Mirror X/Y/Z, X>Y, Z>X, Z>Y, XY>Z; Array Curve, Number, Slide, Spacing. Geometry Sweep1/2/Pipe/Loft frames/materials; Keep giữ qua end frame, Rebuild/Refit sweep, Sweep1 x 1 profile order, Smart Trim smaller trim plane; Tween profile morph, Unload reverse from frame. Pipe Radius/Radius 2 mm, Cap None Flat Round; Loft Normal/Loose. Save/Cancel từng component, Templates doubleclick/Pieces doubleclick và Save Components.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Objects` | `object` | 0 | Không đặt trong mẫu |
| `CameraPaths` | `object` | 0 | Không đặt trong mẫu |
| `GeometryCurves` | `curve` | 0 | Không đặt trong mẫu |
| `Destination` | `path` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Image Type` | `choice` | Viewport hoặc VRay Render; lựa chọn renderer cần capability. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Size Width` | `number` | Chiều rộng frame, pixels. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Size Height` | `number` | Chiều cao frame, pixels. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Viewport` | `choice` | Một trong bốn views để chụp animation. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Total Frames` | `number` | Tổng số frames của animation, count nguyên theo policy OM9. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Save Name` | `text` | Tên sequence/movie output. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Save Format` | `choice` | Định dạng frame có encoder adapter. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `SaveTo` | `text` | Folder output của animation. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Record` | `boolean` | On xuất frames khi chạy preview/render; Off chỉ playback preview. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Extra Display Modes` | `reference` | Danh sách display modes bổ sung cần cùng số frames; không thay Total Frames. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Camera Path` | `reference` | Curve hoặc point/location của camera component; resolver kiểm tra timeline. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Target Path` | `reference` | Curve hoặc point/location target camera theo frame. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Start Frame` | `number` | Frame bắt đầu component camera/object/geometry; scope component được chọn. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `End Frame` | `number` | Frame kết thúc component hiện hành; không tự xóa output giữ Keep. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Ease` | `choice` | Ease In, Ease Out, Ease In Out, Bounce hoặc Elastic cho component hiện hành. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Lens Sets/Enable` | `boolean` | Bật lens sets riêng với tối đa ba sets. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Lens Sets` | `reference` | Tối đa ba bộ lens gồm Start/End Frame, Start/End Size và Ease Type; unit lens size cần camera adapter xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Lens Tilts/Enable` | `boolean` | Bật các bộ lens tilt riêng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Lens Tilts` | `reference` | Tối đa ba bộ tilt gồm Start/End Frame, Start/End Size và Ease Type; không gộp state với Lens Sets. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Object/Operation` | `choice` | Selector đề xuất OM9 cho Move, Rotate, Scale hoặc Copies; các tab tương ứng có scope riêng, có thể cùng active. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Move/Enable` | `boolean` | Bật component dịch object theo path. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Move/Path` | `reference` | Curve đường dịch object center hoặc Pivot Plane. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Move/Pivot Plane` | `reference` | Plane pivot của component khi path không áp trực tiếp center. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Move/Orient` | `boolean` | Định hướng object sao cho bottom theo Move Path; tắt giữ orientation trong không gian. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Object/Sets` | `reference` | Tối đa ba bộ keyframe cho các phép biến đổi, gồm frames/ease và giá trị nhánh; schema OM9 cần type từng record. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Repeat` | `choice` | None, Loop hoặc Loop Reverse cho object component. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Rotate/Enable` | `boolean` | Bật component rotation theo timeline. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Rotate/Enable X` | `boolean` | Bật trục X trong rotation component. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Rotate/Enable Y` | `boolean` | Bật trục Y trong rotation component. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Rotate/Enable Z` | `boolean` | Bật trục Z trong rotation component. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Rotate X` | `number` | Góc quay X component, độ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Rotate Y` | `number` | Góc quay Y component, độ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Rotate Z` | `number` | Góc quay Z component, độ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Rotate/Pivot` | `reference` | Pivot plane/point của rotation component; không suy pivot từ object center khi đã chọn custom. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Scale/Enabled` | `boolean` | Bật thay đổi scale theo timeline; khi tắt biểu diễn 100% giữ kích thước. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Scale X` | `number` | Tỷ lệ theo X, %, control 0–100 khi scale bật. | 100% khi Scale không Enabled; giá trị bật cần lựa chọn thực tế. |
| `Scale Y` | `number` | Tỷ lệ theo Y, %, control 0–100 khi scale bật. | 100% khi Scale không Enabled; giá trị bật cần lựa chọn thực tế. |
| `Scale Z` | `number` | Tỷ lệ theo Z, %, control 0–100 khi scale bật. | 100% khi Scale không Enabled; giá trị bật cần lựa chọn thực tế. |
| `Copies/Mode` | `choice` | Mirror hoặc Array. | Mirror |
| `Hide Original` | `boolean` | Ẩn object gốc theo component copies; không xóa geometry gốc. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Copies/Mirror Enable` | `boolean` | Bật copies dạng mirror. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Copies/Array Enable` | `boolean` | Bật copies dạng array. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Mirror/Plane` | `choice` | X, Y, Z, X>Y, Z>X, Z>Y hoặc XY>Z; giữ từng mapping plane và kiểm chứng orientation bằng fixture. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Copies/Pivot` | `reference` | Plane pivot của Mirror/Array theo component. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Array/Curve` | `reference` | Curve dẫn array. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Array/Number` | `number` | Số copies trong array; policy topology kiểm tra count nguyên. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Array/Slide` | `boolean` | Bật đưa copies từ vị trí ngoài màn hình vào vị trí array theo hiệu ứng slide; không phải scalar khoảng dịch. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Array/Spacing` | `number` | Khoảng cách copies; units/thang phải xác minh theo curve adapter. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Geometry/Mode` | `choice` | Sweep 1, Sweep 2, Pipe hoặc Loft; Tween và Unload là phases/options riêng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Geometry/Enable` | `boolean` | Bật geometry component đã gán trong animation. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Geometry/Material` | `reference` | Material của geometry component trong render. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Keep` | `boolean` | Giữ output geometry qua End Frame thay vì bỏ khi timeline qua component. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Sweep/Curve Processing` | `choice` | Rebuild hoặc Refit; count/tolerance fields cần mở rộng theo lựa chọn sweep được kiểm chứng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Sweep 1 x 1` | `boolean` | Bật nhập từng profile đúng thứ tự để sửa automatic ordering của sweep. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Sweep/Smart Trim` | `boolean` | Trim theo plane nhỏ hơn trong sweep animation; cần solver riêng, không tự dùng generic boolean. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Tween/Enable` | `boolean` | Bật morph giữa các profiles trong frame range riêng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Tween/Profiles` | `reference` | Profiles và correspondence morph theo timeline; cần resolve order/topology. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Unload/Enable` | `boolean` | Bật trình diễn geometry đang dỡ/đảo operation. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Unload/Frame` | `number` | Frame bắt đầu đảo/dỡ geometry animation. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pipe/Radius` | `number` | Bán kính thứ nhất pipe, mm. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pipe/Radius 2` | `number` | Bán kính thứ hai pipe, mm. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Pipe/Cap` | `choice` | None, Flat hoặc Round. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Loft/Type` | `choice` | Normal hoặc Loose. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-RENDER-020` và các vai trò Objects, Camera Paths, Geometry Curves, Destination; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Timeline tracks typed theo object/camera/geometry,3 slots hoạt động độc lập; frame evaluate không mutation document, Record output job riêng; validate refs/frame intervals trước apply.
2. C++/Qt: đăng ký và điều phối native command `gvAnimationBuilderAdvanced`; chuyển kế hoạch Animation Builder Advanced sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-RENDER-020`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-020",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 0, max: None },
        Role { name: "CameraPaths", kind: InputKind::Object, min: 0, max: None },
        Role { name: "GeometryCurves", kind: InputKind::Curve, min: 0, max: None },
        Role { name: "Destination", kind: InputKind::Path, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Image Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Size Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Size Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Viewport", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Total Frames", kind: ParameterKind::Number, required: false },
        Parameter { name: "Save Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Save Format", kind: ParameterKind::Choice, required: false },
        Parameter { name: "SaveTo", kind: ParameterKind::Text, required: false },
        Parameter { name: "Record", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Extra Display Modes", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Camera Path", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Target Path", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Start Frame", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Frame", kind: ParameterKind::Number, required: false },
        Parameter { name: "Ease", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Lens Sets/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Lens Sets", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Lens Tilts/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Lens Tilts", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Object/Operation", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Move/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Move/Path", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Move/Pivot Plane", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Move/Orient", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Object/Sets", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Repeat", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rotate/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotate/Enable X", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotate/Enable Y", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotate/Enable Z", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rotate X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotate/Pivot", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Scale/Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Scale X", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale Y", kind: ParameterKind::Number, required: false },
        Parameter { name: "Scale Z", kind: ParameterKind::Number, required: false },
        Parameter { name: "Copies/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Hide Original", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copies/Mirror Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Copies/Array Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mirror/Plane", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Copies/Pivot", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Array/Curve", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Array/Number", kind: ParameterKind::Number, required: false },
        Parameter { name: "Array/Slide", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Array/Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Geometry/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Geometry/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Geometry/Material", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Keep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sweep/Curve Processing", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Sweep 1 x 1", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Sweep/Smart Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tween/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tween/Profiles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Unload/Enable", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Unload/Frame", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pipe/Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pipe/Radius 2", kind: ParameterKind::Number, required: false },
        Parameter { name: "Pipe/Cap", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Loft/Type", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Record off Play không ghi file, Extra Modes đủ same frames; Keep off geometry biến mất sau End, Unload đảo; Move Orient khác rigid heading, New No giữ timeline, Save/Load Template giữ tracks.
- OM9-RENDER-020: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-RENDER-020`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-RENDER-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-RENDER-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-RENDER-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-RENDER-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-RENDER](../../ENGINEERING_CONTRACTS.md#om9-render): kiểm chứng cho `OM9-RENDER-020` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-RENDER-020` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
