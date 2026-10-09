---
id: OM9-SURFACE-009
name: Loft
command: Loft
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

# OM9-SURFACE-009 — Loft

Alias tương thích: `Loft`. Nhóm: `03-surface`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Loft qua ít nhất hai section hoặc cạnh surface theo thứ tự. Flip đảo chiều section; Automatic thử đồng bộ seam và hướng; Natural phục hồi seam lúc bắt đầu. Section phải đồng loạt mở hoặc đóng; giữ thứ tự và chọn gần cùng đầu để tránh xoắn. Point ở đầu/cuối tạo đầu nhọn. Align Curves điều chỉnh hướng. Cross-section có Do Not Simplify, Rebuild theo số điểm và Refit theo dung sai mm.

### Tùy chọn và tương tác

- `Style`: Normal mặc định khớp section và chuyển đều; Loose mượt nhưng xa section; Tight bám sát nhất quanh góc; Straight Sections đoạn ruled; Developable tạo riêng từng cặp để unroll, có thể thất bại/cho kết quả từng phần; Uniform dùng knot đồng đều
- `Closed loft`: bật/tắt; từ ba section nối cuối về đầu
- `Match Start/End Tangent`: bật/tắt từng đầu; chỉ áp dụng nếu section đầu/cuối là cạnh mặt
- `SplitAtTangents`: bật/tắt; quyết định tách mặt tại segment tangent
- `Rebuild PointCount`: số điểm section
- `Refit Tolerance`: dung sai mm
- `Preview`: bật/tắt; cập nhật động

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `sections` | `curve` | 2 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Style` | `choice` | Normal mặc định khớp section và chuyển đều; Loose mượt nhưng xa section; Tight bám sát nhất quanh góc; Straight Sections đoạn ruled; Developable tạo riêng từng cặp để unroll, có thể thất bại/cho kết quả từng phần; Uniform dùng knot đồng đều | Normal |
| `Closed loft` | `boolean` | bật/tắt; từ ba section nối cuối về đầu | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Match Start Tangent` | `boolean` | bật/tắt từng đầu; chỉ áp dụng nếu section đầu/cuối là cạnh mặt Giá trị độc lập của Match Start Tangent. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Match End Tangent` | `boolean` | bật/tắt từng đầu; chỉ áp dụng nếu section đầu/cuối là cạnh mặt Giá trị độc lập của Match End Tangent. | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `SplitAtTangents` | `boolean` | bật/tắt; quyết định tách mặt tại segment tangent | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Rebuild PointCount` | `number` | số điểm section | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Refit Tolerance` | `number` | dung sai mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Preview` | `boolean` | bật/tắt; cập nhật động | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SURFACE-009 và phiên chọn có thứ tự, kiểm tra vai trò sections; chuẩn hóa tùy chọn riêng của Loft.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Loft trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-SURFACE-009",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "sections", kind: InputKind::Curve, min: 2, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Closed loft", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Match Start Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Match End Tangent", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "SplitAtTangents", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Rebuild PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refit Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Preview", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Normal qua ba section khớp từng section; Straight Sections ruled; Closed loft khép với ba section; Developable có fixture unroll và fixture không khả thi.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Scope:** Native Loft with bounded Normal/Tight support-edge tangent matching, section Refit, optional History and earlier styles/seams; unsupported fitting/closed/rational tangent combinations and SplitAtTangents remain.

Menu ID: `OM9_SurfaceLoft`. CMD alias: `Loft`.
Verified supported slice: 2026-10-09; full feature remains partially implemented.

### Reference and implementation contract

Matrix 8 Book 1 PDF pp.193–195 (printed pp.183–185) were read through the Loft
continuation. Shares [Sweep 1](../../../docs/features/OM9-SURFACE-001.md)'s profile selection,
Reverse/seams, Automatic/Natural, section Rebuild, Preview and lifecycle contract.
No rails; at least two ordered sections, all open or all closed.
Closed Loft connects last to first and requires three or more sections.
Host algorithms below do not claim exact proprietary solver equivalence.

