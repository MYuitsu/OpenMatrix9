# OM9-M09-L4-GFULL-008 — User library: asset, metadata và roundtrip

**Gate:** `G-FULL` · **Loại:** `PERSISTENCE_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Kiểm catalog, insert/edit/save library theo thư viện do người dùng kiểm soát; không dùng asset thương mại.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Tạo user library fixtures tự cấp phép; scan catalog/index, preview và insert vào document.
4. Save/open, rename/move library root, mất asset và duplicate names; kiểm stable refs, warnings và fallback.
5. Kiểm path traversal/unsafe archive metadata và quyền đọc; không tự tải code hoặc asset không xác minh.
6. So hashes provenance và version của library items, vật liệu, dimensions sau roundtrip.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-05-03-001` · `OM9-L2-05-03` · `E13` · `cap.05-03.user-library-folder-catalog`
- `OM9-L3-05-03-002` · `OM9-L2-05-03` · `E13` · `cap.05-03.user-library-asset-register-and-preview`
- `OM9-L3-05-03-003` · `OM9-L2-05-03` · `E13` · `cap.05-03.user-library-insert-model`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-002`

## Acceptance

**Oracle cụ thể:** User library operations retain permitted assets and stable references through import, move and document save/open; missing/unsafe assets generate explicit safe errors.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Silent asset loss/reference hijack hoặc unsafe path handling: FAIL.
- Không đưa tài sản Gemvision Matrix vào gói phân phối.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-008-01` (ACCEPTANCE): Tạo user library fixtures tự cấp phép; scan catalog/index, preview và insert vào document.
- `M09-TC-GFULL-008-02` (ACCEPTANCE): Save/open, rename/move library root, mất asset và duplicate names; kiểm stable refs, warnings và fallback.
- `M09-TC-GFULL-008-03` (ACCEPTANCE): Kiểm path traversal/unsafe archive metadata và quyền đọc; không tự tải code hoặc asset không xác minh.
- `M09-TC-GFULL-008-04` (ACCEPTANCE): So hashes provenance và version của library items, vật liệu, dimensions sau roundtrip.
- `M09-TC-GFULL-008-N01` (NEGATIVE): Silent asset loss/reference hijack hoặc unsafe path handling: FAIL.
- `M09-TC-GFULL-008-N02` (NEGATIVE): Không đưa tài sản Gemvision Matrix vào gói phân phối.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-008/user-library-roundtrip.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- E13 user-library canonical; Matrix 9 Addendum Chapter 3
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `matrix9.0_addendum_manual.pdf`: trang PDF 89, 90

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
