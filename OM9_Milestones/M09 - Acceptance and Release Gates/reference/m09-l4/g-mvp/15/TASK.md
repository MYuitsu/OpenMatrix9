# OM9-M09-L4-GMVP-015 — Kiểm thử thao tác người dùng theo OS/GPU

**Gate:** `G-MVP` · **Loại:** `MANUAL_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Tổ chức phiên acceptance thực tế với tester khác tác giả và điền actual vs expected cho bộ thao tác chính.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Chạy ứng dụng cài từ artifact sẽ phát hành trên từng target OS/GPU đã công bố.
4. Tester không phải tác giả kịch bản thao tác New/Open/Save, model/edit, undo, export và re-open kết quả; có screen recording nếu có thể.
5. Đối chiếu chính xác phiên build SHA, action sequence, accessibility/UX diagnostics và lỗi/limitation.
6. Reviewer ghi tên/role, timestamp, signature; tự động test không thay manual acceptance.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-03-01-001` · `OM9-L2-03-01` · `E02` · `cap.03-01.matrix-four-viewport-grid-layout`
- `OM9-L3-03-09-001` · `OM9-L2-03-09` · `E03` · `cap.03-09.f6-selected-object-class-dispatch`
- `OM9-L3-03-15-001` · `OM9-L2-03-15` · `E02` · `cap.03-15.matrix-file-new-open-save-actions`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-005`
- `OM9-M09-L4-GMVP-009`
- `OM9-M09-L4-GMVP-010`
- `OM9-M09-L4-GMVP-013`
- `OM9-M09-L4-GMVP-014`

## Acceptance

**Oracle cụ thể:** A designated independent human tester completes real installation/execution workflows on every supported OS/GPU profile and signs actual-versus-expected evidence.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Thiếu phiên kiểm thử OS/GPU đã quảng bá: BLOCKED_MANUAL.
- UI no-op/inoperable action: FAIL.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-015-01` (ACCEPTANCE): Chạy ứng dụng cài từ artifact sẽ phát hành trên từng target OS/GPU đã công bố.
- `M09-TC-GMVP-015-02` (ACCEPTANCE): Tester không phải tác giả kịch bản thao tác New/Open/Save, model/edit, undo, export và re-open kết quả; có screen recording nếu có thể.
- `M09-TC-GMVP-015-03` (ACCEPTANCE): Đối chiếu chính xác phiên build SHA, action sequence, accessibility/UX diagnostics và lỗi/limitation.
- `M09-TC-GMVP-015-04` (ACCEPTANCE): Reviewer ghi tên/role, timestamp, signature; tự động test không thay manual acceptance.
- `M09-TC-GMVP-015-N01` (NEGATIVE): Thiếu phiên kiểm thử OS/GPU đã quảng bá: BLOCKED_MANUAL.
- `M09-TC-GMVP-015-N02` (NEGATIVE): UI no-op/inoperable action: FAIL.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-015/human-acceptance-matrix.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- M09 gates; project UX acceptance
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
