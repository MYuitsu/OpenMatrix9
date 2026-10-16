# OM9-M09-L4-GFULL-009 — Cross-feature history, Cancel và Save/Open nâng cao

**Gate:** `G-FULL` · **Loại:** `TRANSACTION_QA` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Kiểm giao nhau SmartFlow → pattern/rope → object/history với baseline MVP.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Dùng combined fixture flow ornament và rigid gems; chỉnh target/UV, rope twists và pattern pitch.
4. So feature DAG/EntityID, regeneration order, deterministic BRep, pending conflict trước-sau.
5. Cancel preview giữa recalculation, injection invalid parameter; kiểm rollback và no phantom preview.
6. Save/open rồi Undo/Redo theo chính sách session; xác minh MVP ring/cutter vẫn nguyên ownership.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-05-002` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-mapping-base-to-target`
- `OM9-L3-05-01-002` · `OM9-L2-05-01` · `E13` · `cap.05-01.smartpattern-uv-distribution`
- `OM9-L3-05-02-003` · `OM9-L2-05-02` · `E13` · `cap.05-02.rope-builder-twists-and-spacing`
- `OM9-L3-01-04-003` · `OM9-L2-01-04` · `E09` · `cap.01-04.dirty-propagation-and-topological-order`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-003`
- `OM9-M09-L4-GFULL-004`
- `OM9-M09-L4-GFULL-006`
- `OM9-M09-L4-GFULL-007`
- `OM9-M09-L4-GFULL-008`

## Acceptance

**Oracle cụ thể:** Cross-feature rebuild, Undo/Redo, Cancel and Save/Open are atomic; no detached dependency or stale MVP geometry remains.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Infinite recompute loop, silently detached history hoặc inconsistent load: FAIL.
- Không tự sửa code E09/E11 từ M09; trả issue đúng owner.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-009-01` (ACCEPTANCE): Dùng combined fixture flow ornament và rigid gems; chỉnh target/UV, rope twists và pattern pitch.
- `M09-TC-GFULL-009-02` (ACCEPTANCE): So feature DAG/EntityID, regeneration order, deterministic BRep, pending conflict trước-sau.
- `M09-TC-GFULL-009-03` (ACCEPTANCE): Cancel preview giữa recalculation, injection invalid parameter; kiểm rollback và no phantom preview.
- `M09-TC-GFULL-009-04` (ACCEPTANCE): Save/open rồi Undo/Redo theo chính sách session; xác minh MVP ring/cutter vẫn nguyên ownership.
- `M09-TC-GFULL-009-N01` (NEGATIVE): Infinite recompute loop, silently detached history hoặc inconsistent load: FAIL.
- `M09-TC-GFULL-009-N02` (NEGATIVE): Không tự sửa code E09/E11 từ M09; trả issue đúng owner.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-009/advanced-crossworkflow-history.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- E13 + E09 feature graph contracts
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
