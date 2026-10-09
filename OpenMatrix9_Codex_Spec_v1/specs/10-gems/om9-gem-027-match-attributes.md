---
id: OM9-GEM-027
name: Match Attributes
command: gvMatchAttributes
domain: 10-gems
module: Placing Gems
kind: command
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
implementation_validation_record: ../../../docs/validation/2026-10-09-reusable-builder-cage.md
---

# OM9-GEM-027 — Match Attributes

Alias tương thích: `gvMatchAttributes`. Nhóm: `10-gems`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

gvMatchAttributes chọn đá đích Enter rồi đá có settings/cutters và chọn style sheets thực có trong pop up, Create áp. Gems nhớ builder parameters của items đã tạo. Chỉ áp cùng shape theo workflow; khác shape có thể không tương thích, không hứa tự chuyển mọi kiểu.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Targets` | `gem` | 1 | Không đặt trong mẫu |
| `Source` | `gem` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Style Sheets` | `reference` | Bộ thuộc tính chọn để chuyển từ viên mẫu tới viên đích | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-GEM-027` và các vai trò Targets, Source; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Đọc styles metadata từ source, validate shape/capability từng style, lập kế hoạch rebuild settings/cutters riêng trên targets.
2. C++/Qt: đăng ký và điều phối native command `gvMatchAttributes`; chuyển kế hoạch Match Attributes sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-GEM-027`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-GEM-027",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "Targets", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Source", kind: InputKind::Gem, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Style Sheets", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Source thiếu style không có pop up entry; chỉ styles chọn được áp; khác shape báo in compatibility, pavé prongs không được match.
- OM9-GEM-027: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Status:** partially_implemented; validated_supported_slice.

### Supported programmatic Match Attributes

`OpenMatrix9Gui.matchBuilderAttributes(sourceGem, targetGems)` matches all supported durable Builder records from the source, preserving each raw parameters string and using the registered versioned affine-template evaluator. Source and targets must be distinct native gems in the active editable document and carry the same explicit nonempty `App::PropertyString OM9GemShape` tag. Every record/target and aggregate replay budget is validated before one native transaction; incompatible inputs fail without partial outputs.

Records can outlive deleted outputs; `restoreBuilderOutputs(record)` explicitly recreates missing owned slots. This is a reusable native API slice. `gvMatchAttributes` / OthersMatchAttributes remains disabled: selective style-sheet popup, original Gem settings/cutter solvers, per-builder defaults and .mss interchange are not implemented. Rust owns recipe/shape compatibility and finite/budget validation; C++ hosts native geometry, document links and transaction/FCStd persistence. See [Builder API contract](../../../docs/features/builder-history-native-contract.md).

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
- [ ] Automated tests use feature ID `OM9-GEM-027`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-GEM-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-GEM-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-GEM-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-GEM-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-GEM-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-GEM-027` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-GEM-027` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
