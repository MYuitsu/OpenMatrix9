---
id: OM9-MAIN-001
name: Main Menu
command: null
domain: 01-core
module: The Main Menu
kind: menu
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MAIN-001 — Main Menu

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-MAIN-001` — Main Menu.**

Main Menu chỉ mở một nhóm tại thời điểm, nút scroll tiến hai hàng; tooltip có tại icon và title bar. Click command cập nhật Icon History chín lệnh. Undo/Redo dùng Ctrl+Z/Ctrl+Y, Builder có thể cần undo cấp phiên. Reset Tools kết thúc lệnh đang chạy, phục hồi view, Wireframe, unlock object, show object/layer và bỏ background bitmap. Menu bao gồm File/View/Utilities/Measure, Curve/Surface/Solid/Transform, Builders/Tools/Gems/Settings/Cutters/Render/Matrix Art và Custom. Builders dựng phần tử có parameters/handles; Tools gồm rail/profile, Boolean/mesh/resize/check/weight/placement; Gems gồm gem load/layout/report/Pavé; Settings thêm vật liệu quanh gem; Cutters tạo vật liệu cắt; Render tổ chức materials/lights/render/animation/layout; Matrix Art dựng heightfield mesh ở dock phải. Mill/toolpath là khả năng tùy cấu hình, không bật khi thiếu backend. Custom kéo icon vào slots, đổi thứ tự, Save/Load bộ riêng. Top 11 luôn hiển thị. Curve, surface và closed solid phân biệt loại geometry trước dispatch.

### Chi tiết mở Main Menu

Lệnh mở Main chuyển yêu cầu tới panel Main của shell Matrix. Bộ code hiện
có chưa chứa phía nhận thông điệp và dựng shell, nên chưa xác nhận thêm
bằng code được thứ tự nhóm, scroll hai hàng, chín icon History hoặc Reset
Tools. Các chi tiết layout trong hợp đồng hiện có tiếp tục là thiết kế cần
nghiệm thu trên UI.

OM9 cần một panel instance theo workspace, trạng thái available/opened/failed,
đường đưa focus về panel đang mở và cleanup khi đổi workspace. Mở panel không
tạo geometry hoặc Undo entry. Mỗi mục menu đi qua command router và capability;
feature chưa hỗ trợ vẫn hiện nhưng disabled. Không lấy giá trị success của
lệnh gửi thông điệp làm xác nhận panel đã hiện.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Action` | `choice` | Open group, Scroll, Reset Tools hoặc Edit Custom | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-MAIN-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Qt dùng catalog Rust cho nhóm/tooltips và capability; Reset Tools chạy kế hoạch thay state từng phần có snapshot, không xóa geometry model; chỉ load approved SVG.
3. Tích hợp `Main Menu` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MAIN-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Mở nhóm mới đóng nhóm cũ; scroll tiến đúng hai hàng; Reset hủy preview và hiện layer/object bị ẩn; Custom Save/Load giữ thứ tự và command ID.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

Supported slice dưới đây là hợp đồng để triển khai; trạng thái native giữ nguyên
và các ca runtime vẫn cần nghiệm thu.

### Inputs

Workspace và command capability registry; dùng command identity ổn định cho mỗi menu item.

### Parameters and defaults

Ordering/layout theo hợp đồng UI hiện có; export wrapper chưa xác nhận thêm scroll, icon count hay Reset Tools.

### Output

Panel Main và focus state; mở panel không tạo geometry hoặc Undo entry.

### Preview / commit / cancel

Một instance theo workspace, tái focus panel đã mở; thao tác chọn lệnh chuyển vào command session riêng.

### History / dependency model

Menu layout là UI preference; History icon và document Undo là hai state có mục đích khác nhau.

### Error / invalid-input behavior

Unavailable/failed hiển thị diagnostic; thiếu command adapter giữ mục disabled, không dispatch macro không được hỗ trợ.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-MAIN-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-MAIN-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
