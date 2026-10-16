# OM9-M05-L4-11-006-01 — Orient To Gem

**Stage:** E10 (M05) | **Owner:** `OM9-L2-04-11` | **Parent:** `OM9-L3-04-11-006` | **Phase:** `SOURCE_AUDIT_CONTRACT`.

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

## Các bước Luna phải thực hiện

1. Inspect local pinned source; git rev-parse HEAD; cargo metadata; AGENTS/CLAUDE rules; license scope; source API and tests
2. Select REUSE, ADAPT or ADD with source file:line proof; list actual affected crate paths
3. Inspect owning service and callers; resolve BE/FEBE split and document rollback strategy
4. Record exact command surface in Matrix 8 Book 2 without assuming original command names exist in OpenMatrix9
5. If G00 or E09 is missing, only produce contract/audit and BLOCKED evidence, NOT feature completion

## Tiêu chí kiểm chứng

- [ ] Source path ownership, QymCAD symbol and line ranges verified locally
- [ ] REUSE/ADAPT/ADD decision backed by local source evidence
- [ ] Typed input/output and ABI boundary reviewed
- [ ] Dependencies and acceptance oracle recorded

## Phụ thuộc task trong M05

Không có task L4 nội bộ (vẫn phải PASS G00 và E09).

## Hướng dẫn test tay

Không áp dụng cho giai đoạn này; không giả mạo bằng chứng kiểm thử thủ công.

## Nguồn và bằng chứng

- Canonical L3: `specs/05-ms05-gems-and-layout/03-febe-gem-halo-cluster-baguette-layouts/006-febe-orient-to-gem/l3.json`.
- Manual: `[{'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_page': 428, 'pdf_page_1based': 202, 'precision': 'section_heading'}, {'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_pages': [429, 446], 'pdf_pages_1based': [203, 220], 'precision': 'parent_l2_range'}]`.
- Ghi chú đối chiếu: Manual references inherited from canonical L3; if parent_l2_range only, do not present as command-level proof. Reinspect relevant PDF pages before asserting behavior.
- Repository pin **để kiểm chứng tại runtime**: `746a17f127897e921a57118b6caad08968602bf5`. Không có source checkout trong ZIP.
- Kết quả phải ghi trong: `implementation/evidence/M05/11-006/01/` (đúng fields trong `l4.json`).
- **Không đánh dấu DONE** nếu không có source audit, build/test log, fixture, geometry oracle và sign-off theo loại task.
