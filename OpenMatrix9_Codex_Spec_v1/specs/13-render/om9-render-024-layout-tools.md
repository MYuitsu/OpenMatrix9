---
id: OM9-RENDER-024
name: Layout Tools
command: null
domain: 13-render
module: Render Menu
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-RENDER-024 — Layout Tools

Alias tương thích: `Chưa có alias command`. Nhóm: `13-render`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Layout Tools New Layout tab blank, Print setup, Layout Properties name/printer/view/page size, Add Picture bitmap PictureFrame với two Corners/Shift Ortho; Remove Background/Frame, Save Current Layout As Template, Remove Current Layout. Template Mode default lighter wireframe và presets/custom. Views Shade Mode/VRay phân biệt snapshot với live Detail có camera riêng, double-click để edit/return. Đường render VRay tạo ảnh tĩnh, nhưng code xử lý kết hợp VRay+Detail khác nhau giữa chọn thumbnail và Change Display Modes; không coi mọi lựa chọn VRay là tự động tắt Detail. Change Display Modes có thể đổi mode tại chỗ hoặc chuyển loại item bằng tạo mới/xóa cũ; Backgrounds phía sau, Frames/Graphics picture frames transform/delete; Restore Viewports khôi phục 4 views nếu accidentally deleted.

### Chi tiết Detail, snapshot và thay page item

Form khởi tạo Detail bật. Chọn thumbnail và nút Change Display Modes là hai
handler khác nhau; điều kiện selection, đường render và cách thay item không
được gộp thành một quy tắc. Các mô tả dưới đây là hành vi đọc được từ code,
chưa phải kết quả chạy UI hoặc renderer.

**Chọn thumbnail.** Handler chỉ làm việc khi active view là page view và
ModelSpace không active. Với đúng một object được chọn, object là PageSpace
Brep và Detail tắt thì handler thay bitmap/material trên surface hiện có;
object là PageSpace Detail và Detail bật thì đổi display mode, copy projection
từ view nguồn rồi commit viewport. Selection khác đi vào nhánh thêm mới.

Detail tắt yêu cầu capture 1600×1200 pixel; mode VRay truyền cờ dùng đường
render thay cho viewport capture. Detail bật vẫn đi vào nhánh live Detail,
kể cả khi chọn VRay: handler tìm display mode theo tên và dùng Shaded nếu
không tìm được. Nó không tự tắt Detail hoặc gọi render trong nhánh này.
Live Detail không vì tên lựa chọn VRay mà trở thành viewport render V-Ray.

Ảnh mới được nhúng vào bitmap table và gắn material lên PageSpace surface.
Chỉ khi tìm thấy cả source view và object vừa tạo, nhánh thêm ảnh mới ghi
metadata camera/projection, CPlane, lens và kích thước viewport. Nhánh thay
ảnh trên Brep hiện có trả về sau khi đổi material; nó không cập nhật metadata
camera, nên metadata cũ có thể khác view vừa capture. Metadata không làm ảnh
thành live Detail: model thay đổi vẫn cần capture lại ảnh.

Nhánh thêm live Detail dùng rectangle ban đầu (0,0)–(100,80), khởi tạo
perspective rồi copy projection view nguồn và commit. Nhánh thêm ảnh đặt
surface rộng 140 đơn vị tọa độ page, chiều cao làm tròn theo tỷ lệ bitmap.
Đó là tọa độ khởi tạo của handler, không phải paper size, millimeter mặc định
hoặc numeric scale. Kích thước trang dùng PageUnitSystem của document;
adapter phải xử lý units rõ ràng. Tỷ lệ kỹ thuật chỉ áp dụng Detail parallel,
không suy tỷ số paper/model từ zoom perspective. Camera, scale, projection
lock và vị trí item trên page là các trạng thái khác nhau.

**Change Display Modes.** Nút này duyệt nhiều object được chọn và xử lý
Detail/Brep. Brep thiếu camera location metadata bị bỏ qua. Ma trận nhánh
đọc được từ code như sau:

