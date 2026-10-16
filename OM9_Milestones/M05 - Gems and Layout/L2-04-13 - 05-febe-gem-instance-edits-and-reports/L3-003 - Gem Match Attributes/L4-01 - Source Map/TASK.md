# OM9-M05-L4-13-003-01 — Gem Match Attributes

**Stage:** E10 (M05) | **Owner:** `OM9-L2-04-13` | **Parent:** `OM9-L3-04-13-003` | **Phase:** `SOURCE_AUDIT_CONTRACT`.

**Trạng thái:** `PLANNED_BLOCKED_G00_E09`. Đây là **kế hoạch**, không phải code hay test đã chạy.

## Mục tiêu và hợp đồng

copy cut/shape/size characteristics between stones.

- **Input dự kiến:** source GemId, destination GemIds, allowed attribute whitelist.
- **Output dự kiến:** transferred compatible settings and documented exclusions.
- **Kiểm tra độc lập/lỗi trọng yếu:** reject incompatible shapes/styles rather than corrupt recipient metadata.
- **Ranh giới:** Gem metadata owner là CAD Base, không nhân đôi.. Không tạo ra tính năng do M06/M07 sở hữu.

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

- Canonical L3: `specs/05-ms05-gems-and-layout/05-febe-gem-instance-edits-and-reports/003-febe-gem-match-attributes/l3.json`.
- Manual: `[{'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_page': 461, 'pdf_page_1based': 235, 'precision': 'section_heading'}, {'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_pages': [450, 467], 'pdf_pages_1based': [224, 241], 'precision': 'parent_l2_range'}]`.
- Ghi chú đối chiếu: Matrix 8 Book 2 printed 461: Match Attributes transmits selected setting/cutter style sheets; avoid claiming indiscriminate cut/size cloning.
- Repository pin **để kiểm chứng tại runtime**: `746a17f127897e921a57118b6caad08968602bf5`. Không có source checkout trong ZIP.
- Kết quả phải ghi trong: `implementation/evidence/M05/13-003/01/` (đúng fields trong `l4.json`).
- **Không đánh dấu DONE** nếu không có source audit, build/test log, fixture, geometry oracle và sign-off theo loại task.
