---
id: OM9-CURVE-031
name: Adjust Closed Curve Seam
command: CrvSeam
domain: 02-curve
module: Curve Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-CURVE-031 — Adjust Closed Curve Seam

Alias tương thích: `CrvSeam`. Nhóm: `02-curve`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-031` — Adjust Closed Curve Seam.**

Chọn một hoặc nhiều curve kín, kéo seam marker dọc từng curve và Enter. Hiển thị marker, đường nối giữa seam và mũi tên chiều; mục tiêu là đồng bộ seam/hướng để sweep ít twist. Flip đảo chiều curve; Automatic thử căn seam và hướng; Natural khôi phục seam lúc bắt đầu phiên. Chỉ đổi parameterization/hướng, không đổi hình curve.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `input_curves` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Flip` | `boolean` | Đảo hướng curve | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Automatic` | `boolean` | Thử căn seam | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Natural` | `boolean` | Phục hồi seam đầu phiên | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-031` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Reparameterize curve kín tại UV/parameter pick; giữ bản snapshot seam ban đầu cho Natural và preview.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-031` tới command native `CrvSeam` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-031",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "input_curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Flip", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Automatic", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Natural", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Di chuyển seam không đổi tập điểm hình học; Flip đảo hướng; Natural phục hồi seam; curve mở bị chặn.
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
- [ ] Automated tests use feature ID `OM9-CURVE-031`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CURVE-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CURVE-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CURVE-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CURVE-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CURVE-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-CURVE-031` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-CURVE-031` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
