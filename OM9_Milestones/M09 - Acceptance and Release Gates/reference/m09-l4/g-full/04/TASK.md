# OM9-M09-L4-GFULL-004 — SmartFlow Rigid giữ hình đá/setting

**Gate:** `G-FULL` · **Loại:** `HISTORY_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Thẩm định gem/setting flows rigid và liên kết lịch sử, tránh biến dạng đá hoặc sai tâm/normal.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Đặt gem/setting instance lên Base rồi map sang bề mặt có curvature thay đổi với rigid flag.
4. So kích thước/shape của gem trước-sau bằng metrics; tâm/normal đúng với tangent target, không méo viên đá.
5. Sửa target surface, Undo/Redo và save/open; xác minh dependent features cập nhật hoặc conflict rõ.
6. Kiểm khác biệt rigid với deformable setting trên cùng fixture, không coi mọi object đều rigid.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-05-003` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-rigid-object-placement`
- `OM9-L3-04-05-002` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-mapping-base-to-target`
- `OM9-L3-04-09-003` · `OM9-L2-04-09` · `E10` · `cap.04-09.gem-loader-shape-cut-size-picker`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-003`

## Acceptance

**Oracle cụ thể:** Rigid gems/settings preserve shape and dimensions under flow while matching target frame; source edits recalc or raise explicit conflict.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Rigid gem bị scale/deform hoặc lost feature link: FAIL.
- Không tạo copy BRep detached âm thầm.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-004-01` (ACCEPTANCE): Đặt gem/setting instance lên Base rồi map sang bề mặt có curvature thay đổi với rigid flag.
- `M09-TC-GFULL-004-02` (ACCEPTANCE): So kích thước/shape của gem trước-sau bằng metrics; tâm/normal đúng với tangent target, không méo viên đá.
- `M09-TC-GFULL-004-03` (ACCEPTANCE): Sửa target surface, Undo/Redo và save/open; xác minh dependent features cập nhật hoặc conflict rõ.
- `M09-TC-GFULL-004-04` (ACCEPTANCE): Kiểm khác biệt rigid với deformable setting trên cùng fixture, không coi mọi object đều rigid.
- `M09-TC-GFULL-004-N01` (NEGATIVE): Rigid gem bị scale/deform hoặc lost feature link: FAIL.
- `M09-TC-GFULL-004-N02` (NEGATIVE): Không tạo copy BRep detached âm thầm.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-004/smartflow-rigid-history.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- Matrix 8 Book 2 Smart Flow Rigid printed p244
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

## Trang PDF để xem workflow (không dùng làm test oracle tự động)
- `matrix_8_manual_book2.pdf`: trang PDF 18

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
