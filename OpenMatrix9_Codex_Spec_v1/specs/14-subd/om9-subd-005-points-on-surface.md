---
id: OM9-SUBD-005
name: Points On Surface
command: null
domain: 14-subd
module: SubD
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SUBD-005 — Points On Surface

Alias tương thích: `Chưa có alias command`. Nhóm: `14-subd`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

PointsOn Surface toggles hiển thị verts/edges của Clayoo Smooth mode; HUD/Clayoo options. Cỡ vertex 4/5/6, edge 2/2/3, opacity 40/95, level 3 và colors thấy chỉ ví dụ settings.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Objects` | `mesh` | 0 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `PointsOn Surface` | `boolean` | Hiện hoặc ẩn vertices/edges trên bề mặt trong Smooth mode. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Vertex Size` | `number` | Kích thước marker vertices lần lượt thường/hover/selected; unit hiển thị cần host xác định, không lấy số minh họa làm default. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Vertex Over Size` | `number` | Kích thước marker vertices lần lượt thường/hover/selected; unit hiển thị cần host xác định, không lấy số minh họa làm default. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Vertex Selected Size` | `number` | Kích thước marker vertices lần lượt thường/hover/selected; unit hiển thị cần host xác định, không lấy số minh họa làm default. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Edge Size` | `number` | Độ dày edges lần lượt thường/hover/selected; unit hiển thị cần host xác định. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Edge Over Size` | `number` | Độ dày edges lần lượt thường/hover/selected; unit hiển thị cần host xác định. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Edge Selected Size` | `number` | Độ dày edges lần lượt thường/hover/selected; unit hiển thị cần host xác định. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Over Opacity` | `number` | Độ đục overlay hover, %; không lấy 40 minh họa làm default. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Selected Opacity` | `number` | Độ đục overlay selected, %; không lấy 95 minh họa làm default. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Vertex Border` | `text` | Màu biên marker vertex; không đổi material geometry. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Crease Edges` | `text` | Màu edges crease trong overlay. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Naked Edges` | `text` | Màu edges naked trong overlay. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Over Faces` | `text` | Màu faces đang hover. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Selected Faces` | `text` | Màu faces selected. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SUBD-005` và các vai trò Objects; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Render overlay control cage trên smooth limit, giữ cage và view state riêng.
2. C++/Qt: đăng ký và điều phối native command `OM9-SUBD-005`; chuyển kế hoạch Points On Surface sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Giữ thao tác ở phạm vi trạng thái hoặc đầu ra đã chọn. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SUBD-005`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SUBD-005",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Mesh, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "PointsOn Surface", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Vertex Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertex Over Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertex Selected Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Over Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Selected Size", kind: ParameterKind::Number, required: false },
        Parameter { name: "Over Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Selected Opacity", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertex Border", kind: ParameterKind::Text, required: false },
        Parameter { name: "Crease Edges", kind: ParameterKind::Text, required: false },
        Parameter { name: "Naked Edges", kind: ParameterKind::Text, required: false },
        Parameter { name: "Over Faces", kind: ParameterKind::Text, required: false },
        Parameter { name: "Selected Faces", kind: ParameterKind::Text, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Tắt ẩn points/edges overlay vẫn Smooth surface, switch không xóa verts; không lấy minh họa cỡ point làm default.
- OM9-SUBD-005: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra chuyển chế độ hoặc hủy đầu ra không làm thay đổi hình học.

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
- [ ] Automated tests use feature ID `OM9-SUBD-005`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SUBD-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SUBD-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SUBD-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SUBD-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SUBD-005` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-SUBD-005` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
