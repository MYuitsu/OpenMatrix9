# OM9-M09-L4-GMVP-007 — Undo, Redo, Cancel và selection state

**Gate:** `G-MVP` · **Loại:** `TRANSACTION_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Kiểm nguyên tử document transaction, tính ổn định EntityID, undo stack và rollback sau lỗi.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Chạy chuỗi create-edit-delete-undo-redo có cả feature graph/selected instances; lưu entity count/hash trước và sau.
4. Cancel giữa preview của builder, transform, cutter; xác nhận tài liệu gốc, selection, history, GPU preview không bị ghi nửa chừng.
5. Inject invalid input/kernel exception và kiểm một thao tác người dùng tương ứng một transaction Undo.
6. Lưu/mở lại, thực hiện Undo/Redo theo chính sách sản phẩm được công bố và ghi rõ nếu session undo không persisted.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-03-001` · `OM9-L2-01-03` · `E01` · `cap.01-03.document-entity-ids`
- `OM9-L3-01-04-001` · `OM9-L2-01-04` · `E09` · `cap.01-04.feature-record-and-typed-dependency-edges`
- `OM9-L3-03-13-001` · `OM9-L2-03-13` · `E09` · `cap.03-13.matrix-history-record-toggle`
- `OM9-L3-01-07-002` · `OM9-L2-01-07` · `E03` · `cap.01-07.object-selection-set-core`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-005`
- `OM9-M09-L4-GMVP-006`

## Acceptance

**Oracle cụ thể:** For each test transaction, Cancel and failure preserve the pre-action scene/document snapshot; Undo/Redo restore expected feature DAG, entity IDs and geometry without phantom preview.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Silent history detach, orphan feature, stale selection hoặc phantom preview: FAIL.
- Không coi chụp ảnh UI là đủ để xác minh document IDs.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-007-01` (ACCEPTANCE): Chạy chuỗi create-edit-delete-undo-redo có cả feature graph/selected instances; lưu entity count/hash trước và sau.
- `M09-TC-GMVP-007-02` (ACCEPTANCE): Cancel giữa preview của builder, transform, cutter; xác nhận tài liệu gốc, selection, history, GPU preview không bị ghi nửa chừng.
- `M09-TC-GMVP-007-03` (ACCEPTANCE): Inject invalid input/kernel exception và kiểm một thao tác người dùng tương ứng một transaction Undo.
- `M09-TC-GMVP-007-04` (ACCEPTANCE): Lưu/mở lại, thực hiện Undo/Redo theo chính sách sản phẩm được công bố và ghi rõ nếu session undo không persisted.
- `M09-TC-GMVP-007-N01` (NEGATIVE): Silent history detach, orphan feature, stale selection hoặc phantom preview: FAIL.
- `M09-TC-GMVP-007-N02` (NEGATIVE): Không coi chụp ảnh UI là đủ để xác minh document IDs.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-007/transaction-rollback-matrix.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- GATES_AND_EVIDENCE; stage E09 feature graph
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
