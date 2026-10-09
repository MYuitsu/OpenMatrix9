---
id: OM9-MATRIXTOOLS-001
name: Smart Pattern
command: gvSmartPattern
domain: 16-matrix-tools
module: Matrix Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-MATRIXTOOLS-001 — Smart Pattern

Alias tương thích: `gvSmartPattern`. Nhóm: `16-matrix-tools`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Smart Pattern bố trí một hoặc nhiều object trên surface. Gán surface đích và các object mẫu, hoặc chọn mẫu từ Browse Pattern; chỉnh preview rồi Go để tạo kết quả. Reset xóa các input đang gán. Group Output gom các bản sao khi commit; Output Mesh yêu cầu đầu ra mesh, Flip đảo hướng theo normal. Columns và Rows điều khiển số lần lặp theo hai chiều surface; Z Offset dịch theo normal. Scale X/Y/Z điều chỉnh từng trục trong khoảng điều khiển 0.2–3; Rotate X/Y/Z dùng độ với khoảng điều khiển −180..180. Handle, slider và ô nhập phải cập nhật cùng một trạng thái.

Các layout được phân biệt: Normal, Hex, OverlapHex, OverlapCol, OverlapRow, OverlapBoth, Random, AttractCurve và SkipSections. Towards chỉ có nghĩa ở AttractCurve, kết hợp độ mạnh Attractor và các curve hút để dịch hướng tới/ra xa. Skip Columns/Rows cùng Start Column/Row dùng cho bố trí bỏ nhịp; Pattern Scale/Move U/V biến đổi layout trong miền tham số. Chính sách mẫu OM9 yêu cầu Columns/Rows và các chỉ số Skip/Start là số nguyên; điều này chưa xác nhận mọi phiên bản tương thích đều từ chối giá trị lẻ. Distance dùng kích thước Width/Height thay cách đặt theo số lượng. Direction cho phép đảo U, V hoặc cả hai. AutoTrim và Relax cần adapter riêng; quy tắc chính xác của hai nhánh này chưa xác định, không bật chỉ vì có tên option. Surface nhiều face, singularity, trims, nối seam và phân phối Random cần nghiệm thu riêng; không xem phép dựng trên một face như hỗ trợ toàn polysurface.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `target_surface` | `surface` | 1 | 1 |
| `pattern_objects` | `object` | 1 | Không đặt trong mẫu |
| `attractor_curves` | `curve` | 0 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `PatternType` | `choice` | Normal/Hex/OverlapHex/OverlapCol/OverlapRow/OverlapBoth/Random/AttractCurve/SkipSections; adapter kiểm tra capability từng mode. | Chưa xác định; không tự điền. |
| `Columns` | `number` | Số cột, yêu cầu nguyên dương; khoảng điều khiển 1..50. | Chưa xác định; không tự điền. |
| `Rows` | `number` | Số hàng, yêu cầu nguyên dương; khoảng điều khiển 1..50. | Chưa xác định; không tự điền. |
| `ZOffset` | `number` | Dịch theo normal, mm; điều khiển −5..5. | Chưa xác định; không tự điền. |
| `ScaleX` | `number` | Tỷ lệ theo trục X, không đơn vị; điều khiển 0.2..3. | Chưa xác định; không tự điền. |
| `ScaleY` | `number` | Tỷ lệ theo trục Y, không đơn vị; điều khiển 0.2..3. | Chưa xác định; không tự điền. |
| `ScaleZ` | `number` | Tỷ lệ theo trục Z, không đơn vị; điều khiển 0.2..3. | Chưa xác định; không tự điền. |
| `RotateX` | `number` | Góc theo trục X (độ); điều khiển −180..180. | Chưa xác định; không tự điền. |
| `RotateY` | `number` | Góc theo trục Y (độ); điều khiển −180..180. | Chưa xác định; không tự điền. |
| `RotateZ` | `number` | Góc theo trục Z (độ); điều khiển −180..180. | Chưa xác định; không tự điền. |
| `GroupOutput` | `boolean` | Nhóm các object đã commit. | Chưa xác định; không tự điền. |
| `OutputMesh` | `boolean` | Tạo mesh thay kết quả BRep; adapter phải xác định tessellation. | Chưa xác định; không tự điền. |
| `Flip` | `boolean` | Đảo chiều tương đối normal surface. | Chưa xác định; không tự điền. |
| `Towards` | `boolean` | AttractCurve: chọn hướng tới/ra xa curve hút. | Chưa xác định; không tự điền. |
| `Attractor` | `number` | Cường độ dịch theo curve hút; quy tắc/đơn vị chưa xác lập. | Chưa xác định; không tự điền. |
| `SkipColumns` | `number` | Chỉ số nhịp/hàng/cột nguyên cho SkipSections; không dùng ở mode khác. | Chưa xác định; không tự điền. |
| `SkipRows` | `number` | Chỉ số nhịp/hàng/cột nguyên cho SkipSections; không dùng ở mode khác. | Chưa xác định; không tự điền. |
| `StartColumn` | `number` | Chỉ số nhịp/hàng/cột nguyên cho SkipSections; không dùng ở mode khác. | Chưa xác định; không tự điền. |
| `StartRow` | `number` | Chỉ số nhịp/hàng/cột nguyên cho SkipSections; không dùng ở mode khác. | Chưa xác định; không tự điền. |
| `PatternScaleU` | `number` | Tỷ lệ layout theo U; không đơn vị. | Chưa xác định; không tự điền. |
| `PatternScaleV` | `number` | Tỷ lệ layout theo V; không đơn vị. | Chưa xác định; không tự điền. |
| `PatternMoveU` | `number` | Dịch layout theo U; đơn vị miền UV cần adapter xác lập. | Chưa xác định; không tự điền. |
| `PatternMoveV` | `number` | Dịch layout theo V; đơn vị miền UV cần adapter xác lập. | Chưa xác định; không tự điền. |
| `Distance` | `boolean` | Chọn bố trí dựa trên Width/Height. | Chưa xác định; không tự điền. |
| `Width` | `number` | Khoảng layout khi Distance bật; units cần chốt. | Chưa xác định; không tự điền. |
| `Height` | `number` | Khoảng layout khi Distance bật; units cần chốt. | Chưa xác định; không tự điền. |
| `Direction` | `choice` | None/ReverseU/ReverseV/ReverseUV. | Chưa xác định; không tự điền. |
| `AutoTrim` | `boolean` | Nhánh cắt theo biên; behavior chưa xác định. | Chưa xác định; không tự điền. |
| `Relax` | `boolean` | Nhánh điều chỉnh layout; behavior chưa xác định. | Chưa xác định; không tự điền. |

