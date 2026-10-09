---
id: OM9-MATRIXTOOLS-005
name: Styles
command: null
domain: 16-matrix-tools
module: Matrix Tools
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MATRIXTOOLS-005 — Styles

Alias tương thích: `Chưa có alias command`. Nhóm: `16-matrix-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Styles là thư viện cấu hình Builder. Khi Builder đang hoạt động, mở Styles, chọn category/style và Apply để cập nhật preview. Một style có thể điều chỉnh theo nhóm kích thước đá; các tham số ở sát biên nhóm có thể cần sửa tiếp. Không coi hệ số scale hoặc ranh giới nhóm size là đã xác định.

Để lưu, chỉnh thiết kế khi Builder còn handles và trước commit, chọn Add to Library. Thư viện nhận cấu hình cùng ảnh preview; người dùng có thể sửa mô tả và thay ảnh. Chọn mục trong browser có thể cập nhật preview; Cancel phải khôi phục snapshot cấu hình trước khi mở browser. Style là dữ liệu tham số, không phải plugin code hoặc capability mới. Builder không hỗ trợ Styles phải báo đúng phạm vi. Phiên bản schema, units, tham chiếu asset và quy tắc merge cần thiết kế rõ; lỗi nạp không được ghi từng phần vào session.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `builder_session` | `selection` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Action` | `choice` | Load/Save/EditMetadata; lựa chọn OM9 để tách luồng. | Chưa xác định; không tự điền. |
| `Style` | `reference` | Identity style để load/edit. | Chưa xác định; không tự điền. |
| `Category` | `text` | Category người dùng chọn. | Chưa xác định; không tự điền. |
| `Description` | `text` | Mô tả style. | Chưa xác định; không tự điền. |
| `PreviewImage` | `reference` | Ảnh preview mới nếu thay ảnh. | Chưa xác định; không tự điền. |

## Hướng dẫn triển khai

1. Rust snapshot toàn bộ cấu hình Builder có kiểu, feature ID, units và schema version. Apply validate toàn bộ trước cập nhật session; builder giữ capability thực tế.
2. C++/Qt dùng browser thư viện để chọn style và xem ảnh. Scale theo gem-size chỉ bật khi có quy tắc/range đã kiểm chứng; không scale mọi chiều theo một tỷ lệ đoán.
3. Save kiểm tra Builder còn active, serialize cấu hình mới và ghi file tạm rồi thay nguyên tử. Load cập nhật preview, không tự commit geometry; lưu ảnh/mô tả xử lý lỗi độc lập.
4. Đăng ký command/menu/CMD trong C++ native; Rust giữ request/state. Python đăng ký workbench và fixture. Giữ nhánh thiếu adapter disabled/unsupported.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-005",
    effect: Effect::ExternalOutput,
    roles: &[
        Role { name: "builder_session", kind: InputKind::Selection, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Action", kind: ParameterKind::Choice, required: true },
        Parameter { name: "Style", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Category", kind: ParameterKind::Text, required: false },
        Parameter { name: "Description", kind: ParameterKind::Text, required: false },
        Parameter { name: "PreviewImage", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Save khi Builder active rồi Load cho cùng request sau chuyển units; schema/feature sai bị từ chối trước sửa state.
- Cancel browser và lỗi ghi không làm mất style cũ; thay ảnh/mô tả không đổi geometry.
- Size-group boundary có fixture sau khi rule được xác lập; unsupported builder không nhận style.

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
- [ ] Automated tests use feature ID `OM9-MATRIXTOOLS-005`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MATRIXTOOLS-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MATRIXTOOLS-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MATRIXTOOLS-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MATRIXTOOLS-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MATRIXTOOLS-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-MATRIXTOOLS-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MATRIXTOOLS-005` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
