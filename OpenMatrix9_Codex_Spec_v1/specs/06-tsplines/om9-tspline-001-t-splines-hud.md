---
id: OM9-TSPLINE-001
name: T-Splines HUD
command: null
domain: 06-tsplines
module: T-Splines
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-TSPLINE-001 — T-Splines HUD

Alias tương thích: `Chưa có alias command`. Nhóm: `06-tsplines`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

HUD panel dock dưới main menu, kéo titlebar đổi vị trí. Main On/Off bật công cụ; indicator ở Perspective và TS trong Job Bag cho biết có T-Spline/HUD active. Chọn component Mode trước MSR, Drag Mode định frame, Compatibility định tốc độ. HUD tập hợp convert, loop/ring select, symmetry, Box/Smooth, MSR, edit modes. Selected hiện tên object đang highlight. Ngoại trừ Hot Keys, các switch dưới HUD reset khi khởi động lại.

### Tùy chọn và tương tác

- `Enabled`: bật/tắt HUD
- `Soft Manipulate`: bật/tắt; mặc định Off; phân bố dịch vertex mượt sang lân cận
- `Set Soft Radius`: khoảng ảnh hưởng, nhỏ local, lớn toàn model
- `Paint`: bật/tắt; kéo qua faces để thêm chọn không cần Shift
- `Hotkeys`: bật/tắt; mặc định Off; On ưu tiên MSR/edit hotkeys và chặn nhập command text
- `Selected`: trường đọc tên selections

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Enabled` | `boolean` | bật/tắt HUD | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Soft Manipulate` | `boolean` | bật/tắt; mặc định Off; phân bố dịch vertex mượt sang lân cận | Off |
| `Set Soft Radius` | `number` | khoảng ảnh hưởng, nhỏ local, lớn toàn model | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Paint` | `boolean` | bật/tắt; kéo qua faces để thêm chọn không cần Shift | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Hotkeys` | `boolean` | bật/tắt; mặc định Off; On ưu tiên MSR/edit hotkeys và chặn nhập command text | Off |
| `Selected` | `text` | trường đọc tên selections | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-TSPLINE-001 và phiên chọn có thứ tự, kiểm tra vai trò document; chuẩn hóa tùy chọn riêng của T-Splines HUD.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt ánh xạ thao tác T-Splines HUD sang trạng thái chọn/hiển thị của native viewport; giữ dữ liệu hình học ổn định và phục hồi trạng thái khi hủy.
4. Native C++ đăng ký command/menu và dispatch sang Rust/native geometry. Python đăng ký workbench và hỗ trợ fixtures/macro kiểm chứng. Capability chưa có hoặc chưa kiểm chứng phải giữ disabled với lý do cụ thể.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-TSPLINE-001",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Enabled", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Soft Manipulate", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Set Soft Radius", kind: ParameterKind::Number, required: false },
        Parameter { name: "Paint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Hotkeys", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Selected", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- HUD drag/dock và enable hiển thị indicator; Soft radius tăng vùng ảnh hưởng, Paint chọn nhiều face, Hotkeys không nuốt command khi off.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Chuyển chế độ rồi phục hồi; kiểm tra tọa độ/topology hình học không đổi nếu lệnh chỉ đổi trạng thái hiển thị/chọn.

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
- [ ] Automated tests use feature ID `OM9-TSPLINE-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-TSPLINE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-TSPLINE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-TSPLINE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-TSPLINE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-TSPLINE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-TSPLINE-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
