# OM9-M09-L4-GMVP-012 — Cutter và an toàn hình học sản xuất

**Gate:** `G-MVP` · **Loại:** `CUTTER_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Đánh giá chính xác lỗ, kênh và negative volume cutter; không viết lại owner E12.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Kiểm single/multi-gem cutters, channel và cut-to-finger rail trên fixtures gồm close clearance/tangency.
4. Đo removed volume và topological validity của kim loại; xác minh không trừ khỏi gem/reference geometry.
5. Phát hiện naked edge, inverted normal, self intersections và zero-thickness theo production fixture.
6. Nếu fail, mở defect về L2 04-16/17/18 hoặc OCCT owner có bằng chứng, M09 không sửa thuật toán.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-16-001` · `OM9-L2-04-16` · `E12` · `cap.04-16.gem-cutter-azure-for-single-stone`
- `OM9-L3-04-17-001` · `OM9-L2-04-17` · `E12` · `cap.04-17.channel-builder-cutter`
- `OM9-L3-04-18-001` · `OM9-L2-04-18` · `E12` · `cap.04-18.cut-to-finger-rail`
- `OM9-L3-01-10-002` · `OM9-L2-01-10` · `E07` · `cap.01-10.brep-boolean-difference`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-006`
- `OM9-M09-L4-GMVP-010`

## Acceptance

**Oracle cụ thể:** Cutter removes only intended metal volume and preserves valid topology as documented by fixtures; intentionally invalid/unsafe outputs are rejected.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Negative volume ngược chiều, manifold thất bại hoặc cắt sai đối tượng: FAIL.
- Không tự đặt kích thước an toàn sản xuất ngoài fixture được phê duyệt.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-012-01` (ACCEPTANCE): Kiểm single/multi-gem cutters, channel và cut-to-finger rail trên fixtures gồm close clearance/tangency.
- `M09-TC-GMVP-012-02` (ACCEPTANCE): Đo removed volume và topological validity của kim loại; xác minh không trừ khỏi gem/reference geometry.
- `M09-TC-GMVP-012-03` (ACCEPTANCE): Phát hiện naked edge, inverted normal, self intersections và zero-thickness theo production fixture.
- `M09-TC-GMVP-012-04` (ACCEPTANCE): Nếu fail, mở defect về L2 04-16/17/18 hoặc OCCT owner có bằng chứng, M09 không sửa thuật toán.
- `M09-TC-GMVP-012-N01` (NEGATIVE): Negative volume ngược chiều, manifold thất bại hoặc cắt sai đối tượng: FAIL.
- `M09-TC-GMVP-012-N02` (NEGATIVE): Không tự đặt kích thước an toàn sản xuất ngoài fixture được phê duyệt.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-012/cutter-safety.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- E12 cutter contract; Matrix 8 Book 2 cutter chapter
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `matrix_8_manual_book2.pdf`: trang PDF 272, 273

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
