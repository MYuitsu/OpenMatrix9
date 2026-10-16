# OM9-M09-L4-GMVP-016 — Đóng gói, cài đặt, rollback và kênh phân phối

**Gate:** `G-MVP` · **Loại:** `DISTRIBUTION` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Kiểm khả năng cài/gỡ/cập nhật và phục hồi project an toàn trên artifact mang release version.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Tạo release package từ build đã kiểm; chốt version, checksum, source tag/SHA, bundled libs, notices và provenance.
4. Kiểm clean install, app startup, load project; kiểm update path được tuyên bố hỗ trợ, không âm thầm chuyển định dạng.
5. Thử uninstall/rollback có backup/recovery; đối chiếu project hashes và user-data retention contract.
6. Nếu có code signing, verify chain; nếu không, công bố chính xác trạng thái không ký, không giả chữ ký.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-01-001` · `OM9-L2-01-01` · `E02` · `cap.01-01.application-startup-and-main-window`
- `OM9-L3-01-12-001` · `OM9-L2-01-12` · `E02` · `cap.01-12.native-project-container-format`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-003`
- `OM9-M09-L4-GMVP-004`
- `OM9-M09-L4-GMVP-008`
- `OM9-M09-L4-GMVP-014`
- `OM9-M09-L4-GMVP-015`

## Acceptance

**Oracle cụ thể:** Release installer/package checksum matches tested binary and licenses; clean install works; advertised update/rollback retains projects and published limitations are consistent.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Không rollback/recover được khi tính năng đó được cam kết: FAIL.
- License notices và executable mismatch: BLOCKED_RELEASE.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-016-01` (ACCEPTANCE): Tạo release package từ build đã kiểm; chốt version, checksum, source tag/SHA, bundled libs, notices và provenance.
- `M09-TC-GMVP-016-02` (ACCEPTANCE): Kiểm clean install, app startup, load project; kiểm update path được tuyên bố hỗ trợ, không âm thầm chuyển định dạng.
- `M09-TC-GMVP-016-03` (ACCEPTANCE): Thử uninstall/rollback có backup/recovery; đối chiếu project hashes và user-data retention contract.
- `M09-TC-GMVP-016-04` (ACCEPTANCE): Nếu có code signing, verify chain; nếu không, công bố chính xác trạng thái không ký, không giả chữ ký.
- `M09-TC-GMVP-016-N01` (NEGATIVE): Không rollback/recover được khi tính năng đó được cam kết: FAIL.
- `M09-TC-GMVP-016-N02` (NEGATIVE): License notices và executable mismatch: BLOCKED_RELEASE.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-016/packaging-install-rollback.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- M00 G00-07 packaging; native release evidence
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
