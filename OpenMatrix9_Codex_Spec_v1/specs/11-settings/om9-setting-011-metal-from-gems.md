---
id: OM9-SETTING-011
name: Metal from Gems
command: gvMetalPiece
domain: 11-settings
module: Setting Menu
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SETTING-011 — Metal from Gems

Alias tương thích: `gvMetalPiece`. Nhóm: `11-settings`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

gvMetalPiece tạo metal/cutter có History từ gem line hoặc loop. Open modes Outside rim, Inside floor+side open ends, Top cutter, Round Corners rim, Pinched Ends rim, Outside Center rim+floor; closed modes Closed Outside rim, Closed Inside floor, Closed Top cutter không end controls. Top Profile library; Position: Profile Offset 0 tại girdle, âm trên/dương dưới; Gem Offset 0 match girdle, âm overlap/dương ra xa; Gem End Offset riêng ends. Dimensions Cap Height giữa top và rectangular side profile, Profile Depth/Width/End Width mm. Corner Heights/Widths Bottom/Top Inside/Outside đổi bốn corners, top height âm lên; Corner End Widths cho end section. Round Corners mới End Curvature. Z tiến level/X lùi; advanced không sliders gồm End Adjustments One, Top, Side, Full (16 handle sets). Mirror control hai ends chung; off bộ riêng. Extension Type Line mặc định, Arc, Smooth; Zero End, Styles, Reset. CMD Tweak Corners nới góc tránh pinching; Distance Girdle/Size đổi cách đo; Corner Angle thêm profiles để đều surface.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `Gems` | `gem` | 1 | Không đặt trong mẫu |
| `Profile` | `curve` | 0 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Mode` | `choice` | Outside, Inside, Top, Round Corners, Pinched Ends hoặc Outside Center; curve khép chỉ Outside/Inside/Top | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Top Profile` | `reference` | Profile phần đỉnh metal | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile Offset` | `number` | 0 ở girdle; âm lên trên, dương xuống dưới Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Gem Offset` | `number` | 0 khớp girdle; âm chồng vào gem, dương ra ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Gem End Offset` | `number` | Độ lệch riêng phần cuối Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Cap Height` | `number` | Chiều cao cap Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile Depth` | `number` | Chiều sâu profile Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile Width` | `number` | Bề rộng profile Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile End Width` | `number` | Bề rộng profile ở đầu cuối metal Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bottom Inside Height` | `number` | Cao góc đáy trong Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bottom Outside Height` | `number` | Cao góc đáy ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Top Inside Height` | `number` | Cao góc đỉnh trong Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Top Outside Height` | `number` | Cao góc đỉnh ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bottom Inside Width` | `number` | Rộng góc đáy trong Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Bottom Outside Width` | `number` | Rộng góc đáy ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Top Inside Width` | `number` | Rộng góc đỉnh trong Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Top Outside Width` | `number` | Rộng góc đỉnh ngoài Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Curvature` | `number` | Độ cong cuối, chỉ Round Corners; đơn vị chưa xác lập Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `End Adjustments` | `choice` | One, Top, Side hoặc Full cho cấp điều khiển cuối | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Mirror` | `boolean` | Liên kết hai đầu | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Extension Type` | `choice` | Line, Arc hoặc Smooth | Line |
| `Tweak Corners` | `boolean` | Nới corner để giảm pinching | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Corner Angle` | `number` | Điều chỉnh chuyển tiếp góc; ánh xạ/đơn vị cụ thể cần xác nhận Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Distance` | `choice` | Girdle hoặc Size để chọn cách tính khoảng cách | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Styles` | `reference` | Tham chiếu bộ cấu hình builder; codec và cách lưu cần được adapter xác nhận. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile Corner End Widths: Bottom Inside` | `number` | Vị trí điểm Bottom Inside của end profile; chỉ bố trí mở có end controls Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile Corner End Widths: Bottom Outside` | `number` | Vị trí điểm Bottom Outside của end profile; chỉ bố trí mở có end controls Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile Corner End Widths: Top Inside` | `number` | Vị trí điểm Top Inside của end profile; chỉ bố trí mở có end controls Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |
| `Profile Corner End Widths: Top Outside` | `number` | Vị trí điểm Top Outside của end profile; chỉ bố trí mở có end controls Đơn vị mm. Mẫu chỉ kiểm tra số hữu hạn; ràng buộc hình học xử lý tại adapter. | Chưa xác lập; mẫu giữ giá trị được nhập hoặc trạng thái hiện hành, không tự điền. |

## Hướng dẫn triển khai

1. Rust: tạo trạng thái riêng cho `OM9-SETTING-011` và các vai trò Gems, Profile; phân biệt đầu vào bị thiếu với tùy chọn chưa đặt. State machine mode/level có scoped params, geometry sweep profile theo gem graph; ends chỉ open; History gems→metal có units/signed offsets rõ.
2. C++/Qt: đăng ký và điều phối native command `gvMetalPiece`; chuyển kế hoạch Metal from Gems sang adapter FreeCAD theo phép toán đã nêu; kiểm tra kết quả và hiển thị lỗi theo bước. Dựng đối tượng xem trước ngoài tài liệu, rồi commit trong một transaction; Cancel bỏ bản xem trước. Giữ command disabled nếu adapter cần thiết chưa có.
3. Python: chỉ đăng ký workbench và cung cấp fixture tích hợp cho `OM9-SETTING-011`; menu dùng bộ điều phối command native của C++/Qt.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SETTING-011",
    effect: Effect::InteractivePreview,
    roles: &[
        Role { name: "Gems", kind: InputKind::Gem, min: 1, max: None },
        Role { name: "Profile", kind: InputKind::Curve, min: 0, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Top Profile", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Profile Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Gem End Offset", kind: ParameterKind::Number, required: false },
        Parameter { name: "Cap Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Depth", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile End Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Inside Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Outside Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Inside Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Outside Height", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Inside Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Bottom Outside Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Inside Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "Top Outside Width", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Curvature", kind: ParameterKind::Number, required: false },
        Parameter { name: "End Adjustments", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Mirror", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Extension Type", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Tweak Corners", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Corner Angle", kind: ParameterKind::Number, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Styles", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Profile Corner End Widths: Bottom Inside", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Corner End Widths: Bottom Outside", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Corner End Widths: Top Inside", kind: ParameterKind::Number, required: false },
        Parameter { name: "Profile Corner End Widths: Top Outside", kind: ParameterKind::Number, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Move gem cập nhật metal; Closed bỏ end controls, Top ra cutter; Profile Offset âm lên; Mirror off cho hai ends riêng, advanced Full đủ 16 sets khi support.
- OM9-SETTING-011: kiểm tra đầu vào sai vai trò, thiếu lựa chọn, NaN và vô cực; lỗi phải giữ nguyên tài liệu. Kiểm tra Cancel, Undo/Redo và lưu/mở lại cấu hình cùng liên kết đối tượng theo capability đã nêu.

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
- [ ] Automated tests use feature ID `OM9-SETTING-011`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SETTING-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SETTING-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SETTING-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SETTING-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SETTING-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SETTING-011` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SOLID](../../ENGINEERING_CONTRACTS.md#om9-solid): kiểm chứng cho `OM9-SETTING-011` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
