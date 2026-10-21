# OM9 Milestone Dashboard

> Sinh tự động lúc **2026-10-20T01:37:29Z** · mã nguồn `813c2b00` · nguồn: `OM9_Milestones/` + `implementation/{evidence,design,fixtures}/`
>
> Cập nhật: chạy lại `python tools/om9_milestone_dashboard.py` sau mỗi mục hoàn thành.
>
> **53** L3 đã nghiệm thu · **49** có bằng chứng chưa ký · **689** mới ở mức kế hoạch (tổng 791 L3).
>
> Dashboard chỉ đọc bằng chứng thật; cây kế hoạch vẫn giữ `SPEC_ONLY_NOT_IMPLEMENTED` và không trạng thái nào được nâng thay cho nghiệm thu.
>
> ⚠️ 204 bản ghi còn đường dẫn cũ sau đợt rút ngắn tên thư mục; dashboard khớp theo ID nên vẫn hiển thị đủ. Xem `OM9_Milestones/PATH_RENAMES.json`.

Chú giải: ✅ đã nghiệm thu · 🟡 có bằng chứng, chưa ký · ⬜ mới ở mức kế hoạch

## Tổng quan theo milestone

| Milestone | Thư mục | L2 | L3 | ✅ | 🟡 | ⬜ | Phụ thuộc |
|---|---|---:|---:|---:|---:|---:|---|
| **M00** Feasibility, QymCAD source, OCCT, legal and environment | `M00 - Feasibility, Legal` | 7 | 19 | 8 | 7 | 4 | — |
| **M01** CAD shell, OCCT bridge, document, undo, GPU and viewport foundation | `M01 - CAD Shell, Viewport` | 13 | 117 | 45 | 41 | 31 | M00 |
| **M02** Viewport picking, snapping, navigation, gumball and Matrix interaction | `M02 - Viewport, Gumball` | 9 | 59 | 0 | 1 | 58 | M00, M01 |
| **M03** Curve, Surface, Solid và trao đổi 3DM | `M03 - Geometry 3DM` | 14 | 243 | 0 | 0 | 243 | M01 |
| **M04** Builder, History và Ring | `M04 - Builder and History` | 7 | 51 | 0 | 0 | 51 | M01 |
| **M05** Gems và Layout | `M05 - Gems and Layout` | 5 | 43 | 0 | 0 | 43 | M03 |
| **M06** Settings và Prongs | `M06 - Settings and Prongs` | 2 | 19 | 0 | 0 | 19 | M03, M05 |
| **M07** Cutters và kiểm tra chế tạo | `M07 - Cutters` | 3 | 19 | 0 | 0 | 19 | M03 |
| **M08** Advanced Jewelry Modeling | `M08 - Jewelry Modeling` | 8 | 51 | 0 | 0 | 51 | M03, M05, M06 |
| **M09** Cổng nghiệm thu và phát hành (không phát sinh L2 owner) | `M09 - Acceptance and Release Gates` | 0 | 0 | 0 | 0 | 0 | M00, M01, M02, M03, M04, M05, M06, M07, M08 |
| **ZAFTER** Phạm vi mở rộng sau các release gate chính | `ZAFTER - Extensions` | 34 | 170 | 0 | 0 | 170 | M09 |

## Cấu trúc chi tiết (Mxx → L1 → L2 → L3)

<details><summary><b>M00 — Feasibility, QymCAD source, OCCT, legal and environment</b> · ✅8 🟡7 ⬜4 · `OM9-M00` · NOT_STARTED</summary>

**L1** — Feasibility, QymCAD source, OCCT, legal and environment · phụ thuộc: —

Bằng chứng cấp milestone: `G00-01`, `G00-02`, `G00-03`, `G00-04`, `G00-05`, `G00-06`, `G00-07`, `OM9-L2-02`, `OM9-L4-02-01-01`, `OM9-L4-02-01-02`, `OM9-L4-02-01-03`

**OM9-M00-L2-01 — QymCAD GitHub & source reuse** · ✅0 🟡0 ⬜3

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-M00-L3-01-01` | Repository provenance & pinned checkout |  |  | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-M00-L3-01-02` | Workspace map and dependency audit |  |  | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-M00-L3-01-03` | REUSE ADAPT ADD architecture decision |  |  | 0/4 | ⬜ mới ở mức kế hoạch | — |

**OM9-M00-L2-02 — OCCT bridge & numerical geometry** · ✅3 🟡0 ⬜0

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-M00-L3-02-01` | Toolchain and C ABI bridge proof |  |  | 4/4 | ✅ đã nghiệm thu — ký bởi project owner | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-M00-L3-02-02` | Lifetime, threading and error safety |  |  | 4/4 | ✅ đã nghiệm thu — ký bởi project owner | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-M00-L3-02-03` | Geometry numeric invariants |  |  | 4/4 | ✅ đã nghiệm thu — ký bởi project owner | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-M00-L2-03 — UI/Viewport/GPU technology** · ✅3 🟡0 ⬜0

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-M00-L3-03-01` | Source UI architecture and constraints |  |  | 3/3 | ✅ đã nghiệm thu — ký bởi project owner | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-M00-L3-03-02` | GPU renderer stability and performance |  |  | 4/4 | ✅ đã nghiệm thu — ký bởi project owner | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-M00-L3-03-03` | Picking + transactional selection |  |  | 3/3 | ✅ đã nghiệm thu — ký bởi project owner | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-M00-L2-04 — 3DM / openNURBS interoperability** · ✅0 🟡2 ⬜1

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-M00-L3-04-01` | 3DM capability gap and dependency candidate |  |  | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-M00-L3-04-02` | NURBS / trimmed BRep / units proof |  |  | 2/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-M00-L3-04-03` | Import security and transparent limitations |  |  | 3/3 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-M00-L2-05 — Associative history & stable topology** · ✅0 🟡2 ⬜0

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-M00-L3-05-01` | Naming and reference architecture audit |  |  | 2/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-M00-L3-05-02` | Regeneration and persistence stress proof |  |  | 3/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-M00-L2-06 — License / redistribution compliance** · ✅2 🟡0 ⬜0

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-M00-L3-06-01` | Software license and reuse strategy |  |  | 3/4 | ✅ đã nghiệm thu — ký bởi project owner | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-M00-L3-06-02` | Assets, trademarks and release notices |  |  | 2/4 | ✅ đã nghiệm thu — ký bởi project owner | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-M00-L2-07 — Reproducible build, CI and release baseline** · ✅0 🟡3 ⬜0

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-M00-L3-07-01` | Windows 10/11 x64 baseline |  |  | 3/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-M00-L3-07-02` | Linux optional & packaged OCCT |  |  | 3/3 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-M00-L3-07-03` | Continuous verification and gate handoff |  |  | 3/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

</details>

<details><summary><b>M01 — CAD shell, OCCT bridge, document, undo, GPU and viewport foundation</b> · ✅45 🟡41 ⬜31 · `OM9-M01` · NOT_STARTED</summary>

**L1** — CAD shell, OCCT bridge, document, undo, GPU and viewport foundation · phụ thuộc: M00

Bằng chứng cấp milestone: `OM9-L2-01-01`, `OM9-L2-01-02`, `OM9-L2-01-03`, `OM9-L2-01-05`, `OM9-L2-03-14`

**OM9-L2-01-01 — Febe Application Shell Docking** · ✅3 🟡5 ⬜0

_Khởi động/đóng ứng dụng, cửa sổ chính, docking cơ bản để bố trí giao diện Matrix._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-01-001` | Application Startup And Main Window *(thủ công)* | E02 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi project owner (human reviewer), running the app on Windows | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-01-002` | Workbench Docking Regions *(thủ công)* | E02 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi project owner (human reviewer) running the app on Windows | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-01-003` | Reorder Collapse And Float Panels *(thủ công)* | E02 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi project owner (human reviewer) running the app on Windows | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-01-004` | Docking Layout Default Restore *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-01-005` | Docking Layout Persistence *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-01-006` | Active Document Window State *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-01-007` | Global Status And Error Surface *(thủ công)* | E02 | MEDIUM | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-01-008` | Builder Panel Docking Slot *(thủ công)* | E02 | MEDIUM | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-L2-01-02 — Be Command Dispatch Undo Redo** · ✅11 🟡0 ⬜0

_Registry lệnh, state-machine nhận điểm/đối tượng, transaction, Undo/Redo/Cancel thống nhất._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-02-001` | Command IDs And Alias Registry | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-002` | Typed Command Descriptor | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-003` | Command Context And Active Document | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-004` | Interactive Command State Machine | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-005` | Command Option Parser | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-006` | Atomic Command Transactions | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-007` | Undo Redo Journal | E01 | NORMAL | 5/5 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-008` | Command Cancel Rollback | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-009` | Command Repeat And Dispatch | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-010` | Command Event And Observer API | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-02-011` | Command Availability Preconditions | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-L2-01-03 — Be Document Scene Entities** · ✅13 🟡0 ⬜0

