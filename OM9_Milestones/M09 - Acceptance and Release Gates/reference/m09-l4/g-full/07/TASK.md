# OM9-M09-L4-GFULL-007 — Smart Pattern và Rope Builder

**Gate:** `G-FULL` · **Loại:** `E2E_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Chứng minh SmartPattern/Rope có deterministic outputs, pattern orientation đúng và rebuild theo tham số.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Sinh patterns trên surface/UV có seam, evaluate spacing, count, transforms và giao điểm theo golden manifest.
4. Dựng rope nhiều thread count/diameter/twists trên đường cong, đo pitch, clearance và bounds.
5. Chỉnh source curve/UV/reference; kiểm feature history, Save/Open, Undo/Redo và deterministic output fingerprint.
6. Thử degenerate curve, invalid counts, zero pitch và heavy input; không treo app, lỗi phải có thông báo.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-05-01-001` · `OM9-L2-05-01` · `E13` · `cap.05-01.smartpattern-source-pattern-choice`
- `OM9-L3-05-01-002` · `OM9-L2-05-01` · `E13` · `cap.05-01.smartpattern-uv-distribution`
- `OM9-L3-05-02-001` · `OM9-L2-05-02` · `E13` · `cap.05-02.rope-builder-curve-input`
- `OM9-L3-05-02-002` · `OM9-L2-05-02` · `E13` · `cap.05-02.rope-builder-thread-count-and-diameter`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-002`
- `OM9-M09-L4-GFULL-005`

## Acceptance

**Oracle cụ thể:** SmartPattern and Rope deliver documented pattern counts, UV distribution and thread pitch/clearances under signed tolerances; edits recompute deterministically.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Sai số lượng/cỡ/angle hoặc self-intersection ngoài tolerance contract: FAIL.
- Không thay thế geometry bằng texture render để PASS.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-007-01` (ACCEPTANCE): Sinh patterns trên surface/UV có seam, evaluate spacing, count, transforms và giao điểm theo golden manifest.
- `M09-TC-GFULL-007-02` (ACCEPTANCE): Dựng rope nhiều thread count/diameter/twists trên đường cong, đo pitch, clearance và bounds.
- `M09-TC-GFULL-007-03` (ACCEPTANCE): Chỉnh source curve/UV/reference; kiểm feature history, Save/Open, Undo/Redo và deterministic output fingerprint.
- `M09-TC-GFULL-007-04` (ACCEPTANCE): Thử degenerate curve, invalid counts, zero pitch và heavy input; không treo app, lỗi phải có thông báo.
- `M09-TC-GFULL-007-N01` (NEGATIVE): Sai số lượng/cỡ/angle hoặc self-intersection ngoài tolerance contract: FAIL.
- `M09-TC-GFULL-007-N02` (NEGATIVE): Không thay thế geometry bằng texture render để PASS.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-007/smartpattern-rope.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- Matrix 9 Addendum Chapter 3 Smart Pattern/Rope
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `matrix9.0_addendum_manual.pdf`: trang PDF 86, 87, 88, 89

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
