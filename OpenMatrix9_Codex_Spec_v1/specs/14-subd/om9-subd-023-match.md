---
id: OM9-SUBD-023
name: Match
command: ClayMatch
domain: 14-subd
module: SubD
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SUBD-023 — Match

Alias tương thích: `ClayMatch`. Nhóm: `14-subd`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

ClayMatch một selection cùng lúc mặc định average về nhau; two stages Source Enter Target Enter first→second; defaults phụ thuộc trình tự nhưng CMD override được. Weld Yes merge thành one surface, No distinct; Average Yes hai phía move, No source only; Align sửa twist. Curve target có Flip và Domain 1 default 0, Domain 2 default 1, normalized 0–1 (Domain 2 cue nhắc start nhưng mục đích end cần confirm).

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `SourceEdges` | `selection` | 1 | 1 |
| `Target` | `object` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Weld` | `boolean` | Yes merge vertices sau match, No giữ các surfaces riêng. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Average` | `boolean` | Yes hai phía di chuyển tới nhau; No nguồn di chuyển tới đích; khởi tạo phụ thuộc trình tự selection. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Align` | `boolean` | Đảo correspondence giữa bề mặt để sửa twist. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Flip` | `boolean` | Đảo direction trong nhánh target curve; không áp cho nhánh surface match. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Domain 1` | `number` | Tham số curve chuẩn hóa 0–1 của đầu miền match; 0 là đầu curve, 1 là cuối. | 0 |
| `Domain 2` | `number` | Tham số curve chuẩn hóa 0–1 của biên còn lại; mô tả start/end chưa thống nhất cần kiểm chứng khi adapter triển khai. | 1 |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SUBD-023` và các vai trò Source Edges, Target; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Resample edge chains/target curve, correspondence by direction/seam/domain và weld policy; choose default from invocation style.
2. C++/Qt: đăng ký và điều phối native command `ClayMatch`; chuyển kế hoạch Match sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SUBD-023`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-023",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "SourceEdges", kind: InputKind::Selection, min: 1, max: Some(1) },
        Role { name: "Target", kind: InputKind::Object, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Weld", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Average", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Domain 1", kind: ParameterKind::Number, required: false },
        Parameter { name: "Domain 2", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Average No target fixed, Yes both move, Weld No ids distinct, domains 0/1 full curve; Flip sửa twist mà không đổi curve.
- OM9-SUBD-023: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SUBD-023`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SUBD-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SUBD-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SUBD-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SUBD-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SUBD-023` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-SUBD-023` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
