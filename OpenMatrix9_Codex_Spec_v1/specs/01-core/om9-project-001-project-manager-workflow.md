---
id: OM9-PROJECT-001
name: Project Manager Workflow
command: null
domain: 01-core
module: Project Manager
kind: manager
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-PROJECT-001 — Project Manager Workflow

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-PROJECT-001` — Project Manager Workflow.**

Project Manager quản lý rows và Job Bags snapshot theo giai đoạn. Create đặt tên row, Cancel không tạo; rows sắp alphabet. Row mới có 10 slots, dùng slot cuối mở thêm; không suy capacity tối đa. Row menu Save/Load archive .mpj khi codec hỗ trợ, Delete/Rename, Compact xóa slots rỗng và renumber, Timer Start/Stop/Reset với hh:mm:ss, Update. Job Bag có In/Load/thumbnail/index/Save/X, scroll ngang; rows scroll dọc, +/- expand. In lưu selection hoặc type-filter right-click; thumbnail chụp view hiện hành. Load bag thường import vào document đang có, có type-filter; X chỉ clear nội dung rồi Compact mới xóa slot. Save bag xuất file riêng. Master/Creation/Parts/Render/Output khác tên nhưng cùng snapshot toàn document: In không cần selection, gồm hidden/locked/off layers, camera/shading/materials/options; Load thay document sau prompt Save/Discard/Cancel. Master groups cho thêm slots cùng tên. Creation giữ inputs/History trước Boolean; Parts giữ thành phần cùng gem metadata; Render giữ materials/props; Output giữ cấu hình chế tạo. Các storage roots phải cấu hình, archive/import giữ metadata và paths an toàn.

### Chi tiết mở panel và phạm vi storage đã biết

Lệnh Project Manager yêu cầu mở panel qua window messaging. Dependency dữ
liệu project và receiver/UI của shell chưa có trong bộ export hiện tại.
Vì vậy tên lệnh chưa xác nhận codec .mpj, rows, Job Bags/Master, 10 slots,
Compact hoặc cách ghi file. Các workflow trong hợp đồng vẫn phải có store
và UI implementation được nghiệm thu riêng.

OM9 tách open panel, load project, restore scene và save store thành các
thao tác có kết quả riêng. Store có schema version, identity ổn định,
diagnostic lỗi path/codec và staging trước thay file đích. Cancel hoặc lỗi
load giữ project/document hiện hành; save thành công chỉ được báo sau khi
file đã ghi và kiểm tra được. Rust giữ state/storage policy, adapter host
restore dữ liệu theo supported scope. Mở được panel chưa xác nhận đã restore
project hoặc mọi asset phụ thuộc.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `RowName` | `text` | Tên project row | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Bag` | `reference` | Slot nội dung | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Action` | `choice` | Create, Save, Load, Rename, Delete, Compact, Update hoặc Timer | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `ObjectTypes` | `text` | Bộ lọc types của bag thường | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-PROJECT-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust store row/slot IDs tách display index, timer và content manifest; C++ document snapshot/import transaction; filesystem writer atomic và Qt prompts bảo vệ unsaved document khi replace.
3. Tích hợp `Project Manager Workflow` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-PROJECT-001",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "RowName", kind: ParameterKind::Text, required: false },
        Parameter { name: "Bag", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ObjectTypes", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Bag thường load thêm object; Master load thay toàn document sau Save/Discard/Cancel; Compact đổi index nhưng giữ content ID; Timer resume không reset; hidden object chỉ có trong full snapshot.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Supported slice dưới đây là hợp đồng để triển khai; trạng thái native giữ nguyên
và các ca runtime vẫn cần nghiệm thu.

### Inputs

Project identity, store path và supported document/assets; state mở panel khác load project.

### Parameters and defaults

Schema/version/codec OM9 có policy riêng; .mpj, Job Bags/Master và slot/UI details chưa được xác nhận thêm từ wrapper.

### Output

Project listing hoặc restored document theo action; success chỉ sau bước I/O/restore thực.

### Preview / commit / cancel

Stage trước thay store/document; Cancel/lỗi giữ project hiện hành; cleanup jobs/callbacks theo generation.

### History / dependency model

Project persistence cần identity/resources/version migration; không tự coi project save là History replay.

### Error / invalid-input behavior

Path/codec/resource lỗi được chỉ rõ; shell receiver và dependency dữ liệu project còn thiếu trong export, không đoán codec từ tên lệnh.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-PROJECT-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-PROJECT-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
