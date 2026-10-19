# OM9-M09-L4-GMVP-009 — Đối chiếu phạm vi nhập 3DM

**Gate:** `G-MVP` · **Loại:** `INTEROP_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Kiểm importer được công bố: units, trimmed BRep, curves, blocks, layer và cảnh báo dữ liệu không hỗ trợ.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Sử dụng corpus 3DM được phép phân phối với dữ liệu mm/inch, trimmed surfaces, blocks, user data và version header.
4. Import từng fixture rồi so đơn vị/dimensions, NURBS trims, layer hierarchy, transform và error report với nguồn độc lập.
5. Thử file mới hơn/chưa hỗ trợ, corrupt, truncated và unsupported object; bảo đảm rõ warning, không im lặng bỏ nội dung.
6. Phân biệt kiểm thử import 3DM với export 3DM, vốn ngoài phạm vi cam kết của v0.12.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-13-001` · `OM9-L2-01-13` · `E08` · `cap.01-13.3dm-file-signature-and-version-probe`
- `OM9-L3-01-13-002` · `OM9-L2-01-13` · `E08` · `cap.01-13.opennurbs-reader-bridge-contract`
- `OM9-L3-01-13-003` · `OM9-L2-01-13` · `E08` · `cap.01-13.3dm-document-units-tolerance-mapping`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-006`
- `OM9-M09-L4-GMVP-008`

## Acceptance

**Oracle cụ thể:** 3DM import retains verified supported units, trimmed BRep geometry, layers and block transforms; unsupported data are named in warnings; no silent loss or fabricated 3DM export.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Silent data loss hoặc sai tỷ lệ mm/inch: FAIL.
- Không có openNURBS license approval hoặc bridge build: BLOCKED_LEGAL/ENV.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-009-01` (ACCEPTANCE): Sử dụng corpus 3DM được phép phân phối với dữ liệu mm/inch, trimmed surfaces, blocks, user data và version header.
- `M09-TC-GMVP-009-02` (ACCEPTANCE): Import từng fixture rồi so đơn vị/dimensions, NURBS trims, layer hierarchy, transform và error report với nguồn độc lập.
- `M09-TC-GMVP-009-03` (ACCEPTANCE): Thử file mới hơn/chưa hỗ trợ, corrupt, truncated và unsupported object; bảo đảm rõ warning, không im lặng bỏ nội dung.
- `M09-TC-GMVP-009-04` (ACCEPTANCE): Phân biệt kiểm thử import 3DM với export 3DM, vốn ngoài phạm vi cam kết của v0.12.
- `M09-TC-GMVP-009-N01` (NEGATIVE): Silent data loss hoặc sai tỷ lệ mm/inch: FAIL.
- `M09-TC-GMVP-009-N02` (NEGATIVE): Không có openNURBS license approval hoặc bridge build: BLOCKED_LEGAL/ENV.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-009/3dm-import-scope.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- M00 3DM GAP proof; E08 contract; NO 3DM export scope
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
