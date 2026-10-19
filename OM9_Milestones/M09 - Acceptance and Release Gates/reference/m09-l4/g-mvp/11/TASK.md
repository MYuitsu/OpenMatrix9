# OM9-M09-L4-GMVP-011 — Recompute nhẫn và đá, conflict minh bạch

**Gate:** `G-MVP` · **Loại:** `HISTORY_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Kiểm sự phụ thuộc ring ↔ gem ↔ setting ↔ cutter qua thay đổi tham số và history topology.

## Thứ tự làm
1. Kiểm các gate đã ký: G00 → E01..E12; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Dùng fixture ban đầu, thay ring size/gem size/profile; thu feature DAG và old/new BRep IDs/labels.
4. Kiểm các dependent features recompute topo đúng hoặc báo conflict có vị trí và cách xử lý, không detach im lặng.
5. Undo/Redo thay đổi parameter và Save/Open; so scene graph, topological naming, geometry metric.
6. Nếu topology ambiguous, kiểm typed conflict thay vì tự nối sang mặt/edge sai.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-01-04-003` · `OM9-L2-01-04` · `E09` · `cap.01-04.dirty-propagation-and-topological-order`
- `OM9-L3-04-02-001` · `OM9-L2-04-02` · `E09` · `cap.04-02.ring-rail-jewelry-size-standard`
- `OM9-L3-04-13-002` · `OM9-L2-04-13` · `E10` · `cap.04-13.gem-control-and-size-edits`
- `OM9-L3-03-13-003` · `OM9-L2-03-13` · `E09` · `cap.03-13.matrix-history-broken-link-report`
- `OM9-L3-04-15-002` · `OM9-L2-04-15` · `E11` · `cap.04-15.prong-editor-selected`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GMVP-007`
- `OM9-M09-L4-GMVP-010`

## Acceptance

**Oracle cụ thể:** Editing ring/gem dimensions recomputes dependent geometry correctly or returns an actionable conflict; topology naming never silently rebinds to the wrong entity.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Stale cutter/prong hoặc stable ID bị gán sai sau recompute: FAIL.
- Auto-recompute không đủ evidence phải BLOCKED_REFERENCE thay vì PASS.

## Ca kiểm thử có mã định danh

- `M09-TC-GMVP-011-01` (ACCEPTANCE): Dùng fixture ban đầu, thay ring size/gem size/profile; thu feature DAG và old/new BRep IDs/labels.
- `M09-TC-GMVP-011-02` (ACCEPTANCE): Kiểm các dependent features recompute topo đúng hoặc báo conflict có vị trí và cách xử lý, không detach im lặng.
- `M09-TC-GMVP-011-03` (ACCEPTANCE): Undo/Redo thay đổi parameter và Save/Open; so scene graph, topological naming, geometry metric.
- `M09-TC-GMVP-011-04` (ACCEPTANCE): Nếu topology ambiguous, kiểm typed conflict thay vì tự nối sang mặt/edge sai.
- `M09-TC-GMVP-011-N01` (NEGATIVE): Stale cutter/prong hoặc stable ID bị gán sai sau recompute: FAIL.
- `M09-TC-GMVP-011-N02` (NEGATIVE): Auto-recompute không đủ evidence phải BLOCKED_REFERENCE thay vì PASS.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-MVP/OM9-M09-L4-GMVP-011/recompute-dependency-check.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- E09 History + E10 Gem + E11 settings contracts
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
