# OM9-M06-L4-14-004-02 — Bezel Builder Full: Hiện thực lõi không giao diện

**Owner:** `OM9-L2-04-14` · **L3:** `OM9-L3-04-14-004` · **capability:** `cap.04-14.bezel-builder-full` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/01-febe-head-and-bezel-brep-settings/004-febe-bezel-builder-full/l3.json`.
- Matrix 8 Book 2: printed 475/pdf 249 (section_heading); printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [249, 250, 251].
- Sách 2 trang in 475–477, Bezel Builder / Bezel Mode.
- Input: một gem đã đặt; reference girdle/seat; bezel style.
- Controls: top/base thickness, dome height, bezel angle/height, scale, placement, seat angle/length.

## Phạm vi đúng 1 thay đổi

Triển khai một đơn vị tính toán/domain cho capability với OCCT qua bridge của dự án; test geometry/logic, đầu vào xấu và lỗi nguyên tử. Không chứa menu/F6 hay renderer.

**Target invariant:** bezel/seat NURBS/BRep khép kín, không đâm xuyên quá mức vào gem, theo hình girdle.

## Thứ tự làm, từng bước

1. Review reviewed phase-01 schema and prior L4 dependency evidence; verify toolchain/kernel API actually present.
2. Implement only Full Bezel computation with typed parameters (top/base thickness, dome height, bezel angle/height, scale, placement, seat angle/length).
3. Ensure output: bezel/seat NURBS/BRep khép kín, không đâm xuyên quá mức vào gem, theo hình girdle.
4. Run positive/negative/update fixtures incl. top thickness bằng 0; seat vướng pavilion; gem không hỗ trợ; fillet thất bại; commit output atomically and error cleanly.
5. Log OCCT validity/topology/units and golden numeric comparisons using G00-approved thresholds, not screenshot equivalence.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: `OM9-M06-L4-14-004-01`, `OM9-M06-L4-14-008-02`, `OM9-M06-L4-14-009-01`

## Acceptance / tests

- [ ] VALID: kiểm mặt cắt tại seat, thickness positivity, BRep validity và mm conversion
- [ ] INVALID: top thickness bằng 0; seat vướng pavilion; gem không hỗ trợ; fillet thất bại ⇒ typed failure, no mutation
- [ ] For BRep outputs: IsValid, orientation/open-edge counts; for curve/library outputs: curve validity, continuity and referential integrity; G00 tolerance
- [ ] Rebuild same params twice ⇒ deterministic geometry measurements and stable references
- [ ] Measure no errors/leaks under repeated construct/dispose in existing FFI test harness

## Manual UI

Không nghiệm thu GUI ở task contract/BE này; task FEBE/QA sẽ kiểm tra sau.

## Lỗi phải xử lý

- top thickness bằng 0; seat vướng pavilion; gem không hỗ trợ; fillet thất bại.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Unit/geometry tests recorded real PASS on installed OCCT/toolchain
- [ ] Real BRep/typed curves produced (not solely triangulation/display screenshot)
- [ ] No failures leak handles or leave document geometry dirty

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-14-004/geometry-and-unit-test.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
