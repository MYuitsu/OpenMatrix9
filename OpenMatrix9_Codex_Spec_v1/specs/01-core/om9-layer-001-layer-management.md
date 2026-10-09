---
id: OM9-LAYER-001
name: Layer Management
command: null
domain: 01-core
module: Layers
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-LAYER-001 — Layer Management

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-LAYER-001` — Layer Management.**

Layers tổ chức object theo group màu và visibility/lock; current layer nhận object không có semantic layer chuyên biệt. Vai trò màu Settings tím, Cutters cam, Profiles vàng, Rails nâu đỏ giữ nhận diện nhưng không dùng màu làm loại geometry. In arrow đổi layer selection; right-click swatch chọn objects visible trên layer; lock ngăn pick/region nhưng vẫn hiện; toggle lớp/group bật tắt display. Hide/Show object độc lập layer: Hide giấu selection, Show hiện mọi hidden trong scope phiên, right-Hide giữ selection và ẩn phần khác, right-Show cho chọn một phần hidden để hiện. Job Bag thường lưu selection nên không tự gồm hidden/off-layer; full save/Master lưu mọi object cùng trạng thái. Advanced có thêm 16 layers có thể đổi tên/màu qua Shift+right-click, có giới hạn với Matrix Art/Lights theo capability; lưu document để giữ tùy biến, New dùng defaults. Không dùng swatch index thay persistent layer ID.

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

Supported slice dưới đây là hợp đồng để triển khai; trạng thái native giữ nguyên
và các ca runtime vẫn cần nghiệm thu.

### Inputs

Document layer table, current layer và selected object identities; hierarchical visibility/lock phải xét effective state.

### Parameters and defaults

UI actions theo hợp đồng layer. Bảng 32 layer chuẩn trong helper material không xác nhận thứ tự swatch của panel.

### Output

Layer/object state hoặc material links theo action; persistent layer identity khác index hiển thị.

### Preview / commit / cancel

Mọi thay document trong transaction; nhánh đổi viewport và material initialization có scope riêng. Cancel không giữ mutation một phần.

### History / dependency model

Layer/material links phải save/reload đúng; đổi appearance chưa đồng nghĩa phải replay geometry History.

### Error / invalid-input behavior

Layer/resource thiếu hoặc native modify thất bại báo diagnostic; nhánh material repair bỏ qua layer thiếu thay vì tạo ngầm khi add-missing tắt.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-LAYER-001`.

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

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
