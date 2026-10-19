# OM9-M09-L4-GMVP-001 — Đối chiếu tiền điều kiện G00 đến E12

**Gate:** `G-MVP` · **Loại:** `GOVERNANCE` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Khóa phiên bản nguồn, các gate đã ký và bộ điều kiện kiểm thử trước khi bắt đầu nghiệm thu.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Thu bản ghi PASS có reviewer và build SHA cho G00 cùng từng E01–E12; xác nhận không stage nào tự ký.
4. Đối chiếu SHA source/OCCT/Rust/compiler, OS, flags và fixture baseline giữa các gate; ghi ngoại lệ rõ.
5. Đối chiếu blocker severity và công việc chưa triển khai; không coi manifest hoặc lời hứa là test evidence.
6. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-05-001` · `OM9-L2-01-05` · `E01` · `cap.01-05.occt-ffi-safe-handle-lifecycle`
- `OM9-L3-01-12-001` · `OM9-L2-01-12` · `E02` · `cap.01-12.native-project-container-format`

## Phụ thuộc giữa các L4
- Không có phụ thuộc L4 nội bộ (nhưng vẫn có external gates).

## Acceptance

**Oracle cụ thể:** Signed G00 and every E01–E12 PASS record is present and tied to the selected source/build baseline; missing or inconsistent proof blocks this task.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Thiếu chữ ký E12 hoặc chỉ có source-review: BLOCKED_GATE.
- Mâu thuẫn source SHA/toolchain mà không rebuild: BLOCKED_PROVENANCE.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-001-01` (ACCEPTANCE): Thu bản ghi PASS có reviewer và build SHA cho G00 cùng từng E01–E12; xác nhận không stage nào tự ký.
- `M09-TC-GMVP-001-02` (ACCEPTANCE): Đối chiếu SHA source/OCCT/Rust/compiler, OS, flags và fixture baseline giữa các gate; ghi ngoại lệ rõ.
- `M09-TC-GMVP-001-03` (ACCEPTANCE): Đối chiếu blocker severity và công việc chưa triển khai; không coi manifest hoặc lời hứa là test evidence.
- `M09-TC-GMVP-001-N01` (NEGATIVE): Thiếu chữ ký E12 hoặc chỉ có source-review: BLOCKED_GATE.
- `M09-TC-GMVP-001-N02` (NEGATIVE): Mâu thuẫn source SHA/toolchain mà không rebuild: BLOCKED_PROVENANCE.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-001/prerequisite-audit.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- G00 feasibility, GATES_AND_EVIDENCE, signed stage records
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