_Document, EntityID, FeatureID, layers, blocks/instances, metadata đá và tham chiếu đối tượng._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-03-001` | Document Entity IDs | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-002` | Typed Scene Entity Records | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-003` | Geometry Handle And Attributes | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-004` | Layer Domain Records | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-005` | Entity To Layer Membership | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-006` | Layer Selection Visibility Rules | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-007` | Group Membership Model | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-008` | Block Definition And Instance Graph | E01 | NORMAL | 5/5 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-009` | Jewelry Semantic Extension Record | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-010` | Selection State Document Contract | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-011` | Document Change Notification | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-012` | Document Snapshot And Restore | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-03-013` | Import Source Provenance Metadata | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-L2-01-05 — Be Occt Brep Kernel Bridge** · ✅11 🟡0 ⬜0

_Adapter OCCT, BRep, NURBS evaluation, tolerances, memory ownership/FFI safety._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-05-001` | OCCT FFI Safe Handle Lifecycle | E01 | HIGH | 4/5 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-002` | B-Rep Shape Type And Validation | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-003` | NURBS Curve Evaluation Adapter | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-004` | NURBS Surface Evaluation Adapter | E01 | HIGH | 4/5 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-005` | Face Wire Edge Topology Access | E01 | HIGH | 4/5 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-006` | OCCT Transform Matrix Service | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-007` | Geom Intersection Query Adapter | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-008` | Shape Tessellation Interface | E01 | HIGH | 4/5 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-009` | Geometry Operation Error Contract | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-010` | Kernel Thread And Transaction Policy | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-05-011` | Reference Shape Fixture Evaluation | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-L2-01-06 — Febe Viewport Gpu Draw Backend** · ✅0 🟡9 ⬜0

_GPU viewport wireframe/shaded, gem display, camera draw, overlays và mesh tessellation._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-06-001` | GPU Device And Render Loop *(thủ công)* | E02 | HIGH | 3/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |
| `OM9-L3-01-06-002` | Shared Scene Graphics Resources *(thủ công)* | E02 | HIGH | 3/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |
| `OM9-L3-01-06-003` | NURBS B-Rep Display Tessellation Cache *(thủ công)* | E02 | HIGH | 3/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |
| `OM9-L3-01-06-004` | Wireframe And Isocurve Pass *(thủ công)* | E02 | HIGH | 3/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |
| `OM9-L3-01-06-005` | Shaded Face And Solid Pass *(thủ công)* | E02 | HIGH | 3/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |
| `OM9-L3-01-06-006` | Depth Clipping And Grid Overlays *(thủ công)* | E02 | HIGH | 3/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |
| `OM9-L3-01-06-007` | Selection Highlight Render Pass *(thủ công)* | E02 | HIGH | 3/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |
| `OM9-L3-01-06-008` | Viewport Invalidation And Redraw *(thủ công)* | E02 | HIGH | 3/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |
| `OM9-L3-01-06-009` | GPU Device Loss Graceful Recovery *(thủ công)* | E02 | HIGH | 3/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |

**OM9-L2-01-08 — Be Units Tolerances Basic Parameters** · ✅7 🟡0 ⬜0

_Millimeter, degrees, tolerances, numeric input và tham số builder đơn giản._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-08-001` | Millimeter World Unit Model | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-08-002` | Angle Conversion And Degrees | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-08-003` | Document Absolute Relative Tolerances | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-08-004` | Typed Numeric Parameter Values | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-08-005` | Builder Parameter Range Validation | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-08-006` | Unit Safe Parameter Serialization | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-01-08-007` | Coordinate Numeric Input Primitives | E01 | NORMAL | 4/4 | ✅ đã nghiệm thu — ký bởi cổng AUTO_ONLY (bằng chứng tự động) | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-L2-01-12 — Be Native Project Codec Stl Export** · ✅0 🟡9 ⬜1

_Save/load native document đầy đủ geometry/history/layer/gem instance; xuất STL watertight và báo lỗi hình học._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-12-001` | Native Project Container Format | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-01-12-002` | Scene And Layer State Encoding | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-01-12-003` | B-Rep Mesh And Instance Encoding | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-01-12-004` | Builder History Stub Persistence | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-01-12-005` | Atomic Save And Write Failure Recovery | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-01-12-006` | Project Load Validation And Compatibility | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-01-12-007` | Document Reload And Restoration | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-01-12-008` | STL Tessellation Export Codec | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-12-009` | Export Mesh Units And Winding | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-01-12-010` | Project Roundtrip Regression Fixtures | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |

**OM9-L2-02-01 — Be Rhino Nurbs Object Semantics** · ✅0 🟡10 ⬜0

_Định nghĩa Rhino-like Curve/Surface/TrimmedSurface/Polysurface/solid/block và seam, UV/normal._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-02-01-001` | Point And Polyline Semantic Types | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-02-01-002` | NURBS Curve Control Points Knots Weights | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-02-01-003` | Open Closed And Planar Curve Rules | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-02-01-004` | Surface UV Domain And Seam Rules | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-02-01-005` | Trimmed Vs Underlying Surface Model | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-02-01-006` | Surface Edge And Isocurve Semantics | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-02-01-007` | Polysurface And Closed Solid Semantics | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-02-01-008` | Extrusion Vs Full B-Rep Representations | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-02-01-009` | Mesh Vs NURBS Object Type Rules | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |
| `OM9-L3-02-01-010` | Instance And Block Semantic Type | E02 | HIGH | 5/5 | 🟡 có bằng chứng, chưa ký | `implementation/evidence` |

**OM9-L2-03-01 — Febe Jewelry Viewport Layout And Cplanes** · ✅0 🟡0 ⬜7

_Viewport Looking Down/Perspective/Side/Through Finger, grid trang sức, active view_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-01-001` | Matrix Four Viewport Grid Layout *(thủ công)* | E02 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-01-002` | Jewelry Viewport Camera Presets *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-01-003` | Active Viewport Indicator *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-01-004` | F4 World Origin And Ring Center Display *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-01-005` | Matrix Viewport Grid And Axes *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-01-006` | Maximize And Restore Viewport Layout *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-01-007` | Viewport Tab Switch When Maximized *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-07 — Febe Matrix Main Menu Iconhistory And Docking** · ✅0 🟡0 ⬜8

_Main Menu, Icon History, panel arrangement Matrix_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-07-001` | Matrix Left Sidebar Stack And Order *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-07-002` | Matrix Main Menu Category Tabs *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-07-003` | Command Icon Tooltip And Rightclick Slot *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-07-004` | Menu Scrolling And Icon Grid *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-07-005` | Recent Command Icon History *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-07-006` | Main Menu Collapse Reorder And Float *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-07-007` | Default Menu Layout Reset *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-07-008` | Disabled Menu Action Visibility *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-10 — Febe Matrix Commandline Feedback Adapter** · ✅0 🟡0 ⬜6

_Command Line kiểu Rhino/Matrix, prompt, options/history_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-10-001` | Commandline Entry And Submit *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-10-002` | Interactive Command Prompts *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-10-003` | Command Options Clickable Tokens *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-10-004` | Numeric And Coordinate Feedback *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-10-005` | Command History Readable Panel *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-10-006` | Commandline Escape Repeat Shortcuts *(thủ công)* | E02 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-14 — Febe Matrix Layers Objectinfo Superselect** · ✅0 🟡8 ⬜0

_Layers palette, Object Info, SuperSelect cần modeling_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-14-001` | Matrix Layer Category Palette *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-03-14-002` | Current Layer And Assignment UI *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-03-14-003` | Layer Visible And Locked Controls *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-03-14-004` | Select All Objects By Layer *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-03-14-005` | Matrix Object Info Overlay *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-03-14-006` | Superselect By Jewelry Object Type *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-03-14-007` | Object Property Editor Adapter *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |
| `OM9-L3-03-14-008` | Layer State Persistence Visual Check *(thủ công)* | E02 | NORMAL | 4/4 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence`, `implementation/fixtures` |

**OM9-L2-03-15 — Febe Matrix File Project And Jobbags** · ✅0 🟡0 ⬜9

_File menu, native Open/Save, Import .3dm action, Job Bags/Project Manager_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-15-001` | Matrix File New Open Save Actions *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-15-002` | Matrix File Import Export Entrypoints *(thủ công)* | E02 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-15-003` | Project Manager Project Row List *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-15-004` | Jobbag Slot Grid And Thumbnail *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-15-005` | Jobbag Capture Current Selection *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-15-006` | Jobbag Restore And Insert *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-15-007` | Master Jobbag Document Snapshot *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-15-008` | Project Notes And Basic Metadata *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-15-009` | Project Manager Dock Interaction *(thủ công)* | E02 | NORMAL | 0/4 | ⬜ mới ở mức kế hoạch | — |

</details>

<details><summary><b>M02 — Viewport picking, snapping, navigation, gumball and Matrix interaction</b> · ✅0 🟡1 ⬜58 · `OM9-M02` · NOT_STARTED</summary>

**L1** — Viewport picking, snapping, navigation, gumball and Matrix interaction · phụ thuộc: M00, M01

**OM9-L2-01-07 — Febe Picking Selection Generic Handles** · ✅0 🟡0 ⬜8

_Picking object/face/edge/vertex, selection set, gizmo hooks và hover hit-tests._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-07-001` | Screen Ray And Hit Test Query *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-07-002` | Object Selection Set Core *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-07-003` | Subobject Face Edge Vertex Picking *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-07-004` | Selection Window And Crossing *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-07-005` | Object Selection Disambiguation *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-07-006` | Selection Highlighting And Hover *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-07-007` | Generic Translation Rotation Scale Handles *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-07-008` | Drag Transaction And Commit Cancel *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-02-02 — Febe Cplanes And Coordinate Input** · ✅0 🟡0 ⬜7

_CPlane/World coordinates, gốc F4, nhập điểm chính xác, dựng hình theo view._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-02-02-001` | World And Construction Plane Coordinate Context *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-02-002` | CPlane Preset Worldtop Front Right *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-02-003` | CPlane Orientation And Origin Change *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-02-004` | Coordinate Text Point Entry *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-02-005` | Matrix F4 Origin Shortcut *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-02-006` | Ortho Planar Project Cursor Constrain *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-02-007` | Cross Viewport Coordinate Consistency *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-02-03 — Febe Osnaps And Basic Smarttrack** · ✅0 🟡0 ⬜7

