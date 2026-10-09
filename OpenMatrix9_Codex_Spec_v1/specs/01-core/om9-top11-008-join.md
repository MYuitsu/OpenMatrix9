---
id: OM9-TOP11-008
name: Join
command: Join
domain: 01-core
module: Top 11 Buttons
kind: command
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOP11-008 — Join

Alias tương thích: `Join`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-TOP11-008` — Join.**

Join nhận curves theo chuỗi đầu hoặc surfaces/polysurfaces có naked edges tiếp giáp. Curves bỏ originals và tạo joined curve; endpoint gần trong tolerance được thay bằng midpoint theo compatibility mô tả, không kéo hai đầu rất xa. Surfaces Join đổi topology/sewing, không sửa underlying surface thành một face; muốn một face dùng Merge Surface. Undo trong phiên đảo bước cuối. Không dùng exact equality thay tolerance document, cũng không mở rộng tolerance ngầm.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `selected_objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Undo` | `boolean` | Bỏ bước join cuối | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-TOP11-008`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Phân loại curve-wire assembly và BRep sewing; đo residual, cập nhật endpoint được phép rồi validate output và one transaction replacement.
3. Tích hợp `Join` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOP11-008",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Undo", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Hai lines gặp đầu thành wire/polycurve; gap vượt tolerance không join; surfaces joined vẫn nhiều faces; Undo khôi phục originals.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Status:** partially implemented; supported slice validated.

**Scope:** Curve endpoint midpoint snapping with explicit bounded tolerance and sewn surface shells; disconnected/mixed/ambiguous inputs rejected. Document-tolerance compatibility and rich History remain unverified.

Menu/F6 ID: `OM9_TopIconJoin`. CMD: `Join`. Matrix 8 Book 1 PDF pp.85–86
re-read through command continuation on 2026-10-09. This evidence is separate
from the engineering contract and sample request planner above.

See [per-feature record](../../../docs/features/OM9-TOP11-008.md) and
[native Edit contract](../../../docs/features/edit-native-contract.md) for
explicit host defaults, tolerance, inputs, output placement, options and limits.
Rust validates session state; C++/Qt performs typed native Part/Mesh operations.
Transient preview adds no document objects. Successful geometry/removal is one
transaction with stale-input and dependency guards. Outputs are snapshots with
provenance strings; Explode restores containing groups and supported display fields.

Native options evidence: `build/edit_options_smoke-1/19b793add82d48de9e92ffcbf2c43343/results.json` (57/57).
Core regression: `build/edit_commands_smoke-1/4ebfad5e7e844c999ca94dc0a4493dea/results.json` (115/115). Matching SDK build,
Rust (140 tests) and Python (23 tests) passed. Independent review found no
remaining correctness blocker in this supported slice. See the live
[ledger](../../../docs/openmatrix9-progress.json) for further regressions,
source-audit limitations and remaining work. No full Matrix History,
document-tolerance compatibility or exhaustive topology is claimed.

## Acceptance checklist

- [x] Hành vi của supported slice đã được đối chiếu.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-TOP11-008`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOP11-008` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
