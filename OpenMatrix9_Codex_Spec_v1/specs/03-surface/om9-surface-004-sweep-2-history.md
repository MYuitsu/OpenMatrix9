---
id: OM9-SURFACE-004
name: Sweep 2 History
command: gvSweep2History
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

# OM9-SURFACE-004 — Sweep 2 History

Alias tương thích: `gvSweep2History`. Nhóm: `03-surface`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Sweep 2 có History: hai rail và section theo thứ tự; sửa rail hoặc profile làm cập nhật mặt. Flip đảo chiều section; Automatic thử đồng bộ seam và hướng; Natural phục hồi seam lúc bắt đầu. Section phải đồng loạt mở hoặc đóng; giữ thứ tự và chọn gần cùng đầu để tránh xoắn. Preview đường chỉ hướng và thứ tự section.

### Tùy chọn và tương tác

- `Closed`: Yes/No; mặc định Yes; cần ít nhất hai profile, No để khe đầu/cuối
- `Flip`: Yes/No; đảo hướng sweep
- `Maintain Height`: Yes/No; mặc định Yes; giữ cao độc lập khoảng cách rail

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `rails` | `curve` | 2 | 2 |
| `sections` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Closed` | `boolean` | Yes/No; mặc định Yes; cần ít nhất hai profile, No để khe đầu/cuối | Yes |
| `Flip` | `boolean` | Yes/No; đảo hướng sweep | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Maintain Height` | `boolean` | Yes/No; mặc định Yes; giữ cao độc lập khoảng cách rail | Yes |

## Hướng dẫn triển khai

1. Rust lưu OM9-SURFACE-004 và phiên chọn có thứ tự, kiểm tra vai trò rails, sections; chuẩn hóa tùy chọn riêng của Sweep 2 History.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Sweep 2 History trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-SURFACE-004",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rails", kind: InputKind::Curve, min: 2, max: Some(2) },
        Role { name: "sections", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Closed", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Maintain Height", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Sửa một rail rồi profile cập nhật đúng kết quả; đổi Maintain Height kiểm tra biến thiên chiều cao.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Scope:** Native gvSweep2History CMD alias with exact ID, one-profile Maintain Height Yes, conditional Closed Yes, source/parent recompute and native persistence; full History/menu parity remains unverified.

CMD alias `gvSweep2History` reuses the native Sweep2 input/options workflow and
forces associative output. Reference: Matrix 8 Book1 PDF186 / printed176.
The manual describes automatic rebuilding after edits and Closed/Flip/
Maintain Height options. Native output preserves two ordered rails, sections,
all component subreferences, options and parent frames.

Closed defaults Yes for closed rails with at least two profiles; open-rail
closure is unsupported. Maintain Height defaults Yes with one profile; the
multiple-profile host strategy cannot implement it. Reverse/Flip is per input.
Other geometric bounds match [Sweep2](../../../docs/features/OM9-SURFACE-003.md).

The native dependency graph handles source/parent changes, suspend/resume,
invalid/deleted inputs, direct child edit detachment, native Undo and fresh-process
FCStd reconstruction. These lifecycle rules share the bounded
[advanced contract](../../../docs/features/surface-advanced-options.md). Dedicated History menu/F6
placement, global Record/Update controls and full Matrix parity remain unverified.
This feature remains partial. See [native validation](../../../docs/validation/2026-10-09-surface-constraints-history.md).

## Acceptance checklist

- [x] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] History/dependencies tested if applicable.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-SURFACE-004`.

- [x] Alias004, Maintain Height một profile mặc định Yes, closed default có điều kiện và native persistence.
- [ ] Hoàn tất tương thích đầy đủ ngoài supported slice đã công bố.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SURFACE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SURFACE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SURFACE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SURFACE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SURFACE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SURFACE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SURFACE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-SURFACE-004` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SURFACE-004` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID. (xem validation constraints/History: 530 native checks).