- Normal (initial host style): native smooth `Part.makeLoft`.
- Straight Sections: native ruled loft.
- Loose: OCCT's profiler harmonizes native single-edge NURBS profiles; their
  exact control rows/weights form a uniformly knotted V control net. Internal
  sections pull away rather than being interpolated. Closed V is periodic.
  Multi-edge profiles require explicit section Rebuild.
- Tight: native section interpolation with centripetal parameterization,
  maximum degree three and unchanged explicit profile correspondence.
- Uniform: harmonize single-edge U profiles; Rust globally interpolates the
  homogeneous coordinates and weights on genuinely uniform distinct V knots.
  Reject nonpositive resulting weights. U distinct knots must be uniformly
  spaced after harmonization, otherwise enable section Rebuild explicitly.
  Multi-edge profiles also need Rebuild. Closed V is periodic. Host bounds:
  256 sections, 4096 U poles and `U_poles * sections^3 <= 50,000,000` to bound
  dense solves; reduce sections/Rebuild count when exceeded.
- Developable: separate native ruled surface for each consecutive profile pair.
  Reject two nonparallel straight profiles. Each face must have defined
  curvature and `abs(GaussianCurvature)*max(1 mm², faceArea) <= 1e-7` at 81
  interior samples. Any failing pair rejects the whole operation.
  Closed Developable is unsupported. This bounded check is not an unroll
  certificate or a general developable-surface solver.
- Output must be valid, contain faces and no solids. Independent source copies
  are used for construction. History OFF creates a snapshot; History ON creates
  a native associative feature.

### Advanced options

Match Start/End Tangents use eligible supporting face edges with open nonrational
single-edge profiles, Normal/Tight styles and original section mode. Refit adapts
the section spline to a finite millimetre tolerance and applies continuous native
chord/hull deviation bounds. The History checkbox persists inputs/settings and
recomputes from original shape/placement changes. See the precise
[algorithms, bounds and lifecycle](../../../docs/features/surface-advanced-options.md).

### Remaining options

SplitAtTangents, Point endpoints, dragged seams and exhaustive profile correspondence remain
unsupported. Complete Matrix/Rhino geometry equivalence remains unverified.
Tangent matching with closed/rational/multi-edge sections, other styles or
section fitting remains unsupported; numeric Refit bounds are not formal
floating-point certificates.

### Validation

Tests measure Loose interior deviation, Tight/Uniform interpolation, six-row
uniform knot spacing, periodic Uniform, rational weighted profiles, nonuniform-U
rejection/Rebuild recovery, separate Developable pairs and analytic area,
nonparallel-pair recovery, source preservation and lifecycle/persistence.
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
- [x] Automated tests use feature ID `OM9-SURFACE-009`.


Kiểm chứng đầy đủ hợp đồng và tương thích hình học vẫn chưa hoàn tất.
Checked items apply only to the supported implementation slice above.

- [x] Section Rebuild mở/periodic, Automatic/Natural và preview thủ công/động được kiểm chứng native.
- [x] Loose/Tight/Uniform/Developable trong giới hạn host được kiểm chứng; Uniform periodic/rational và khôi phục sau lỗi U knot đã qua.
- [ ] Hoàn tất các tùy chọn còn thiếu và chứng minh tương thích geometry đầy đủ.

- [x] Match Start/End Tangents đo pháp tuyến độc lập; Refit và History được kiểm chứng trong giới hạn host.
- [ ] Hoàn tất tương thích đầy đủ ngoài supported slice đã công bố.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SURFACE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SURFACE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SURFACE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SURFACE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SURFACE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SURFACE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SURFACE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-SURFACE-009` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SURFACE-009` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID (180 option checks + 97 hồi quy; xem validation record).
