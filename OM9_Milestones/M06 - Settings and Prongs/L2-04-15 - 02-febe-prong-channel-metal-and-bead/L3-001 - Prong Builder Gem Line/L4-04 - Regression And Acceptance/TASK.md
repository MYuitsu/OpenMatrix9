# OM9-M06-L4-15-001-04 — Prong Builder Gem Line: Hồi quy, liên kết, nghiệm thu E11

**Owner:** `OM9-L2-04-15` · **L3:** `OM9-L3-04-15-001` · **capability:** `cap.04-15.prong-builder-gem-line` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/02-febe-prong-channel-metal-and-bead-settings/001-febe-prong-builder-gem-line/l3.json`.
- Matrix 8 Book 2: printed 483/pdf 257 (section_heading); printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [257, 258, 259, 260].
- Sách 2 trang in 483–486, Prong Builder.
- Input: ordered open/closed gem row từ E10; gem transform và spacing.
- Controls: prong modes, count, diameter, height, taper, orientation.

## Phạm vi đúng 1 thay đổi

Bổ sung kiểm thử tích hợp, roundtrip, thao tác tay có kịch bản; ghi evidence còn thiếu. Không tự ký PASS, chỉ nâng trạng thái khi có log thực và sign-off.

**Target invariant:** prongs theo thứ tự gems, không sinh duplicate ở shared junctions/closed loop.

## Thứ tự làm, từng bước

1. Use fixtures VALID, INVALID, UPDATE; check exact goal: open/closed rows, varied spacing, count and shared-prong symmetry; valid BRep.
2. Test end-to-end in real app: create or open E10 gem(s), generate setting, modify gem, observe update or explicit conflict.
3. Test save/open, cross-module associations, stable IDs, mm units, Undo/Redo and no invisible orphan shapes.
4. Run E01–E10 required regression suites and E11 affected tests using exact commands and capture stdout/stderr/exit codes.
5. Record screenshot/video for manual FEBE and reviewer + machine profile; pass only with external sign-off.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: `OM9-M06-L4-15-001-03`

## Acceptance / tests

- [ ] E2E VALID: open/closed rows, varied spacing, count and shared-prong symmetry; valid BRep
- [ ] E2E INVALID: 0/1 gem ở mode không hỗ trợ; chồng lấn; mixed gems; invalid row order ⇒ typed failure, unchanged doc
- [ ] Roundtrip persistence: IDs, params, feature graph, solids and mass (where applicable), unit conversion
- [ ] Undo/Redo and canceled preview leave no artifacts; reset/restart shows same persisted model
- [ ] Re-run actual E10 and kernel/OCCT contract regressions; log differences as blockers

## Manual UI

- [ ] chọn gem line, F6/Settings Prong Builder, đổi mode và sửa diam/height
- [ ] Expected: render only reflects committed geometry; details and material remain stable after reopen
- [ ] Expected: reviewer documents OS/GPU/toolchain, expected vs actual and sign-off/FAIL

## Lỗi phải xử lý

- 0/1 gem ở mode không hỗ trợ; chồng lấn; mixed gems; invalid row order.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Tests and manual run actually performed; attached log and golden geometry metrics
- [ ] No unsolved P0/P1 data/geometry blockers; no license/proprietary-asset blockers
- [ ] Human signed E11-UI checklist; QA task remains PLANNED/BLOCKED otherwise

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-15-001/acceptance-and-regression.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
