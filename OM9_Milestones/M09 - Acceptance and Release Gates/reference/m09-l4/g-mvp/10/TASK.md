# OM9-M09-L4-GMVP-010 — Luồng thiết kế nhẫn từ rail tới setting

**Gate:** `G-MVP` · **Loại:** `E2E_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Chạy kịch bản sản phẩm end-to-end: ring rail/profile → gem placement → head/bezel/prong → cutter.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Tạo rail + profile từ signed fixture, dựng ring BRep và kiểm dimensions/closedness.
4. Đặt gem row hoặc solitaire, tạo head/bezel/prong theo hợp đồng E10/E11; check collision, alignment và source gem refs.
5. Chạy cutter thuộc E12 và so Boolean difference, feature graph và BRep validity; không chỉ nhìn mesh.
6. Save/open và kiểm mass/metal vs nonmetal attribution theo fixtures, ghi lệch measured/expected.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-02-001` · `OM9-L2-04-02` · `E09` · `cap.04-02.ring-rail-jewelry-size-standard`
- `OM9-L3-04-02-003` · `OM9-L2-04-02` · `E09` · `cap.04-02.profile-placer-on-rail`
- `OM9-L3-04-09-003` · `OM9-L2-04-09` · `E10` · `cap.04-09.gem-loader-shape-cut-size-picker`
- `OM9-L3-04-10-001` · `OM9-L2-04-10` · `E10` · `cap.04-10.gem-on-curve-basic`
- `OM9-L3-04-14-001` · `OM9-L2-04-14` · `E11` · `cap.04-14.head-builder-basket`
- `OM9-L3-04-15-001` · `OM9-L2-04-15` · `E11` · `cap.04-15.prong-builder-gem-line`
- `OM9-L3-04-16-001` · `OM9-L2-04-16` · `E12` · `cap.04-16.gem-cutter-azure-for-single-stone`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-005`
- `OM9-M09-L4-GMVP-006`
- `OM9-M09-L4-GMVP-007`

## Acceptance

**Oracle cụ thể:** Ring/profile/gem/setting/cutter scenario yields geometrically correct outputs, stable refs and metal-only mass per approved oracles; result survives save/open.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Prong pierces gem, cutter leaves invalid solid, hoặc mass tính cả guide/gem: FAIL.
- Nếu L3 cụ thể chưa đóng PASS, không waiving capability bắt buộc.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-010-01` (ACCEPTANCE): Tạo rail + profile từ signed fixture, dựng ring BRep và kiểm dimensions/closedness.
- `M09-TC-GMVP-010-02` (ACCEPTANCE): Đặt gem row hoặc solitaire, tạo head/bezel/prong theo hợp đồng E10/E11; check collision, alignment và source gem refs.
- `M09-TC-GMVP-010-03` (ACCEPTANCE): Chạy cutter thuộc E12 và so Boolean difference, feature graph và BRep validity; không chỉ nhìn mesh.
- `M09-TC-GMVP-010-04` (ACCEPTANCE): Save/open và kiểm mass/metal vs nonmetal attribution theo fixtures, ghi lệch measured/expected.
- `M09-TC-GMVP-010-N01` (NEGATIVE): Prong pierces gem, cutter leaves invalid solid, hoặc mass tính cả guide/gem: FAIL.
- `M09-TC-GMVP-010-N02` (NEGATIVE): Nếu L3 cụ thể chưa đóng PASS, không waiving capability bắt buộc.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-010/jewelry-e2e-mvp.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- Matrix 8 Book 1 Ring/Tools; Book 2 Gems/Settings/Cutters
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `matrix_8_manual_book2.pdf`: trang PDF 177, 242, 272

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
