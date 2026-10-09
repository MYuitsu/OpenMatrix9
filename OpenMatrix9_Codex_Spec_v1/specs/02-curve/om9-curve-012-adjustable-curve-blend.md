---
id: OM9-CURVE-012
name: Adjustable Curve Blend
command: BlendCrv
domain: 02-curve
module: Curve Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-CURVE-012 — Adjustable Curve Blend

Alias tương thích: `BlendCrv`. Nhóm: `02-curve`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-012` — Adjustable Curve Blend.**

Chọn curve hoặc edge, có thể Point để blend tới điểm; Curves là bộ lọc ban đầu được mô tả và khôi phục sau Point/Edges. Chọn từng handle kéo preview, Shift sửa đối xứng, Alt đổi góc của nhánh shape khỏi vuông góc edge. Continuity Curve 1/2 độc lập: G0 vị trí, G1 hướng, G2 độ cong; G3/G4 đòi ràng buộc đạo hàm cao và cần bộ giải kiểm chứng. Reset trả bulge về lúc bắt đầu; Flip 1/2 chọn lại chiều/đầu, Trim cắt input, Join gộp đầu ra, Show Curvature hiện đồ thị. History phải theo input khi dịch/chuyển hướng.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `input_curves_or_edges` | `curve` | 2 | 2 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `SelectionMode` | `choice` | Curves, Edges hoặc Point | Curves — bộ lọc ban đầu được mô tả; mẫu operation plan không tự điền khi thiếu. |
| `Continuity Curve 1` | `choice` | G0, G1, G2, G3 hoặc G4 | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Continuity Curve 2` | `choice` | G0, G1, G2, G3 hoặc G4 | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Reset` | `boolean` | Khôi phục bulge | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Flip 1` | `boolean` | Đảo hướng input đầu | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Flip 2` | `boolean` | Đảo hướng input sau | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Trim` | `boolean` | Cắt input | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Join` | `boolean` | Ghép kết quả | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Show Curvature` | `boolean` | Hiện curvature graph | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-012` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Lưu jet mỗi đầu và handle bulge trong Rust; native solver tạo preview không ghi document, recompute dùng links/endpoint ổn định.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-012` tới command native `BlendCrv` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-012",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves_or_edges", kind: InputKind::Curve, min: 2, max: Some(2) },
    ],
    parameters: &[
        Parameter { name: "SelectionMode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Continuity Curve 1", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Continuity Curve 2", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Reset", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip 1", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Flip 2", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Trim", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Join", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Show Curvature", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Shift cho biến đổi handle đối xứng; Reset phục hồi bulge; Flip chỉ đảo đầu tương ứng; Trim/Join xảy ra cùng transaction lúc xác nhận.
- Cancel ở giữa phiên không thay object/layer; lỗi solver không commit output một phần. Undo/Redo phục hồi toàn bộ thao tác; save/reload bảo toàn geometry, placement và links theo hợp đồng.
- Kiểm tra NaN/Infinity bị từ chối và UI cùng CMD cho cùng kết quả với cùng input; các options chưa có mặc định yêu cầu xác nhận giá trị đang hiển thị.

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
- [ ] Automated tests use feature ID `OM9-CURVE-012`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CURVE-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CURVE-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CURVE-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CURVE-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CURVE-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-CURVE-012` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-CURVE-012` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
