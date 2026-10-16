# OM9-M09-L4-GFULL-012 — Quyết định ký G-FULL GO/NO-GO

**Gate:** `G-FULL` · **Loại:** `SIGNOFF` · **Trạng thái:** `PLANNED_NOT_EXECUTED` (chưa kiểm thử)
**Milestone owner:** `M09` (chỉ kiểm định; **không có L2/L3 mới**).

## Mục tiêu
Ký gate FULL chỉ khi E13, test advanced và mọi hồi quy MVP đều hoàn tất có bằng chứng.

## Thứ tự làm
1. Kiểm các gate đã ký: G-MVP signed → E13; kiểm source, artifact, fixtures và môi trường thật.
2. Đọc L3 canonical và kiểm mã nguồn thật từ repository đã checkout (không suy luận tên crate, API hoặc phiên bản OCCT).
3. Tập hợp 11 work order evidence, defect severity/retest, manual acceptance và legal delta.
4. Độc lập xác minh G-MVP vẫn PASS trên binary FULL và E13 features theo scope; phân loại NO_GO nếu blocker.
5. Reviewer có thẩm quyền ký hoặc từ chối; đóng băng Full artifact hashes, release notes và regression baseline.
6. Ghi rõ FULL = advanced jewelry CAD trong phạm vi v0.12, không phải feature parity toàn bộ Matrix 9.
7. Thu log/sha/evidence; nếu lỗi, ghi issue và gửi lại owner L2; nếu thiếu điều kiện, đánh dấu BLOCKED.

## Traceability đến L3 hiện có
- `OM9-L3-04-05-002` · `OM9-L2-04-05` · `E13` · `cap.04-05.smartflow-mapping-base-to-target`
- `OM9-L3-05-03-003` · `OM9-L2-05-03` · `E13` · `cap.05-03.user-library-insert-model`

## Phụ thuộc giữa các L4
- `OM9-M09-L4-GFULL-001`
- `OM9-M09-L4-GFULL-002`
- `OM9-M09-L4-GFULL-003`
- `OM9-M09-L4-GFULL-004`
- `OM9-M09-L4-GFULL-005`
- `OM9-M09-L4-GFULL-006`
- `OM9-M09-L4-GFULL-007`
- `OM9-M09-L4-GFULL-008`
- `OM9-M09-L4-GFULL-009`
- `OM9-M09-L4-GFULL-010`
- `OM9-M09-L4-GFULL-011`

## Acceptance

**Oracle cụ thể:** Human release authority signs GO only with signed G-MVP, E13, all 11 FULL evidence items and entire MVP regression PASS; otherwise NO_GO or AWAITING_SIGNOFF.

- [ ] All task-specific steps have reproducible actual vs expected evidence tied to source SHA, artifact hash, signed tolerance/fixture and environment.
- [ ] Every negative case is run or explicitly BLOCKED with issue owner; no silent waivers.
- [ ] Any issue requiring code changes is filed to the canonical owning module and re-tested, never implemented by M09.
- [ ] PASS requires independent reviewer approval of this task evidence; agent cannot self-approve release gates.

## Bắt lỗi / negative tests
- Agent không được tự ký; thiếu chữ ký hoặc MVP regression: AWAITING_SIGNOFF/NO_GO.
- Không được phát hành chỉ dựa trên metadata kế hoạch.

## Ca kiểm thử có mã định danh

- `M09-TC-GFULL-012-01` (ACCEPTANCE): Tập hợp 11 work order evidence, defect severity/retest, manual acceptance và legal delta.
- `M09-TC-GFULL-012-02` (ACCEPTANCE): Độc lập xác minh G-MVP vẫn PASS trên binary FULL và E13 features theo scope; phân loại NO_GO nếu blocker.
- `M09-TC-GFULL-012-03` (ACCEPTANCE): Reviewer có thẩm quyền ký hoặc từ chối; đóng băng Full artifact hashes, release notes và regression baseline.
- `M09-TC-GFULL-012-04` (ACCEPTANCE): Ghi rõ FULL = advanced jewelry CAD trong phạm vi v0.12, không phải feature parity toàn bộ Matrix 9.
- `M09-TC-GFULL-012-N01` (NEGATIVE): Agent không được tự ký; thiếu chữ ký hoặc MVP regression: AWAITING_SIGNOFF/NO_GO.
- `M09-TC-GFULL-012-N02` (NEGATIVE): Không được phát hành chỉ dựa trên metadata kế hoạch.

## Bằng chứng
- **Đường dẫn đích dự kiến (CHƯA tạo):** `implementation/evidence/M09/G-FULL/OM9-M09-L4-GFULL-012/g-full-decision.json`
- Ghi từng lệnh thật + exit code, expected/actual, SHA nguồn và binary, OCCT version/ABI, OS/GPU, test fixture provenance, sai số theo G00, reviewer và timestamp.
- M09 chỉ xác nhận chất lượng; sửa lỗi thực hiện tại L2 gốc qua phiếu lỗi và rerun liên quan.

## Tài liệu nguồn
- M09_RELEASE_GATES; human signoff policy
- planning/M09_RELEASE_GATES.md
- planning/GATES_AND_EVIDENCE.md
- planning/DEPENDENCY_AND_RELEASE_POLICY.md

**Quy tắc dừng:** không có signed gates/release artifact/toolchain/fixtures thì chỉ audit tài liệu, ghi BLOCKED; không giả log, không tự ký G-MVP/G-FULL.
