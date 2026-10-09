---
id: OM9-SURFACE-028
name: Match Surfaces
command: MatchSrf
domain: 03-surface
module: Surface Tools
kind: command
implementation_status: not_started
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
---

# OM9-SURFACE-028 — Match Surfaces

Alias tương thích: `MatchSrf`. Nhóm: `03-surface`. Trạng thái: `not_started`.

## Hợp đồng thiết kế

### Hợp đồng hành vi đề xuất

Match sửa cạnh untrimmed tới cạnh mục tiêu theo G0/G1/G2. Cạnh mở không match cạnh đóng; chỉ cạnh hoàn chỉnh, muốn một phần phải trim/split trước. MultipleMatches nhận các cạnh liên tiếp. Auto Chain/AutoChain chọn chuỗi theo Chain Continuity/ChainContinuity; Direction Forward/Backward/Both quyết định hướng tìm. Gap Tolerance/GapTolerance bỏ khe nhỏ; Angle Tolerance/AngleTolerance kiểm tra góc khi chọn Tangency. Undo bỏ đoạn cuối, Next nhận đoạn tiếp, All chọn toàn bộ.

### Tùy chọn và tương tác

- `Continuity`: Position/Tangency/Curvature
- `Preserve Other End`: None/Position/Tangency/Curvature ràng buộc biên đối diện
- `Average Surfaces`: bật/tắt; nếu hai mặt untrimmed chỉnh cả hai tới hình giữa
- `Match Edges by Closest Points`: nối end-to-end bằng stretch/compress hoặc kéo mỗi điểm tới điểm gần nhất
- `Refine Match`: bật/tắt; kiểm tra/refine theo tolerance
- `Distance`: dung sai G0 theo model units
- `Tangency`: dung sai góc độ
- `Curvature`: dung sai phần trăm bán kính
- `Isocurve Direction Adjustment`: Automatic theo trimmed/untrimmed target; Preserve Isocurve Direction; Match Target Isocurve Direction; Make Perpendicular to Target Edge

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `source_edges` | `curve` | 1 | Không đặt trong mẫu |
| `target_edges` | `curve` | 1 | Không đặt trong mẫu |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Continuity` | `choice` | Position/Tangency/Curvature | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Preserve Other End` | `choice` | None/Position/Tangency/Curvature ràng buộc biên đối diện | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Average Surfaces` | `boolean` | bật/tắt; nếu hai mặt untrimmed chỉnh cả hai tới hình giữa | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Match Edges by Closest Points` | `reference` | nối end-to-end bằng stretch/compress hoặc kéo mỗi điểm tới điểm gần nhất | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Refine Match` | `boolean` | bật/tắt; kiểm tra/refine theo tolerance | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Distance` | `number` | dung sai G0 theo model units | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Tangency` | `number` | dung sai góc độ | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Curvature` | `number` | dung sai phần trăm bán kính | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |
| `Isocurve Direction Adjustment` | `choice` | Automatic theo trimmed/untrimmed target; Preserve Isocurve Direction; Match Target Isocurve Direction; Make Perpendicular to Target Edge | Chưa xác lập; cần lựa chọn tường minh hoặc TODO_EVIDENCE. |

## Hướng dẫn triển khai

1. Rust lưu OM9-SURFACE-028 và phiên chọn có thứ tự, kiểm tra vai trò source_edges, target_edges; chuẩn hóa tùy chọn riêng của Match Surfaces.
2. Tách các bước chọn, chỉnh tùy chọn và xác nhận; kiểm tra điều kiện hình học và options theo hợp đồng thiết kế ở trên.
3. C++/Qt giải tham chiếu native, tính Match Surfaces trên bản sao bằng typed Part/OpenCascade hoặc adapter topology tương ứng; kiểm tra kết quả theo hợp đồng trên trước khi giao dịch tài liệu.
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
    id: "OM9-SURFACE-028",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "source_edges", kind: InputKind::Curve, min: 1, max: None },
        Role { name: "target_edges", kind: InputKind::Curve, min: 1, max: None },
    ],
    parameters: &[
        Parameter { name: "Continuity", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Preserve Other End", kind: ParameterKind::Choice, required: false },
        Parameter { name: "Average Surfaces", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Match Edges by Closest Points", kind: ParameterKind::Reference, required: false },
        Parameter { name: "Refine Match", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Distance", kind: ParameterKind::Number, required: false },
        Parameter { name: "Tangency", kind: ParameterKind::Number, required: false },
        Parameter { name: "Curvature", kind: ParameterKind::Number, required: false },
        Parameter { name: "Isocurve Direction Adjustment", kind: ParameterKind::Choice, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- G1 match đo tangent; Preserve Other End giữ biên đối diện; Average sửa cả hai; target trimmed dùng Automatic vuông góc cạnh.
- Số NaN/Infinity bị từ chối trước host; tham chiếu sai kiểu hoặc hết hạn không thay đổi tài liệu.
- Hủy/lỗi giữ trạng thái đầu vào; Undo/Redo và save/reload giữ kết quả, tùy chọn và liên kết đã công bố.

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
- [ ] Automated tests use feature ID `OM9-SURFACE-028`.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-SURFACE-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-SURFACE-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-SURFACE-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-SURFACE-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-SURFACE-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SURFACE](../../ENGINEERING_CONTRACTS.md#om9-surface): kiểm chứng cho `OM9-SURFACE-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-EDIT](../../ENGINEERING_CONTRACTS.md#om9-edit): kiểm chứng cho `OM9-SURFACE-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-POINTS](../../ENGINEERING_CONTRACTS.md#om9-points): kiểm chứng cho `OM9-SURFACE-028` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-SURFACE-028` hoặc ghi lý do không áp dụng.

- [ ] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID.