| Item đầu vào | Detail / mode | Hành vi của handler |
|---|---|---|
| Detail | Bật, mode khác VRay | Đổi display mode và commit trên Detail hiện có. |
| Detail | Bật, VRay | Capture/render thành surface ảnh mới rồi xóa Detail cũ khi nhánh xử lý ảnh đi tiếp. |
| Detail | Tắt, mọi mode | Capture/render thành surface ảnh mới rồi xóa Detail cũ khi nhánh xử lý ảnh đi tiếp. |
| Brep có camera metadata | Bật, kể cả VRay | Tạo live Detail mới từ bounds và camera đã lưu, commit rồi xóa Brep cũ; nhánh này không có điều kiện loại VRay. |
| Brep có camera metadata | Tắt, mọi mode | Recapture/render thành surface ảnh mới từ bounds, ghi metadata cho ảnh mới rồi xóa Brep cũ khi nhánh xử lý ảnh đi tiếp. |
| Brep thiếu camera location metadata | Mọi lựa chọn | Bỏ qua item; không có đủ dữ liệu để phục hồi camera. |

Camera cho recapture được lấy từ Detail hoặc phần metadata mà handler đọc
được ở Brep: location/target/direction, lens và kích thước viewport; chuyển
Brep sang Detail còn dùng frustum nếu có. Không suy rằng mọi field metadata
đã ghi đều được phục hồi đầy đủ. Rectangle/surface thay thế lấy từ bounding
box của item cũ. Kích thước capture lấy theo viewport dùng để chụp; nhánh
VRay tăng gấp đôi mỗi chiều. Các nhánh này không dùng chung mặc định
1600×1200 pixel hay rectangle 100×80 của thao tác chọn thumbnail.

Chuyển loại bằng Change Display Modes tạo object có UUID mới rồi xóa object
cũ; không phải mọi thao tác thay nội dung đều giữ identity. Đổi mode tại chỗ
và thumbnail thay material có lifecycle khác. Các điều kiện VRay giữa hai
handler, và giữa đầu vào Detail/Brep, không đồng nhất; đây là khác biệt cần
ghi nhận, không phải policy OM9 phải sao chép nguyên trạng.

OM9 cần một policy capability nhất quán: VRay render cho output snapshot;
live Detail chỉ dùng viewport backend đã hỗ trợ. Kết hợp mode/Detail không
hợp lệ phải được UI giải thích và xử lý rõ, không âm thầm đổi sang Shaded
rồi gọi đó là render VRay. Phân biệt embedded snapshot, external image và
live Detail trong type/UI, storage, resource failure và lifecycle.

