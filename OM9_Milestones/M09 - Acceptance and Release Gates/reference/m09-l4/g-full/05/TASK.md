# OM9-M09-L4-GFULL-005 — Bend, Twist, Taper và Cage topology

**Gate:** `G-FULL` · **Loại:** `GEOMETRY_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Kiểm transform/deformation trên curve, surface, solid theo loại geometry với mức biến dạng hợp lệ.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Áp Bend/Twist/Taper/Cage lên các fixture có control points, trimmed BRep, nested instance và số đo chuẩn.
4. So bbox, continuity, validity, self-intersection và deviation đúng loại output và signed tolerance.
5. Thử extreme deformation và invalid spine/axis; yêu cầu typed errors, rollback, no leak.
6. Kiểm undo/save-open/persistent references sau biến dạng.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-08-001` · `OM9-L2-04-08` · `E13` · `cap.04-08.bend-deformation-command`
- `OM9-L3-04-08-002` · `OM9-L2-04-08` · `E13` · `cap.04-08.stretch-and-taper-deformation`
- `OM9-L3-04-08-003` · `OM9-L2-04-08` · `E13` · `cap.04-08.twist-and-maelstrom-deformation`
- `OM9-L3-04-01-008` · `OM9-L2-04-01` · `E13` · `cap.04-01.transform-command-bend`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-002`

## Acceptance

**Oracle cụ thể:** Deformed BRep/surfaces/curves pass their distinct validity/continuity checks and numeric budgets; invalid deformation rolls back.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Closed solid không còn valid, inversion không báo lỗi hoặc curve bị bắt kiểm closed BRep: FAIL.
- Không ép mọi deformation phải manifold nếu output hợp lệ là open surface.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-005-01` (ACCEPTANCE): Áp Bend/Twist/Taper/Cage lên các fixture có control points, trimmed BRep, nested instance và số đo chuẩn.
- `M09-TC-GFULL-005-02` (ACCEPTANCE): So bbox, continuity, validity, self-intersection và deviation đúng loại output và signed tolerance.
- `M09-TC-GFULL-005-03` (ACCEPTANCE): Thử extreme deformation và invalid spine/axis; yêu cầu typed errors, rollback, no leak.
- `M09-TC-GFULL-005-04` (ACCEPTANCE): Kiểm undo/save-open/persistent references sau biến dạng.
- `M09-TC-GFULL-005-N01` (NEGATIVE): Closed solid không còn valid, inversion không báo lỗi hoặc curve bị bắt kiểm closed BRep: FAIL.
- `M09-TC-GFULL-005-N02` (NEGATIVE): Không ép mọi deformation phải manifold nếu output hợp lệ là open surface.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-005/advanced-deformation.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- Matrix 8 Book 2 Transform, Cage Edit
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `matrix_8_manual_book2.pdf`: trang PDF 9, 14, 15

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