## Hướng dẫn triển khai

1. Rust tạo session giữ thứ tự object, surface identity, layout mode và các tham số; kiểm tra số nguyên, units và nhánh mode phù hợp trước solver.
2. C++ resolve face/world placement và xây frame UV/normal. Đặt từng bản sao bằng transform mới, kiểm tra trims/seam và orientation; không biến object input tại chỗ.
3. Attractor phải được chiếu/ánh xạ vào frame tương ứng. Random cần seed/reproducibility policy riêng; nếu chưa có quy tắc định lượng thì để unsupported.
4. Tạo preview ngoài document. Go kiểm tra lại revision, ghi output/group/metadata trong một transaction; mesh conversion thất bại phải abort toàn bộ.
5. Đăng ký command/menu/CMD trong C++ native; Rust giữ request/state. Python đăng ký workbench và fixture. Giữ nhánh thiếu adapter disabled/unsupported.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-MATRIXTOOLS-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "target_surface", kind: InputKind::Surface, min: 1, max: Some(1) },
        Role { name: "pattern_objects", kind: InputKind::Object, min: 1, max: None },
        Role { name: "attractor_curves", kind: InputKind::Curve, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "PatternType", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Columns", kind: ParameterKind::Number, required: false },
        Parameter { name: "Rows", kind: ParameterKind::Number, required: false },
        Parameter { name: "ZOffset", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleX", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleY", kind: ParameterKind::Number, required: false },
        Parameter { name: "ScaleZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotateX", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotateY", kind: ParameterKind::Number, required: false },
        Parameter { name: "RotateZ", kind: ParameterKind::Number, required: false },
        Parameter { name: "GroupOutput", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "OutputMesh", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Towards", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Attractor", kind: ParameterKind::Number, required: false },
        Parameter { name: "SkipColumns", kind: ParameterKind::Number, required: false },
        Parameter { name: "SkipRows", kind: ParameterKind::Number, required: false },
        Parameter { name: "StartColumn", kind: ParameterKind::Number, required: false },
        Parameter { name: "StartRow", kind: ParameterKind::Number, required: false },
        Parameter { name: "PatternScaleU", kind: ParameterKind::Number, required: false },
        Parameter { name: "PatternScaleV", kind: ParameterKind::Number, required: false },
        Parameter { name: "PatternMoveU", kind: ParameterKind::Number, required: false },
        Parameter { name: "PatternMoveV", kind: ParameterKind::Number, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Choice, required: false },
        Parameter { name: "AutoTrim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Relax", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Surface phẳng 3×2 cho đúng số instance và orientation; Flip đảo normal, ZOffset có dấu đúng.
- Từng mode/skip và Direction có fixture riêng; object trong hierarchy giữ world placement.
- Surface trimmed/degenerate và số lượng không nguyên bị xử lý rõ; Cancel không tạo object, Undo/Redo và FCStd giữ grouping/metadata.

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
- [ ] Automated tests use feature ID `OM9-MATRIXTOOLS-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-MATRIXTOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-MATRIXTOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-MATRIXTOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-MATRIXTOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-MATRIXTOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-MATRIXTOOLS-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-MATRIXTOOLS-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