_End/Mid/Cen/Near/Point/Int/Tan/Perp/Quad và SmartTrack cơ bản._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-02-03-001` | Osnap End Mid Center Quadrant *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-03-002` | Osnap Near Point Intersection *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-03-003` | Osnap Tangent Perpendicular *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-03-004` | Osnap Surface And Polysurface *(thủ công)* | E03 | MEDIUM | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-03-005` | Osnap Priority And One Shot Overrides *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-03-006` | Smarttrack Inferred Alignment Lines *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-03-007` | Osnap Debug Marker And Tooltip *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-02 — Febe Viewport Navigation Pan Zoom Orbit** · ✅0 🟡0 ⬜7

_Pan/Zoom/Orbit, double-click maximize, view resets_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-02-001` | Perspective Right Drag Orbit *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-02-002` | Orthographic Right Drag Pan *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-02-003` | Wheel Zoom To Cursor *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-02-004` | Perspective Shift Right Drag Pan *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-02-005` | Zoom Extents And Selected *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-02-006` | Viewport Center Reset And Home *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-02-007` | Viewport Maximize By Title *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-03 — Febe Viewport Object Selection And Gumball** · ✅0 🟡1 ⬜5

_Chọn đối tượng theo khung và Gumball trực quan_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-03-001` | Matrix Click Shift Ctrl Select *(thủ công)* | E03 | NORMAL | 1/5 | 🟡 có bằng chứng, chưa ký | `implementation/design`, `implementation/evidence` |
| `OM9-L3-03-03-002` | Matrix Left Right Drag Selection *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-03-003` | Matrix Ambiguous Selection Popup *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-03-004` | Matrix Gumball Axis Drag *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-03-005` | Matrix Gumball Alignment And Relocate *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-03-006` | Matrix Gumball Numeric Edit *(thủ công)* | E03 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-04 — Febe Viewport Snaps Ortho Smarttrack** · ✅0 🟡0 ⬜5

_Snaps palette, Ortho/Project/Planar UI_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-04-001` | Matrix Snap Toggle Panel *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-04-002` | Matrix Ortho Planar Project Toggles *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-04-003` | Matrix Snap Activation And Indicator *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-04-004` | Matrix Smarttrack Toggle And Feedback *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-04-005` | Matrix Snap Shortcut Modes *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-05 — Febe Viewport Display Shading And Gems** · ✅0 🟡0 ⬜6

_Wireframe/Shaded/Gem View, display presets khi modeling_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-05-001` | Wireframe Shaded Mode Switcher *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-05-002` | Gem View Material Color Cues *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-05-003` | Curve And Cutter Visibility Presets *(thủ công)* | E03 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-05-004` | Surface Shading And Backface Cues *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-05-005` | Selection Highlight Overrides *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-05-006` | Display Settings Preview Presets *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-06 — Febe Viewport Pictureframe And Calibration** · ✅0 🟡0 ⬜5

_Picture Frame và scale tham chiếu ảnh_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-06-001` | Pictureframe Image Import *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-06-002` | Pictureframe Position And Scale *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-06-003` | Reference Image Transparency And Lock *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-06-004` | Known Distance Image Calibration *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-06-005` | Viewport Reference Asset Persistence *(thủ công)* | E03 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-09 — Febe Matrix F6 Context Actions** · ✅0 🟡0 ⬜8

_F6 menu theo selection type, auto-prefill builder_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-09-001` | F6 Selected Object Class Dispatch *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-09-002` | F6 Default General Mode *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-09-003` | F6 Curve And Surface Modes *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-09-004` | F6 Gem And Report Modes *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-09-005` | F6 Custom User Mode *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-09-006` | F6 Builder Input Prefill *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-09-007` | F6 Floating Pin And Position *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-09-008` | F6 Disabled Unimplemented Actions *(thủ công)* | E03 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |

</details>

<details><summary><b>M03 — Curve, Surface, Solid và trao đổi 3DM</b> · ✅0 🟡0 ⬜243 · `OM9-M03` · NOT_STARTED</summary>

**L1** — Curve, Surface, Solid và trao đổi 3DM · phụ thuộc: M01

**OM9-L2-01-09 — 01-be-occt-geometry-creation-ops** · ✅0 🟡0 ⬜9

_Primitives, pipe, extrude, revolve, loft/sweep BRep và curve/surface adapters phục vụ trang sức._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-09-001` | Analytic Solid Primitive Factories | E04 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-09-002` | Curve To Solid Extrusion | E06 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-09-003` | Revolve Profile And Rail Revolve | E06 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-09-004` | Pipe From Curve | E06 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-09-005` | Planar Boundary Surface Generator | E06 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-09-006` | Loft Through Crosssections | E06 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-09-007` | Sweep Single And Double Rail | E06 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-09-008` | Surface From Edge Curves | E06 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-09-009` | General Profile To B-Rep Adapter | E06 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-01-10 — 02-be-occt-boolean-fillet-shell-ops** · ✅0 🟡0 ⬜9

