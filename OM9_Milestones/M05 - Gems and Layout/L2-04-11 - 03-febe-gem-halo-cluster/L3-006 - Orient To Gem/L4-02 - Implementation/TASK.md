# OM9-M05-L4-11-006-02 — Orient To Gem

**Stage:** E10 (M05) | **Owner:** `OM9-L2-04-11` | **Parent:** `OM9-L3-04-11-006` | **Phase:** `DOMAIN_IMPLEMENTATION`.

**Trạng thái:** `PLANNED_BLOCKED_G00_E09`. Đây là **kế hoạch**, không phải code hay test đã chạy.

## Mục tiêu và hợp đồng

orient companion objects relative to gemstone frame.

- **Input dự kiến:** source object frames and target GemId/feature reference.
- **Output dự kiến:** transformed companion objects relative to gem orientation.
- **Kiểm tra độc lập/lỗi trọng yếu:** invalid reference axis or missing gem rejected; no direct edits to gem asset geometry.
- **Ranh giới:** Không duplicate gem placement primitive.. Không tạo ra tính năng do M06/M07 sở hữu.

## Điều kiện bắt buộc

- [ ] G00-01..G00-07 PASS, documented license/legal approval
- [ ] E01..E09 PASS with actual test evidence and compatible contracts
- [ ] QymCAD HEAD == pinned SHA or explicit approved rebaseline
- [ ] Relevant module source/contract audited locally; never infer paths from planning ZIP
- [ ] Rust-first app + OCCT C++ bridge for OCCT operations; approved FFI boundary
- [ ] Audit task contract and dependency domain outputs have passed

## Các bước Luna phải thực hiện

1. Implement ONLY owning domain feature or adapter dispatch, using Rust service + existing OCCT bridge; never copy algorithm into UI
2. Define typed inputs/outputs, domain error cases, transaction semantics, stable IDs and in-module ownership
3. For pure adapter tasks, call previously implemented service; do NOT create second geometry engine
4. Unit-test deterministic fixture and adversarial input; integration-test persistence/history and geometric reference
5. Capture exact cargo/FFI build/test log, numeric differences, source SHA and fixtures; block if geometry oracle unavailable

## Tiêu chí kiểm chứng

- [ ] Actual implementation compiled and tests executed with evidence
- [ ] No invalid geometry or orphaned IDs; numerical oracle passes
- [ ] Errors/cancel are atomic; Undo/Redo and roundtrip succeed
- [ ] No duplicative service or downstream M06/M07 code

## Phụ thuộc task trong M05

`OM9-M05-L4-11-006-01`, `OM9-M05-L4-09-006-02`, `OM9-M05-L4-09-008-02`

## Hướng dẫn test tay

Không áp dụng cho giai đoạn này; không giả mạo bằng chứng kiểm thử thủ công.

## Nguồn và bằng chứng

- Canonical L3: `specs/05-ms05-gems-and-layout/03-febe-gem-halo-cluster-baguette-layouts/006-febe-orient-to-gem/l3.json`.
- Manual: `[{'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_page': 428, 'pdf_page_1based': 202, 'precision': 'section_heading'}, {'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_pages': [429, 446], 'pdf_pages_1based': [203, 220], 'precision': 'parent_l2_range'}]`.
- Ghi chú đối chiếu: Manual references inherited from canonical L3; if parent_l2_range only, do not present as command-level proof. Reinspect relevant PDF pages before asserting behavior.
- Repository pin **để kiểm chứng tại runtime**: `746a17f127897e921a57118b6caad08968602bf5`. Không có source checkout trong ZIP.
- Kết quả phải ghi trong: `implementation/evidence/M05/11-006/02/` (đúng fields trong `l4.json`).
- **Không đánh dấu DONE** nếu không có source audit, build/test log, fixture, geometry oracle và sign-off theo loại task.
