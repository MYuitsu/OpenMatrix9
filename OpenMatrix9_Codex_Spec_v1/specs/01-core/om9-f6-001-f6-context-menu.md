---
id: OM9-F6-001
name: F6 Context Menu
command: F6
domain: 01-core
module: The F6 Menu
kind: menu
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-F6-001 — F6 Context Menu

Alias tương thích: `F6`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-F6-001` — F6 Context Menu.**

F6 mở menu gần cursor dựa trên loại/số object selected và mode. Tám mode: General (khởi tạo được mô tả), Curve, Gem, Surface, T-Splines, User, Report, Materials. Không chọn cho starter tools; một/hai curve, một/nhiều gem, surface cho tập khác nhau. Chọn Builder tự đưa selection hợp lệ vào In Box; sửa object có dữ liệu Builder khôi phục input/settings cũ. MSR dùng cho gem/profile và curve theo capability. Report xem tổng hợp selected; Materials chọn material và metal quality/specific gravity, áp bằng nút xác nhận, lọc material hệ thống/user. Customize lưu command text, icon, Name, vị trí vào ngữ cảnh selection đang dùng; Global hiển thị mọi context/mode; +/- thêm/xóa và up/down đổi vị trí; Save ghi cấu hình. Import nhận bộ menu .MENU khi định dạng đã hỗ trợ; underscore command là định danh English, không tự dịch. Refresh rebuild menu. Kéo header đổi vị trí; Pin giữ menu mở sau dispatch.

### Chi tiết context, Pin và cập nhật selection

F6 tạo context từ cả object và subobject: Brep edge được xử lý như curve,
Brep face như surface. Ngoài type/count, context còn dùng object identity,
metadata ID/SubID, nhận diện gem và báo cáo metal/gem. OM9 phải giữ owner,
subelement, document/revision và semantic role; tên hoặc màu object chưa đủ
để nhận diện gem/Builder. Metadata không resolve được phải có trạng thái
unknown, không tự coi như object thường đã được phân loại đầy đủ.

Mở và refresh được chuyển tới UI dispatcher, có guard chống re-entry trong
lúc lập context. Selection đổi khi cửa sổ đang hiện làm rebuild context.
Code dùng timer selection một lần 400 ms và đường kiểm lại sau mouse-down
750 ms; đây là hằng số của các nhánh cụ thể. OM9 gộp event, kiểm revision
và đo latency riêng; không đưa delay cố định này vào mọi command.

Escape đóng menu khi chưa Pin. Chọn command cũng đóng menu chưa Pin trước
khi dispatch; menu đã Pin tiếp tục hiện. Đóng F6 khác với hủy command đang
chạy. Adapter phải ngăn hai phiên command trùng nhau và vô hiệu hóa context
cũ sau đổi document, xóa object hoặc thay topology.

Vị trí mở thường lấy cursor với offset (-10,-100); Pin-on-startup dùng vị
trí đã lưu. Cửa sổ có owner host và không hiện trên taskbar. Nhánh clamp đã
đọc chỉ kiểm một số mép, nên OM9 bổ sung kiểm bốn mép, DPI, tháo màn hình và
panel lớn hơn working area. ViewModel dựng menu nằm ở dependency chưa xuất;
thứ tự/filter đầy đủ của tám mode vẫn cần xác minh.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | General, Curve, Gem, Surface, T-Splines, User, Report hoặc Materials | General — mô tả khởi tạo tương thích; không auto-fill trong mẫu. |
| `Global` | `boolean` | Hiện command mọi context | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Pin` | `boolean` | Giữ menu sau dispatch | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Name` | `text` | Tên mục menu | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Command` | `text` | Command text đã kiểm tra | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Position` | `number` | Vị trí mục | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-F6-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust khóa context key gồm mode/type/count và capability; Qt render menu, marshal selection và history attributes vào phiên Builder; chỉ parser macro được hỗ trợ được thực thi.
3. Tích hợp `F6` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-F6-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Global", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pin", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Name", kind: ParameterKind::Text, required: false },
        Parameter { name: "Command", kind: ParameterKind::Text, required: false },
        Parameter { name: "Position", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Một curve và hai curve sinh menu khác; custom nonglobal chỉ hiện đúng context; Global hiện mọi mode; Pin giữ cửa sổ; Builder nhận đúng input cùng settings đã lưu.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Supported slice dưới đây là hợp đồng để triển khai; trạng thái native giữ nguyên
và các ca runtime vẫn cần nghiệm thu.

### Inputs

Selection snapshot gồm object IDs, owner/subelement, geometry types, revision và metadata gem/Builder đã resolve.

### Parameters and defaults

Pin và vị trí panel có preference. Timers 400/750 ms thuộc nhánh code gốc; latency OM9 phải đo riêng. Full menu ordering/filter còn thiếu ViewModel export.

### Output

Context/menu state và yêu cầu command; bản thân dựng menu không tạo shape.

### Preview / commit / cancel

Refresh qua UI dispatcher, có guard/gộp event và bỏ context cũ. Esc/chọn command đóng menu chưa Pin; đã Pin giữ menu. Window positioning kiểm bốn mép/DPI.

### History / dependency model

F6 đọc metadata, chưa tự tạo HistoryRecord; command được chọn quyết định dependency và transaction.

### Error / invalid-input behavior

Object/subelement mất hoặc revision cũ làm context invalid; không resolve role bằng màu/tên. Cancel command và close menu được xử lý riêng.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-F6-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-F6-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
