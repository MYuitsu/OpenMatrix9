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

GumballAlignment chọn C-Plane, Object hoặc World. CPlane theo frame viewport, World theo XYZ, Object theo định hướng curve/extrusion/light/mesh face-edge-vertex/planar subobject/camera mà adapter có thể xác định. Không có frame object duy nhất thì báo unresolved thay vì lấy world ngầm. Chỉ đổi gizmo frame, không xoay shape.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `selected_objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Alignment` | `choice` | C-Plane, Object hoặc World | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

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
Native CPlane/World and one unambiguous whole-object/curve/face frame; menu/CMD preserve geometry. Mesh component/light/camera frames remain unsupported explicitly.

Source evidence: Matrix 8 Book 1 PDF45–47 / printed35–37, continuation inspected
until History PDF48. Source Y cyan/blue; approved OM9 Y green is a design choice.
[Inputs, parameters/defaults/units, outputs, lifecycle, dependencies, errors and tolerances](../../../docs/features/OM9-INFO-021.md).
[Native/Rust verification and limits](../../../docs/validation/2026-10-11-gumball-core.md).
No new associative History graph; existing native owner dependencies retained.
Geometry/report metadata persists in FCStd; widget cache is session view state.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
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
