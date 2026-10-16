# OM9-M09-L4-GMVP-005 — Workbench 4 viewport, command và điều khiển

**Gate:** `G-MVP` · **Loại:** `UI_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Kiểm đủ hành vi native shell: bốn vùng nhìn, camera, chọn đối tượng, snap, menu F6, docking và trạng thái lệnh.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Mở tài liệu có mesh/curve/BRep: kiểm 4 viewport đồng bộ scene, hướng camera preset và active indicator.
4. Kiểm pan/orbit/zoom, window vs crossing selection, subobject selection, snaps, F6 theo selection, docking/undocking.
5. Chạy command qua menu và command line; kiểm enabled/disabled theo capability, Esc/Cancel không để ghost preview.
6. Thực hiện manual test trên từng OS/DPI/input device tuyên bố; ghi ảnh/video, thao tác và observed.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-03-01-001` · `OM9-L2-03-01` · `E02` · `cap.03-01.matrix-four-viewport-grid-layout`
- `OM9-L3-03-01-003` · `OM9-L2-03-01` · `E02` · `cap.03-01.active-viewport-indicator`
- `OM9-L3-03-09-001` · `OM9-L2-03-09` · `E03` · `cap.03-09.f6-selected-object-class-dispatch`
- `OM9-L3-03-07-002` · `OM9-L2-03-07` · `E02` · `cap.03-07.matrix-main-menu-category-tabs`
- `OM9-L3-03-02-001` · `OM9-L2-03-02` · `E03` · `cap.03-02.perspective-right-drag-orbit`
- `OM9-L3-03-03-001` · `OM9-L2-03-03` · `E03` · `cap.03-03.matrix-click-shift-ctrl-select`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-002`
- `OM9-M09-L4-GMVP-004`

## Acceptance

**Oracle cụ thể:** All 4 synchronized viewports, active indicator, F6, snaps, selection, docking, command state and Cancel meet the documented per-platform steps; enabled commands never silently no-op.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Command hiển thị enabled nhưng no-op: FAIL.
- UI rendering khác camera không được suy ra geometry corruption nếu BRep vẫn đúng, phân loại defect riêng.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-005-01` (ACCEPTANCE): Mở tài liệu có mesh/curve/BRep: kiểm 4 viewport đồng bộ scene, hướng camera preset và active indicator.
- `M09-TC-GMVP-005-02` (ACCEPTANCE): Kiểm pan/orbit/zoom, window vs crossing selection, subobject selection, snaps, F6 theo selection, docking/undocking.
- `M09-TC-GMVP-005-03` (ACCEPTANCE): Chạy command qua menu và command line; kiểm enabled/disabled theo capability, Esc/Cancel không để ghost preview.
- `M09-TC-GMVP-005-04` (ACCEPTANCE): Thực hiện manual test trên từng OS/DPI/input device tuyên bố; ghi ảnh/video, thao tác và observed.
- `M09-TC-GMVP-005-N01` (NEGATIVE): Command hiển thị enabled nhưng no-op: FAIL.
- `M09-TC-GMVP-005-N02` (NEGATIVE): UI rendering khác camera không được suy ra geometry corruption nếu BRep vẫn đúng, phân loại defect riêng.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-005/workbench-manual-matrix.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- Matrix 8 Book 1 Ch.1-4; Rhino 5 viewport and selection
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `matrix8_book_1.pdf`: trang PDF 11, 12, 19
- `rhino5_manual.pdf`: trang PDF 39, 40

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
