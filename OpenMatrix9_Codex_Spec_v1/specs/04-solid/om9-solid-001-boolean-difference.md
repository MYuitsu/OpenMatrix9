---
id: OM9-SOLID-001
name: Boolean Difference
command: BooleanDifference
domain: 04-solid
module: Solid Tools
kind: command
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SOLID-001 — Boolean Difference

Alias tương thích: `BooleanDifference`. Nhóm: `04-solid`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Chọn nhóm vật thể bị cắt, Enter, rồi nhóm cutter. Tính A trừ B, giữ vùng còn lại với cavity dạng cutter. BRep surface/polysurface dùng kernel riêng, mesh đi tuyến Boolean Builder với cùng kiểu cả hai nhóm. Closed solid hợp lệ là trường hợp ưu tiên; mặt mở phụ thuộc orientation và có thể cho nghĩa đảo. Cạnh đồng phẳng, góc hẹp/giao nhiều lần, object bao trọn object khác cần fixture riêng và báo kết quả kernel; không sửa input âm thầm để né lỗi.

### Tùy chọn và tương tác

- `DeleteInput`: Yes/No; No giữ cutter, Yes xóa cutter sau thành công

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `targets` | `solid` | 1 | Không đặt trong mẫu |
| `cutters` | `solid` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `DeleteInput` | `boolean` | Yes/No; No giữ cutter, Yes xóa cutter sau thành công | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SOLID-001 và phiên chọn có thứ tự, kiểm tra vai trò targets, cutters; chuẩn hóa tùy chọn riêng của Boolean Difference.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Boolean Difference trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
4. Preview nếu cần dùng dữ liệu tạm; xác nhận ghi kết quả và dependency trong một transaction. Hủy/lỗi không tạo output dở dang; Undo/Redo và save/reload giữ contract.
5. Native C++ đăng ký command/menu và dispatch sang Rust/native geometry. Python đăng ký workbench và hỗ trợ fixtures/macro kiểm chứng. Capability chưa có hoặc chưa kiểm chứng phải giữ disabled với lý do cụ thể.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SOLID-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "targets", kind: InputKind::Solid, min: 1, max: None },
        Role { name: "cutters", kind: InputKind::Solid, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "DeleteInput", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Hai box giao cho volume V(A)-V(intersection); DeleteInput No giữ cutter, No intersection không tạo shape hỏng.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Status:** partially implemented; supported slice validated.

**Scope:** Ordered target/cutter sets for solid BRep, surface-area BRep or closed Mesh via polyhedral OCCT; target replacement and cutter DeleteInput, cavities and persistence. Mixed categories and invalid/empty results reject.

Menu/F6 ID: `OM9_SolidDifference`. CMD: `BooleanDifference`. Matrix 8 Book 1 PDF pp.223
re-read through command continuation on 2026-10-09. This evidence is separate
from the engineering contract and sample request planner above.

See [per-feature record](../../../docs/features/OM9-SOLID-001.md) and
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
- [x] Automated tests use feature ID `OM9-SOLID-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SOLID-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SOLID-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SOLID-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SOLID-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SOLID-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SOLID-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SOLID-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SOLID-001` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
