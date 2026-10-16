# OM9-M09-L4-GFULL-010 — Giữ nguyên MVP và đo hiệu năng nâng cao

**Gate:** `G-FULL` · **Loại:** `REGRESSION_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Chạy chính bộ kiểm G-MVP trên artifact FULL và đánh giá load/memory/geometry khi E13 được bật.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Tái chạy full MVP acceptance artifact theo frozen MVP manifest và same fixture hashes/units/tolerances.
4. So trước/sau về valid geometry, history, save/open, 3DM import, STL dimensions, signed license notices.
5. Chạy robustness benchmark E13 trên target hardware cùng baseline G00; báo delta có lý do.
6. Các thay đổi công khai behavior chỉ được chấp nhận qua formal approval; không silently regress MVP.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-12-008` · `OM9-L2-01-12` · `E12` · `cap.01-12.stl-tessellation-export-codec`
- `OM9-L3-01-13-002` · `OM9-L2-01-13` · `E08` · `cap.01-13.opennurbs-reader-bridge-contract`
- `OM9-L3-04-05-002` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-mapping-base-to-target`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-001`
- `OM9-M09-L4-GFULL-004`
- `OM9-M09-L4-GFULL-005`
- `OM9-M09-L4-GFULL-006`
- `OM9-M09-L4-GFULL-007`
- `OM9-M09-L4-GFULL-008`
- `OM9-M09-L4-GFULL-009`

## Acceptance

**Oracle cụ thể:** All frozen MVP test cases still pass on FULL binary with the same approved oracles; advanced stress workloads meet approved budgets.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- MVP previously passed giờ hỏng: BLOCKED_REGRESSION.
- Thiếu cơ sở đo/perf threshold: BLOCKED_REFERENCE, không gọi PASS.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-010-01` (ACCEPTANCE): Tái chạy full MVP acceptance artifact theo frozen MVP manifest và same fixture hashes/units/tolerances.
- `M09-TC-GFULL-010-02` (ACCEPTANCE): So trước/sau về valid geometry, history, save/open, 3DM import, STL dimensions, signed license notices.
- `M09-TC-GFULL-010-03` (ACCEPTANCE): Chạy robustness benchmark E13 trên target hardware cùng baseline G00; báo delta có lý do.
- `M09-TC-GFULL-010-04` (ACCEPTANCE): Các thay đổi công khai behavior chỉ được chấp nhận qua formal approval; không silently regress MVP.
- `M09-TC-GFULL-010-N01` (NEGATIVE): MVP previously passed giờ hỏng: BLOCKED_REGRESSION.
- `M09-TC-GFULL-010-N02` (NEGATIVE): Thiếu cơ sở đo/perf threshold: BLOCKED_REFERENCE, không gọi PASS.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-010/full-mvp-regression.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- G-MVP frozen baseline; E13 test suite
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
