---
id: OM9-HISTORY-001
name: History Workflow
command: null
domain: 01-core
module: History
kind: workflow
implementation_status: partially_implemented
guide_date: '2026-10-07'
guide_scope: engineering_contract_and_request_planner
sample_status: compiled_and_validation_tests_passed
implementation_validation_status: validated_supported_slice
implementation_verified_at: '2026-10-09'
implementation_validation_record: ../../../docs/validation/2026-10-09-reusable-builder-cage.md
---

# OM9-HISTORY-001 — History Workflow

Alias tương thích: `Chưa có alias command`. Nhóm: `01-core`. Trạng thái: `partially_implemented`.

## Hợp đồng thiết kế

**Hợp đồng thiết kế `OM9-HISTORY-001` — History Workflow.**

History là graph nhiều thế hệ parent/child; child có thể là parent của thế hệ sau. Sửa parent cập nhật descendants; sửa child độc lập phá link về parent nhưng descendants của child vẫn có thể theo nó. BrokenHistoryWarning có thể tắt bằng tùy chọn cùng tên; Undo phục hồi link sau thao tác phá. Join/Split/Trim/CurveBoolean/Boolean hoặc xóa input/output có thể thay topology và phá links nên adapter phải báo phạm vi; Join History tạo child của các curve rời khi có backend. Snapshot thiết kế trước thay topology giữ khả năng dựng lại. Gem giữ parameters nhiều Settings/Cutters để Match Attributes dựng lại, kể cả output bị xóa: Heads, Bezels, Bezel Cutter, Prongs, Metal Piece, Channel Border, Azure/Channel/MicroProng/Bright Cutters và Bright Cut Channel. Không mặc định ghi cho Prong/Bead on Surface, Emerald Profile & Cluster, Milgrain hay Gem Cutter Library. F6 edit/Builder input có thể khôi phục settings cuối. Gem Follow/Gem Control giữ north/culet hướng về gem/object target; bezels/prongs/metal/channel có thể recompute từ gems. Curve 2 Views History và Sweep History là ví dụ dependency explicit. Record/Update khởi tạo On được mô tả; Off chỉ suspend links cũ, object mới khi Off không nhận link sau. Clear Object History detach object cụ thể. Màu parent/child giúp chọn đúng nhưng không là dependency. Danh sách command từng hỗ trợ History không bảo đảm capability hiện có của OM9.

## Vai trò đầu vào của mẫu

Cardinality là chính sách của ví dụ; các nhánh tương tác và loại geometry phải được host kiểm tra tiếp theo hợp đồng.

| Role key | Kiểu | Tối thiểu | Tối đa |
|---|---|---:|---:|
| `active_document` | `document` | 1 | 1 |

## Tham số và điều kiện

Mẫu không tự điền default, không kiểm tra miền số hay tập enum của solver. Các điều kiện này phải được thực thi ở adapter theo bảng và hợp đồng.

| Option key | Kiểu mẫu | Ý nghĩa / đơn vị | Default / trạng thái |
|---|---|---|---|
| `Record` | `boolean` | Ghi dependency mới | On — khởi tạo History được mô tả; từng command còn cần capability. |
| `Update` | `boolean` | Cho recompute | On — khởi tạo History được mô tả; từng command còn cần capability. |
| `Lock` | `boolean` | Chặn sửa geometry child | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |
| `BrokenHistoryWarning` | `boolean` | Báo detach | Chưa xác lập; yêu cầu giao diện hiển thị lựa chọn hiện hành. |

## Hướng dẫn triển khai

1. Rust: khai báo state/operation plan theo `OM9-HISTORY-001`, kiểm tra phase selection, parameter kinds/units và capability; mọi số cần hữu hạn.
2. C++/Qt–FreeCAD: Rust dependency graph có operation ID, inputs, versioned parameters, dirty status và scheduler; C++ document links/recompute/transactions lưu detach có Undo; gem Builder records tách khỏi output objects.
3. Tích hợp `History Workflow` ở native command router; UI/CMD cùng handler. Python hỗ trợ workbench/fixture và native runtime checks theo ID, không đăng ký command trùng.
4. Tách preview/report/view state khỏi geometry document; commit transaction chỉ cho tác động được yêu cầu, cleanup event handlers khi panel/document đóng; serializer giữ identity và fields đúng scope.

## Code mẫu Rust

Đây là bộ kiểm tra request và tạo operation plan cho feature này. Dùng [thư viện mẫu](../../examples/rust/README.md); còn phải triển khai adapter/solver nêu trên để chạy trong FreeCAD. [Luồng Rust/C++/Qt/Python](../../CODE_GUIDE.md).

```rust
use om9_spec_examples::{
    validate_request, Effect, FeatureContract, InputKind,
    OperationPlan, Parameter, ParameterKind, PlanError, Request, Role,
};

pub const CONTRACT: FeatureContract = FeatureContract {
    id: "OM9-HISTORY-001",
    effect: Effect::DocumentWrite,
    roles: &[
        Role { name: "active_document", kind: InputKind::Document, min: 1, max: Some(1) },
    ],
    parameters: &[
        Parameter { name: "Record", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Update", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "Lock", kind: ParameterKind::Boolean, required: false },
        Parameter { name: "BrokenHistoryWarning", kind: ParameterKind::Boolean, required: false },
    ],
};

pub fn plan(request: &Request) -> Result<OperationPlan, PlanError> {
    validate_request(&CONTRACT, request)
}
```

