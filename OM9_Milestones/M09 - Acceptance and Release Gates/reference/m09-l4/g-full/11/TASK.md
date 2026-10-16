# OM9-M09-L4-GFULL-011 — Đóng gói FULL, license delta và known gaps

**Gate:** `G-FULL` · **Loại:** `DISTRIBUTION` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Xác minh kênh phát hành FULL, upgrade project, tài sản thư viện và nghĩa vụ license mới.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Đối chiếu Full release artifact với MVP bản gốc: ABI/dependency/SBOM/asset/NOTICE thay đổi.
4. Kiểm clean install, upgrade từ signed G-MVP và rollback; xác nhận old MVP project usable theo version policy.
5. Tạo release notes thể hiện advanced chính xác, known gaps AFTER, API incompatibilities và limitations.
6. Kiểm license delta và human legal approval khi thêm C++/font/asset mới; không dùng silent waiver.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-05-03-001` · `OM9-L2-05-03` · `E13` · `cap.05-03.user-library-folder-catalog`
- `OM9-L3-01-12-001` · `OM9-L2-01-12` · `E02` · `cap.01-12.native-project-container-format`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-008`
- `OM9-M09-L4-GFULL-010`

## Acceptance

**Oracle cụ thể:** FULL install/upgrade/rollback, SBOM/license delta, user-library assets and release notes match tested FULL binary without jeopardizing MVP documents.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Upgrade phá project MVP hoặc unsupported-after bị quảng cáo đã hỗ trợ: FAIL.
- Legal delta chưa được duyệt: BLOCKED_LEGAL.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-011-01` (ACCEPTANCE): Đối chiếu Full release artifact với MVP bản gốc: ABI/dependency/SBOM/asset/NOTICE thay đổi.
- `M09-TC-GFULL-011-02` (ACCEPTANCE): Kiểm clean install, upgrade từ signed G-MVP và rollback; xác nhận old MVP project usable theo version policy.
- `M09-TC-GFULL-011-03` (ACCEPTANCE): Tạo release notes thể hiện advanced chính xác, known gaps AFTER, API incompatibilities và limitations.
- `M09-TC-GFULL-011-04` (ACCEPTANCE): Kiểm license delta và human legal approval khi thêm C++/font/asset mới; không dùng silent waiver.
- `M09-TC-GFULL-011-N01` (NEGATIVE): Upgrade phá project MVP hoặc unsupported-after bị quảng cáo đã hỗ trợ: FAIL.
- `M09-TC-GFULL-011-N02` (NEGATIVE): Legal delta chưa được duyệt: BLOCKED_LEGAL.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-011/full-release-packaging.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- GATES_AND_EVIDENCE; G-MVP release artifact
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
