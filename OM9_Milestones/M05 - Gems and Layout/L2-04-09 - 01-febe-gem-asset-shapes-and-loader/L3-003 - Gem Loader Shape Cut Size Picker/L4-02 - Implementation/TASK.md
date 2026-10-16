# OM9-M05-L4-09-003-02 — Gem Loader Shape Cut Size Picker

**Stage:** E10 (M05) | **Owner:** `OM9-L2-04-09` | **Parent:** `OM9-L3-04-09-003` | **Phase:** `DOMAIN_IMPLEMENTATION`.

**Trạng thái:** `PLANNED_BLOCKED_G00_E09`. Đây là **kế hoạch**, không phải code hay test đã chạy.

## Mục tiêu và hợp đồng

create one gemstone with specified shape and dimensions.

- **Input dự kiến:** shape/cut/size selection, optional existing GemId.
- **Output dự kiến:** create or explicitly replace a stable gem instance through document transaction.
- **Kiểm tra độc lập/lỗi trọng yếu:** preview does not insert duplicate objects; invalid custom size cannot commit.
- **Ranh giới:** Không dùng chung hardcode render gems.. Không tạo ra tính năng do M06/M07 sở hữu.

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

`OM9-M05-L4-09-003-01`, `OM9-M05-L4-09-001-02`, `OM9-M05-L4-09-002-02`

## Hướng dẫn test tay

Không áp dụng cho giai đoạn này; không giả mạo bằng chứng kiểm thử thủ công.

## Nguồn và bằng chứng

- Canonical L3: `specs/05-ms05-gems-and-layout/01-febe-gem-asset-shapes-and-loader/003-febe-gem-loader-shape-cut-size-picker/l3.json`.
- Manual: `[{'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_page': 402, 'pdf_page_1based': 176, 'precision': 'section_heading'}, {'manual_id': 'book2', 'filename': 'matrix_8_manual_book2.pdf', 'printed_pages': [400, 406], 'pdf_pages_1based': [174, 180], 'precision': 'parent_l2_range'}]`.
- Ghi chú đối chiếu: Matrix 8 Book 2 printed 402–403: gvLoader shape/cut/size selector; Keep Original Size and custom Length/Width/Depth; do not assume `.mss` formats.
- Repository pin **để kiểm chứng tại runtime**: `746a17f127897e921a57118b6caad08968602bf5`. Không có source checkout trong ZIP.
- Kết quả phải ghi trong: `implementation/evidence/M05/09-003/02/` (đúng fields trong `l4.json`).
- **Không đánh dấu DONE** nếu không có source audit, build/test log, fixture, geometry oracle và sign-off theo loại task.