## Các ca nghiệm thu cần thực hiện

- Graph A→B→C sửa A cập nhật B/C; sửa B detach A→B nhưng B→C còn; suspend không xóa links; Clear một object không clear toàn graph.
- Hủy panel/phiên giữ dữ liệu trước Apply; lỗi input/adapter không ghi kết quả một phần. Kiểm tra NaN/Infinity và document switch/close; không giữ callbacks tới object đã xóa.
- Đối với thay document: Undo/Redo và save/reload giữ kết quả cùng metadata/links. Đối với view/read-only: geometry/count không đổi. Đối với file output: path/format lỗi không ghi file hỏng hoặc đổi model ngoài yêu cầu.

Các ca trên là yêu cầu kiểm thử geometry/state của feature; kết quả biên dịch mẫu không thay thế nghiệm thu native.

## OpenMatrix9 implementation notes

**Verified:** 2026-10-09. **Status:** partially_implemented; validated_supported_slice.

Native HistorySettings, HistoryJoin and SurfaceHistory retain parent/child relationships, ordered recompute, global Record/Update/Lock/BrokenHistoryWarning/Clear, independent child detach and native Undo/Redo/FCStd. The reusable Builder graph adds gem -> durable record -> output, keeps parameters after outputs are deleted, and provides explicit restore/match APIs. Native CageBinding is a separate persistent dependency which continues to update when global History Record or Update is Off.

**RCORE-09 policy revision (2026-10-09):** Record controls new supported links.
Update alone suspends/resumes existing recorded descendants, including when
Record is Off; enabling Record does not retrofit snapshot outputs. This newer
OM9 policy supersedes the earlier coupled suspension behavior. The source
description above is retained as its original contract, not rewritten as
evidence of the newer implementation.

Rust owns portable graph ordering, recipe/version/numeric validation and cage bind/evaluation data. C++ is the required FreeCAD document/property/transaction/signal and OCCT adapter; Python is limited to host fixtures/bootstrap. This does not establish arbitrary original Gem builders, deleted-output auto-resurrection, topology correspondence or all-command Matrix/Rhino History. See the existing [Join/Surface contract](../../../docs/features/history-native-contract.md), [Builder API contract](../../../docs/features/builder-history-native-contract.md) and [Cage command contract](../../../docs/features/cage-command-native-contract.md).

[Matching native validation](../../../docs/validation/2026-10-09-reusable-builder-cage.md) records the accepted API/command scope and limits. Original design requirements above remain broader than this implemented slice.

## Acceptance checklist

- [ ] Hợp đồng feature và điều kiện hình học được kiểm chứng trước khi triển khai.
- [x] Inputs and selection state documented.
- [x] Parameters/defaults/units documented.
- [x] Output geometry documented.
- [x] Preview/commit/cancel behavior tested if applicable.
- [x] History/dependencies tested if applicable. (supported native Join/Surface/Builder graph slice only).
- [x] Undo/redo tested. (supported native Join/Surface/Builder graph slice only).
- [x] Save/reload persistence tested. (supported native Join/Surface/Builder graph slice only).
- [ ] Automated tests use feature ID `OM9-HISTORY-001`.

- [x] Surface graph nhiều thế hệ, detach/Undo, cycle refusal và cold FCStd source-edit rebuild đã qua.
- [ ] Hoàn tất tương thích đầy đủ ngoài supported slice đã công bố.

## Hợp đồng kỹ thuật chung

Áp dụng có điều kiện theo workflow; mỗi mục cần evidence riêng hoặc lý do không áp dụng.

- [ ] [OM9-CMD](../../ENGINEERING_CONTRACTS.md#om9-cmd): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-OBJECT](../../ENGINEERING_CONTRACTS.md#om9-object): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-SELECT](../../ENGINEERING_CONTRACTS.md#om9-select): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-VIEW](../../ENGINEERING_CONTRACTS.md#om9-view): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-DISPLAY](../../ENGINEERING_CONTRACTS.md#om9-display): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-PICK](../../ENGINEERING_CONTRACTS.md#om9-pick): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-COORD](../../ENGINEERING_CONTRACTS.md#om9-coord): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ORGANIZE](../../ENGINEERING_CONTRACTS.md#om9-organize): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANNOTATE](../../ENGINEERING_CONTRACTS.md#om9-annotate): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.
- [ ] [OM9-ANALYSIS](../../ENGINEERING_CONTRACTS.md#om9-analysis): kiểm chứng cho `OM9-HISTORY-001` hoặc ghi lý do không áp dụng.

- [x] Ghi rõ supported slice, tolerance, options còn thiếu và kết quả native theo exact feature ID. (xem validation constraints/History: 530 native checks).
