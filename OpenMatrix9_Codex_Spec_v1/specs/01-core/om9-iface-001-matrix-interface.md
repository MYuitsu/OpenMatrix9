---
id: OM9-IFACE-001
name: Matrix Interface
command: null
domain: 01-core
module: A Tour of the Interface
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-IFACE-001 — Matrix Interface

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-IFACE-001` — Matrix Interface.**

Giao diện gồm Icon History, Main Menu, Display, Snaps, Info & Settings, Layers, Projects và Builder phía dưới Projects; vùng còn lại chứa bốn viewport và CMD/feedback. Icon History chứa chín command gần nhất và Undo/Redo. Title bar cho collapse, đổi thứ tự, kéo float hoặc dock hai phía; một số thanh thu gọn vẫn giữ controls quan trọng. Dock bên phải có bề rộng tương thích 300 pixel nhưng UI OM9 phải điều chỉnh theo DPI/màn hình; hỗ trợ nhiều màn hình. Set as Default lưu bố cục người dùng, Reset Menus phục hồi bố cục mặc định, Reload Menus nạp bố cục cấu hình khởi chạy. CMD nhận tên lệnh, số, tùy chọn click hoặc mnemonic và Enter; feedback hiển thị kết quả. Menu ngang bổ sung chức năng host. Việc mở panel không tạo geometry.

### Chi tiết giao diện và vòng đời panel

Các lệnh mở Main, Layers, Snaps, Info & Settings và Project Manager chuyển
yêu cầu tới cửa sổ Matrix bên ngoài. Gửi yêu cầu thành công chưa bảo đảm
panel đã được tạo, vì nhánh không có window handle vẫn có thể kết thúc
không báo lỗi. OM9 phải phân biệt unavailable, opening, opened và failed;
panel chỉ nhận focus sau khi adapter xác nhận cửa sổ còn tồn tại.

Panel có owner là main window của host. Khi chuyển sang pick, helper có
nhánh modal và modeless: ẩn panel, chuyển focus tới vùng CAD, gọi pick rồi
hiện lại panel. OM9 phải phục hồi visibility/focus cả khi Cancel hoặc lỗi,
và hủy callback khi panel/document đóng. Đây là yêu cầu cleanup OM9 bổ sung;
code đã đọc chưa bảo đảm xử lý exception ở mọi nhánh helper.

Vị trí cửa sổ được lưu theo plugin và loại window, kèm font family/size.
Restore có thể bị từ chối khi thiếu dữ liệu hoặc font đã đổi; kích thước
chỉ phục hồi cho cửa sổ cho resize. OM9 cần schema version, DPI/font context,
màn hình còn khả dụng và fallback layout. Tọa độ panel là preference UI,
không thuộc geometry transaction.

Control số đã đọc có value/min/max nullable, increment 0.1, bốn chữ số sau
dấu thập phân và parse theo locale. Đây là cấu hình của control đó. Mỗi Builder phải
công bố units/range/default riêng; số chữ số hiển thị không phải tolerance
CAD. Rust giữ raw text, parsed quantity, accepted value và validation error;
Qt chịu ownership, binding, focus và lifetime widget.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `LayoutAction` | `choice` | Set as Default, Reset Menus hoặc Reload Menus | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-IFACE-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Lưu bố cục có version trong Rust; Qt dock widgets và command line đưa input tới command router; panel kéo ra màn hình khác cần phục hồi vị trí còn nhìn thấy được.
3. Tích hợp `Matrix Interface` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-IFACE-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "LayoutAction", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Collapse không mất controls; kéo dock/float giữ nội dung; lưu bố cục rồi mở lại giữ vị trí; command thành công thứ mười làm danh sách chỉ còn chín entry.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Supported slice dưới đây là hợp đồng để triển khai; trạng thái native giữ nguyên
và các ca runtime vẫn cần nghiệm thu.

### Inputs

Active workspace/document cùng layout preference; pick session phải giữ document generation và owner window.

### Parameters and defaults

Layout action có Set as Default/Reset/Reload. Font/DPI/bounds là preference UI. Mọi ô số có units/range/default theo feature; increment 0.1 và bốn chữ số sau dấu thập phân chỉ thuộc control đã đọc.

### Output

Dock/float panels, command area và feedback state. Mở/đổi bố cục không tạo geometry.

### Preview / commit / cancel

Panel→pick→panel có cleanup khi success/Cancel/exception; đóng panel/document hủy callbacks. Restore có fallback khi font/screen khác.

### History / dependency model

UI preference không thuộc dependency graph geometry; command được dispatch có lifecycle riêng.

### Error / invalid-input behavior

Phân biệt panel unavailable/open failed/restore invalid; không coi gửi window message là xác nhận panel đã mở.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-IFACE-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-IFACE-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
