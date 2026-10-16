# OM9-M09-L4-GFULL-002 — Ma trận truy vết nâng cao E13

**Gate:** `G-FULL` · **Loại:** `GOVERNANCE` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Ràng buộc chức năng nâng cao vào 73 L3 canonical của E13 và từng acceptance oracle.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Đọc manifest E13 và tạo ma trận 73 L3 với owner, supported vs limited, fixture/golden/evidence và human acceptance.
4. Phân loại SmartFlow, rigid flow, transform/deform, builders, SmartPattern, Rope, user library thành nhóm test rõ.
5. Xác nhận AFTER/Clayoo/SubD/Emboss/Render/CAM không bị quảng cáo thành FULL; FULL không có nghĩa 100% Matrix 9.
6. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-05-001` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-autobase-surface-generation`
- `OM9-L3-04-06-001` · `OM9-L2-04-06` · `E13` · `cap.04-06.knot-builder`
- `OM9-L3-04-08-001` · `OM9-L2-04-08` · `E13` · `cap.04-08.bend-deformation-command`
- `OM9-L3-05-01-001` · `OM9-L2-05-01` · `E13` · `cap.05-01.smartpattern-source-pattern-choice`
- `OM9-L3-05-02-001` · `OM9-L2-05-02` · `E13` · `cap.05-02.rope-builder-curve-input`
- `OM9-L3-05-03-001` · `OM9-L2-05-03` · `E13` · `cap.05-03.user-library-folder-catalog`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-001`

## Acceptance

**Oracle cụ thể:** All 73 E13 canonical L3 are traced once with verified owner and explicit supported/limited disposition; AFTER excluded from release claims.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- L3 E13 chưa có provenance hoặc acceptance: BLOCKED_COVERAGE.
- Không thêm duplicate L2 release owner.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-002-01` (ACCEPTANCE): Đọc manifest E13 và tạo ma trận 73 L3 với owner, supported vs limited, fixture/golden/evidence và human acceptance.
- `M09-TC-GFULL-002-02` (ACCEPTANCE): Phân loại SmartFlow, rigid flow, transform/deform, builders, SmartPattern, Rope, user library thành nhóm test rõ.
- `M09-TC-GFULL-002-03` (ACCEPTANCE): Xác nhận AFTER/Clayoo/SubD/Emboss/Render/CAM không bị quảng cáo thành FULL; FULL không có nghĩa 100% Matrix 9.
- `M09-TC-GFULL-002-N01` (NEGATIVE): L3 E13 chưa có provenance hoặc acceptance: BLOCKED_COVERAGE.
- `M09-TC-GFULL-002-N02` (NEGATIVE): Không thêm duplicate L2 release owner.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-002/full-traceability.csv`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- E13 execution manifest and scope policy
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