_Union/Difference/Intersection, trim/split support, fillet/chamfer, shell/offset cần cho nhẫn và cutter._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-10-001` | B-Rep Boolean Union | E07 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-10-002` | B-Rep Boolean Difference | E07 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-10-003` | B-Rep Boolean Intersection | E07 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-10-004` | Curve And Face Split Intersections | E07 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-10-005` | Planar Cap And Close Solid | E07 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-10-006` | Fillet And Chamfer B-Rep Edges | E07 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-10-007` | Offset Face And Thicken Shell | E07 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-10-008` | Face Extract Unjoin And Rejoin | E07 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-10-009` | Geometry Health And Naked Edge Query | E07 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-01-11 — 03-be-occt-transforms-and-pattern-ops** · ✅0 🟡0 ⬜8

_Move/Rotate/Scale/Mirror/Array transformations + transform matrices làm primitive dịch vụ dùng chung._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-11-001` | Translation Rotation Scaling Matrices | E04 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-11-002` | Mirror Across Plane | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-11-003` | Nonuniform Scaling And Units | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-11-004` | Polar And Linear Instance Pattern | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-11-005` | Curve Sampled Placement Frames | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-11-006` | Surface UV Normal Placement Frames | E07 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-11-007` | Orientation Two Three Point Solver | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-11-008` | Array Bake Vs Instance Policy | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-01-13 — 14-febe-import-rhino-3dm** · ✅0 🟡0 ⬜12

_Đọc file Rhino .3dm (NURBS/trimmed BRep, layers, units, blocks, meshes), chuyển sang scene OCCT, giao dịch atomic; lệnh Import được gắn qua File Menu Matrix._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-13-001` | .3dm File Signature And Version Probe | E08 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-002` | Opennurbs Reader Bridge Contract | E08 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-003` | .3dm Document Units Tolerance Mapping | E08 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-004` | .3dm Layer And Object Attribute Mapping | E08 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-005` | .3dm Analytic And NURBS Curves | E08 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-006` | .3dm NURBS Surface And Trimmed B-Rep | E08 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-007` | .3dm Mesh And Instance Blocks | E08 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-008` | .3dm Unsupported Proprietary Object Policy | E08 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-009` | .3dm Import Atomic Scene Transaction | E08 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-010` | .3dm File Picker And Options Dialog *(thủ công)* | E08 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-011` | .3dm Import Summary Warning UI *(thủ công)* | E08 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-13-012` | .3dm Golden Roundtrip Fixtures | E08 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-02-04 — 04-febe-freeform-curve-modeling** · ✅0 🟡0 ⬜12

_Line/Polyline/InterpCurve/Circle/Arc/Offset/Fillet/Blend/Rebuild/Match và point edit, G0/G1/G2 theo scope Curve cần cho Matrix._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-02-04-001` | Line Polyline Point Creation *(thủ công)* | E04 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-002` | Circle Arc Ellipse Polygon Creation *(thủ công)* | E04 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-003` | Interpolated And Controlpoint NURBS Curve *(thủ công)* | E05 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-004` | Curve Edit Points And Grips *(thủ công)* | E05 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-005` | Join Explode Close And Split Curves *(thủ công)* | E05 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-006` | Offset Fillet Chamfer Curves *(thủ công)* | E05 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-007` | Blend Arcblend And Match Curve Continuity *(thủ công)* | E05 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-008` | Rebuild Refit And Knot Operations *(thủ công)* | E05 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-009` | Project Pullback And Curve Intersections *(thủ công)* | E05 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-010` | Duplicate Edge Border Isocurve *(thủ công)* | E05 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-011` | Curve Seams Directions And Divisions *(thủ công)* | E05 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-04-012` | Sketch On Surface Mesh And Two Views *(thủ công)* | E05 | HIGH | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-02-05 — 05-febe-freeform-surface-modeling** · ✅0 🟡0 ⬜10

_Sweep1/2, Loft, Network, Patch, RailRevolve, Trim/Split/Untrim/MatchSurface/Offset cùng UV surface._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-02-05-001` | Sweep1 Sweep2 And Profile Sweep Workflows *(thủ công)* | E06 | MEDIUM | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-05-002` | Loft And Curve Network Workflows *(thủ công)* | E06 | MEDIUM | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-05-003` | Revolve Railrevolve And Extrude Workflows *(thủ công)* | E06 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-05-004` | Edge Planar And Patch Surface Workflows *(thủ công)* | E06 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-05-005` | Trim Split Untrim And Shrink Surface *(thủ công)* | E06 | MEDIUM | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-05-006` | Surface Rebuild Refit And Softedit *(thủ công)* | E06 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-05-007` | Surface Match Blend And Join Continuity *(thủ công)* | E06 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-05-008` | Surface Offset Fillet Chamfer *(thủ công)* | E06 | HIGH | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-05-009` | Surface UV Direction And Isocurve Handling *(thủ công)* | E06 | MEDIUM | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-05-010` | Surface Normal Extrude And Unroll *(thủ công)* | E06 | MEDIUM | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-02-06 — 06-febe-rhino-transform-orient-workflows** · ✅0 🟡0 ⬜6

_Command sessions cho Move/Scale/Rotate/Array/Orient với pivot/ref/copy/preview kiểu Rhino._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-02-06-001` | Interactive Move Copy And Rotate *(thủ công)* | E04 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-06-002` | Interactive Scale 1d 2d 3d *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-06-003` | Interactive Mirror And Array *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-06-004` | Orient By Two Three Reference Points *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-06-005` | Align And Setpoints *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-02-06-006` | Transform Cancel Preview And Undo *(thủ công)* | E07 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-08 — 07-febe-matrix-top11-and-shortcut-actions** · ✅0 🟡0 ⬜6

_11 lệnh thường dùng theo toolbar Matrix_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-08-001` | Top11 Duplicate Action *(thủ công)* | E04 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-08-002` | Top11 Editpoints And Controlpoints *(thủ công)* | E05 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-08-003` | Top11 Mirror Move Rotate Actions *(thủ công)* | E04 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-08-004` | Top11 Explode Join Split Trim Actions *(thủ công)* | E07 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-08-005` | Top11 Ringrail Entrypoint *(thủ công)* | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-08-006` | Top11 Toolbar Availability State *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-16 — 08-febe-matrix-curve-menu-workflows** · ✅0 🟡0 ⬜58

_Curve menu và lệnh chỉnh curve của Matrix8 Book1_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-16-001` | Curve · PolyLine — menu binding *(thủ công)* | E04 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-002` | Curve · Line — menu binding *(thủ công)* | E04 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-003` | Curve · Interp Curve — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-004` | Curve · Rectangle — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-005` | Curve · Circle — menu binding *(thủ công)* | E04 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-006` | Curve · Ellipse — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-007` | Curve · Arc — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-008` | Curve · Arc Direction — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-009` | Curve · Rebuild — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-010` | Curve · Refit to Tolerance — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-011` | Curve · Quick Curve Blend — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-012` | Curve · Adjustable Curve Blend — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-013` | Curve · Curve from 2 Views — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-014` | Curve · Curve 2 Views with History — menu binding *(thủ công)* | E05 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-015` | Curve · Pullback — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-016` | Curve · Project — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-017` | Curve · Intersection — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-018` | Curve · Offset Curve — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-019` | Curve · Offset Curve on Surface — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-020` | Curve · Extract Isocurve — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-021` | Curve · Fillet Curves — menu binding *(thủ công)* | E05 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-022` | Curve · Fillet Corners — menu binding *(thủ công)* | E05 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-023` | Curve · Chamfer Curves — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-024` | Curve · Create UV Curves — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-025` | Curve · Apply UV Curves — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-026` | Curve · Through Points — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-027` | Curve · Extend Curve — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-028` | Curve · Polygon — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-029` | Curve · Spiral — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-030` | Curve · Helix — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-031` | Curve · Point — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-032` | Curve · Mark Curve Start — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-033` | Curve · Mark Curve End — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-034` | Curve · Adjust Closed Curve Seam — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-035` | Curve · Continue Interpolated Curve — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-036` | Curve · Divide Curve — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-037` | Curve · Curve Boolean — menu binding *(thủ công)* | E05 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-038` | Curve · Tween Curves — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-039` | Curve · Duplicate Edge — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-040` | Curve · Duplicate Border — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-041` | Curve · Curve — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-042` | Curve · Continue Control Point Curve — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-043` | Curve · Extract Angle Isocurve — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-044` | Curve · Cross Section Profiles — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-045` | Curve · Sketch — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-046` | Curve · Sketch on Surface — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-047` | Curve · Sketch on Mesh — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-048` | Curve · Polyline on Surface — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-049` | Curve · Interp Curve on Surface — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-050` | Curve · Curve to Lines — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-051` | Curve · Section — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-052` | Curve · Match — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-053` | Curve · Silhouette — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-054` | Curve · Extract Wire Frame — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-055` | Curve · Soft Edit — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-056` | Curve · Offset Normal to Surface — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-057` | Curve · Arc Blend — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-16-058` | Curve · Intersect Two Sets — menu binding *(thủ công)* | E05 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-17 — 09-febe-matrix-surface-menu-workflows** · ✅0 🟡0 ⬜38

_Surface menu, Sweep/Loft/Trim settings theo Matrix_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-17-001` | Surface · Sweep 1 — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-002` | Surface · Sweep 1 History — menu binding *(thủ công)* | E06 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-003` | Surface · Sweep 2 — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-004` | Surface · Sweep 2 History — menu binding *(thủ công)* | E06 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-005` | Surface · Profile Sweep — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-006` | Surface · Rebuild — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-007` | Surface · Blend Surface — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-008` | Surface · Variable Blend Surfaces — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-009` | Surface · Loft — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-010` | Surface · Curve Network — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-011` | Surface · Shrink Trimmed Surface — menu binding *(thủ công)* | E06 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-012` | Surface · Patch — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-013` | Surface · Planar Curves — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-014` | Surface · Edge Curves — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-015` | Surface · Plane from Corners — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-016` | Surface · Untrim — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-017` | Surface · Surface Extrude All — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-018` | Surface · Rail Revolve — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-019` | Surface · Sweep 4 — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-020` | Surface · Fillet Surfaces — menu binding *(thủ công)* | E06 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-021` | Surface · Variable Fillet Surfaces — menu binding *(thủ công)* | E06 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-022` | Surface · Chamfer Surfaces — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-023` | Surface · Variable Chamfer Surfaces — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-024` | Surface · Offset Surface — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-025` | Surface · Variable Offset Surface — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-026` | Surface · Tween Surfaces — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-027` | Surface · Merge Surfaces — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-028` | Surface · Match Surfaces — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-029` | Surface · Drape — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-030` | Surface · Sweep Multi — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-031` | Surface · Split at Isocurve — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-032` | Surface · Extend Surface — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-033` | Surface · Extrude Normal to Surface — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-034` | Surface · Unroll Surface — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-035` | Surface · Soft Edit — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-036` | Surface · Cutting Plane — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-037` | Surface · Adjust Closed Surface Seam — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-17-038` | Surface · Set Surface Tangent — menu binding *(thủ công)* | E06 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-18 — 10-febe-matrix-solid-menu-workflows** · ✅0 🟡0 ⬜21

_Solid menu phù hợp trang sức: primitives, bool, extrusion, shell, text geometry_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-18-001` | Solid · Boolean Union — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-002` | Solid · Boolean Difference — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-003` | Solid · Boolean Intersection — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-004` | Solid · Boolean Two Objects — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-005` | Solid · Cap Planar — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-006` | Solid · Extract Surface — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-007` | Solid · Fillet Edge — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-008` | Solid · Text — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-009` | Solid · Extrude Curve Straight — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-010` | Solid · Solid Extrude All — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-011` | Solid · Pipe — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-012` | Solid · Box — menu binding *(thủ công)* | E04 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-013` | Solid · Sphere — menu binding *(thủ công)* | E04 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-014` | Solid · Ellipsoid — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-015` | Solid · Torus — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-016` | Solid · Cylinder — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-017` | Solid · Tube — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-018` | Solid · Pyramid — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-019` | Solid · Cone — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-020` | Solid · Truncated Cone — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-18-021` | Solid · Shell — menu binding *(thủ công)* | E07 | HIGH | 0/3 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-19 — 11-febe-matrix-utilities-and-mesh-health** · ✅0 🟡0 ⬜7

_Check/ShowEdges/validity, mesh inspection, model health_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-19-001` | Check Object Validity And Bad Objects *(thủ công)* | E07 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-19-002` | Show And Classify Naked Edges *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-19-003` | Bounding Box And Center Object *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-19-004` | Group And Ungroup Selection *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-19-005` | Mesh From NURBS For Production *(thủ công)* | E07 | NORMAL | 0/6 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-19-006` | Mesh Reduce Basic *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-19-007` | Object Health Report Panel *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-20 — 12-febe-matrix-measure-and-model-statistics** · ✅0 🟡0 ⬜5

_Distance/Angle/Length/Radius, kích thước và thống kê mẫu_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-20-001` | Distance And Length Measurement *(thủ công)* | E04 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-20-002` | Angle And Radius Measurement *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-20-003` | Bounding Size And Center Measurement *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-20-004` | Jewelry Object Count And Type Summary *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-20-005` | Measurement Label And Command Feedback *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-01 — 13-febe-matrix-transform-toolbar-bridges** · ✅0 🟡0 ⬜42

