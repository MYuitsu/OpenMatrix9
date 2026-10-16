# OM9-M05-L4-11-007-03 — Halo Cluster Edit Parameters

**Stage:** E10 (M05) | **Owner:** `OM9-L2-04-11` | **Parent:** `OM9-L3-04-11-007` | **Phase:** `UI_INTEGRATION_AND_ACCEPTANCE`.

**Trạng thái:** `PLANNED_BLOCKED_G00_E09`. Đây là **kế hoạch**, không phải code hay test đã chạy.

## Mục tiêu và hợp đồng

after creation edits through builder UI and history.

- **Input dự kiến:** existing halo/cluster feature id with edited parameters.
- **Output dự kiến:** atomic regenerated preview and final geometry.
- **Kiểm tra độc lập/lỗi trọng yếu:** stable child ids where correspondence is well-defined, else conflict/replacement report.
- **Ranh giới:** Không duplicate gem placement primitive.. Không tạo ra tính năng do M06/M07 sở hữu.

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
3. Run guided UI scenario: Select created halo; reopen parameter editor, update width; cancel/commit
4. Check viewport response, Undo/Redo, save/open and source-edit history with screenshots/log
5. Mark READY_FOR_MANUAL_TEST only after runtime UI and BE tests; DONE only after human sign-off

## Tiêu chí kiểm chứng

- [ ] Live command/selection/preview/commit/undo/save-open behavior is correct
- [ ] BE regression and UI smoke have actual logs
- [ ] Screenshots, expected-versus-actual and human sign-off recorded
- [ ] No placeholder actions; unresolved blockers prevent DONE

## Phụ thuộc task trong M05

`OM9-M05-L4-11-007-02`, `OM9-M05-L4-11-001-03`, `OM9-M05-L4-11-002-03`

## Hướng dẫn test tay

Select created halo; reopen parameter editor, update width; cancel/commit

## Nguồn và bằng chứng

- Canonical L3: `specs/05-ms05-gems-and-layout/03-febe-gem-halo-cluster-baguette-layouts/007-febe-halo-cluster-edit-parameters/l3.json`.
- Manual: `[{'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_pages': [429, 446], 'pdf_pages_1based': [203, 220], 'precision': 'parent_l2_range'}]`.
- Ghi chú đối chiếu: Manual references inherited from canonical L3; if parent_l2_range only, do not present as command-level proof. Reinspect relevant PDF pages before asserting behavior.
- Repository pin **để kiểm chứng tại runtime**: `746a17f127897e921a57118b6caad08968602bf5`. Không có source checkout trong ZIP.
- Kết quả phải ghi trong: `implementation/evidence/M05/11-007/03/` (đúng fields trong `l4.json`).
- **Không đánh dấu DONE** nếu không có source audit, build/test log, fixture, geometry oracle và sign-off theo loại task.
