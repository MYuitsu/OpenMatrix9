# OM9-M09-L4-GMVP-017 — Triage lỗi và kiểm toán bằng chứng

**Gate:** `G-MVP` · **Loại:** `GOVERNANCE` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Đối soát mọi test failure/blocker, waiver có lý do, build provenance và completeness trước quyết định.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Tổng hợp test results với command/exit code/timestamp/source SHA/fixture provenance; đối chiếu artifact hashes.
4. Triage defects theo severity, reproduction fixture, canonical owner, fixed SHA và independent retest.
5. Chỉ cân nhắc waiver noncritical có người duyệt và limitation công khai; tuyệt đối không waive data loss, wrong geometry, license hoặc security blocker.
6. Tạo release-readiness matrix các nhóm phải PASS; giữ FAIL/BLOCKED cho ô chưa có chứng cứ.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-05-002` · `OM9-L2-01-05` · `E01` · `cap.01-05.brep-shape-type-and-validation`
- `OM9-L3-01-12-001` · `OM9-L2-01-12` · `E02` · `cap.01-12.native-project-container-format`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-002`
- `OM9-M09-L4-GMVP-003`
- `OM9-M09-L4-GMVP-004`
- `OM9-M09-L4-GMVP-005`
- `OM9-M09-L4-GMVP-006`
- `OM9-M09-L4-GMVP-007`
- `OM9-M09-L4-GMVP-008`
- `OM9-M09-L4-GMVP-009`
- `OM9-M09-L4-GMVP-010`
- `OM9-M09-L4-GMVP-011`
- `OM9-M09-L4-GMVP-012`
- `OM9-M09-L4-GMVP-013`
- `OM9-M09-L4-GMVP-014`
- `OM9-M09-L4-GMVP-015`
- `OM9-M09-L4-GMVP-016`

## Acceptance

**Oracle cụ thể:** All 16 prior required evidence bundles resolve to real raw logs/fixtures/hash; no unresolved blocker; any waiver is noncritical and approved by authorized reviewer.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Thiếu nguồn log/raw evidence => BLOCKED_PROVENANCE.
- Critical blocker/waiver thiếu thẩm quyền => NO_GO.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-017-01` (ACCEPTANCE): Tổng hợp test results với command/exit code/timestamp/source SHA/fixture provenance; đối chiếu artifact hashes.
- `M09-TC-GMVP-017-02` (ACCEPTANCE): Triage defects theo severity, reproduction fixture, canonical owner, fixed SHA và independent retest.
- `M09-TC-GMVP-017-03` (ACCEPTANCE): Chỉ cân nhắc waiver noncritical có người duyệt và limitation công khai; tuyệt đối không waive data loss, wrong geometry, license hoặc security blocker.
- `M09-TC-GMVP-017-04` (ACCEPTANCE): Tạo release-readiness matrix các nhóm phải PASS; giữ FAIL/BLOCKED cho ô chưa có chứng cứ.
- `M09-TC-GMVP-017-N01` (NEGATIVE): Thiếu nguồn log/raw evidence => BLOCKED_PROVENANCE.
- `M09-TC-GMVP-017-N02` (NEGATIVE): Critical blocker/waiver thiếu thẩm quyền => NO_GO.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-017/release-evidence-audit.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- GATES_AND_EVIDENCE; DEPENDENCY_AND_RELEASE_POLICY
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
