# OM9-M06-L4-14-009-02 — Head Bezel Geometry Validity And Update: Hiện thực lõi không giao diện

**Owner:** `OM9-L2-04-14` · **L3:** `OM9-L3-04-14-009` · **capability:** `cap.04-14.head-bezel-geometry-validity-and-update` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/01-febe-head-and-bezel-brep-settings/009-febe-head-bezel-geometry-validity-and-update/l3.json`.
- Matrix 8 Book 2: printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [243, 249, 251].
- Quy tắc M06/E11 + phần manual về Head/Bezel; không có heading riêng.
- Input: setting BRep/feature và gem dependency graph; journal/transactions có sẵn.
- Controls: validity report, associative recompute, replace vs preserve geometry.

## Phạm vi đúng 1 thay đổi

Triển khai một đơn vị tính toán/domain cho capability với OCCT qua bridge của dự án; test geometry/logic, đầu vào xấu và lỗi nguyên tử. Không chứa menu/F6 hay renderer.

**Target invariant:** IsValid, closedness theo loại, stable IDs, rebuild deterministic, no phantom references.

## Thứ tự làm, từng bước

1. Review reviewed phase-01 schema and prior L4 dependency evidence; verify toolchain/kernel API actually present.
2. Implement only Head/Bezel validity & update computation with typed parameters (validity report, associative recompute, replace vs preserve geometry).
3. Ensure output: IsValid, closedness theo loại, stable IDs, rebuild deterministic, no phantom references.
4. Run positive/negative/update fixtures incl. OCCT failure; stale feature links; repeated rebuild; circular history graph; commit output atomically and error cleanly.
5. Log OCCT validity/topology/units and golden numeric comparisons using G00-approved thresholds, not screenshot equivalence.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: `OM9-M06-L4-14-009-01`

## Acceptance / tests

- [ ] VALID: gem resize → rebuild; invalid params → original doc intact; save/open/Undo/Redo diff
- [ ] INVALID: OCCT failure; stale feature links; repeated rebuild; circular history graph ⇒ typed failure, no mutation
- [ ] For BRep outputs: IsValid, orientation/open-edge counts; for curve/library outputs: curve validity, continuity and referential integrity; G00 tolerance
- [ ] Rebuild same params twice ⇒ deterministic geometry measurements and stable references
- [ ] Measure no errors/leaks under repeated construct/dispose in existing FFI test harness

## Manual UI

Không nghiệm thu GUI ở task contract/BE này; task FEBE/QA sẽ kiểm tra sau.

## Lỗi phải xử lý

- OCCT failure; stale feature links; repeated rebuild; circular history graph.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Unit/geometry tests recorded real PASS on installed OCCT/toolchain
- [ ] Real BRep/typed curves produced (not solely triangulation/display screenshot)
- [ ] No failures leak handles or leave document geometry dirty

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-14-009/geometry-and-unit-test.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
