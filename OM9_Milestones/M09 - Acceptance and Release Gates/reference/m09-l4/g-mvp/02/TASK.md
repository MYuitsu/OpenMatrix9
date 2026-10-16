# OM9-M09-L4-GMVP-002 — Tạo ma trận truy vết MVP

**Gate:** `G-MVP` · **Loại:** `GOVERNANCE` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Xác định mọi năng lực MVP thực sự thuộc in-scope và mã kiểm thử liên kết tới chủ sở hữu canonical.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Tải planning/EXECUTION_MANIFEST.json và lọc E01–E12, 529 L3 theo runtime manifest (kiểm bằng validator, không hardcode trong quyết định).
4. Tạo bảng requirement → canonical L3 → owner L2 → fixture → command/test → evidence → current disposition.
5. Chia MUST_PASS, approved limitation và AFTER; phân biệt import 3DM với export 3DM và công cụ nâng cao.
6. Với mỗi yêu cầu bắt buộc thiếu test, mở finding BLOCKED_COVERAGE.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-03-01-001` · `OM9-L2-03-01` · `E02` · `cap.03-01.matrix-four-viewport-grid-layout`
- `OM9-L3-01-13-001` · `OM9-L2-01-13` · `E08` · `cap.01-13.3dm-file-signature-and-version-probe`
- `OM9-L3-04-02-001` · `OM9-L2-04-02` · `E09` · `cap.04-02.ring-rail-jewelry-size-standard`
- `OM9-L3-04-09-003` · `OM9-L2-04-09` · `E10` · `cap.04-09.gem-loader-shape-cut-size-picker`
- `OM9-L3-04-14-001` · `OM9-L2-04-14` · `E11` · `cap.04-14.head-builder-basket`
- `OM9-L3-04-16-001` · `OM9-L2-04-16` · `E12` · `cap.04-16.gem-cutter-azure-for-single-stone`
- `OM9-L3-01-12-008` · `OM9-L2-01-12` · `E12` · `cap.01-12.stl-tessellation-export-codec`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-001`

## Acceptance

**Oracle cụ thể:** Every one of 529 E01–E12 canonical L3 has exactly one ownership trace and an explicit disposition; every REQUIRED release capability maps to acceptance evidence or a blocking gap.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Không chèn capabilities AFTER/Clayoo/Render/CAM vào gate.
- Không gắn một tính năng vào hai owner khác nhau.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-002-01` (ACCEPTANCE): Tải planning/EXECUTION_MANIFEST.json và lọc E01–E12, 529 L3 theo runtime manifest (kiểm bằng validator, không hardcode trong quyết định).
- `M09-TC-GMVP-002-02` (ACCEPTANCE): Tạo bảng requirement → canonical L3 → owner L2 → fixture → command/test → evidence → current disposition.
- `M09-TC-GMVP-002-03` (ACCEPTANCE): Chia MUST_PASS, approved limitation và AFTER; phân biệt import 3DM với export 3DM và công cụ nâng cao.
- `M09-TC-GMVP-002-04` (ACCEPTANCE): Với mỗi yêu cầu bắt buộc thiếu test, mở finding BLOCKED_COVERAGE.
- `M09-TC-GMVP-002-N01` (NEGATIVE): Không chèn capabilities AFTER/Clayoo/Render/CAM vào gate.
- `M09-TC-GMVP-002-N02` (NEGATIVE): Không gắn một tính năng vào hai owner khác nhau.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-002/mvp-traceability.csv`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- planning/README, M09_RELEASE_GATES, EXECUTION_MANIFEST
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
