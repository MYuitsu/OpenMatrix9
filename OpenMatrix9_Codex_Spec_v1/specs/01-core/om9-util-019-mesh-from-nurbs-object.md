---
id: OM9-UTIL-019
name: Mesh from NURBS Object
command: null
domain: 01-core
module: Utilities
kind: tool_or_workflow
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-UTIL-019 — Mesh from NURBS Object

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-UTIL-019` — Mesh from NURBS Object.**

Mesh chuyển NURBS surfaces/polysurfaces thành mesh object với preview và options; giữ source theo policy explicit. Joined edges cần coincident vertices, valid solid phải kiểm tra watertight output. Density 0–1 điều khiển count, nhưng formula host không có nên OM9 map có nhãn; Maximum Angle giữa normals, Maximum Aspect Ratio ban đầu quads với 0 no-limit (khởi tạo mô tả 0), Minimum Edge Length 0 ngừng giới hạn (số default được capture mơ hồ 0.0001/0 nên chưa chốt), Maximum Edge Length và Maximum Distance Edge to Surface 0 no-limit (khởi tạo 0), Minimum Initial Grid Quads khởi tạo 16, đề xuất dải tương thích không thành range validation tự động. Angle khởi tạo 20 độ theo mô tả. Refine Mesh recursive theo các giới hạn; Off giữ grid quads vùng không trim. Jagged Seams mesh faces độc lập không stitch, có thể hở; Off cần weld shared edges. Simple Planes triangulate ít polygons và bỏ hầu hết density/refine controls trừ Jagged Seams. Pack Textures chia unit square thành UV regions riêng mỗi face. Mesh không giữ mọi khả năng edit NURBS; không tự thay source hoặc cam kết toàn BRep operation cho mesh.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `selected_objects` | `object` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Density` | `number` | Map chất lượng không đơn vị | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Maximum Angle` | `number` | Góc normals, độ tại UI, 0 tắt | 20 độ — khởi tạo tương thích được mô tả, map kernel cần explicit. |
| `Maximum Aspect Ratio` | `number` | Tỷ số quads, 0 no-limit | 0 — no-limit được mô tả. |
| `Minimum Edge Length` | `number` | Chiều dài theo document, 0 tắt | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Maximum Edge Length` | `number` | Chiều dài theo document, 0 tắt | 0 — tắt giới hạn được mô tả. |
| `Maximum Distance Edge to Surface` | `number` | Lệch midpoint theo document, 0 tắt | 0 — tắt giới hạn được mô tả. |
| `Minimum Initial Grid Quads` | `number` | Số quads khởi tạo | 16 — khởi tạo tương thích được mô tả. |
| `Refine Mesh` | `boolean` | Refinement theo criteria | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Jagged Seams` | `boolean` | Không stitch shared edges | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Simple Planes` | `boolean` | Minimal triangulation planes | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Pack Textures` | `boolean` | UV packing | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Preview` | `boolean` | Tính mesh preview | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-UTIL-019`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust meshing options/presets và explicit disable semantics; native tessellator adaptive theo deviation/angle/length với shared-edge welding, UV packing independent; render cache không dùng như geometry mesh mặc định.
3. Tích hợp `Mesh from NURBS Object` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-UTIL-019",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "selected_objects", kind: InputKind::Object, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Density", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maximum Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maximum Aspect Ratio", kind: ParameterKind::Number, required: false },
        Parameter { name: "Minimum Edge Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maximum Edge Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Maximum Distance Edge to Surface", kind: ParameterKind::Number, required: false },
        Parameter { name: "Minimum Initial Grid Quads", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refine Mesh", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Jagged Seams", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Simple Planes", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Pack Textures", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Solid box tạo watertight mesh khi Jagged Seams Off; holes trimmed giữ holes; Simple Planes giữ biên; Density map tăng chi tiết; 0 options không divide vô hạn.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

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
- [ ] Automated tests use feature ID `OM9-UTIL-019`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-UTIL-019` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
