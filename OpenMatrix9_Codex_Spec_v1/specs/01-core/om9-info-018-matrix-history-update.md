---
id: OM9-INFO-018
name: Matrix History Update
command: null
domain: 01-core
module: Info & Settings
kind: tool_or_workflow
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-INFO-018 — Matrix History Update

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-INFO-018` — Matrix History Update.**

Matrix History Update toggle cập nhật dependency. Record On/Update Off giữ edit đã ghi nhưng output chưa refresh; bật Update lại tính toàn graph dirty theo thứ tự. Không xóa History khi tắt. Nhiều edits vào cùng input cần coalesce và không chạy solver trên GUI thread.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Enabled` | `boolean` | Chính sách cập nhật Matrix History | On — khởi tạo History được mô tả. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-INFO-018`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust dirty queue và generation tokens; C++ recompute giao diện UI-safe, bỏ kết quả stale khi document/inputs đổi.
3. Tích hợp `Matrix History Update` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-INFO-018",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Ba edits khi Update Off chỉ cập nhật một kết quả mới nhất khi On; dependency chain cập nhật thứ tự; solver failure giữ lỗi không ghi shape sai.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Implemented:** 2026-10-09. **Status:** partially implemented; supported slice validated (SDK83/83; cold4/4).

**Scope — RCORE-09 policy revision (2026-10-09):** Update suspends/resumes existing
recorded descendants without clearing links, independently of Record. Record
controls new relationships only. The original source description above is
retained separately from this newer OM9 implementation policy.

See the [per-feature record](../../../docs/features/OM9-INFO-018.md) and
[native History contract](../../../docs/features/history-native-contract.md)
for supported inputs, policy defaults, output ownership, command routes,
preview/commit/cancel behavior, persistent native links and limitations.
Rust owns policy/graph validation/scheduling; C++/Qt hosts document links,
transactions and native curve Join recompute. Ordinary Edit snapshots detach
affected Join/Surface History records inside their transaction with Lock/warning guards.

Fixtures: rust/tests/history_graph.rs and tests/history_smoke.FCMacro.
Build integration is present; native SDK History83/83 and latest cold4/4 pass. Checked acceptance boxes below refer only to this supported native slice;
full contract compatibility is not claimed.
Full Matrix History, Gem Builder records and arbitrary operation histories
remain outside this supported slice.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] History/dependencies tested if applicable.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-INFO-018`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-INFO-018` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
