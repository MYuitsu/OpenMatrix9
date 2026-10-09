---
id: OM9-BUILDER-001
name: Eternity Band Builder
command: gvEternity
domain: 08-builders
module: Builders
kind: builder
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-BUILDER-001 — Eternity Band Builder

Alias tương thích: `gvEternity`. Nhóm: `08-builders`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvEternity dùng Ring Rail, có +RingRail nếu thiếu, Start tạo preview nhẫn channel eternity. Edge Profile từ thư viện cập nhật khi hover và cho chỉnh/lưu profile riêng. Add Gems mở Gem Loader; một kích thước lấp channel đồng nhất, nhiều kích thước luân phiên theo Gem List; X xóa đá, mũi tên lên/xuống đổi thứ tự. Ring Width mm là bề ngang tổng; Channel Width là phần trăm kim loại che đá; Channel Depth mm là khoảng đá–Finger Rail; Extra Height mm là phần kim loại trên đá; Culet to Finger mm là khoảng culet–rail và tác động chiều dày/spacing; Gem Spacing mm là khe đá. Thanh trượt, CMD và các handle phải cùng giá trị. Draw Band tắt riêng band để xem đá; Reset về cấu hình mặc định đã xác minh; Styles lưu/nạp thiết kế.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `RingRail` | `curve` | 1 | 1 |
| `Profiles` | `curve` | 0 | Không đặt trong mẫu |
| `Gems` | `gem` | 0 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Ring Width` | `number` | Bề rộng band nhẫn. | Chưa xác định; không tự gán giá trị. |
| `Channel Width` | `number` | Phần trăm kim loại che đá, %.  | Chưa xác định; không tự gán giá trị. |
| `Culet to Finger` | `number` | Khoảng culet tới Finger Rail, mm. | Chưa xác định; không tự gán giá trị. |
| `Gem Spacing` | `number` | Khe giữa đá. | Chưa xác định; không tự gán giá trị. |
| `Draw Band` | `boolean` | Ẩn/hiện band để xem đá; không đổi layout đá. | Chưa xác định; không tự gán giá trị. |
| `Styles` | `reference` | Cấu hình builder từ thư viện style; không phải phép phối hợp độ cao. | Chưa xác định; không tự gán giá trị. |
| `Channel Depth` | `number` | Khoảng đá tới Finger Rail, mm. | Chưa xác định; không tự gán giá trị. |
| `Extra Height` | `number` | Phần kim loại trên đá, mm. | Chưa xác định; không tự gán giá trị. |
| `Edge Profile` | `reference` | Mặt cắt band từ thư viện/editor. | Chưa xác định; không tự gán giá trị. |
| `Gem List` | `reference` | Các cỡ đá theo thứ tự luân phiên. | Chưa xác định; không tự gán giá trị. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-BUILDER-001` và các vai trò RingRail, Profiles, Gems; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. Sweep Edge Profile theo ring rail, bố trí danh sách đá luân phiên và tính channel theo chiều đá; tái tính các kích thước phụ thuộc Culet to Finger.
2. C++/Qt: đăng ký và điều phối native command `gvEternity`; chuyển kế hoạch Eternity Band Builder sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-BUILDER-001`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-BUILDER-001",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "RingRail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "Profiles", kind: InputKind::Curve, min: 0, max: None },
        Role { name: "Gems", kind: InputKind::Gem, min: 0, max: None },
    ],
    parameters: &[
        Parameter { name: "Ring Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Channel Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Culet to Finger", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Spacing", kind: ParameterKind::Number, required: false },
        Parameter { name: "Draw Band", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Channel Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Extra Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Edge Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Gem List", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Danh sách hai cỡ A/B phải ra A/B luân phiên; đổi thứ tự/xóa cập nhật preview; Draw Band không xóa đá hay đổi layout.
- OM9-BUILDER-001: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-BUILDER-001`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-BUILDER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-BUILDER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-BUILDER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-BUILDER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-BUILDER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-BUILDER-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-TRANSFORM](../../ENGINEERING_CONTRACTS.md#om9-transform): kiểm chứng cho `OM9-BUILDER-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
