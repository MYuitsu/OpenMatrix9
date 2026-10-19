# OM9-M09-L4-GFULL-003 — SmartFlow UV, normal và bám bề mặt

**Gate:** `G-FULL` · **Loại:** `GEOMETRY_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Đánh giá chuyển hình học từ base sang target surface trên fixtures có seam/normal đảo và độ cong khác nhau.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Tạo Flat Autobase/Destination surfaces theo signed reference, lưu mapping UV/normal và seam.
4. Flow đường cong và BRep ornament qua target; đo deviation, orientation và overhang theo signed tolerance.
5. Lặp đảo surface normal/đổi UV/seam để kiểm mapping deterministic, cảnh báo sai hướng, no silent geometry inversion.
6. Test break/relink surface history nếu E13 contract hỗ trợ; không trộn rigid behavior vào biến dạng tự do.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-05-001` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-autobase-surface-generation`
- `OM9-L3-04-05-002` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-mapping-base-to-target`
- `OM9-L3-04-05-006` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-pulldown-reference-curves`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-002`

## Acceptance

**Oracle cụ thể:** SmartFlow mapping is deterministic for signed UV/seam/normal fixtures and meets numeric deviation/orientation targets; inversions are rejected or visibly corrected.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- UV direction đảo sai, hidden shape below surface hoặc BRep invalid: FAIL.
- Tính năng manual chỉ mô tả không thay geometric oracle.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-003-01` (ACCEPTANCE): Tạo Flat Autobase/Destination surfaces theo signed reference, lưu mapping UV/normal và seam.
- `M09-TC-GFULL-003-02` (ACCEPTANCE): Flow đường cong và BRep ornament qua target; đo deviation, orientation và overhang theo signed tolerance.
- `M09-TC-GFULL-003-03` (ACCEPTANCE): Lặp đảo surface normal/đổi UV/seam để kiểm mapping deterministic, cảnh báo sai hướng, no silent geometry inversion.
- `M09-TC-GFULL-003-04` (ACCEPTANCE): Test break/relink surface history nếu E13 contract hỗ trợ; không trộn rigid behavior vào biến dạng tự do.
- `M09-TC-GFULL-003-N01` (NEGATIVE): UV direction đảo sai, hidden shape below surface hoặc BRep invalid: FAIL.
- `M09-TC-GFULL-003-N02` (NEGATIVE): Tính năng manual chỉ mô tả không thay geometric oracle.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-003/smartflow-uv-normal.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- Matrix 8 Book 2 Smart Flow printed p242-245
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `matrix_8_manual_book2.pdf`: trang PDF 16, 17, 18, 19

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
