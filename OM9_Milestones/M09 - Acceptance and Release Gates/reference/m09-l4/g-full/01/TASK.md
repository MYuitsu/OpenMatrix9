# OM9-M09-L4-GFULL-001 — Khóa G-MVP signed và E13 PASS

**Gate:** `G-FULL` · **Loại:** `GOVERNANCE` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Khởi tạo FULL từ bản MVP đã được con người phê duyệt và chứng cứ E13 hoàn chỉnh.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Xác nhận G-MVP decision có chữ ký reviewer, source/build SHA, fixture checksums và status PASS.
4. Xác nhận E13 PASS độc lập, mọi 73 canonical L3 trong E13 có owner/evidence hoặc approved scope disposition.
5. Khóa baseline MVP để test hậu phát triển E13; không tự gắn advanced vào MVP trước gate.
6. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-05-001` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-autobase-surface-generation`
- `OM9-L3-05-01-001` · `OM9-L2-05-01` · `E13` · `cap.05-01.smartpattern-source-pattern-choice`

## Phụ thuộc giữa các L4
- Không có phụ thuộc L4 nội bộ (nhưng vẫn có external gates).

## Acceptance

**Oracle cụ thể:** Signed G-MVP GO evidence and frozen MVP baseline match an accepted E13 signed PASS revision; never run FULL on unsigned MVP.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- G-MVP thiếu chữ ký hoặc E13 chưa PASS: BLOCKED_GATE.
- Baseline mất/hash khác mà không giải thích: BLOCKED_PROVENANCE.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-001-01` (ACCEPTANCE): Xác nhận G-MVP decision có chữ ký reviewer, source/build SHA, fixture checksums và status PASS.
- `M09-TC-GFULL-001-02` (ACCEPTANCE): Xác nhận E13 PASS độc lập, mọi 73 canonical L3 trong E13 có owner/evidence hoặc approved scope disposition.
- `M09-TC-GFULL-001-03` (ACCEPTANCE): Khóa baseline MVP để test hậu phát triển E13; không tự gắn advanced vào MVP trước gate.
- `M09-TC-GFULL-001-N01` (NEGATIVE): G-MVP thiếu chữ ký hoặc E13 chưa PASS: BLOCKED_GATE.
- `M09-TC-GFULL-001-N02` (NEGATIVE): Baseline mất/hash khác mà không giải thích: BLOCKED_PROVENANCE.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-001/full-prerequisite-baseline.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- M09_RELEASE_GATES; E13; G-MVP signed report
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
