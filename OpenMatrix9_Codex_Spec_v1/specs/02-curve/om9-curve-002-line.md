---
id: OM9-CURVE-002
name: Line
command: Line
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

# OM9-CURVE-002 — Line

Alias tương thích: `Line`. Nhóm: `02-curve`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-CURVE-002` — Line.**

Dựng một đoạn qua điểm đầu và điểm cuối. BothSides dùng điểm đầu làm trung điểm nên độ dài đầy đủ gấp đôi khoảng kéo. Normal chọn surface, điểm trên surface và chiều dài theo pháp tuyến; IgnoreTrims quyết định miền được phép đặt điểm. Angled dùng hai điểm của hướng tham chiếu và góc xoay; Vertical theo pháp tuyến CPlane. FourPoint tách cặp điểm xác định hướng khỏi cặp xác định chiều dài; Bisector dùng đỉnh và hai nhánh góc. Perpendicular và Tangent chọn curve cùng vị trí bắt đầu; Point cho phép điểm ngoài curve; PointOnCurve giữ điểm trên curve; FromFirstPoint khóa điểm đầu thay vì cho trượt, chỉ có với BothSides/PointOnCurve. 2Curves đòi bộ giải hai curve; điều kiện của nhánh Tangent còn cần xác nhận vì mô tả không nhất quán. Extension tạo phần kéo dài bằng đoạn từ đầu được chọn. Menu chuột phải kích hoạt BothSides.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `picked_points` | `point` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `BothSides` | `boolean` | Điểm đầu thành trung điểm | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Normal` | `boolean` | Dùng pháp tuyến surface | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `IgnoreTrims` | `boolean` | Cho pick trên surface cơ sở ngoài trim | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Angled` | `number` | Góc so với hướng tham chiếu, độ ở UI | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Vertical` | `boolean` | Theo pháp tuyến CPlane | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `FourPoint` | `boolean` | Tách hướng và chiều dài | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Bisector` | `boolean` | Theo phân giác | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Perpendicular` | `boolean` | Ràng buộc vuông góc curve | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Tangent` | `boolean` | Ràng buộc tiếp tuyến | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `FromFirstPoint` | `boolean` | Khóa điểm bắt đầu | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `2Curves` | `boolean` | Dùng ràng buộc hai curve | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `Extension` | `boolean` | Tạo đoạn kéo dài | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo `OM9-CURVE-002` với phase và options đã nêu; chuẩn hóa selection role theo từng nhánh, giữ endpoint/orientation/UV và từ chối NaN/Infinity.
2. C++/Qt–Part: Tách các bài toán hai điểm, pháp tuyến, góc, phân giác, tiếp tuyến và vuông góc thành solver độc lập; trả edge và chẩn đoán vô nghiệm.
3. Tách preview khỏi object document; giữ source placement, layer và liên kết cần thiết. Tính lại khi input đổi chỉ ở nhánh History đã khai báo; ghi mọi thay thế/xóa/tạo trong một transaction.
4. C++/Qt: ánh xạ `OM9-CURVE-002` tới command native `Line` và cùng handler cho menu/chuột phải/CMD. Python chỉ hỗ trợ workbench/fixture kiểm tra, không đăng ký lại command. Vô hiệu nhánh thiếu adapter; kiểm tra runtime theo ID.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-CURVE-002",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "picked_points", kind: InputKind::Point, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "BothSides", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Normal", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "IgnoreTrims", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Angled", kind: ParameterKind::Number, required: false },
        Parameter { name: "Vertical", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "FourPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Bisector", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Perpendicular", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "FromFirstPoint", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "2Curves", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Extension", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- BothSides qua (0,0,0) với điểm kéo (5,0,0) tạo chiều dài 10; Normal trên mặt phẳng nghiêng song song pháp tuyến; cặp curve không có nghiệm phải giữ phiên nhập.
- Cancel ở giữa phiên không thay object/layer; lỗi solver không commit output một phần. Undo/Redo phục hồi toàn bộ thao tác; save/reload bảo toàn geometry, placement và links theo hợp đồng.
- Kiểm tra NaN/Infinity bị từ chối và UI cùng CMD cho cùng kết quả với cùng input; các options chưa có mặc định yêu cầu xác nhận giá trị đang hiển thị.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-06. **Scope:** Two-point Line, CMD/mouse, BothSides, shared CPlane/snaps/Ortho; specialist construction modes remain unsupported.

### Inputs

Two ordered world points from CMD or the native viewport picker. CMD coordinates,
relative coordinates and unit suffixes use the active CPlane. Existing mouse
snaps and Ortho/Shift rules apply. No selected input objects are required.

### Parameters and defaults

OpenMatrix9 choices: millimetres; BothSides=No. BothSides=Yes/No is accepted before
the first point. When enabled, the first point is the midpoint and the second
defines one end; the opposite end is reflected about the midpoint. Normal,
IgnoreTrims, Angled, Vertical, FourPoint and Bisector modes are unsupported.
Original defaults/ranges remain TODO_EVIDENCE.

### Output

One root-level Part::Feature containing one straight native edge. Coordinates
must be finite and within 1e9 mm; endpoints must differ by at least 1e-7 mm.
It is an open curve with no solid/surface or associative dependencies.

### Preview / commit / cancel

The second valid point commits immediately. Menu, CMD and actual viewport clicks
share the same Rust session. Esc/Cancel, document replacement/switch/close and
workbench exit cancel without geometry. A parallel CPlane/view ray reports an
editable error and permits subsequent CMD input. A live rubber-band Line preview
is not implemented; commit/cancel is verified.

### History / dependency model

Snapshot geometry, without Builder/Styles/associative History. One native document
transaction supports Undo/Redo; FCStd stores the native edge. Success history is
recorded only after geometry commit.

### Error / invalid-input behavior

Nonfinite/out-of-range coordinates, coincident endpoints, malformed coordinates
and unsupported options create no output and keep the current session editable.
Cancel clears pending points and constraints.

### Verification and remaining work

Verified 2026-10-06: tests/curve_smoke.FCMacro passed 26/26 shared Line/Polyline
checks, including independent mouse ray projection, 3-4-5 line length, menu/CMD
equivalence, Esc, Undo/Redo, save/reload and document lifecycle. Rust session tests
continuations, specialist construction modes and full Rhino foundation acceptance
remain TODO_EVIDENCE/unverified. This is a validated supported slice.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented (OpenMatrix9 choices; giá trị tương thích còn chưa xác định).
- [x] Output geometry documented.
- [ ] Preview/commit/cancel behavior tested if applicable.
- [x] History/dependency scope documented; native snapshot persistence tested.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-CURVE-002`.


Kiểm chứng đầy đủ hợp đồng và tương thích hình học vẫn chưa hoàn tất.
Checked items apply only to the supported implementation slice above.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-CURVE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-CURVE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-CURVE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-CURVE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-CURVE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-CURVE-002` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-CURVE-002` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