_Matrix Transform menu binds to Rhino/OCCT transforms_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-01-001` | Transform · Scale 3D — menu binding *(thủ công)* | E04 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-002` | Transform · Scale2D — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-003` | Transform · Scale1D — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-004` | Transform · Scale Non-Uniform — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-005` | Transform · Orient 2 Points — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-006` | Transform · Orient 3 Points — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-007` | Transform · Rotate3D — menu binding *(thủ công)* | E04 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-008` | Transform · Bend — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-009` | Transform · Array Polar — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-010` | Transform · Project To C-Plane — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-011` | Transform · Smart Flow — menu binding *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-012` | Transform · Smart Flow Rigid — menu binding *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-013` | Transform · Smart Flow Break — menu binding *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-014` | Transform · Smart Flow Pull Down Curves — menu binding *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-015` | Transform · Orient on Surface — menu binding *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-016` | Transform · Orient Perpendicular to Curve — menu binding *(thủ công)* | E07 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-017` | Transform · Orient on Curve — menu binding *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-018` | Transform · Symmetry — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-019` | Transform · Solid Points ON — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-020` | Transform · Cage Edit — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-021` | Transform · ReleaseFromCage — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-022` | Transform · Array Along Curve — menu binding *(thủ công)* | E07 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-023` | Transform · Array Linear — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-024` | Transform · Array Along Surface — menu binding *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-025` | Transform · Array Curve on Surface — menu binding *(thủ công)* | E07 | MEDIUM | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-026` | Transform · Array — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-027` | Transform · Flow Along Curve — menu binding *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-028` | Transform · Flow Along Surface — menu binding *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-029` | Transform · Splop — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-030` | Transform · Set Points — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-031` | Transform · Scale By Plane — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-032` | Transform · Stretch — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-033` | Transform · Taper — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-034` | Transform · Shear — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-035` | Transform · Twist — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-036` | Transform · Maelstrom — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-037` | Transform · Make Down Facing — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-038` | Transform · Remap to C-Plane — menu binding *(thủ công)* | E07 | NORMAL | 0/3 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-039` | Transform · MoveUVN — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-040` | Transform · Soft Move — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-041` | Transform · Create Cage — menu binding *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-01-042` | Transform menu action availability *(thủ công)* | E07 | NORMAL | 0/5 | ⬜ mới ở mức kế hoạch | — |

</details>

<details><summary><b>M04 — Builder, History và Ring</b> · ✅0 🟡0 ⬜51 · `OM9-M04` · L1_L4_SPECIFIED_NOT_IMPLEMENTED</summary>

**L1** — Builder, History và Ring · phụ thuộc: M01

**OM9-L2-01-04 — 01-be-associative-dependency-rebuild** · ✅0 🟡0 ⬜9

_DAG/history tối thiểu cho rail/profile/gems/settings/cutters, dirty propagation, rollback, stable semantic references._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-01-04-001` | Feature Record And Typed Dependency Edges | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-04-002` | Feature Provenance From Command | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-04-003` | Dirty Propagation And Topological Order | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-04-004` | Stable Semantic Reference Linking | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-04-005` | Incremental Recompute And Cache | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-04-006` | Builder Preview Vs Committed Features | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-04-007` | History Recompute Failure Rollback | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-04-008` | Delete Parent Or Change Input Policy | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-01-04-009` | Associative Rebuild Geometry Fixtures | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-11 — 02-febe-matrix-builder-ui-and-live-handles** · ✅0 🟡0 ⬜9

_Builder panel, live preview, viewport control handles_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-11-001` | Builder Typed Input Boxes *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-11-002` | Builder Mode Tabs And Context *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-11-003` | Builder Parameter Controls And Sliders *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-11-004` | Builder Live Preview Lifecycle *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-11-005` | Builder Viewport Control Handles *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-11-006` | Builder Handle And Panel Bidirectional Sync *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-11-007` | Builder Profile Selector Window *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-11-008` | Builder Accept Cancel Reset And Undo *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-11-009` | Builder F6 And Commandline Input Binding *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-12 — 03-febe-matrix-builder-styles-and-presets** · ✅0 🟡0 ⬜6

_Preset/Style theo Builder, snapshot các tham số thiết kế_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-12-001` | Builder Style Param Snapshot Schema | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-12-002` | Builder Style Browser And Thumbnail *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-12-003` | Builder Style Apply And Preview *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-12-004` | Builder Style Save New Name *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-12-005` | Builder Style Mode And Shape Filter *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-12-006` | Builder Style Match Attributes *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-03-13 — 04-febe-matrix-history-associative-updates** · ✅0 🟡0 ⬜6

_UI bật/tắt và chỉnh association của builder history_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-03-13-001` | Matrix History Record Toggle *(thủ công)* | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-13-002` | Matrix History Update Toggle *(thủ công)* | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-13-003` | Matrix History Broken Link Report *(thủ công)* | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-13-004` | Matrix Clear Single Object History *(thủ công)* | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-13-005` | Edit Existing Builder By History *(thủ công)* | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-03-13-006` | History Commandline And Preview Status *(thủ công)* | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-02 — 05-febe-ring-rail-profile-and-resize-tools** · ✅0 🟡0 ⬜9

_Ring rail, outside rail, profile placer, ring resizer_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-02-001` | Ring Rail Jewelry Size Standard *(thủ công)* | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-02-002` | Outside Ring Rail Generator *(thủ công)* | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-02-003` | Profile Placer On Rail *(thủ công)* | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-02-004` | Custom Rail And Advanced Rail *(thủ công)* | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-02-005` | Four Rail Profile Tool *(thủ công)* | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-02-006` | Profile Edit Merge And Endcap *(thủ công)* | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-02-007` | Ring Resizer Builder *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-02-008` | Rail Profile Sweep Associative Output *(thủ công)* | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-02-009` | Rail And Profile Viewport Handles *(thủ công)* | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-03 — 06-febe-jewelry-ring-builder-family** · ✅0 🟡0 ⬜7

_Eternity/Signet/Raised/Award ring builders theo NURBS_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-03-001` | Eternity Band Builder *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-03-002` | Signet Ring Builder *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-03-003` | Raised Band Builder *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-03-004` | Award Ring Builder *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-03-005` | Ring Builder Style Presets *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-03-006` | Ring Builder History Updates *(thủ công)* | E09 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-03-007` | Ring Builder Final B-Rep Validation *(thủ công)* | E09 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-19 — 07-be-jewelry-metal-weight-calculations** · ✅0 🟡0 ⬜5

_Tính thể tích, vật liệu, trọng lượng kim loại trang sức_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-19-001` | Metal Volume From Closed B-Rep | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-19-002` | Metal Material Density Registry | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-19-003` | Metal Estimated Mass Calculation | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-19-004` | Multi Material Model Mass Breakdown | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-19-005` | Metal Weight Error And Open B-Rep Policy | E09 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

</details>

<details><summary><b>M05 — Gems và Layout</b> · ✅0 🟡0 ⬜43 · `OM9-M05` · L1_L4_SPECIFIED_NOT_IMPLEMENTED</summary>

**L1** — Gems và Layout · phụ thuộc: M03

**OM9-L2-04-09 — 01-febe-gem-asset-shapes-and-loader** · ✅0 🟡0 ⬜8

_Gem shapes/cuts, size, loader/instances_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-09-001` | Gem Shape Cut Dimension Schema | E10 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-09-002` | Gem Reference Mesh And B-Rep Assets | E10 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-09-003` | Gem Loader Shape Cut Size Picker *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-09-004` | Gem Custom Shape From Profile *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-09-005` | Gem Emerald And Baguette Builders *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-09-006` | Gem Instance Create Rotate And Position *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-09-007` | Gem Asset Properties And Layer Defaults *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-09-008` | Gem Facet Outline Girdle Reference API | E10 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-10 — 02-febe-gem-on-curves-and-surfaces** · ✅0 🟡0 ⬜11

_Gems on Curve/Surface, spacing, orientation, history_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-10-001` | Gem On Curve Basic *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-002` | Gem On Curve Advanced *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-003` | Gem Count On Curve *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-004` | Gem List On Curve *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-005` | Gem On Curve Multi *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-006` | Gem Profile Curve And Guides *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-007` | Gem On Surface *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-008` | Gem Follow Underlying Curve *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-009` | Gem Flow Placement *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-010` | Gem Splop On Surface *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-10-011` | Gem To Curve Surface History Links *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-11 — 03-febe-gem-halo-cluster-baguette-layouts** · ✅0 🟡0 ⬜7

