# OM9-M09-L4-GFULL-006 — Orient/Array, Knot và ornament builder

**Gate:** `G-FULL` · **Loại:** `E2E_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Xác minh vị trí instance, sao chép theo curve/surface và hình học ornament builders bằng toán học độc lập.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Chạy OrientOnCurve/OnSurface và Array trên curve/closed rail với input frames rõ; so count, step, normal, collision.
4. Chạy Knot/Freeform/JumpRing theo fixture; đo bounding, profile width, self intersections theo contract của từng builder.
5. Chỉnh curve source; check instance updates/ID preservation và undo/delete atomic.
6. Tách lỗi domain owners E13 04-06/07 khỏi lỗi kernel core E07 bằng repro fixture.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-07-001` · `OM9-L2-04-07` · `E13` · `cap.04-07.orient-object-on-curve`
- `OM9-L3-04-07-002` · `OM9-L2-04-07` · `E13` · `cap.04-07.orient-object-on-surface`
- `OM9-L3-04-06-001` · `OM9-L2-04-06` · `E13` · `cap.04-06.knot-builder`
- `OM9-L3-04-06-003` · `OM9-L2-04-06` · `E13` · `cap.04-06.jump-ring-builder`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-002`
- `OM9-M09-L4-GFULL-005`

## Acceptance

**Oracle cụ thể:** Orient, array and ornament results have exact fixture instance count, transforms, expected topology and stable references after edits.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Array duplicate lệch frame/rotation, history orphan hoặc geometry invalid: FAIL.
- Không đặt UI screenshot là numerical oracle.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-006-01` (ACCEPTANCE): Chạy OrientOnCurve/OnSurface và Array trên curve/closed rail với input frames rõ; so count, step, normal, collision.
- `M09-TC-GFULL-006-02` (ACCEPTANCE): Chạy Knot/Freeform/JumpRing theo fixture; đo bounding, profile width, self intersections theo contract của từng builder.
- `M09-TC-GFULL-006-03` (ACCEPTANCE): Chỉnh curve source; check instance updates/ID preservation và undo/delete atomic.
- `M09-TC-GFULL-006-04` (ACCEPTANCE): Tách lỗi domain owners E13 04-06/07 khỏi lỗi kernel core E07 bằng repro fixture.
- `M09-TC-GFULL-006-N01` (NEGATIVE): Array duplicate lệch frame/rotation, history orphan hoặc geometry invalid: FAIL.
- `M09-TC-GFULL-006-N02` (NEGATIVE): Không đặt UI screenshot là numerical oracle.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-006/orient-ornament-arrays.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- Matrix 8 Book 2 Builders/Transform
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `matrix_8_manual_book2.pdf`: trang PDF 26, 27, 115

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
