---
id: OM9-LAYER-001
name: Layer Management
command: null
domain: 01-core
module: Layers
kind: tool_or_workflow
implementation_status: partially_implemented
implementation_validation_status: validated_supported_slice
implementation_reviewed_at: '2026-10-11'
implementation_validation_record: ../../../docs/validation/2026-10-11-layer-panel-implementation.md
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-LAYER-001 — Layer Management

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `partially_implemented` / `validated_supported_slice`; các thao tác panel đã được kiểm tra trên plugin và shared host tương thích. Giới hạn được ghi ở implementation notes.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-LAYER-001` — Layer Management.**

Layers tổ chức object theo group màu và visibility/lock; current layer nhận object không có semantic layer chuyên biệt. Vai trò màu Settings tím, Cutters cam, Profiles vàng, Rails nâu đỏ giữ nhận diện nhưng không dùng màu làm loại geometry. In arrow gán các object đã chọn sang layer đích; đây là thao tác khác với chọn current layer. Right-click swatch chọn objects visible trên layer; lock ngăn pick/region nhưng vẫn hiện; toggle lớp/group bật tắt display. Hide/Show object độc lập layer: Hide giấu selection, Show hiện mọi hidden trong scope phiên, right-Hide giữ selection và ẩn phần khác, right-Show cho chọn một phần hidden để hiện. Job Bag thường lưu selection nên không tự gồm hidden/off-layer; full save/Master lưu mọi object cùng trạng thái. Advanced có thêm 16 layers có thể đổi tên/màu qua Shift+right-click, có giới hạn với Matrix Art/Lights theo capability; lưu document để giữ tùy biến, New dùng defaults. Không dùng swatch index thay persistent layer ID.

### Nguồn và phạm vi đối chiếu ngày 2026-10-11

Đã đọc và kiểm hình cả bốn trang của bản trích người dùng cung cấp
`matrix8_book_1 Copy.pdf`: trang PDF 1-4, trang in 44-47, Chapter 9,
The Layers Menu. Đây là nguồn mô tả Matrix8; ảnh người dùng là tham chiếu
bố cục panel. Không coi ảnh, code mẫu hay tên control là chứng nhận runtime
hoặc bằng chứng mọi hành vi Matrix9 giống Matrix8.

| Trang PDF / in | Nội dung được đối chiếu |
|---|---|
| 1 / 44 | Vai trò nhóm Metal, Gem, User, Heads, Finger, Cutting, Creation. |
| 2 / 45 | Anatomy: In arrow gán selection; swatch chuột phải chọn theo layer; current-layer highlight; layer/group toggle. |
| 3 / 46 | Lock chặn cả Region Select; layer-off; Hide/Show trái/phải; scope Job Bag và full save. |
| 4 / 47 | Advanced thêm 16 layer; Shift+right-click tên/màu; lưu tùy biến và reset theo phiên/document. |

### Bố cục panel và các thao tác cần có

Hàng đầu là Lights cùng Hide/Show. Khối cơ bản có hai cột, tám hàng:
trái Metal 01-04 rồi Gem 01-04; phải User 01-04 rồi Heads, Finger,
Cutting, Creation. Khối Advanced có hai cột, tám hàng: trái User 17-20
rồi User 25-28; phải User 21-24 rồi User 29-32. Advanced có control mở/thu
riêng. Mỗi hàng có tên, In arrow, swatch, lock và visibility; thanh dài
điều khiển visibility theo nhóm. Màu highlight báo current layer.

| Thành phần / thao tác | Hợp đồng nguồn | Phạm vi đã triển khai |
|---|---|---|
| Current layer | Output mới dùng current nếu không có layer nghiệp vụ riêng. | Tên hoặc swatch trái chọn current; CMD nhận index 1..33, tên chuẩn/tên tùy biến, Lights và None. Arrow không đổi current. |
| In arrow trái | Gán selection sang layer đích và đổi màu. | Validate cả batch trước mutation; plain root geometry/members được gán trong một transaction. Reject layer khóa, foreign document, nhóm structural/placed/import ownership không an toàn. |
| Swatch phải | Chọn objects visible trên layer. | Chỉ chọn member visible, không Hidden/off/locked, có quyền Selectable. |
| Lock | Chặn pick và Region Select; vẫn thấy để tham chiếu. | Mặt nạ Selectable native và OM9 dispatcher/mouse/SelectAll/edit guards; unlock giữ quyền Selectable trước đó. Visibility không ghi đè quyền chọn. |
| Visibility layer | Tắt/bật display layer và members. | Giữ explicit object visibility và OM9 Hidden riêng; Off/On không hiện object đã ẩn. |
| Visibility nhóm | Bật/tắt cả nhóm. | Nhóm 4 slot: tất cả đang hiện thì tắt, trạng thái mixed thì hiện. |
| Hide / Show trái | Hide selection; Show hidden. | Hidden riêng theo document; Show khôi phục object do OM9 Hide, không bật layer đang Off hoặc object explicit invisible khác. |
| Hide / Show phải | Reverse Hide và selective Show. | Giữ selection/ẩn phần visible khác; dialog chọn hidden, Cancel không ghi và stale document/object identity bị từ chối. |
| Advanced | Thêm 16 layer, mở/thu. | Mặc định thu; đúng thứ tự hai cột 17–20/25–28 và 21–24/29–32. Thu không mất state. |
| Shift+right tên/màu | Tùy biến Advanced. | Chỉ 17..32, trim tên không rỗng/controls, tối đa 128 UTF-8 bytes, tên unique không trùng slot chuẩn; đổi màu members. Dialog revalidates session/generation. |
| Lights | Hàng riêng, không có trong MatrixArt. | Slot33 trắng, đủ current/Assign/selection/lock/visibility. Không tự tạo light geometry. Rust có Art restriction; MatrixArt runtime chưa hỗ trợ. |
| Footer combo / C A L V C | Người dùng xác nhận giao diện gốc không có (2026-10-11). | Bỏ toàn bộ, không suy chức năng hoặc để placeholder. |

Mã RGB tuyệt đối, kích thước pixel, default current layer và cách click tên
để đặt current không được suy từ ảnh minh họa. Màu/slot đang có trong Rust
là defaults OM9 hiện hữu; chưa chứng nhận palette thương mại chính xác.

### Chi tiết panel layer và liên kết material

Lệnh mở Layers gửi yêu cầu tới panel Layers bên ngoài. Wrapper chưa chứa
handler swatch, lock/current-layer hoặc right-click, nên không đủ để suy
bố cục và tương tác panel. OM9 phải nối panel với identity layer, effective
visibility/lock và current layer của document, thay vì dùng index UI làm
identity bền vững.

Helper material đã đọc sử dụng 32 định nghĩa layer chuẩn gồm nhóm Metal,
Gem, User Layer và các layer nghiệp vụ. Danh sách này phục vụ khởi tạo hoặc
bổ sung material, chưa chứng minh thứ tự swatch trên UI. Nhánh được Shade
Mode gọi tìm layer hiện có theo tên và bỏ qua layer thiếu; nó không tạo
layer thiếu trong nhánh đó. Với layer chưa có material, helper tạo material;
nếu thiếu environment texture tương ứng thì thêm texture, đổi tên material
theo định nghĩa layer, đặt texture decal mode và cập nhật liên kết material.
Material hiện có có thể bị sửa; index dùng chung cần kiểm ảnh hưởng tới
object/layer khác. Không thấy nhánh này đổi visibility,
lock, current layer hoặc màu của layer đã tồn tại.

OM9 dùng layer IDs và semantic roles có schema, kiểm khả dụng resource và
xử lý lỗi material riêng. Material initialization phải được công bố đúng
scope; việc đổi chế độ hiển thị không được âm thầm ghi đè material tùy biến.
Không nhập bitmap thư viện thương mại vào repository để hoàn thiện panel.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Layer` | `reference` | Layer đích | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Action` | `choice` | Set current, Assign, Select, Lock, Toggle, Hide hoặc Show | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Name` | `text` | Tên layer user | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Color` | `text` | Màu hiển thị | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-LAYER-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust layer schema ID/name/color/role/visible/locked/current, native adapter group/view-provider và selection guards; full serializer giữ layer/object visibility riêng.
3. Tích hợp `Layer Management` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-LAYER-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Layer", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Color", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Layer off không pick; object hidden vẫn lưu trong FCStd; đổi swatch không đổi semantic type; current layer nhận output không có role; right-Hide/Show tuân selection.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

[Bằng chứng triển khai và kiểm thử](../../../docs/validation/2026-10-11-layer-panel-implementation.md)
thay thế hiện trạng của [audit trước triển khai](../../../docs/validation/2026-10-11-layer-management-audit.md).
Giữ `partially_implemented` cho toàn feature; panel và supported output slice đã nghiệm thu native.

### Inputs, identity và defaults

Rust sở hữu slot/defaults/parser, name/action policy, effective visibility và selection permission.
C++ là ngoại lệ native cho FreeCAD properties/groups/transactions/selection và Qt Unicode label matching.
32 slot model cộng Lights33; 0/None là root legacy. Group tạo lazy, mặc định visible/unlocked;
document New có current=None, palette OM9 hiện hữu. Circle/Ellipse root giữ xanh legacy;
các family khác giữ style hiện hữu khi current=None. Không chứng nhận palette thương mại.
Rename đổi Label, giữ native Name/ID/slot; modal giữ document session và Rust ObjectStamp,
không giữ pointer để commit sau đóng/mở lại cùng FCStd. Name và màu Cancel không ghi.

### Durable state và permissions

`OM9LayerSchema`, `OM9LayerIndex`, `OM9LayerColor`, `OM9LayerLocked`,
`OM9LayerStateSchema`, `OM9ActiveLayer`, `OM9OutputLayerIndex`,
`OM9LayerExplicitVisible`, `OM9LayerExplicitSelectable` và `OM9ObjectHidden`
được lưu trong FCStd. Layer visibility, explicit visibility và Hidden là các trạng thái riêng.
Native Selectable chỉ bị mask bởi lock; visibility tự ngăn native picking, OM9 guards
còn kiểm layer visibility/Hidden. Bật object qua cây FreeCAD không làm mất quyền Selectable.
Raw host commands ngoài OM9 dispatcher không được coi là permission service hoàn chỉnh.
Snap/reference là policy riêng, không thay đổi thành editing permission.

### Output, preview và transactions

Root Curve (gồm Circle History/Deformable, Ellipse/foci), Box/Sphere,
Sweep1/2/Loft và History, Join/Trim/Boolean, Rebuild dùng current khi commit.
Explode giữ group/source ownership và output-layer metadata để không thay placement.
Storage/rig/cage/import hierarchy không tự bị chuyển sang current.
Layer khóa từ chối output; layer Off nhận output invisible. Capture ngay trước transaction,
không freeze layer khi bắt đầu preview. Mọi layer mutation có transaction riêng/abort;
geometry output dùng transaction command hiện hữu. Rebuild DeleteInput mặc định được phép
với validated OM9 layer ownership; real dependents vẫn chặn. Undo/Redo phục hồi cả input/group.

### Phạm vi chưa nghiệm thu

MatrixArt runtime/material initialization, general imported-layer hierarchy/inheritance,
mọi semantic builder family và session-time output-layer binding chưa được bật bằng slice này.
Lights tổ chức object, chưa tạo illumination. Job Bag/Master không được chứng nhận từ FCStd tests.
Legacy files có visibility đã bị adapter cũ ghi đè không đủ dữ liệu để tự khôi phục ý định trước đó.
Không copy plugin SDK mới vào launcher FreeCAD1.1.4 khác ABI.

### Nghiệm thu panel

- [x] `LAYER-ASSIGN`: tách Assign/current, atomic rejection, membership/color và Undo/Redo.
- [x] `LAYER-SELECT`: right swatch và SelectAll giữ visible/unlocked/selectable scope.
- [x] `LAYER-LOCK`: native click, window/crossing Region Select, OM9 Delete và unlock permission.
- [x] `LAYER-VISIBILITY`: Hidden độc lập Off/On; explicit visibility/Selectable không bị mất.
- [x] `LAYER-HIDE-SHOW`: trái/phải, selective Show Cancel/commit, FCStd.
- [x] `LAYER-ADVANCED`: mở/thu, rename/color, Cancel, duplicate rejection và persistence.
- [x] `LAYER-MODAL`: đổi document, đóng/mở lại cùng persistent UID/native ID không commit stale actions.
- [x] `LAYER-OUTPUT`: native Line/Box/Sphere/Loft, Circle/Ellipse, Join/Explode/Rebuild và dependency protection.
- [x] `LAYER-LIGHTS`: slot33 không lẫn 32 model slots; assignment/visibility/state persistence.
- [x] `LAYER-FOOTER`: bỏ combo/C A L V C theo yêu cầu trực tiếp của người dùng.
- [ ] Art/material/general hierarchy và mọi output family: giới hạn ở trên, không suy từ panel tests.

## Acceptance checklist (supported slice)

- [x] Nguồn PDF 1–4 / trang in 44–47 đã đọc; quyết định footer của người dùng ghi riêng.
- [x] Inputs, defaults, current/Assign, explicit visibility và permission được mô tả.
- [x] Native commit/cancel, stale modal, Undo/Redo và FCStd được kiểm tra.
- [x] Fixtures panel/output/permission/modal dùng exact feature ID `OM9-LAYER-001`.
- [x] Plugin-only build và matching shared host kiểm tra module path/runner exit0.
- [ ] Toàn feature và toàn compatibility nguồn: còn scope exclusions được công bố.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-LAYER-001` hoặc ghi lý do không áp dụng.

- [x] Supported slice, native reports và giới hạn ghi trong report triển khai; audit cũ giữ riêng làm lịch sử. Checklist kỹ thuật chung còn cần evidence riêng cho toàn feature.