_Halo/Cluster/Baguette Channel/Between Curves_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-11-001` | Halo Builder *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-11-002` | Cluster Builder *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-11-003` | Baguette Channel Layout *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-11-004` | Tapered Baguettes Between Two Curves *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-11-005` | Gems Between Two Curves *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-11-006` | Orient To Gem *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-11-007` | Halo Cluster Edit Parameters *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-12 — 04-febe-gem-pave-layout-and-azure** · ✅0 🟡0 ⬜8

_Auto Pavé/Pavé/Azure layout_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-12-001` | Auto Pave Builder *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-12-002` | Manual Pave Builder *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-12-003` | Pave Azure Builder *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-12-004` | Pave Prong Layout *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-12-005` | Pave Sphere Layout *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-12-006` | Gem Springs And Relaxation *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-12-007` | Pave Region Controls And Preview *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-12-008` | Pave History Geometry Refresh *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-13 — 05-febe-gem-instance-edits-and-reports** · ✅0 🟡0 ⬜9

_Gem Control/Map/Reporter, gemstone editing_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-13-001` | Gem Positioner MSR *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-13-002` | Gem Control And Size Edits *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-13-003` | Gem Match Attributes *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-13-004` | Gem Style Save *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-13-005` | Gem Style Load *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-13-006` | Gem Update And Regenerate *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-13-007` | Gem Map Viewport Overlay *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-13-008` | Gem Reporter Count And Weight *(thủ công)* | E10 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-13-009` | Gem Attribute History And ID Preservation *(thủ công)* | E10 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

</details>

<details><summary><b>M06 — Settings và Prongs</b> · ✅0 🟡0 ⬜19 · `OM9-M06` · L1_L4_SPECIFIED_NOT_IMPLEMENTED</summary>

**L1** — Settings và Prongs · phụ thuộc: M03, M05

**OM9-L2-04-14 — 01-febe-head-and-bezel-brep-settings** · ✅0 🟡0 ⬜9

_Head Builder, Bezel Builder với NURBS/BRep_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-14-001` | Head Builder Basket *(thủ công)* | E11 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-14-002` | Head Builder Half Bezel Mode *(thủ công)* | E11 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-14-003` | Head Builder Prong Rail Level Edit *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-14-004` | Bezel Builder Full *(thủ công)* | E11 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-14-005` | Bezel Builder Under Bezel And Gallery *(thủ công)* | E11 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-14-006` | Scallop Bezel Builder *(thủ công)* | E11 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-14-007` | Head Bezel Profile Browser *(thủ công)* | E11 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-14-008` | Head Bezel Girdle Seat Attachment *(thủ công)* | E11 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-14-009` | Head Bezel Geometry Validity And Update *(thủ công)* | E11 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-15 — 02-febe-prong-channel-metal-and-bead-settings** · ✅0 🟡0 ⬜10

_Prong/Channel/Metal Piece/Bead settings_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-15-001` | Prong Builder Gem Line *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-15-002` | Prong Editor Selected *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-15-003` | Prong On Surface *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-15-004` | Metal Piece From Gems *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-15-005` | Channel Border Curve *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-15-006` | Bead On Curve Milgrain *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-15-007` | Bead On Surface *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-15-008` | Pull Object To Rail Setting *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-15-009` | Setting Row End Modes And Prong Spacing *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-15-010` | Settings Associative Edit And Mass *(thủ công)* | E11 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

</details>

<details><summary><b>M07 — Cutters và kiểm tra chế tạo</b> · ✅0 🟡0 ⬜19 · `OM9-M07` · L1_L4_SPECIFIED_NOT_IMPLEMENTED</summary>

**L1** — Cutters và kiểm tra chế tạo · phụ thuộc: M03

**OM9-L2-04-16 — 01-febe-single-gem-seat-cutter-workflows** · ✅0 🟡0 ⬜7

_Gem Seat/Gem Cutters cho 1 viên đá_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-16-001` | Gem Cutter Azure For Single Stone *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-16-002` | Gem Cutter Multiple Stones *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-16-003` | Gem Cutter Library Profiles *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-16-004` | Bezel Cutter Builder *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-16-005` | Cutter Pavilion Clearance And Depth Controls *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-16-006` | Interactive Boolean On Metal *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-16-007` | Cutter Body Preserve Remove Option *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-17 — 02-febe-channel-microprong-brightcut-cutters** · ✅0 🟡0 ⬜6

_Channel/MicroProng/Bright Cut cutters_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-17-001` | Channel Builder Cutter *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-17-002` | Microprong Cutter *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-17-003` | Bright Cut Channel Builder *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-17-004` | Bright Cutter Single Gem *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-17-005` | Channel Cutter Profile And End Controls *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-17-006` | Multiple Stone Interactive Boolean *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-18 — 03-febe-ring-finger-plane-quad-cutters** · ✅0 🟡0 ⬜6

_Cut to finger rail/plane/cube/quad cut jewelry_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-18-001` | Cut To Finger Rail *(thủ công)* | E12 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-18-002` | Plane And Cube Cutters *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-18-003` | Quad Flip Routines *(thủ công)* | E12 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-18-004` | Boolean Builder Operations *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-18-005` | Cutting Plane Mirror History *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-18-006` | Cutters Geometry Watertight Check *(thủ công)* | E12 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

</details>

<details><summary><b>M08 — Advanced Jewelry Modeling</b> · ✅0 🟡0 ⬜51 · `OM9-M08` · L1_L4_SPECIFIED_NOT_IMPLEMENTED</summary>

**L1** — Advanced Jewelry Modeling · phụ thuộc: M03, M05, M06

**OM9-L2-04-04 — 01-febe-smart-target-curve-blend-network** · ✅0 🟡0 ⬜5

_Smart Target/MSR/Blend curve networks phục vụ form nhẫn_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-04-001` | Smart Target Place And Add *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-04-002` | Smart Target Curve End *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-04-003` | Smart Blend Curve Network *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-04-004` | Smart MSR Control Handles *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-04-005` | Smart Target Selection And Update *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-05 — 02-febe-matrix-smartflow-history-morph** · ✅0 🟡0 ⬜8

_Smart Flow auto-base/surface mapping, rigid objects, associative history_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-05-001` | Smartflow Autobase Surface Generation *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-05-002` | Smartflow Mapping Base To Target *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-05-003` | Smartflow Rigid Object Placement *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-05-004` | Smartflow History Recompute *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-05-005` | Smartflow Break And Reconnect *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-05-006` | Smartflow Pulldown Reference Curves *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-05-007` | Smartflow UV Normal Align Assistance *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-05-008` | Smartflow Overhang Tolerance Controls *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-06 — 03-febe-jewelry-ornament-builder-family** · ✅0 🟡0 ⬜6

_Knot/FreeformKnot/JumpRing/Nautilus và pattern ornaments_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-06-001` | Knot Builder *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-06-002` | Freeform Knot Builder *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-06-003` | Jump Ring Builder *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-06-004` | Nautilus Builder *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-06-005` | Ornament Pattern Builder *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-06-006` | Ornament Builder Style And History *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-07 — 04-febe-jewelry-surface-object-placement-tools** · ✅0 🟡0 ⬜9

_Orient/array và đặt chi tiết trên curve/surface trang sức_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-07-001` | Orient Object On Curve *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-07-002` | Orient Object On Surface *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-07-003` | Array Along Curve Jewelry *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-07-004` | Array Along Surface UV *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-07-005` | Array Curve On Surface *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-07-006` | Flow Along Curve Jewelry *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-07-007` | Flow Along Surface Jewelry *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-07-008` | Splop Object On Surface *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-07-009` | Object On Curve Tool *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-04-08 — 05-febe-matrix-advanced-deformation-ux** · ✅0 🟡0 ⬜8

_Bend/Twist/Taper/CageEdit/SoftMove dùng thiết kế đường nét trang sức_

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-04-08-001` | Bend Deformation Command *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-08-002` | Stretch And Taper Deformation *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-08-003` | Twist And Maelstrom Deformation *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-08-004` | Shear And Softmove Deformation *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-08-005` | Cage Edit And Release Command *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-08-006` | Move Uvn And Setpoints *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-08-007` | Symmetry And Live Mirror *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-04-08-008` | Deformation Preview And Undo *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-05-01 — 06-febe-matrix9-smart-pattern-builder** · ✅0 🟡0 ⬜5

_Smart Pattern trên surface: phân phối chi tiết trang sức, group/mesh, attraction và preview._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-05-01-001` | Smartpattern Source Pattern Choice *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-01-002` | Smartpattern UV Distribution *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-01-003` | Smartpattern Group And Segment Controls *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-01-004` | Smartpattern Attractor Region Controls *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-01-005` | Smartpattern History And Viewport Preview *(thủ công)* | E13 | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-05-02 — 07-febe-matrix9-rope-builder** · ✅0 🟡0 ⬜5

_Rope Builder: braided rope quanh rail curve; cỡ sợi, xoắn, spacing, cap._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-05-02-001` | Rope Builder Curve Input *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-02-002` | Rope Builder Thread Count And Diameter *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-02-003` | Rope Builder Twists And Spacing *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-02-004` | Rope Builder End Cap Modes *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-02-005` | Rope Builder Interactive Preview *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-05-03 — 08-febe-user-design-libraries** · ✅0 🟡0 ⬜5

