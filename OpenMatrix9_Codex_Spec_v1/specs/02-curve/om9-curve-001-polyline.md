---
id: OM9-CURVE-001
name: PolyLine
command: Polyline
domain: 02-curve
module: Curve Tools
kind: command
implementation_status: partially_implemented
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-06'
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-CURVE-001 — PolyLine

Alias tương thích: `Polyline`. Nhóm: `02-curve`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-001` — PolyLine.**

Nhập chuỗi điểm có thứ tự để dựng một wire mở hoặc kín. Enter kết thúc; Close thêm đoạn về điểm đầu rồi kết thúc khi đã có ít nhất ba điểm. PersistentClose giữ đường kín trong lúc bổ sung điểm, bắt đầu từ hai điểm. AutoClose nhận điểm đầu khi con trỏ tiến gần; Alt tạm ngăn đóng tự động. Mode=Line dùng đoạn bậc một và là chế độ ban đầu được mô tả; Length ràng buộc chiều dài đoạn kế tiếp, chỉ hiện trong chế độ Line. Mode=Arc cho phép trộn cung và đoạn thẳng; Direction đặt tiếp tuyến và giữ hướng cho các cung tiếp theo tới khi đổi; Center lấy tâm để suy bán kính từ điểm đầu và chọn điểm cuối. Helpers bật dấu hướng tiếp tuyến/vuông góc. Đừng gộp Arc Steps hay Center Steps thành tham số số.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `picked_points` | `point` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `PersistentClose` | `boolean` | Giữ đường kín trong khi bổ sung điểm | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Close` | `boolean` | Đóng và kết thúc | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Mode` | `choice` | Line hoặc Arc | Line — hành vi khởi tạo được mô tả; mẫu operation plan không tự điền khi thiếu. |
| `Length` | `number` | Chiều dài đoạn kế tiếp theo đơn vị document | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Direction` | `reference` | Hướng tiếp tuyến cung | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Center` | `reference` | Tâm cung | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Helpers` | `boolean` | Chỉ dẫn tiếp tuyến và vuông góc | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-001` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Tạo từng edge theo chế độ của đoạn, ghép wire theo thứ tự và kiểm tra đầu nối trước khi tạo Part::Feature.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-001` tới command native `Polyline` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "PersistentClose", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Close", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Length", kind: ParameterKind::Number, required: false },
        Parameter { name: "Direction", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Center", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Helpers", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Ba điểm vuông góc tạo hai đoạn mở; Close tạo thêm đúng một đoạn. PersistentClose cập nhật cạnh đóng khi thêm điểm. Chế độ cung phải giữ tiếp tuyến đã chọn hoặc báo chưa hỗ trợ.
- Cancel ở giữa phiên không thay object/layer; lỗi solver không commit output một phần. Undo/Redo phục hồi toàn bộ thao tác; save/reload bảo toàn geometry, placement và links theo hợp đồng.
- Kiểm tra NaN/Infinity bị từ chối và UI cùng CMD cho cùng kết quả với cùng input; các options chưa có mặc định yêu cầu xác nhận giá trị đang hiển thị.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-06. **Scope:** Straight-segment Polyline, CMD/mouse, Close, PersistentClose, Length, Undo; arc modes remain unsupported.

### Inputs

Ordered world points from the shared CMD or native viewport picker. CMD x,y / x,y,z,
relative r-coordinates and unit suffixes use the active CPlane. Existing native
snaps and Ortho/Shift apply to mouse input. No selection is required.

### Parameters and defaults

OpenMatrix9 choices: millimetres; Mode=Line; PersistentClose=No. Supports Close,
PersistentClose=Yes/No, Length (positive, at least 1e-7 mm), Undo and Enter.
The host limit is 4096 picked points. Mode=Arc, Direction/Center arc construction,
Arc Steps and Helpers are not implemented. Original defaults remain TODO_EVIDENCE.

### Output

One native Part::Feature containing an open/closed polygon wire. At least two
distinct points are required; Close needs at least three. This is straight-segment
geometry, not an interpolated spline. Output is at the document root.

### Preview / commit / cancel

Picked points and hover segments are temporary unpickable scene geometry.
Enter commits; Close commits a closed polygon; Esc/Cancel discards the session.
Menu/CMD/mouse share the same handler. Pending Length blocks point picking until
a valid length is entered. Document replacement/switch/close or workbench exit
cancels. Native commit/cancel, invalid-input recovery and menu/CMD/mouse equivalence
are verified; exhaustive scene-node rendering behavior is not claimed.

### History / dependency model

Standalone snapshot geometry; no Builder, Styles or associative source History.
Successful command history is recorded after commit. One document transaction
supports native Undo/Redo and FCStd save/reload.

### Error / invalid-input behavior

Reject nonfinite or out-of-range coordinates (absolute value above 1e9 mm),
consecutive points closer than 1e-7 mm, invalid lengths, unsupported modes and
premature Close. Errors keep the command editable; cancellation creates no object.

### Verification and remaining work

Verified 2026-10-06: tests/curve_smoke.FCMacro passed 26/26 shared Line/Polyline
checks, including mouse/CMD equivalence, Length, Close, Esc, invalid input,
Undo/Redo, save/reload and document replacement. Rust curve session tests cover
continuations remain TODO_EVIDENCE; arc modes and full Rhino foundation acceptance
remain unverified. This is a validated supported slice, not the complete catalog feature.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented (OpenMatrix9 choices; giá trị tương thích còn chưa xác định).
- [x] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [x] History/dependency scope documented; native snapshot persistence tested.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-CURVE-001`.


Kiểm chứng đầy đủ hợp đồng và tương thích hình học vẫn chưa hoàn tất.
Checked items apply only to the supported implementation slice above.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CURVE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CURVE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CURVE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CURVE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CURVE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-CURVE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-CURVE-001` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
