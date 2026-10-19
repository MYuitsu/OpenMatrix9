# OM9-M09-L4-GMVP-003 — Audit giấy phép, nguồn và SBOM

**Gate:** `G-MVP` · **Loại:** `COMPLIANCE` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Xác thực điều kiện tái phân phối AGPL/OCCT/openNURBS, dependencies và toàn bộ asset trước bản phát hành.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Lập SBOM theo binary/crate/C++ library thực tại build SHA; pin versions và licenses/NOTICE.
4. Kiểm license QymCAD AGPL-3.0-or-later, OCCT, openNURBS và nghĩa vụ cung cấp nguồn theo phương án phân phối đã phê duyệt.
5. Quét assets/font/logo/marketing/screenshots và attribution; xác minh không sao chép tài sản thương mại Matrix.
6. Gửi unresolved conflicts cho người phụ trách pháp lý, không tự cho phép ngoại lệ.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-05-001` · `OM9-L2-01-05` · `E01` · `cap.01-05.occt-ffi-safe-handle-lifecycle`
- `OM9-L3-01-13-002` · `OM9-L2-01-13` · `E08` · `cap.01-13.opennurbs-reader-bridge-contract`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-001`
- `OM9-M09-L4-GMVP-002`

## Acceptance

**Oracle cụ thể:** SBOM and all bundled assets/dependencies have reviewed SPDX/license/provenance; no unapproved distribution obligation, trademark or proprietary asset remains.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- License chưa được phê duyệt hoặc asset không có provenance: BLOCKED_LEGAL.
- Lỗi SBOM thiếu direct/transitive dependency: FAIL.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-003-01` (ACCEPTANCE): Lập SBOM theo binary/crate/C++ library thực tại build SHA; pin versions và licenses/NOTICE.
- `M09-TC-GMVP-003-02` (ACCEPTANCE): Kiểm license QymCAD AGPL-3.0-or-later, OCCT, openNURBS và nghĩa vụ cung cấp nguồn theo phương án phân phối đã phê duyệt.
- `M09-TC-GMVP-003-03` (ACCEPTANCE): Quét assets/font/logo/marketing/screenshots và attribution; xác minh không sao chép tài sản thương mại Matrix.
- `M09-TC-GMVP-003-04` (ACCEPTANCE): Gửi unresolved conflicts cho người phụ trách pháp lý, không tự cho phép ngoại lệ.
- `M09-TC-GMVP-003-N01` (NEGATIVE): License chưa được phê duyệt hoặc asset không có provenance: BLOCKED_LEGAL.
- `M09-TC-GMVP-003-N02` (NEGATIVE): Lỗi SBOM thiếu direct/transitive dependency: FAIL.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-003/license-sbom-review.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- M00 SOURCE_AUDIT_V012, license files, packaging manifests
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