_User Library chứa cả mẫu thiết kế; import/open model và phân loại thư mục riêng._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-05-03-001` | User Library Folder Catalog *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-03-002` | User Library Asset Register And Preview *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-03-003` | User Library Insert Model *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-03-004` | User Library Search And Select *(thủ công)* | E13 | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-05-03-005` | User Library Save And Import *(thủ công)* | E13 | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

</details>

<details><summary><b>M09 — Cổng nghiệm thu và phát hành (không phát sinh L2 owner)</b> · ✅0 🟡0 ⬜0 · `OM9-M09` · L1_RELEASE_GATE_L4_PLANNED_NOT_EXECUTED</summary>

**L1** — Cổng nghiệm thu và phát hành (không phát sinh L2 owner) · phụ thuộc: M00, M01, M02, M03, M04, M05, M06, M07, M08

</details>

<details><summary><b>ZAFTER — Phạm vi mở rộng sau các release gate chính</b> · ✅0 🟡0 ⬜170 · `OM9-AFTER` · NOT_STARTED</summary>

**L1** — Phạm vi mở rộng sau các release gate chính · phụ thuộc: M09

**OM9-L2-06-01 — 01-be-advanced-topology-reconciliation** · ✅0 🟡0 ⬜5

_Remap tham chiếu faces/edges khi hình học thay đổi phức tạp._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-06-01-001` | Advanced Topological Name Recovery | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-01-002` | Shape History Graph Reconciliation | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-01-003` | Subshape Geometric Signature Matching | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-01-004` | Ambiguous Reference User Remapping Policy | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-01-005` | Topology Remap Regression Library | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-06-02 — 02-febe-additional-jewelry-cad-formats** · ✅0 🟡0 ⬜4

_Các adapter bổ sung như STEP/IGES khi cần trao đổi với xưởng/đối tác trang sức._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-06-02-001` | Step Part Model Import Adapter | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-02-002` | Iges Surface Import Adapter | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-02-003` | Additional Format File Dialog Integration *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-02-004` | Exchange Format Compatibility Report *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-06-03 — 03-febe-advanced-preferences-i18n-keymap** · ✅0 🟡0 ⬜5

_Cấu hình giao diện chi tiết, hệ phím tắt và bản dịch mở rộng._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-06-03-001` | Advanced Preferences Dialog *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-03-002` | Hotkey Mapping Editor *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-03-003` | Language Resource Localization *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-03-004` | Custom Toolbar And Menu Workspaces *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-03-005` | Preference Profile Import Export *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-06-04 — 04-be-profiling-benchmark-lab** · ✅0 🟡0 ⬜4

_Profiling GPU/OCCT long-run; benchmark chuyên sâu, không phải kiểm thử bắt buộc L4._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-06-04-001` | OCCT Performance Microbenchmarks | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-04-002` | GPU Tessellation Frame Profiling | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-04-003` | Massive Gem Instance Stress Suite | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-06-04-004` | Repeatable Cad Baseline Benchmark Data | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-07-01 — 01-febe-advanced-rhino-macros-and-scripting** · ✅0 🟡0 ⬜4

_Scripting/macro nâng cao để tự động hóa modeling sau._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-07-01-001` | Rhino Command Alias Script Editor *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-01-002` | Macro Recording And Replay *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-01-003` | Macro Permission And Reentrancy Boundary | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-01-004` | Custom Menu To Macro Binding *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-07-02 — 02-febe-advanced-selection-filters** · ✅0 🟡0 ⬜4

_Rhino detailed selection filters beyond jewelry Super Select._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-07-02-001` | Selection Filter By Rhino Object Class *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-02-002` | Selection Filter By Color Name Layer *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-02-003` | Subobject Face Edge Select Filters *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-02-004` | Save Filter Presets *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-07-03 — 03-febe-named-views-and-matrix-viewmanager** · ✅0 🟡0 ⬜4

_Một nơi lưu/phục hồi camera/view names và UI Matrix View Manager._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-07-03-001` | Named View Camera Save *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-03-002` | Named View Manager Palette *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-03-003` | Named View Restore And Rename *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-03-004` | Matrix View Manager Command Bindings *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-07-04 — 04-febe-zebra-curvature-surface-analysis** · ✅0 🟡0 ⬜5

_Zebra, EMap, heatmap curvature và các visual analysis nâng cao._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-07-04-001` | Zebra Surface Continuity Analysis *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-04-002` | Curvature Heatmap Analysis *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-04-003` | Curvature Graph Analysis *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-04-004` | Environment Reflection Surface Analysis *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-04-005` | Draft Angle Visual Analysis *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-07-05 — 05-febe-advanced-layers-groups-blocks** · ✅0 🟡0 ⬜4

_Rhino manager nâng cao cho layers/groups/blocks._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-07-05-001` | Advanced Layer Hierarchy Editor *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-05-002` | Rhino Group Management Panel *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-05-003` | Block Definition And Instances Manager *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-07-05-004` | Worksession Unsupported Deferred Import *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-08-01 — 01-be-shared-offline-render-material-light-pipeline** · ✅0 🟡0 ⬜5

_Offline renderer, scene lights and PBR material engine._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-08-01-001` | Offline Renderer Scene Translation | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-01-002` | PBR Material And Shader Schema | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-01-003` | Scene Light And Environment Model | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-01-004` | Offline Render Job Lifecycle | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-01-005` | Render Output Image Buffer And Color Management | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-08-02 — 02-febe-jewelry-render-lookdev-and-light-rigs** · ✅0 🟡0 ⬜5

_Jewelry lighting/metal/gem lookdev._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-08-02-001` | Jewelry Metal Lookdev Material Picker *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-02-002` | Gem Refraction And Caustic Lookdev *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-02-003` | Studio Light Rig And Reflector Setup *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-02-004` | Render Camera Backdrop And Props *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-02-005` | Material Preview And Style Application *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-08-03 — 03-febe-jewelry-animation-and-moviemaker** · ✅0 🟡0 ⬜4

_Animations / movie maker._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-08-03-001` | Animation Turntable Builder *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-03-002` | Animation Keyframe Timeline *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-03-003` | Matrix Movie Maker Output *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-03-004` | Animation Preview And Cancel *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-08-04 — 04-febe-jewelry-batch-render-and-postprocessing** · ✅0 🟡0 ⬜5

_Batch render and postprocess._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-08-04-001` | Batch Render Design Variants *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-04-002` | Render Job Queue And Scheduler *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-04-003` | Render Editor Crop Watermark And Alpha *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-04-004` | Four Viewport Capture And Layout *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-04-005` | Save Render To Image Formats *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-08-05 — 05-febe-design-report-render-layout-and-presentation** · ✅0 🟡0 ⬜4

_Rendered images in catalogs/layout/report._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-08-05-001` | Design Report Cover And Template *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-05-002` | Report Render Images And Gem Table *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-05-003` | Catalog Layout Pages And Labels *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-08-05-004` | Report Export And Print *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-01 — 01-febe-tsplines-legacy-hud-command-compat** · ✅0 🟡0 ⬜6

_Legacy T-Splines HUD and command workflow._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-01-001` | T-Splines Legacy Hud Activation *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-01-002` | T-Splines Select Vertex Edge Face Object Mode *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-01-003` | T-Splines Gumball Translate Scale Rotate *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-01-004` | T-Splines Box Smooth Toggle *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-01-005` | T-Splines X Y Symmetry Toggles *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-01-006` | T-Splines Compatibility Mode Settings *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-02 — 02-febe-tsplines-legacy-primitive-builder-compat** · ✅0 🟡0 ⬜6

_T-Splines primitive builders incl. T-Spline Signet, Bezel._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-02-001` | T-Spline Box Sphere Cylinder Cone Torus *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-02-002` | T-Spline Plane And Tube From Curves *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-02-003` | T-Spline Bezel Builder *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-02-004` | T-Spline Under Bezel Builder *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-02-005` | T-Spline Signet Ring Builder *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-02-006` | T-Spline Skin Loft Pipe *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-03 — 03-be-tsplines-legacy-topology-tjunction-parity** · ✅0 🟡0 ⬜7

_T-spline topological model including T-junctions._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-03-001` | T-Spline Control Mesh Topology | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-03-002` | T-Spline Tjunction And Extraordinary Points | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-03-003` | T-Spline Insert Edges And Points | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-03-004` | T-Spline Subdivide And Unweld Weld | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-03-005` | T-Spline Bridge Append Fill And Extrude | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-03-006` | T-Spline Crease Weight And Smoothness | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-03-007` | T-Spline Topology Geometry Regression | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-04 — 04-be-tsplines-legacy-import-export-compat** · ✅0 🟡0 ⬜4

_Legacy T-spline conversion/interoperability._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-04-001` | T-Spline To Rhino Surface Converter | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-04-002` | Rhino Surface To T-Spline Eligibility | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-04-003` | T-Spline To Mesh Tessellation Export | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-04-004` | Legacy T-Spline File Data Bridge | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-05 — 05-febe-clayoo-hud-gumball-selection-modes** · ✅0 🟡0 ⬜6

_Clayoo sub-object HUD/controls._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-05-001` | Clayoo Clayedit Enable Hud *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-05-002` | Clayoo Object Vertex Edge Face Modes *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-05-003` | Clayoo Gumball Translation Scale Rotation *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-05-004` | Clayoo Box Smooth Visualization *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-05-005` | Clayoo Hotkey Profiles *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-05-006` | Clayoo Points On Surface Overlay *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-06 — 06-febe-clayoo-subd-primitive-generators** · ✅0 🟡0 ⬜5

_Clayoo SubD primitives._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-06-001` | Clayoo Box And Plane Generators *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-06-002` | Clayoo Sphere Quad And Polygonal *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-06-003` | Clayoo Cylinder Cone Generators *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-06-004` | Clayoo Shape Viewport Control Handles *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-06-005` | Clayoo Subdivision Level Preview *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-07 — 07-febe-clayoo-local-topology-construction** · ✅0 🟡0 ⬜7

_Clayoo topology editing._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-07-001` | Clayoo Divide And Split Sides *(thủ công)* | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-07-002` | Clayoo Inset And Knife *(thủ công)* | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-07-003` | Clayoo Extrude And Offset *(thủ công)* | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-07-004` | Clayoo Bridge And Append Face *(thủ công)* | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-07-005` | Clayoo Fill And Shell *(thủ công)* | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-07-006` | Clayoo Collapse And Merge *(thủ công)* | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-07-007` | Clayoo Topology Operation Undo *(thủ công)* | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-08 — 08-febe-clayoo-curve-driven-shapes** · ✅0 🟡0 ⬜6

