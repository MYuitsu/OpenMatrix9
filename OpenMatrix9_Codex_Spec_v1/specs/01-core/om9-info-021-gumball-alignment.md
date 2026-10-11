---
id: OM9-INFO-021
name: Gumball Alignment
command: GumballAlignment
domain: 01-core
module: Info & Settings
kind: command
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-INFO-021 — Gumball Alignment

Alias tương thích: `GumballAlignment`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-INFO-021` — Gumball Alignment.**

GumballAlignment chọn CPlane, Object hoặc World cho **một frame Gumball dùng
chung** trong document. Mọi overlay thuộc `OM9-INFO-020` hiển thị frame đó từ
camera của mình; lệnh alignment không xoay shape hoặc tạo Undo geometry.

### Đồng bộ hệ trục trong bốn viewport — revision 2026-10-11

- `GumballAlignment World`: axes theo world XYZ ở mọi ô. Đổi focus hoặc orbit
  camera không đổi frame/pivot. Đây là mode dùng cho fixture đối chiếu ảnh
  Top/Front/Right/Perspective; màu theo trục, không theo chiều màn hình.
- `GumballAlignment Object`: axes theo native frame của selection có adapter
  xác định được, gồm curve/extrusion/planar subobject và các owner được hỗ trợ.
  Một frame thế giới được capture rồi project ra mọi camera; không lấy frame
  object riêng theo từng view. Không có frame duy nhất thì báo unresolved,
  giữ frame/alignment trước; không chọn World ngầm.
- `GumballAlignment CPlane`: idle frame theo CPlane của viewport đang active.
  Khi đổi active view/CPlane, recapture một frame và cập nhật tất cả overlays
  cùng lúc; không để bốn ô có bốn hướng khác nhau. Basis đã capture phải được
  giữ nguyên suốt gesture; đổi view/CPlane trong gesture hủy preview đồng bộ.
- Default OM9 CPlane cho profile mới và preference đã lưu vẫn được giữ theo
  quyết định trước. World là cách đối chiếu ảnh, không phải bằng chứng ảnh
  xác lập default startup của Rhino/Matrix.
- Relocated frame override alignment tự động tới khi Reset, explicit Alignment
  hoặc source stamp không còn hợp lệ; đổi focus khi idle không xóa pivot đã
  relocate. Sau Reset/Alignment, Scale extents và hit targets phải được tính
  lại cho mọi view, không dùng geometry screen-space cũ.
- Menu trên bất kỳ ô nào và Command dùng cùng handler/checked state. Chọn
  Alignment không tự bật Gumball nếu đang Off. Khi On, thay basis không tạo
  object, thay placement, thêm transaction hoặc nhân bản bộ gizmo state.

Ảnh dùng làm chuẩn màu/bốn view; bản trích Matrix 8 Book 1 PDF3/trang in37
mô tả CPlane/Object/World. [Rhino 5 Help](https://docs.mcneel.com/rhino/5/help/en-us/commands/gumball.htm)
đối chiếu active CPlane và reset alignment. Quy tắc một frame chia sẻ và
update đồng bộ là hợp đồng OM9 để đạt ảnh, không phải chứng minh code nội bộ
Rhino. Selection groups/multiple owners/control points phải có chính sách
cache rõ; không tự thừa hưởng single-owner relocated frame.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `selected_objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Alignment` | `choice` | CPlane, Object hoặc World | Default OM9 CPlane ở profile mới; giữ preference đã lưu; World cho fixture ảnh; Object cần frame native xác định được. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-INFO-021`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust lưu alignment policy, C++ lấy frame object/subelement hoặc CPlane và Qt dựng gizmo axes.
3. Tích hợp `GumballAlignment` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-021",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Alignment", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Object xoay 30 độ làm axes theo object; World giữ XYZ; đổi alignment không đổi placement/shape.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Ngày 2026-10-11: `partially_implemented` / `validated_supported_slice`.
One shared World/Object/CPlane frame, idle CPlane recapture, frozen gesture and relocated override across focus/maximize. Native whole-owner/curve/face frames; Planar oriented Mesh Segments; Mesh edge/vertex/light/camera or ambiguous frames remain explicit UnsupportedFrame.

[Evidence mới](../../../docs/validation/2026-10-11-gumball-four-view-completion.md).
[Hợp đồng inputs/units/output/lifecycle và giới hạn](../../../docs/features/OM9-INFO-021.md).
Nguồn bản trích Matrix8 Book1 PDF1–3/printed35–37; ảnh Rhino5 bốn view.
Cache widget process-local; geometry/report native giữ FCStd, không thêm History graph.

## Acceptance checklist

- [x] World/Object dùng cùng frame trong bốn ô; idle focus/camera không đổi pivot hoặc geometry.
- [x] CPlane của active view cập nhật tất cả overlays; không recapture giữa gesture.
- [x] Menu/CMD Alignment và Reset đồng bộ basis, Scale extents, colors, pick và checked state ở mọi ô.
- [x] Relocated override giữ qua focus và maximize/restore; explicit Alignment reset đúng selection.
- [x] Native planar oriented Mesh Segment Object frame, rotated Placement và ambiguous/degenerate/nonplanar/cache rejection có evidence riêng.
- [ ] Mesh edge/vertex và light/camera frame adapters còn thiếu trước full original-host parity.
- [x] Mọi case trên giữ object count, placement/shape và Undo; cancel trả frame cũ ở cả bốn ô.

- [x] Hợp đồng và điều kiện hình học được kiểm chứng trong supported native slice; adapter gaps ghi riêng.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] Alignment/Relocate are view state; no dependency mutation or new associative History graph (native unchanged geometry/count/Undo evidence).
- [x] No document geometry/count/Undo writes for view-state changes; successful geometry uses OM9-INFO-020 transaction.
- [ ] Full original-host view/pivot persistence not claimed; owner cache is process-local by approved design.
- [x] Automated tests use feature ID `OM9-INFO-021`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [x] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): native supported slice; evidence/limitations in feature record.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-INFO-021` hoặc ghi lý do không áp dụng.
- [x] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): native supported slice; evidence/limitations in feature record.
- [x] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): native supported slice; evidence/limitations in feature record.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-INFO-021` hoặc ghi lý do không áp dụng.
- [x] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): native supported slice; evidence/limitations in feature record.
- [x] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): native supported slice; evidence/limitations in feature record.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-INFO-021` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-INFO-021` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-INFO-021` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
