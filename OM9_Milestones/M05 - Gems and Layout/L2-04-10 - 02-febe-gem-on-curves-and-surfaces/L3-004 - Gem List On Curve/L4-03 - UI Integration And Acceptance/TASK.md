# OM9-M05-L4-10-004-03 — Gem List On Curve

**Stage:** E10 (M05) | **Owner:** `OM9-L2-04-10` | **Parent:** `OM9-L3-04-10-004` | **Phase:** `UI_INTEGRATION_AND_ACCEPTANCE`.

**Trạng thái:** `PLANNED_BLOCKED_G00_E09`. Đây là **kế hoạch**, không phải code hay test đã chạy.

## Mục tiêu và hợp đồng

place heterogeneous sizes/shapes from gem list.

- **Input dự kiến:** curve and ordered heterogeneous GemSpec list.
- **Output dự kiến:** stable placement order and per-gem size/shape mapping.
- **Kiểm tra độc lập/lỗi trọng yếu:** empty list, too-long list, and duplicate spec references handled explicitly.
- **Ranh giới:** Dùng arc-length / surface evaluator chung.. Không tạo ra tính năng do M06/M07 sở hữu.

## Điều kiện bắt buộc

- [ ] G00-01..G00-07 PASS, documented license/legal approval
- [ ] E01..E09 PASS with actual test evidence and compatible contracts
- [ ] QymCAD HEAD == pinned SHA or explicit approved rebaseline
- [ ] Relevant module source/contract audited locally; never infer paths from planning ZIP
- [ ] Rust-first app + OCCT C++ bridge for OCCT operations; approved FFI boundary
- [ ] Audit task contract and dependency domain outputs have passed

## Các bước Luna phải thực hiện

1. Expose command through real menu/F6/selection context; use existing command dispatch and semantic state, not dummy buttons
2. Implement preview/cancel/confirm with no phantom document objects; ensure control values use mm/deg and invalid controls are disabled
3. Run guided UI scenario: Create layout alternating round and oval sizes; verify exact order
4. Check viewport response, Undo/Redo, save/open and source-edit history with screenshots/log
5. Mark READY_FOR_MANUAL_TEST only after runtime UI and BE tests; DONE only after human sign-off

## Tiêu chí kiểm chứng

- [ ] Live command/selection/preview/commit/undo/save-open behavior is correct
- [ ] BE regression and UI smoke have actual logs
- [ ] Screenshots, expected-versus-actual and human sign-off recorded
- [ ] No placeholder actions; unresolved blockers prevent DONE

## Phụ thuộc task trong M05

`OM9-M05-L4-10-004-02`, `OM9-M05-L4-10-002-03`, `OM9-M05-L4-09-003-03`

## Hướng dẫn test tay

Create layout alternating round and oval sizes; verify exact order

## Nguồn và bằng chứng

- Canonical L3: `specs/05-ms05-gems-and-layout/02-febe-gem-on-curves-and-surfaces/004-febe-gem-list-on-curve/l3.json`.
- Manual: `[{'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_page': 417, 'pdf_page_1based': 191, 'precision': 'section_heading'}, {'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_pages': [403, 434], 'pdf_pages_1based': [177, 208], 'precision': 'parent_l2_range'}]`.
- Ghi chú đối chiếu: Manual references inherited from canonical L3; if parent_l2_range only, do not present as command-level proof. Reinspect relevant PDF pages before asserting behavior.
- Repository pin **để kiểm chứng tại runtime**: `746a17f127897e921a57118b6caad08968602bf5`. Không có source checkout trong ZIP.
- Kết quả phải ghi trong: `implementation/evidence/M05/10-004/03/` (đúng fields trong `l4.json`).
- **Không đánh dấu DONE** nếu không có source audit, build/test log, fixture, geometry oracle và sign-off theo loại task.