_Clayoo Sweep/Loft/Pipe curve-driven meshes._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-08-001` | Clayoo Pipe From Curves *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-08-002` | Clayoo Sweep One Rail *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-08-003` | Clayoo Sweep Two Rails *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-08-004` | Clayoo Loft And Revolve *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-08-005` | Clayoo Wrap Curves And Mesh *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-08-006` | Clayoo Create By One Two Curves *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-09 — 09-febe-clayoo-jewelry-subd-builders** · ✅0 🟡0 ⬜4

_Clayoo Ring, Signet, Bezel builders._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-09-001` | Clayoo Ring Builder *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-09-002` | Clayoo Signet Ring Builder *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-09-003` | Clayoo Bezel Around Gem *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-09-004` | Clayoo Jewelry Builder Handles *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-10 — 10-febe-clayoo-crease-weld-symmetry** · ✅0 🟡0 ⬜6

_Clayoo crease/weld/symmetry._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-10-001` | Clayoo Crease And Uncrease Edges *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-10-002` | Clayoo Weld Vertices With Tolerance *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-10-003` | Clayoo Match Edges To Curve *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-10-004` | Clayoo Symmetry Plane And Loop *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-10-005` | Clayoo Project To Plane *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-10-006` | Clayoo Flip Normals And Winding *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-11 — 11-be-clayoo-nurbs-tspline-mesh-conversion** · ✅0 🟡0 ⬜6

_Clayoo SubD to mesh/NURBS/legacy TSpline conversions._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-11-001` | Clayoo Controlmesh To SubD Evaluator | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-11-002` | Clayoo SubD To NURBS Surfaces | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-11-003` | Clayoo SubD To Triangle Mesh | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-11-004` | Clayoo NURBS To SubD Mesh Policy | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-11-005` | Clayoo T-Spline Conversion Boundary | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-11-006` | Clayoo Conversion Error And Loss Report | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-12 — 12-febe-clayoo-selection-set-operations** · ✅0 🟡0 ⬜6

_Clayoo selection sets and growth/loop operations._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-12-001` | Clayoo Ring Edge Loop And UV Select *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-12-002` | Clayoo Select All None Invert *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-12-003` | Clayoo Paint Selection *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-12-004` | Clayoo Grow Shrink Selection *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-12-005` | Clayoo Crease And Naked Edge Filters *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-12-006` | Clayoo Saved Selection Sets *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-13 — 13-febe-clayemboss-procedural-relief** · ✅0 🟡0 ⬜5

_ClayEmboss profile/relief procedural operations._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-13-001` | Clayemboss Main Procedural Relief *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-13-002` | Clayemboss By Profile *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-13-003` | Clayemboss Extrude And Revolve *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-13-004` | Clayemboss Base On Geometry *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-13-005` | Clayemboss Height Parameter Controls *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-14 — 14-febe-clayemboss-brush-stamp-sculpt** · ✅0 🟡0 ⬜5

_ClayEmboss brush/sculpt/stamp._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-14-001` | Clayemboss Stamp Brush *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-14-002` | Clayemboss Carbon Copy And Brush *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-14-003` | Clayemboss Brush Sculpt *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-14-004` | Clayemboss Smoothing Brush *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-14-005` | Clayemboss Brush Settings And Preview *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-15 — 15-febe-clayemboss-texture-heightfield-decimation** · ✅0 🟡0 ⬜5

_ClayEmboss texture, heightfield and decimation._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-15-001` | Clayemboss Texture From Image *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-15-002` | Clayemboss Heightfield Image *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-15-003` | Clayemboss Resources Browser *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-15-004` | Clayemboss Decimator *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-15-005` | Clayemboss Texture Repeat And Scale *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-16 — 16-febe-clayemboss-text-on-curve** · ✅0 🟡0 ⬜4

_Command gốc ClayEmbossTextOnCurve, text 2D/3D chạy theo curve._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-16-001` | Clayemboss Text On Curve 2d *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-16-002` | Clayemboss Text On Curve 3d *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-16-003` | Clayemboss Text Glyph Spacing Direction *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-16-004` | Clayemboss Text On Curve Builder UI *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-09-17 — 17-febe-clay-blueprint** · ✅0 🟡0 ⬜4

_Blueprint tham chiếu ảnh 3D theo các mặt, command gốc ClayBluePrint._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-09-17-001` | Clay Blueprint Multiview Image Placement *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-17-002` | Clay Blueprint Image Alignment *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-17-003` | Clay Blueprint Transparent Overlays *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-09-17-004` | Clay Blueprint Preset Save Load *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-10-01 — 01-febe-matrix-art-and-surface-texture-builder** · ✅0 🟡0 ⬜5

_Bitmap/heightfield Matrix Art + Texture Builder mesh-on-surface theo grayscale, dùng shared heightfield utilities._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-10-01-001` | Matrix Art Bitmap To Heightfield *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-01-002` | Matrix Art Relief Region Mask *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-01-003` | Matrix Texture Builder On Surface *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-01-004` | Matrix Texture Builder UV Layout *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-01-005` | Heightfield Result To Mesh *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-10-02 — 02-febe-advanced-jewelry-mesh-repair** · ✅0 🟡0 ⬜5

_Advanced repair/decimation for casting or jewelry additive manufacture._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-10-02-001` | Mesh Weld Normals Repair *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-02-002` | Mesh Manifold And Hole Repair *(thủ công)* | AFTER | HIGH | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-02-003` | Mesh Intersection And Island Repair *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-02-004` | Mesh Reduction Advanced *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-02-005` | Mesh Repair Before After Report *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

**OM9-L2-10-03 — 03-febe-jewelry-print-supports-and-sprues** · ✅0 🟡0 ⬜6

_Supports, sprues and manufacturing aids for casting/printing jewelry._

| L3 | Tiêu đề | Stage | Risk | L4 | Trạng thái | Bằng chứng |
|---|---|---|---|---:|---|---|
| `OM9-L3-10-03-001` | Ring Support Sprue Builder *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-03-002` | Support Base And Support Ring *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-03-003` | Cylinder And Tapered Support Generators *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-03-004` | Blend Supports Between Surfaces *(thủ công)* | AFTER | MEDIUM | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-03-005` | Support Region Analysis *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |
| `OM9-L3-10-03-006` | 3d Printing Export Support Workflow *(thủ công)* | AFTER | NORMAL | 0/0 | ⬜ mới ở mức kế hoạch | — |

</details>

## Việc đã ghi gần đây (live ledger)

| Ngày | Mục | Trạng thái | Phạm vi |
|---|---|---|---|
| 2026-10-20 | `m01-l3-03-14-008-layer-state-persistence-visual-check-2026-10-20` | standalone_implementation_verified_awaiting_manual_review | OM9-L3-03-14-008 (Layer State Persistence Visual Check) - 4 L4 tasks of module OM9-L2-03-14 |
| 2026-10-20 | `m01-l3-03-14-007-object-property-editor-adapter-2026-10-20` | standalone_implementation_verified_awaiting_manual_review | OM9-L3-03-14-007 (Object Property Editor Adapter) - 4 L4 tasks of module OM9-L2-03-14 |
| 2026-10-20 | `m01-l3-03-14-006-superselect-by-jewelry-object-type-2026-10-20` | standalone_implementation_verified_awaiting_manual_review | OM9-L3-03-14-006 (Superselect By Jewelry Object Type) - 4 L4 tasks of module OM9-L2-03-14 |
| 2026-10-20 | `m01-l3-03-14-005-matrix-object-info-overlay-2026-10-20` | standalone_implementation_verified_awaiting_manual_review | OM9-L3-03-14-005 (Matrix Object Info Overlay) - 4 L4 tasks of module OM9-L2-03-14 |
| 2026-10-20 | `m01-l3-03-14-004-select-all-objects-by-layer-2026-10-20` | standalone_implementation_verified_awaiting_manual_review | OM9-L3-03-14-004 (Select All Objects By Layer) - 4 L4 tasks of module OM9-L2-03-14 |
| 2026-10-20 | `m01-l3-03-14-003-layer-visible-and-locked-controls-2026-10-20` | standalone_implementation_verified_awaiting_manual_review | OM9-L3-03-14-003 (Layer Visible And Locked Controls) - 4 L4 tasks of module OM9-L2-03-14 |
| 2026-10-20 | `m01-l3-03-14-002-current-layer-and-assignment-ui-2026-10-20` | standalone_implementation_verified_awaiting_manual_review | OM9-L3-03-14-002 (Current Layer And Assignment UI) - 4 L4 tasks of module OM9-L2-03-14 |
| 2026-10-20 | `m01-l3-03-14-001-matrix-layer-category-palette-2026-10-20` | standalone_implementation_verified_awaiting_manual_review | OM9-L3-03-14-001 (Matrix Layer Category Palette) - 4 L4 tasks of module OM9-L2-03-14 |

## Việc kế tiếp

Gom MOT checklist test cho ca 8 L3 va bao cao tong ket module OM9-L2-03-14.
