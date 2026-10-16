# OM9-M09-L4-GMVP-018 — Quyết định ký G-MVP GO/NO-GO

**Gate:** `G-MVP` · **Loại:** `SIGNOFF` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Tổng hợp hồ sơ, yêu cầu chữ ký người có thẩm quyền và chốt snapshot MVP có thể phục hồi.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Đọc độc lập toàn bộ 17 work-order evidence và release checklist; xác nhận G00–E12 signed, defect triage, legal, packaging và OS test.
4. Nếu thiếu bất kỳ MUST_PASS/bản quyền/blocker nghiêm trọng: ghi NO_GO và lý do; không tự nâng PASS.
5. Nếu đủ evidence, trình reviewer con người ký G-MVP và đóng băng source/build/fixture/acceptance SHA; tạo MVP baseline regression set.
6. Công bố phạm vi/known gaps, không gọi là Matrix 9 parity; ghi decision + timestamp/reviewer và link artifacts.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-12-008` · `OM9-L2-01-12` · `E12` · `cap.01-12.stl-tessellation-export-codec`
- `OM9-L3-01-13-002` · `OM9-L2-01-13` · `E08` · `cap.01-13.opennurbs-reader-bridge-contract`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-001`
- `OM9-M09-L4-GMVP-017`

## Acceptance

**Oracle cụ thể:** A human release authority signs GO only after every prerequisite and required test passes, otherwise signs NO_GO/AWAITING_SIGNOFF; MVP scope and baseline hashes are frozen.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Agent không được tự ký; thiếu reviewer => AWAITING_SIGNOFF.
- NO_GO không được diễn đạt như shipped/ready.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-018-01` (ACCEPTANCE): Đọc độc lập toàn bộ 17 work-order evidence và release checklist; xác nhận G00–E12 signed, defect triage, legal, packaging và OS test.
- `M09-TC-GMVP-018-02` (ACCEPTANCE): Nếu thiếu bất kỳ MUST_PASS/bản quyền/blocker nghiêm trọng: ghi NO_GO và lý do; không tự nâng PASS.
- `M09-TC-GMVP-018-03` (ACCEPTANCE): Nếu đủ evidence, trình reviewer con người ký G-MVP và đóng băng source/build/fixture/acceptance SHA; tạo MVP baseline regression set.
- `M09-TC-GMVP-018-04` (ACCEPTANCE): Công bố phạm vi/known gaps, không gọi là Matrix 9 parity; ghi decision + timestamp/reviewer và link artifacts.
- `M09-TC-GMVP-018-N01` (NEGATIVE): Agent không được tự ký; thiếu reviewer => AWAITING_SIGNOFF.
- `M09-TC-GMVP-018-N02` (NEGATIVE): NO_GO không được diễn đạt như shipped/ready.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-018/g-mvp-decision.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- M09_RELEASE_GATES; release signoff policy
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
