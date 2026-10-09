---
id: OM9-TOOLS-017
name: Join History
command: gvJoinHistory
domain: 09-tools
module: Tools
kind: command
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TOOLS-017 — Join History

Alias tương thích: `gvJoinHistory`. Nhóm: `09-tools`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

gvJoinHistory tạo child curve tương đương join hai hay nhiều parent; parents vẫn riêng và chỉnh được, child nằm trùng đường gốc, geometry phụ thuộc child cũng cập nhật. Join thường không thay thế liên kết này.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Parents` | `curve` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

Mẫu không khai báo option riêng; không suy ra rằng toàn bộ feature không có cấu hình.

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-TOOLS-017` và các vai trò Parents; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Dựng graph child join, kiểm tra kết nối endpoints với tolerance cấu hình công khai; giữ identities parents và recompute child khi parents thay đổi.
2. C++/Qt: đăng ký và điều phối native command `gvJoinHistory`; chuyển kế hoạch Join History sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-TOOLS-017`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TOOLS-017",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Parents", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Sửa một parent phải cập nhật child và geometry phụ thuộc; parents không bị join/xóa; lỗi không nối được không tạo child sai.
- OM9-TOOLS-017: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Implemented:** 2026-10-09. **Status:** partially implemented; supported slice validated (SDK83/83; cold4/4).

**Scope:** Connected separate native curves create a retained-parent HistoryJoin child; surfaces and Mesh reject.

**RCORE-09 policy revision (2026-10-09):** Record controls new supported links;
Update controls existing recorded descendants independently of Record. Enabling
Record does not retrofit children created as snapshots. This updates the OM9
implementation policy while retaining the original source contract above.

See the [per-feature record](../../../docs/features/OM9-TOOLS-017.md) and
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
- [ ] Automated tests use feature ID `OM9-TOOLS-017`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TOOLS-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TOOLS-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TOOLS-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TOOLS-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TOOLS-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-TOOLS-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TOOLS-017` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-TOOLS-017` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
