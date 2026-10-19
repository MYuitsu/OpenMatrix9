# OM9-M06-L4-15-004-04 — Metal Piece From Gems: Hồi quy, liên kết, nghiệm thu E11

**Owner:** `OM9-L2-04-15` · **L3:** `OM9-L3-04-15-004` · **capability:** `cap.04-15.metal-piece-from-gems` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/02-febe-prong-channel-metal-and-bead-settings/004-febe-metal-piece-from-gems/l3.json`.
- Matrix 8 Book 2: printed 490/pdf 264 (section_heading); printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [264, 265, 266, 267, 268].
- Sách 2 trang in 490–494, Metal from Gems.
- Input: ordered gem row open/closed; feature params; optional channel border.
- Controls: open/closed modes; gem offset/end offset; wall/height/profile end widths.

## Phạm vi đúng 1 thay đổi

Bổ sung kiểm thử tích hợp, roundtrip, thao tác tay có kịch bản; ghi evidence còn thiếu. Không tự ký PASS, chỉ nâng trạng thái khi có log thực và sign-off.

**Target invariant:** metal rim/channel walls đúng offsets, không lẫn cutter với metal object.

## Thứ tự làm, từng bước

1. Use fixtures VALID, INVALID, UPDATE; check exact goal: open/closed fixture, compare offsets, validity of solids, stable parts after reorder.
2. Test end-to-end in real app: create or open E10 gem(s), generate setting, modify gem, observe update or explicit conflict.
3. Test save/open, cross-module associations, stable IDs, mm units, Undo/Redo and no invisible orphan shapes.
4. Run E01–E10 required regression suites and E11 affected tests using exact commands and capture stdout/stderr/exit codes.
5. Record screenshot/video for manual FEBE and reviewer + machine profile; pass only with external sign-off.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: `OM9-M06-L4-15-004-03`

## Acceptance / tests

- [ ] E2E VALID: open/closed fixture, compare offsets, validity of solids, stable parts after reorder
- [ ] E2E INVALID: row discontinuity; self-crossing offsets; thickness zero; closed-loop seam drift ⇒ typed failure, unchanged doc
- [ ] Roundtrip persistence: IDs, params, feature graph, solids and mass (where applicable), unit conversion
- [ ] Undo/Redo and canceled preview leave no artifacts; reset/restart shows same persisted model
- [ ] Re-run actual E10 and kernel/OCCT contract regressions; log differences as blockers

## Manual UI

- [ ] Settings > Metal from Gems; chọn Outside/Inside/Top và kiểm open/closed modes
- [ ] Expected: render only reflects committed geometry; details and material remain stable after reopen
- [ ] Expected: reviewer documents OS/GPU/toolchain, expected vs actual and sign-off/FAIL

**Boundary M06→E12:** `Metal Piece > Top` mô tả profile tương tự cutter; sinh geometry/preview trung gian, không nhân bản cutter/boolean manufacturing đã thuộc E12.

## Lỗi phải xử lý

- row discontinuity; self-crossing offsets; thickness zero; closed-loop seam drift.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Tests and manual run actually performed; attached log and golden geometry metrics
- [ ] No unsolved P0/P1 data/geometry blockers; no license/proprietary-asset blockers
- [ ] Human signed E11-UI checklist; QA task remains PLANNED/BLOCKED otherwise

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-15-004/acceptance-and-regression.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
