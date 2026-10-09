---
id: OM9-SURFACE-002
name: Sweep 1 History
command: gvSweepHistory
domain: 03-surface
module: Surface Tools
kind: command
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
---

# OM9-SURFACE-002 — Sweep 1 History

Alias tương thích: `gvSweepHistory`. Nhóm: `03-surface`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Tạo Sweep 1 có liên kết sống giữa một rail và chuỗi section. Hiển thị hướng sweep và thứ tự chọn; sau sửa rail/section và xác nhận, tính lại surface. Flip đảo chiều section; Automatic thử đồng bộ seam và hướng; Natural phục hồi seam lúc bắt đầu. Section phải đồng loạt mở hoặc đóng; giữ thứ tự và chọn gần cùng đầu để tránh xoắn.

### Tùy chọn và tương tác

- `Closed`: Yes/No; mặc định Yes; chỉ xuất hiện khi rail đóng và có ít nhất hai section; No để lại khoảng hở
- `Flip`: Yes/No; Yes đảo hướng sweep

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `rail` | `curve` | 1 | 1 |
| `sections` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Closed` | `boolean` | Yes/No; mặc định Yes; chỉ xuất hiện khi rail đóng và có ít nhất hai section; No để lại khoảng hở | Yes |
| `Flip` | `boolean` | Yes/No; Yes đảo hướng sweep | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SURFACE-002 và phiên chọn có thứ tự, kiểm tra vai trò rail, sections; chuẩn hóa tùy chọn riêng của Sweep 1 History.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Sweep 1 History trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-SURFACE-002",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "sections", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Sửa rail và một section lần lượt phải cập nhật surface, giữ thứ tự chọn; Closed No tạo khe giữa section đầu/cuối.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Scope:** Native gvSweepHistory CMD alias with exact ID, conditional Closed Yes, source/parent recompute, detach/Undo and FCStd; dedicated History menu/F6 and full Matrix workflow remain unverified.

CMD alias `gvSweepHistory` reuses the native Sweep1 input/options workflow and
forces associative output. Reference: Matrix 8 Book1 PDF183 / printed173.
The manual describes automatic rebuilding after rail/profile edits and Closed
Yes only for a closed rail with at least two profiles.

Native `OpenMatrix9Gui::SurfaceHistory` retains ordered curve subreferences,
options and parent-frame dependencies. Direct child Shape edits detach its
parents while descendants remain linked, according to the shared OM9-HISTORY-001
contract. Undo restores those parents. Invalid/deleted sources clear the result
and report an error. Source geometry is preserved; Preview, one-transaction
commit, Cancel, Undo/Redo and FCStd restore share Sweep1 behavior.

Closed defaults Yes only within the supported closed-rail/two-profile case.
Reverse/Flip is per input; the remaining geometric bounds match
[Sweep1](../../../docs/features/OM9-SURFACE-001.md). Full Matrix History/global controls and dedicated
History menu/F6 placement remain unverified. This feature remains partial.

[Shared advanced contract](../../../docs/features/surface-advanced-options.md) and
[native validation](../../../docs/validation/2026-10-09-surface-constraints-history.md).

## Acceptance checklist

- [x] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] History/dependencies tested if applicable.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-SURFACE-002`.

- [x] Alias002, closed default có điều kiện, sửa nguồn và Undo/Redo/FCStd được kiểm chứng native.
- [ ] Hoàn tất tương thích đầy đủ ngoài supported slice đã công bố.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SURFACE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SURFACE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SURFACE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SURFACE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SURFACE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SURFACE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SURFACE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-SURFACE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SURFACE-002` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID. (xem validation constraints/History: 530 native checks).
