---
id: OM9-MATRIXTOOLS-007
name: Blueprint
command: ClayBluePrint
domain: 16-matrix-tools
module: Matrix Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MATRIXTOOLS-007 — Blueprint

Alias tương thích: `ClayBluePrint`. Nhóm: `16-matrix-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Blueprint tạo hộp tham chiếu bằng ảnh cho dựng hình. Gán ảnh riêng cho Top (Looking Down), Front (Through Finger), Side (Side View) và Back; Delete tại một view bỏ đúng ảnh đó. OK thêm đối tượng tham chiếu, Cancel bỏ phiên cấu hình. Ảnh chỉ làm nền định hướng, không được xem là hình học chiều sâu đã tái tạo.

OpenMatrix9 cần quyết định kích thước/units, calibration theo model, orientation mặt Back và chiều mirror của mỗi view. Khi chưa có calibration, ghi rõ ảnh chưa có tỷ lệ kích thước đã xác thực. Không scale model chỉ để vừa ảnh. Asset ảnh phải được đóng gói/tham chiếu bền vững để save/reload và đổi máy không làm mất blueprint.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `top_image` | `image` | 0 | 1 |
| `front_image` | `image` | 0 | 1 |
| `side_image` | `image` | 0 | 1 |
| `back_image` | `image` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

Mẫu không khai báo option riêng; không suy ra rằng toàn bộ feature không có cấu hình.

## Hướng dẫn triển khai

1. Rust lưu mapping image→view và trạng thái xóa/thay ảnh; adapter yêu cầu ít nhất một ảnh hợp lệ, khác policy cardinality từng role độc lập.
2. C++/Qt decode ảnh, dựng các plane/reference nodes theo hệ tọa độ đã chốt, hiển thị preview và calibration state. Tách reference rendering khỏi mesh/solid output.
3. OK commit reference objects cùng assets/metadata trong một transaction; Cancel giải phóng ảnh tạm. Delete trong Builder chỉ sửa preview trước commit.
4. Đăng ký command/menu/CMD trong C++ native; Rust giữ request/state. Python đăng ký workbench và fixture. Giữ nhánh thiếu adapter disabled/unsupported.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-007",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "top_image", kind: InputKind::Image, min: 0, max: Some(1) },
        Role { name: "front_image", kind: InputKind::Image, min: 0, max: Some(1) },
        Role { name: "side_image", kind: InputKind::Image, min: 0, max: Some(1) },
        Role { name: "back_image", kind: InputKind::Image, min: 0, max: Some(1) },
    ],
    parameters: &[
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Từng view và mặt Back có orientation đúng theo policy; xóa Front không xóa Top/Side.
- Ảnh hỏng hoặc tất cả ảnh trống bị từ chối; không coi pixel count là kích thước mm.
- Cancel sạch preview, Undo/Redo và save/reload phục hồi reference cùng ảnh.

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
- [ ] Automated tests use feature ID `OM9-MATRIXTOOLS-007`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MATRIXTOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MATRIXTOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MATRIXTOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MATRIXTOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MATRIXTOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-MATRIXTOOLS-007` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MATRIXTOOLS-007` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
