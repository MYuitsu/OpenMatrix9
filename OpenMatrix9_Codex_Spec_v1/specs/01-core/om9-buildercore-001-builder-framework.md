---
id: OM9-BUILDERCORE-001
name: Builder Framework
command: null
domain: 01-core
module: Builders vs. Commands
kind: builder
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
implementation_validation_record: ../../../docs/validation/2026-10-09-reusable-builder-cage.md
---

# OM9-BUILDERCORE-001 — Builder Framework

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-BUILDERCORE-001` — Builder Framework.**

Builder là phiên có UI riêng, In preview lọc type input, Options/Mode, profile preview, graphic controls, slider và viewport handles cùng CMD. In nhận selection đúng loại hoặc F6 tự nạp; Builder không có input dùng Start. Mode đổi loại tạo và highlight lựa chọn. Slider cho đơn vị mm, degree hoặc percent tùy control; Shift drag bước 0.5, +/- bước 0.10 theo tương thích được mô tả. Nhập số trực tiếp thay kéo. Handles hover hiện tên: arrow scale, cube position, orb vị trí trên curve, orb+halo rotation; chữ/symbol còn tùy Builder. Profile preview mở browser, hover xem tạm, click áp; Edit mở editor profile riêng với curve/snaps/grid. Reset trở về defaults của mode đang chọn; Enter commit và bỏ handles/preview, Esc hủy, X đóng UI. Options CMD và UI có phần riêng nên không coi chúng luôn tương đương.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Loại kết quả do Builder riêng khai báo | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `ControlValue` | `number` | Giá trị trong đơn vị control | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Action` | `choice` | Start, Reset, Commit hoặc Cancel | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-BUILDERCORE-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust state machine phân biệt Idle/Input/Preview/EditingProfile/Commit; mỗi Builder khai báo schema riêng và profile asset do OM9 sở hữu; Qt handles/slider đồng bộ qua state events; Part commit đúng transaction.
3. Tích hợp `Builder Framework` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDERCORE-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "ControlValue", kind: ParameterKind::Number, required: false },
        Parameter { name: "Action", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Sai type không khởi tạo preview; cùng giá trị nhập số và handle cho cùng geometry; hover profile không commit; Enter một transaction, Esc không để geometry tạm.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Status:** partially_implemented; validated_supported_slice.

### Native API foundation

`OpenMatrix9Gui.createBuilderRecord(gem, featureId, parametersJson, seedObjects, scaleToGem=False)` copies same-document native Part seed geometry into a durable gem-local affine template. It preserves the inputs and returns a native BuilderRecord whose OutputNames identify separate BuilderOutput slots. Multiple independent records can belong to one gem. `restoreBuilderOutputs(record)` explicitly recreates missing owned slots; deleting output geometry does not trigger implicit resurrection, and occupied unrelated names reject.

The registered recipe is schema 1, evaluator `om9.affine-template`, version 1. Raw settings JSON is retained; it is metadata, not an arbitrary setting/cutter geometry solver. Affine scale/offset and optional current/initial gem dimension ratios evaluate the copied templates in current global gem frames. Rust validates schema/version, finite values, positive dimensions, feature identity and replay budgets. C++ is required for FreeCAD native document properties/transactions/signals/persistence and OCCT seed/result geometry. Rust never retains document/kernel pointers.

Global Record/Update/Lock/Clear integrates with durable output links. Under the RCORE-09 policy revision (2026-10-09), Record controls new links only: new outputs created with Record Off remain unrecorded, while existing recorded outputs update whenever Update is On. Update independently suspends/resumes existing links, independent output edits detach, and Undo/Redo/FCStd preserve the declared record/output scope. The original source contract above remains unchanged. This is API-only infrastructure: original per-builder commands, arbitrary Gem solvers, In/Start/Mode UI, sliders, handles, profile editor, Styles selection and .mss interchange remain disabled or unsupported. See [Builder API contract](../../../docs/features/builder-history-native-contract.md).

[Matching native validation](../../../docs/validation/2026-10-09-reusable-builder-cage.md) records the accepted API/command scope and limits. Original design requirements above remain broader than this implemented slice.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-BUILDERCORE-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-BUILDERCORE-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
