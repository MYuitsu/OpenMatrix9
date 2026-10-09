---
id: OM9-SURFACE-001
name: Sweep 1
command: Sweep1
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

# OM9-SURFACE-001 — Sweep 1

Alias tương thích: `Sweep1`. Nhóm: `03-surface`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Nhận một rail rồi một hoặc nhiều section, có thể là đường mở/đóng hoặc cạnh mặt. Kết quả là surface đi theo rail qua các section. Flip đảo chiều section; Automatic thử đồng bộ seam và hướng; Natural phục hồi seam lúc bắt đầu. Section phải đồng loạt mở hoặc đóng; giữ thứ tự và chọn gần cùng đầu để tránh xoắn. Chain Edges ghép các cạnh tiếp xúc làm một rail; Point chỉ được dùng ở đầu/cuối chuỗi section. Preview phải làm mới sau khi đổi tùy chọn.

### Tùy chọn và tương tác

- `Style`: Freeform mặc định, xoay section giữ góc với rail; Road-like Through Finger/Side View/Looking Down giữ góc với C-Plane tương ứng
- `Closed Sweep`: khép từ section cuối về đầu, khả dụng từ hai section; rail đóng lặp section đầu ở cuối
- `Global Shape Blending`: bật/tắt; bật nội suy tuyến tính đầu-cuối, tắt tạo tiếp tuyến gần section và hòa trộn giữa
- `Untrimmed Miters`: bật/tắt tại góc gãy rail; bật tạo miter untrimmed, tắt cho mặt trimmed
- `Align Shapes`: đảo hướng section
- `Cross-Section Mode`: Do Not Simplify mặc định; Rebuild thay cấu trúc điểm; Refit khớp theo dung sai
- `Rebuild PointCount`: số điểm điều khiển section
- `Refit Tolerance`: sai số section theo mm
- `Simple Sweep`: dùng khi section giao rail tại edit point để tạo mặt đơn giản
- `Refit Rail`: làm mượt rail theo Absolute Tolerance tài liệu

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `rail` | `curve` | 1 | 1 |
| `sections` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Style` | `choice` | Freeform mặc định, xoay section giữ góc với rail; Road-like Through Finger/Side View/Looking Down giữ góc với C-Plane tương ứng | Freeform |
| `Closed Sweep` | `boolean` | khép từ section cuối về đầu, khả dụng từ hai section; rail đóng lặp section đầu ở cuối | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Global Shape Blending` | `boolean` | bật/tắt; bật nội suy tuyến tính đầu-cuối, tắt tạo tiếp tuyến gần section và hòa trộn giữa | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Untrimmed Miters` | `boolean` | bật/tắt tại góc gãy rail; bật tạo miter untrimmed, tắt cho mặt trimmed | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Align Shapes` | `reference` | đảo hướng section | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Cross-Section Mode` | `choice` | Do Not Simplify mặc định; Rebuild thay cấu trúc điểm; Refit khớp theo dung sai | Do Not Simplify |
| `Rebuild PointCount` | `number` | số điểm điều khiển section | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Refit Tolerance` | `number` | sai số section theo mm | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Simple Sweep` | `boolean` | dùng khi section giao rail tại edit point để tạo mặt đơn giản | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Refit Rail` | `boolean` | làm mượt rail theo Absolute Tolerance tài liệu | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SURFACE-001 và phiên chọn có thứ tự, kiểm tra vai trò rail, sections; chuẩn hóa tùy chọn riêng của Sweep 1.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Sweep 1 trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-SURFACE-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "rail", kind: InputKind::Curve, min: 1, max: Some(1) },
        Role { name: "sections", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Style", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Closed Sweep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Global Shape Blending", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Untrimmed Miters", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Align Shapes", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Cross-Section Mode", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Rebuild PointCount", kind: ParameterKind::Number, required: false },
        Parameter { name: "Refit Tolerance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Simple Sweep", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Refit Rail", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Rail thẳng + section tròn cho shell trụ; rail vòng cho surface khép. Kiểm tra hướng/seam, section mở-đóng hỗn hợp bị từ chối và các chế độ chưa hỗ trợ không được nhận âm thầm.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Scope:** Native Sweep1 with section Refit using numerical hull bounds, optional associative History, Chain Edges, seams, Rebuild, Closed Sweep and Preview; Road-like/blending/miters/Refit Rail remain unsupported.

Menu ID: `OM9_SurfaceSweepSweep1Rail`. CMD alias: `Sweep1`.
Verified supported slice: 2026-10-09; full feature remains partially implemented.

### Reference and implementation contract

Matrix 8 Book 1 PDF pp.180–183 (printed pp.170–173) were read through the
Sweep 1 continuation. The package now lives under
`Mod/OpenMatrix9/OpenMatrix9_Codex_Spec_v1`; the requested `ref/matrix9` prefix
is absent. Algorithm choices and bounds below are OM9 decisions.

- Rust owns identity, ordered selection, Chain Edges state and option policy.
  C++/Qt uses native Part/OpenCascade geometry; Python registers the workbench.
- Select one valid edge/connected wire rail, then profiles in order. Profiles
  must all be open or all closed. Explicit `EdgeN`/`WireN` references work.
  Reject duplicates, invalid/zero-length curves and whole faces/solids.
  Host limits: 256 inputs and 1024-byte reference keys.
- `ChainEdges` while selecting a rail collects touching edges into one logical
  rail. Enter validates connectivity; Undo removes its last edge; Cancel clears
  the command. All component subreferences persist in `SourceCurves`.
  Chain mouse selection retains the picked edge of a multi-edge object.
- CorrectedFrenet is the host default; Frenet is an explicit toggle.
  Independent Reverse/Flip and closed-profile numeric seams work.
  Single-edge fractions use the original curve parameter; multi-edge fractions
  use whole-wire arc length, splitting an edge when necessary. Geometry is
  copied independently before seam/kernel operations.
- Automatic compares profiles to the first current profile using centered,
  sampled directions and planar normal alignment. Closed profiles use 128
  candidates; the UI rounds seam fractions to four decimals. Degenerate and
  mismatched closure inputs fail. This is a host alignment heuristic.
  Natural restores original seams while retaining explicit Flip choices.
  Buttons and CMD tokens share these operations.
- Do Not Simplify is the default cross-section mode. Rebuild fits temporary
  copies with Rust, using 2–256 control points (initial host count 16), degree
  `min(3, count−1)`, and `max(513, 4*count+1)` arc-length samples.
  Closed profiles remain periodic. Rebuild is approximate; it provides no
  Refit tolerance or certified maximum error. Sources remain unchanged.
- Closed Sweep requires two or more profiles and closed rails in this host.
  The first section is appended to the native sweep sequence.
  Open-rail closure remains unsupported.
- Dynamic Preview is initially on; turning it off clears displayed preview.
  The Preview button/CMD token explicitly refreshes it. Geometry still validates
  before enabling OK. Preview creates no document object or undo entry.
- OK rebuilds a non-null valid uncapped surface/shell before one transaction.
  Errors disable OK and create no partial output. Esc/Cancel, document close/
  switch and workbench deactivation remove preview. Output retains sources,
  `OM9FeatureId`, `OM9Command`, `SourceCurves` and `SurfaceOptions`.
  Native Undo/Redo and FCStd preserve settings, source links and world placement.
  History OFF creates a snapshot. History ON creates a persistent native
  associative feature; see [advanced options](../../../docs/features/surface-advanced-options.md).

### Advanced options

Cross-section Refit now accepts a finite millimetre tolerance, builds an adaptive
Rust spline and checks continuous native chord/hull deviation bounds before
acceptance. The History checkbox recomputes after original curves or parent
placements change. CMD `gvSweepHistory` forces the native History variant
(OM9-SURFACE-002). Algorithms, numerical limits and lifecycle are documented in
[advanced options](../../../docs/features/surface-advanced-options.md).

### Remaining options

Road-like CPlane orientations, Global Shape Blending, trimmed/untrimmed miters,
Simple Sweep, Refit Rail, Point endpoints and dragged seam markers
remain unsupported. Frenet does not claim the Road-like
options. Full Matrix/Rhino compatibility remains unverified.

### Validation

Native tests cover cylinder/torus areas, closed Sweep with two sections, connected
and disconnected rail chains, grouped world placement, multi-edge seams,
Automatic/Natural on reversed periodic sources, open/periodic Rebuild, Preview,
failure recovery, Cancel, Undo/Redo and FCStd source/placement persistence.
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
- [x] Automated tests use feature ID `OM9-SURFACE-001`.


Kiểm chứng đầy đủ hợp đồng và tương thích hình học vẫn chưa hoàn tất.
Checked items apply only to the supported implementation slice above.

- [x] Section Rebuild mở/periodic, Automatic/Natural và preview thủ công/động được kiểm chứng native.
- [x] Closed Sweep với hai section trên rail đóng được kiểm chứng diện tích và FCStd.
- [ ] Hoàn tất các tùy chọn còn thiếu và chứng minh tương thích geometry đầy đủ.

- [x] Refit với dung sai mm và native hull bounds; History checkbox và alias002 đã kiểm chứng.
- [ ] Hoàn tất tương thích đầy đủ ngoài supported slice đã công bố.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SURFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SURFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SURFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SURFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SURFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SURFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SURFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-SURFACE-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SURFACE-001` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID (180 option checks + 97 hồi quy; xem validation record).
