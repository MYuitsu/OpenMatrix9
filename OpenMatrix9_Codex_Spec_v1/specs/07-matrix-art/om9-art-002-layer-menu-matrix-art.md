---
id: OM9-ART-002
name: Layer Menu (Matrix Art)
command: null
domain: 07-matrix-art
module: Matrix Art
kind: menu
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-ART-002 — Layer Menu (Matrix Art)

Alias tương thích: `Chưa có alias command`. Nhóm: `07-matrix-art`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

Nhập toàn bộ đường kín, đồng phẳng trong Looking Down; gán màu layer trước khi nhập. Cùng màu dùng chung profile và Surface Height, mặc định 1 mm; miền kín lồng trong miền cùng màu tạo lỗ. Xếp lớp thấp từ Creation Yellow, Cutting Orange, Finger Brown, Heads Purple tới Metal 01 cao nhất. Chọn layer chủ động trước chỉnh profile; profile hiển thị một nửa và tự mirror, có thể kéo điểm, Edit Profile, Set on Axis, Auto Scale, Return Profile hoặc lưu vào thư viện. Sửa đường hay đổi màu phải nhập lại tất cả đường. Chọn layer cao và ô giao với layer thấp; chuột trái/phải đi tiến/lùi sáu quan hệ Build, Cut, Grid, Trim/Build, Trim/Cut, Trim/Grid. Build mặc định cộng chiều cao/profile lớp dưới; Cut bỏ ảnh hưởng lớp dưới; Grid cho mỗi lớp lên từ nền và chọn phần cao. Trim giới hạn lớp cao vào giao miền với lớp thấp.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Curves` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Surface Height` | `number` | Chiều cao nền của layer, mm. | 1 mm |
| `Profile` | `reference` | Profile nửa mặt cắt được mirror cho layer chủ động. | Chưa xác định; không tự gán giá trị. |
| `Layer Relation` | `choice` | Build, Cut, Grid hoặc ba biến thể Trim; quan hệ layer cao với layer thấp. | Build |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-ART-002` và các vai trò Curves; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Tạo đồ thị quan hệ layer có thứ tự và mask lỗ; tính profile phản chiếu theo từng layer, rồi kết hợp trường cao theo chế độ; tránh xem màu layer là vật liệu.
2. C++/Qt: đăng ký và điều phối native command `OM9-ART-002`; chuyển kế hoạch Layer Menu (Matrix Art) sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-ART-002`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-ART-002",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Curves", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Surface Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Layer Relation", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Hai miền cùng màu lồng nhau phải có lỗ; Build cộng cao trong giao miền, Cut không cộng, Trim chỉ giữ phần lớp cao trong lớp thấp; nhập lại cập nhật màu.
- OM9-ART-002: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-ART-002`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-ART-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-ART-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-ART-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-ART-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-ART-002` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
