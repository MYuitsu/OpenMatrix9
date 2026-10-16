# OM9-M09-L4-GMVP-006 — Kernel OCCT: BRep, NURBS, trim, boolean

**Gate:** `G-MVP` · **Loại:** `GEOMETRY_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Thẩm định hình học CAD chính xác bằng kiểm tra BRep và bộ mẫu độc lập, không dựa vào ảnh render.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Dùng signed G00 tolerance; fixtures có đơn vị/knot/trim/closed/open/degenerate topology và nguồn đối chiếu.
4. Chạy tạo curve/surface/solid, split/trim/sweep/boolean hợp lệ và cố ý lỗi; kiểm OCCT IsValid, orientation, volume/area bounds và topology.
5. So sánh kết quả BRep với oracle độc lập theo tolerance đã duyệt; xác nhận mesh renderer không được dùng thay chứng cứ exactness.
6. Phát hiện lỗi topology mapping/OCCT exception phải có typed error, không crash hoặc gây shape leak.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-05-002` · `OM9-L2-01-05` · `E01` · `cap.01-05.brep-shape-type-and-validation`
- `OM9-L3-01-05-003` · `OM9-L2-01-05` · `E01` · `cap.01-05.nurbs-curve-evaluation-adapter`
- `OM9-L3-01-10-001` · `OM9-L2-01-10` · `E07` · `cap.01-10.brep-boolean-union`
- `OM9-L3-01-10-002` · `OM9-L2-01-10` · `E07` · `cap.01-10.brep-boolean-difference`
- `OM9-L3-02-05-002` · `OM9-L2-02-05` · `E06` · `cap.02-05.loft-and-curve-network-workflows`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-001`
- `OM9-M09-L4-GMVP-004`

## Acceptance

**Oracle cụ thể:** Closed outputs are OCCT-valid closed BReps; declared open curves/surfaces are checked by their proper topology contracts. All measured invariants stay within approved G00 tolerance.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Open solid khi yêu cầu closed: FAIL.
- Chênh lệch numeric quá tolerance G00 hoặc lack provenance fixture: FAIL/BLOCKED_REFERENCE.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-006-01` (ACCEPTANCE): Dùng signed G00 tolerance; fixtures có đơn vị/knot/trim/closed/open/degenerate topology và nguồn đối chiếu.
- `M09-TC-GMVP-006-02` (ACCEPTANCE): Chạy tạo curve/surface/solid, split/trim/sweep/boolean hợp lệ và cố ý lỗi; kiểm OCCT IsValid, orientation, volume/area bounds và topology.
- `M09-TC-GMVP-006-03` (ACCEPTANCE): So sánh kết quả BRep với oracle độc lập theo tolerance đã duyệt; xác nhận mesh renderer không được dùng thay chứng cứ exactness.
- `M09-TC-GMVP-006-04` (ACCEPTANCE): Phát hiện lỗi topology mapping/OCCT exception phải có typed error, không crash hoặc gây shape leak.
- `M09-TC-GMVP-006-N01` (NEGATIVE): Open solid khi yêu cầu closed: FAIL.
- `M09-TC-GMVP-006-N02` (NEGATIVE): Chênh lệch numeric quá tolerance G00 hoặc lack provenance fixture: FAIL/BLOCKED_REFERENCE.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-006/kernel-geometry-validation.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- Rhino5 NURBS/solids principles; canonical G00 tolerance
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `rhino5_manual.pdf`: trang PDF 25, 26, 27, 30, 31

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
