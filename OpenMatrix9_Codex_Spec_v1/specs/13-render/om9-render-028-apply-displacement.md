---
id: OM9-RENDER-028
name: Apply Displacement
command: ApplyDisplacement
domain: 13-render
module: Render Menu
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-RENDER-028 — Apply Displacement

Alias tương thích: `ApplyDisplacement`. Nhóm: `13-render`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

ApplyDisplacement tạo display mesh geometry thật từ procedural/image, khác bump chỉ shading, property editable. RemoveAll bỏ mọi objects; On, Texture Import.rtex/New/Edit/Duplicate, Mapping Channel. Black Point/White Point displacement theo current units, có thể signed bất kỳ finite. Initial Quality initial subdivision/samples, Max Faces post reduce, Fairing passes, Post WeldAngle, Mesh Memory Limit MB, Refine Steps, Refine Sensitivity 1 split all/.99 contrast nhỏ/.01 contrast lớn. Không mặc định modify BRep.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Objects` | `object` | 1 | Không đặt trong mẫu |
| `Texture` | `image` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `On` | `boolean` | Bật displacement trên display mesh có geometry thật. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Texture` | `reference` | Procedural/image texture; Import .rtex chỉ khi có codec adapter tương thích. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Mapping Channel` | `number` | Channel UV mapping của displacement; adapter kiểm tra channel có trên object. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Black Point` | `number` | Displacement vùng đen theo đơn vị chiều dài tài liệu; giá trị có dấu và hữu hạn. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `White Point` | `number` | Displacement vùng trắng theo đơn vị chiều dài tài liệu; giá trị có dấu và hữu hạn. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Initial Quality` | `number` | Mức subdivide/sample mesh ban đầu; thang quality do renderer xác lập. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Max Faces` | `number` | Số faces mục tiêu/tối đa của bước post reduction; semantics exact cần adapter xác minh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Fairing` | `number` | Số passes làm mượt displacement mesh. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Post Weld Angle` | `number` | Góc weld sau displacement, độ; không phải tolerance chiều dài. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Mesh Memory Limit` | `number` | Giới hạn bộ nhớ mesh, MB. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Refine Steps` | `number` | Số bước refine. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |
| `Refine Sensitivity` | `number` | 1 chia mọi vùng; .99 nhạy với contrast nhỏ; .01 chỉ contrast lớn; không đảo thang. | Chưa xác lập; hiển thị trạng thái thực tế và yêu cầu adapter xác minh. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-RENDER-028` và các vai trò Objects, Texture; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Render-mesh modifier pipe line sampling displacement theo channel, adaptive refine, memorybudget, fairing/reduction/weld; cachederivative mesh.
2. C++/Qt: đăng ký và điều phối native command `ApplyDisplacement`; chuyển kế hoạch Apply Displacement sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Giữ thao tác ở phạm vi trạng thái hoặc đầu ra đã chọn. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-RENDER-028`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-RENDER-028",
    effect: Effect::ViewState,
    roles: &[
        Role { name: "Objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "Texture", kind: InputKind::Image, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "On", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Texture", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Mapping Channel", kind: ParameterKind::Number, required: false },
        Parameter { name: "Black Point", kind: ParameterKind::Number, required: false },
        Parameter { name: "White Point", kind: ParameterKind::Number, required: false },
        Parameter { name: "Initial Quality", kind: ParameterKind::Number, required: false },
        Parameter { name: "Max Faces", kind: ParameterKind::Number, required: false },
        Parameter { name: "Fairing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Post Weld Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Mesh Memory Limit", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refine Steps", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refine Sensitivity", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Black Point âm cho displacement âm, edge highlights thật khác bump; Max Faces re spe ct/report, memory limit error giữ cache cũ, RemoveAll không đổi BRep.
- OM9-RENDER-028: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra chuyển chế độ hoặc hủy đầu ra không làm thay đổi hình học.

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
- [ ] Automated tests use feature ID `OM9-RENDER-028`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-RENDER-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-RENDER-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-RENDER-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-RENDER-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-RENDER](../../ENGINEERING_CONTRACTS.md#om9-render): kiểm chứng cho `OM9-RENDER-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-RENDER-028` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
