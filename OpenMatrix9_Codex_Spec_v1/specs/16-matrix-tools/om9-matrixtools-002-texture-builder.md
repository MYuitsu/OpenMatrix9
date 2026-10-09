---
id: OM9-MATRIXTOOLS-002
name: Texture Builder
command: GVTextureBuilder
domain: 16-matrix-tools
module: Matrix Tools
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MATRIXTOOLS-002 — Texture Builder

Alias tương thích: `GVTextureBuilder`. Nhóm: `16-matrix-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Texture Builder tạo relief dạng mesh từ ảnh grayscale trên surface đã gán. Chọn surface, chọn ảnh rồi Go để xem trước; Create mới thêm mesh vào document. Reset bỏ ảnh khỏi Builder. Resolution gồm Low/Medium/High và phải được chuyển thành một chính sách meshing rõ ràng. TileX/TileY điều khiển số lần lặp, Rotation xoay texture; BlackHeight và WhiteHeight xác lập cao độ vùng đen/trắng. Offset X/Y dịch texture; ý nghĩa units của offset cần chốt ở adapter. Các giá trị xuất hiện trong một cấu hình minh họa không được dùng làm default.

Miền Rotation còn cần xác lập ở biên 0 độ so với phạm vi điều khiển 1–180 độ. Không âm thầm kẹp hay đổi 0 thành 1. Chính sách OpenMatrix9 đề xuất: nội suy cao độ theo luminance chuẩn hóa h=b+(w−b)·L rồi đẩy mesh theo normal; đây là lựa chọn solver cần nghiệm thu, không phải phép khôi phục kernel. Image colorspace/alpha, mapping UV, seam, trimmed face và ngân sách số mặt phải được quyết định trước khi bật capability.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `target_surface` | `surface` | 1 | 1 |
| `texture_image` | `image` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Resolution` | `choice` | Low/Medium/High; từng mức cần mesh budget/tolerance công bố. | Chưa xác định; không tự điền. |
| `TileX` | `number` | Số lần lặp theo X của texture; đơn vị count. | Chưa xác định; không tự điền. |
| `TileY` | `number` | Số lần lặp theo Y của texture; đơn vị count. | Chưa xác định; không tự điền. |
| `Rotation` | `number` | Góc xoay texture (độ); miền biên 0 còn cần quyết định. | Chưa xác định; không tự điền. |
| `BlackHeight` | `number` | Cao độ vùng đen (mm), có thể âm khi hợp đồng solver cho phép. | Chưa xác định; không tự điền. |
| `WhiteHeight` | `number` | Cao độ vùng trắng (mm). | Chưa xác định; không tự điền. |
| `OffsetX` | `number` | Dịch texture theo X; units/mapping chưa chốt. | Chưa xác định; không tự điền. |
| `OffsetY` | `number` | Dịch texture theo Y; units/mapping chưa chốt. | Chưa xác định; không tự điền. |

## Hướng dẫn triển khai

1. Rust giữ image identity, Tile/Rotation/height và phase preview/create; kiểm tra ảnh đã decode, số hữu hạn, units và giới hạn tài nguyên.
2. C++/Qt decode image qua adapter ảnh, tạo mapping trên surface và mesh theo budget Resolution. Tính normal/displacement theo chính sách đã chốt, kiểm tra self-intersection và không tuyên bố mesh luôn là solid.
3. Go chỉ cập nhật scene preview. Create tính lại từ input/revision, commit mesh và metadata trong một transaction; lỗi/hủy giải phóng ảnh/mesh tạm.
4. Đăng ký command/menu/CMD trong C++ native; Rust giữ request/state. Python đăng ký workbench và fixture. Giữ nhánh thiếu adapter disabled/unsupported.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "texture_image", kind: InputKind::Image, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Resolution", kind: ParameterKind::Choice, required: false },
        Parameter { name: "TileX", kind: ParameterKind::Number, required: false },
        Parameter { name: "TileY", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rotation", kind: ParameterKind::Number, required: false },
        Parameter { name: "BlackHeight", kind: ParameterKind::Number, required: false },
        Parameter { name: "WhiteHeight", kind: ParameterKind::Number, required: false },
        Parameter { name: "OffsetX", kind: ParameterKind::Number, required: false },
        Parameter { name: "OffsetY", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Ảnh ba mức đen/xám/trắng cho ba cao độ theo mapping đã chốt; đổi Tile/Rotation không đổi chiều normal ngoài ý muốn.
- Resolution thay mật độ và giữ budget; ảnh hỏng/alpha/seam/trim có fixture riêng.
- Go không tạo document object; Reset bỏ ảnh, Cancel sạch preview; mesh save/reload giữ tham số và hình.

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
- [ ] Automated tests use feature ID `OM9-MATRIXTOOLS-002`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MATRIXTOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MATRIXTOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MATRIXTOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MATRIXTOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MATRIXTOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-MATRIXTOOLS-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MATRIXTOOLS-002` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
