# OM9-M09-L4-GMVP-013 — STL: đơn vị, winding và external viewer

**Gate:** `G-MVP` · **Loại:** `EXPORT_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Thẩm định STL xuất sản xuất trên hình học nguồn được kiểm chứng và chương trình đọc độc lập.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Xuất STL của fixture closed solid ở mm; ghi source BRep hash, triangle count, file hash và export settings.
4. Import vào ít nhất một viewer độc lập được định danh/version; đo bbox, scale, winding, manifold, triangle degeneracy theo oracle.
5. Lặp qua geometry có nhiều solid, hole và unsupported open geometry; kiểm cảnh báo/phân biệt display tessellation.
6. Kiểm giới hạn: thành phẩm CAD/STL không tự chứng nhận đã sản xuất thực tế hoặc đủ quy chuẩn cơ khí.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-12-008` · `OM9-L2-01-12` · `E12` · `cap.01-12.stl-tessellation-export-codec`
- `OM9-L3-04-16-001` · `OM9-L2-04-16` · `E12` · `cap.04-16.gem-cutter-azure-for-single-stone`
- `OM9-L3-01-05-002` · `OM9-L2-01-05` · `E01` · `cap.01-05.brep-shape-type-and-validation`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-006`
- `OM9-M09-L4-GMVP-010`
- `OM9-M09-L4-GMVP-012`

## Acceptance

**Oracle cụ thể:** Independent STL viewer confirms millimeter scale, bbox, manifold and orientation against approved BRep/mesh fixture; no claim of physical-production certification.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Sai mm↔inch, nonmanifold ngoài fixture được phép, normal sai: FAIL.
- Không có independent viewer evidence: BLOCKED_REFERENCE.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-013-01` (ACCEPTANCE): Xuất STL của fixture closed solid ở mm; ghi source BRep hash, triangle count, file hash và export settings.
- `M09-TC-GMVP-013-02` (ACCEPTANCE): Import vào ít nhất một viewer độc lập được định danh/version; đo bbox, scale, winding, manifold, triangle degeneracy theo oracle.
- `M09-TC-GMVP-013-03` (ACCEPTANCE): Lặp qua geometry có nhiều solid, hole và unsupported open geometry; kiểm cảnh báo/phân biệt display tessellation.
- `M09-TC-GMVP-013-04` (ACCEPTANCE): Kiểm giới hạn: thành phẩm CAD/STL không tự chứng nhận đã sản xuất thực tế hoặc đủ quy chuẩn cơ khí.
- `M09-TC-GMVP-013-N01` (NEGATIVE): Sai mm↔inch, nonmanifold ngoài fixture được phép, normal sai: FAIL.
- `M09-TC-GMVP-013-N02` (NEGATIVE): Không có independent viewer evidence: BLOCKED_REFERENCE.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-013/stl-external-audit.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- E12 STL export + independent viewer policy
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