Code có đường tạo Detail trước khi kiểm đủ view nguồn và không kiểm mọi
bool commit. OM9 phải resolve source/resource trước mutation, stage toàn bộ
selection trong transaction và rollback khi Cancel/lỗi. Policy giữ/thay
identity và cập nhật camera metadata phải được công bố cho từng thao tác.
Nghiệm thu cần gồm thêm/thay/chuyển loại, selection hỗn hợp, nguồn view mất,
ảnh thiếu, VRay+Detail, perspective/parallel và Undo/Redo/save/reload.

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
| `Layout Name` | `text` | Tên layout page. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Printer` | `reference` | Identity printer theo host; chưa bảo đảm các print codecs tương thích. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `View` | `choice` | Camera/view của layout hoặc image capture; choices theo capability host. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Page Width` | `number` | Kích thước trang theo đơn vị print settings; không phải pixels. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Page Height` | `number` | Kích thước trang theo đơn vị print settings. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Template` | `reference` | Template layout tự tạo/được phép dùng; không lấy sample library làm tài nguyên đã có. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Template/Mode` | `choice` | Template hiển thị wireframe màu nhẹ/detailed, hoặc shade mode/VRay trong tab Template. | Template |
| `Shade Mode` | `choice` | Chế độ native viewport hoặc VRay static; Detail chỉ live trong các mode có hỗ trợ. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Detail` | `boolean` | Bật live viewport trong layout khi mode có capability; VRay capture là ảnh tĩnh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Picture` | `reference` | Bitmap đặt bằng PictureFrame hai góc; path/asset resolver giữ tham chiếu bền vững. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Background` | `reference` | Asset nền layout, đặt phía sau content. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Frame` | `reference` | Asset frame quanh ảnh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Graphic` | `reference` | Asset đồ họa có transform riêng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-RENDER-024` và các vai trò Document, Images; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Layout model typed live-detail/image/frame/background, render requests cho VRay, template serialization không geometry destructive; delete current layout riêng.
2. C++/Qt: đăng ký và điều phối native command `OM9-RENDER-024`; chuyển kế hoạch Layout Tools sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-RENDER-024`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-024",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Document", kind: InputKind::Document, min: 1, max: Some(1) },
        Role { name: "Images", kind: InputKind::Image, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Layout Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Printer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "View", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Page Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Page Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Template", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Template/Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Shade Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Detail", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Picture", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Background", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Frame", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Graphic", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Detail Wireframe edit camera cập nhật, VRay static không live; Change Display Mode giữ placement, image delete không xóa model; template roundtrip đúng layers/page.
- OM9-RENDER-024: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Supported slice dưới đây là hợp đồng để triển khai; trạng thái native giữ nguyên
và các ca runtime vẫn cần nghiệm thu.

### Inputs

Active page view, source model viewport, Detail toggle, display/render mode và selection.
Chọn thumbnail xét đúng một selected PageSpace Brep/Detail để thay nội dung;
Change Display Modes xử lý nhiều Detail/Brep và cần camera metadata khi đầu vào là ảnh.
Adapter OM9 phải kiểm page/space và capability của từng item trước mutation.

### Parameters and defaults

Form Detail=true. Thumbnail ảnh yêu cầu 1600×1200 px; thumbnail Detail mới bắt đầu
ở rectangle 100×80 theo tọa độ page rồi nhận projection nguồn. Ảnh mới rộng 140
đơn vị tọa độ page, chiều cao theo tỷ lệ bitmap. Change Display Modes dùng bounds
item và kích thước viewport, tăng gấp đôi từng chiều capture khi đi vào nhánh VRay.
Đây là giá trị của từng handler, không phải paper size/mm/global default. Numeric
scale chỉ parallel; page dimensions và unit conversion phải được kiểm riêng.

### Output

Live Detail hoặc bitmap/material trên PageSpace surface. Thumbnail thêm ảnh mới
chỉ ghi camera metadata khi resolve được source view và output object; thumbnail
thay ảnh hiện có không cập nhật metadata này. Change Display Modes có nhánh ghi
metadata lên ảnh thay thế. Metadata không biến snapshot thành live view, cũng không
tự chứng minh mọi thuộc tính camera được phục hồi đầy đủ.

### Preview / commit / cancel

Thumbnail thay material trên một PageSpace Brep khi Detail tắt, hoặc copy/commit
projection trên một PageSpace Detail khi Detail bật; trường hợp khác thêm mới.
Change Display Modes duyệt nhiều item: đổi mode tại chỗ hoặc tạo object mới rồi
xóa object cũ để chuyển Detail/ảnh. Kiểm đủ source/resource và kết quả commit trước
khi xóa cũ; stage theo transaction, rollback lỗi/Cancel và công bố policy identity.

### History / dependency model

Persist page/item identity, camera/projection/lock/scale và embedded resources.
Snapshot chỉ cập nhật qua recapture; metadata stale sau thay ảnh phải được xử lý
rõ. Live Detail giữ view semantics. Chuyển loại không được tự nối History vào UUID
mới chỉ vì bounds giống object cũ; adapter cần policy dependency/identity riêng.

### Error / invalid-input behavior

ModelSpace active, source view mất, renderer/image thiếu, camera metadata không đủ
hoặc commit thất bại không được báo thành công hay để partial page item. OM9 dùng
policy coherent: VRay là snapshot render, live Detail chỉ dùng viewport mode có
capability. Không tái tạo các nhánh VRay+Detail không đồng nhất hoặc fallback Shaded
mà che giấu loại output thực. Kiểm Undo/Redo/cold reload theo policy đã công bố.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-RENDER-024`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-RENDER-024` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-RENDER-024` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-RENDER-024` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-RENDER-024` hoặc ghi lý do không áp dụng.
- [ ] [OM9-RENDER](../../ENGINEERING_CONTRACTS.md#om9-render): kiểm chứng cho `OM9-RENDER-024` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-RENDER-024` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
