---
id: OM9-MATRIXTOOLS-003
name: Rope
command: gvRopeBuilder
domain: 16-matrix-tools
module: Matrix Tools
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MATRIXTOOLS-003 — Rope

Alias tương thích: `gvRopeBuilder`. Nhóm: `16-matrix-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Rope tạo các sợi xoắn quanh một curve. Gán curve vào Builder, chỉnh kích thước/số sợi/số vòng xoắn/khoảng cách rồi Enter để tạo. Thread Size là đường kính sợi; Threads là số sợi trong khoảng 1–6, Twists điều khiển số lượt quấn dọc curve, Thread Spacing là khoảng sợi so với curve dẫn. CapType None để đầu mở, Flat tạo đầu phẳng, Round tạo đầu tròn. Group Output chỉ nhóm các sợi đã tạo, không thay topology của chúng.

Mẫu OpenMatrix9 chọn Threads và Twists là số nguyên khi adapter dùng thuật toán rời rạc; từ chối số lẻ thay vì âm thầm truncate. Frame vận chuyển, hướng xoắn, curve kín và tổng twist tại seam cần nghiệm thu riêng. Không dựng solid cho CapType None nếu sợi thực tế còn mở; khoảng spacing, thread diameter và tự giao cần kiểm tra hình học. Default toàn ứng dụng và History phụ thuộc chưa xác lập.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `guide_curve` | `curve` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `ThreadSize` | `number` | Đường kính sợi theo units chiều dài của session; adapter đổi sang mm. | Chưa xác định; không tự điền. |
| `Threads` | `number` | Số nguyên 1..6. | Chưa xác định; không tự điền. |
| `Twists` | `number` | Số lượt xoắn; mẫu adapter yêu cầu số nguyên, phạm vi cuối cần chốt. | Chưa xác định; không tự điền. |
| `ThreadSpacing` | `number` | Khoảng từ sợi tới curve dẫn theo units chiều dài. | Chưa xác định; không tự điền. |
| `CapType` | `choice` | None/Flat/Round. | Chưa xác định; không tự điền. |
| `GroupOutput` | `boolean` | Nhóm các sợi khi commit. | Chưa xác định; không tự điền. |

## Hướng dẫn triển khai

1. Rust giữ guide identity và các tham số, kiểm tra integer count/finite dimensions và cap mode; không sửa curve dẫn.
2. C++ đánh giá curve và frame liên tục, tạo centerline từng sợi theo phase xoắn, rồi sweep tiết diện bán kính ThreadSize/2. Chính sách frame là quyết định OM9, không dùng transform tùy ý để né singularity.
3. Tạo cap theo mode, kiểm tra tự giao/đầu mở/solid và preview. Commit toàn bộ sợi cùng group/metadata trong một transaction; lỗi giữa một sợi phải rollback.
4. Đăng ký command/menu/CMD trong C++ native; Rust giữ request/state. Python đăng ký workbench và fixture. Giữ nhánh thiếu adapter disabled/unsupported.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-003",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "guide_curve", kind: InputKind::Curve, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "ThreadSize", kind: ParameterKind::Number, required: false },
        Parameter { name: "Threads", kind: ParameterKind::Number, required: false },
        Parameter { name: "Twists", kind: ParameterKind::Number, required: false },
        Parameter { name: "ThreadSpacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "CapType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "GroupOutput", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Curve thẳng với 1/3/6 sợi cho đúng số và đường kính; Twists thay số vòng mà không thay guide.
- None/Flat/Round có topology đúng; curve kín hoặc curvature lớn được kiểm tra seam/tự giao.
- Số sợi 0/7 hoặc count không nguyên bị từ chối; Cancel/Undo/Redo/save-reload bảo toàn input và group.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

> This section is intentionally separate theo hợp đồng feature-derived material. Fill it when implementing this feature. Các hành vi chưa xác định phải được ghi rõ.

### Inputs

- TODO: normalize supported object types and required selection state theo hợp đồng feature.

### Parameters and defaults

- TODO: verify exact semantics, units, ranges, and defaults theo hợp đồng feature.

### Output

- TODO: define resulting OpenMatrix9 object(s), geometry type, and ownership/history relationships.

### Preview / commit / cancel

- TODO: define interactive lifecycle only where supported by source.

### History / dependency model

- TODO: verify whether this feature records/uses History and define the OpenMatrix9 dependency graph.

### Error / invalid-input behavior

- TODO: define explicit errors and no-op/cancel cases.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [ ] Inputs and selection state documented.
- [ ] Parameters/defaults/units documented.
- [ ] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [ ] History/dependencies tested if applicable.
- [ ] Undo/redo tested.
- [ ] Save/reload persistence tested.
- [ ] Automated tests use feature ID `OM9-MATRIXTOOLS-003`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MATRIXTOOLS-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MATRIXTOOLS-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MATRIXTOOLS-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MATRIXTOOLS-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MATRIXTOOLS-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-MATRIXTOOLS-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MATRIXTOOLS-003` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
