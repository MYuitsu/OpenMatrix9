---
id: OM9-SURFACE-003
name: Sweep 2
command: Sweep2
domain: 03-surface
module: Surface Tools
kind: command
implementation_status: partially_implemented
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SURFACE-003 — Sweep 2

Alias tương thích: `Sweep2`. Nhóm: `03-surface`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Nhận rail thứ nhất, rail thứ hai rồi section theo thứ tự để dựng surface giữa hai rail. Flip đảo chiều section; Automatic thử đồng bộ seam và hướng; Natural phục hồi seam lúc bắt đầu. Section phải đồng loạt mở hoặc đóng; giữ thứ tự và chọn gần cùng đầu để tránh xoắn. Chain Edges ghép cạnh cho từng rail; Point chỉ đầu/cuối. Preview làm mới theo thay đổi. Continuity chỉ khả dụng với rail là cạnh mặt và section non-rational có trọng số bằng một, đủ cấu trúc điểm.

### Tùy chọn và tương tác

- `Cross-Section Mode`: Do Not Simplify mặc định; Rebuild hoặc Refit như Sweep1
- `Rebuild PointCount`: số điểm section
- `Refit Tolerance`: dung sai mm
- `Preserve First/Last Shape`: bật/tắt từng đầu; buộc khớp section đầu/cuối khi rail là cạnh mặt
- `Maintain Height`: bật/tắt; bật giữ chiều cao khi khoảng cách hai rail đổi, tắt tỷ lệ cao theo rộng
- `Rail A/B Continuity`: Position G0, Tangency G1, Curvature G2 riêng từng rail
- `Closed Sweep`: bật/tắt; khả dụng từ hai section, nối cuối về đầu
- `Simple Sweep`: tạo mặt đơn giản khi section giao edit point rail
- `Add Slash`: thêm cặp điểm trên hai rail để điều khiển tương ứng giữa section

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `rails` | `curve` | 2 | 2 |
| `sections` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Cross-Section Mode` | `choice` | Do Not Simplify mặc định; Rebuild hoặc Refit như Sweep1 | Do Not Simplify |
| `Rebuild PointCount` | `number` | số điểm section | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Refit Tolerance` | `number` | dung sai mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Preserve First Shape` | `boolean` | bật/tắt từng đầu; buộc khớp section đầu/cuối khi rail là cạnh mặt Giá trị độc lập của Preserve First Shape. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Preserve Last Shape` | `boolean` | bật/tắt từng đầu; buộc khớp section đầu/cuối khi rail là cạnh mặt Giá trị độc lập của Preserve Last Shape. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Maintain Height` | `boolean` | bật/tắt; bật giữ chiều cao khi khoảng cách hai rail đổi, tắt tỷ lệ cao theo rộng | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Rail A Continuity` | `choice` | Position G0, Tangency G1, Curvature G2 riêng từng rail Giá trị độc lập của Rail A Continuity. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Rail B Continuity` | `choice` | Position G0, Tangency G1, Curvature G2 riêng từng rail Giá trị độc lập của Rail B Continuity. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Closed Sweep` | `boolean` | bật/tắt; khả dụng từ hai section, nối cuối về đầu | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Simple Sweep` | `boolean` | tạo mặt đơn giản khi section giao edit point rail | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Add Slash` | `reference` | thêm cặp điểm trên hai rail để điều khiển tương ứng giữa section | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SURFACE-003 và phiên chọn có thứ tự, kiểm tra vai trò rails, sections; chuẩn hóa tùy chọn riêng của Sweep 2.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Sweep 2 trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
4. Preview nếu cần dùng dữ liệu tạm; xác nhận ghi kết quả và dependency trong một transaction. Hủy/lỗi không tạo output dở dang; Undo/Redo và save/reload giữ contract.
5. Native C++ đăng ký command/menu và dispatch sang Rust/native geometry. Python đăng ký workbench và hỗ trợ fixtures/macro kiểm chứng. Capability chưa có hoặc chưa kiểm chứng phải giữ disabled với lý do cụ thể.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-SURFACE-003",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rails", kind: InputKind::Curve, min: 2, max: Some(2) },
        Role { name: "sections", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Cross-Section Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rebuild PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refit Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preserve First Shape", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Preserve Last Shape", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Maintain Height", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rail A Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rail B Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Closed Sweep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Simple Sweep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Add Slash", kind: ParameterKind::Reference, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Fixture hai rail có độ tách tăng: Maintain Height giữ cao khi rộng thay đổi; đo khớp rail và section; continuity phải kiểm tra riêng rail A/B.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Scope:** Native Sweep2 with bounded support-edge G1/G2, section Refit, one-profile/open-rail Add Slash, optional History and earlier options; constraint combinations and multiple-profile Maintain Height remain unsupported.

Menu ID: `OM9_SurfaceSweepSweep2Rails`. CMD alias: `Sweep2`.
Verified supported slice: 2026-10-09; full feature remains partially implemented.

### Reference and implementation contract

Matrix 8 Book 1 PDF pp.184–186 (printed pp.174–176) were read through the
Sweep 2 continuation, including the Maintain Height illustration.
Shares [Sweep 1](../../../docs/features/OM9-SURFACE-001.md)'s ordered selection, Chain Edges,
Reverse/seams, Automatic/Natural, section Rebuild, Preview and lifecycle contract,
with two distinct rails before profiles. Links retain every rail component and
section, including parent world placement.

The matching SDK's ContactOnBorder path fails on minimal fixtures and is not
used. These strategies and bounds are explicit OM9 decisions:

- One profile must intersect both rail starts; rails share closure state.
  Sample 65 equivalent arc-length stations on each rail; Rust computes frames
  using rail separation and primary rail tangent. Width scales with separation.
  Maintain Height ON preserves the section normal coordinate; OFF scales it
  with width. Initial host choice is OFF. Tangent coordinate stays unchanged.
  Native loft joins these transported copies, degree at most three. Closed
  rails omit the duplicate final station and use a closed loft.
  Dense curved-section PipeShell produced invalid geometry on this SDK;
  lofting the same sections passed contact and crown-height checks.
- Multiple profiles use the native auxiliary-spine Contact mode with original
  profiles. Maintain Height is disabled for this case and rejected by the
  adapter if requested. Unsupported correspondence fails explicitly.
- Closed Sweep requires closed rails and at least two sections. A two-profile
  closed annulus and persistence are verified.
- Check 65 samples on each rail/section against the resulting surface, with
  tolerance `max(1e-4 mm, curveLength * 1e-7)`. Crossing/coincident frames and
  unmatched contact fail. Sampling is not certified whole-curve contact.
- Output is an uncapped native surface/shell. Settings include
  `maintainHeight`, fitting, closure, directions/seams and Preview.
  Originals and source links remain intact.

### Advanced options

Independent rail G1/G2 constraints use unambiguous support-face edges and matching
open nonrational sections. Native geometry checks original contact, normals and
curvature. Refit supplies a finite cross-section tolerance with native hull bounds.
Add Slash controls actual section correspondence for one profile on open rails,
using paired viewport picks or `AddSlash=a,b`. History ON makes results
associative; CMD `gvSweep2History` forces OM9-SURFACE-004. See the precise
[bounds, defaults and lifecycle](../../../docs/features/surface-advanced-options.md).

### Remaining options

General multiple-profile Maintain Height, Preserve First/Last Shape constraints,
Simple Sweep, Point endpoints and dragged seams remain unsupported. Multiple-
profile/closed-rail Slash and fitting/closed/height/Slash combinations with G1/G2
remain unsupported. Full compatibility remains unverified.

### Validation

An arch between rails growing from width 5 to 9 mm retains height 2 mm with
Maintain Height ON and becomes 3.6 mm OFF. Both rails keep contact.
Existing one/multiple-profile strip tests, closed annulus area, manual Preview,
Undo/Redo and FCStd options/source links/placements pass.
See [the validation record](../../../docs/validation/2026-10-09-surface-options.md).
Advanced continuation: [constraints, Refit, Slash and History validation](../../../docs/validation/2026-10-09-surface-constraints-history.md).

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented (OpenMatrix9 choices; giá trị tương thích còn chưa xác định).
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested for the supported slice.
- [x] History/dependency scope documented; native snapshot persistence tested.
- [x] Undo/redo tested.
- [x] Save/reload persistence tested.
- [x] Automated tests use feature ID `OM9-SURFACE-003`.


Kiểm chứng đầy đủ hợp đồng và tương thích hình học vẫn chưa hoàn tất.
Checked items apply only to the supported implementation slice above.

- [x] Section Rebuild mở/periodic, Automatic/Natural và preview thủ công/động được kiểm chứng native.
- [x] Closed Sweep với hai section trên rail đóng được kiểm chứng diện tích và FCStd.
- [x] Maintain Height một section: crown 2 mm ON / 3.6 mm OFF khi rộng rail 5→9 mm; đo tiếp xúc cả hai rail.
- [ ] Hoàn tất các tùy chọn còn thiếu và chứng minh tương thích geometry đầy đủ.

- [x] G1/G2 kiểm chứng pháp tuyến/độ cong; Add Slash thay đổi mặt thật; Refit/History trong giới hạn host.
- [ ] Hoàn tất tương thích đầy đủ ngoài supported slice đã công bố.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SURFACE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SURFACE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SURFACE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SURFACE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SURFACE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SURFACE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SURFACE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-SURFACE-003` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SURFACE-003` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID (180 option checks + 97 hồi quy; xem validation record).
