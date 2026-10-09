---
id: OM9-TRANSFORM-011
name: Smart Flow
command: gvSmartFlow
domain: 05-transform
module: Transform Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TRANSFORM-011 — Smart Flow

Alias tương thích: `gvSmartFlow`. Nhóm: `05-transform`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Smart Flow duy trì quan hệ base surface→destination surface và các objects trên base. Autobase khi preselect đúng một target tạo base phẳng tương ứng UV/kích thước trên active layer rồi liên kết. Không Autobase chọn base rồi target. History cần bật trước tạo relation; thay/add object hoặc surface recompute child. Align UV và normal; normal đảo đặt object sang underside. Detect Smart Flow dùng U đỏ, V xanh, backface đỏ.

### Tùy chọn và tương tác

- `Autobase`: tạo base tự động từ target
- `Clear`: bỏ relation, giữ child geometry hiện tại
- `BoundingBoxAddX/Y`: mức overhang cho phép quanh base theo mỗi axis
- `Tolerance`: sai lệch biến dạng, nhỏ tăng độ phức tạp; 0.01–0.25 là khoảng gợi ý, chưa chốt default
- `Around`: Yes/No; Yes giữ overhang tương ứng, No cố giới hạn object vào target

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `base_surface` | `surface` | 0 | 1 |
| `destination_surface` | `surface` | 1 | 1 |
| `objects` | `object` | 0 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Autobase` | `reference` | tạo base tự động từ target | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Clear` | `reference` | bỏ relation, giữ child geometry hiện tại | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `BoundingBoxAddX` | `number` | mức overhang cho phép quanh base theo mỗi axis Giá trị độc lập của BoundingBoxAddX. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `BoundingBoxAddY` | `number` | mức overhang cho phép quanh base theo mỗi axis Giá trị độc lập của BoundingBoxAddY. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Tolerance` | `choice` | sai lệch biến dạng, nhỏ tăng độ phức tạp; 0.01–0.25 là khoảng gợi ý, chưa chốt default | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Around` | `boolean` | Yes/No; Yes giữ overhang tương ứng, No cố giới hạn object vào target | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TRANSFORM-011 và phiên chọn có thứ tự, kiểm tra vai trò base_surface, destination_surface, objects; chuẩn hóa tùy chọn riêng của Smart Flow.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Smart Flow trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-TRANSFORM-011",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "base_surface", kind: InputKind::Surface, min: 0, max: Some(1) },
        Role { name: "destination_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "objects", kind: InputKind::Object, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Autobase", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Clear", kind: ParameterKind::Reference, required: false },
        Parameter { name: "BoundingBoxAddX", kind: ParameterKind::Number, required: false },
        Parameter { name: "BoundingBoxAddY", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tolerance", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Around", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Thêm/sửa/xóa object base cập nhật child; save/reload relation; đảo target normal đổi phía; Around No/Yes xử lý overhang khác.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

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
- [ ] Automated tests use feature ID `OM9-TRANSFORM-011`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TRANSFORM-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TRANSFORM-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TRANSFORM-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TRANSFORM-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TRANSFORM-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TRANSFORM-011` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
