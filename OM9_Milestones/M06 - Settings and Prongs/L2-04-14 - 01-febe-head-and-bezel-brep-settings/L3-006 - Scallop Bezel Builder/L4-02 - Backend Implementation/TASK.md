# OM9-M06-L4-14-006-02 — Scallop Bezel Builder: Hiện thực lõi không giao diện

**Owner:** `OM9-L2-04-14` · **L3:** `OM9-L3-04-14-006` · **capability:** `cap.04-14.scallop-bezel-builder` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/01-febe-head-and-bezel-brep-settings/006-febe-scallop-bezel-builder/l3.json`.
- Matrix 8 Book 2: printed 481/pdf 255 (section_heading); printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [255, 256, 257].
- Sách 2 trang in 481–483, Scallop Bezel / cutter.
- Input: bezel BRep hợp lệ; gem; scallop/cutter profile tùy chọn.
- Controls: scallop cutter type, count, pattern angle, offset theo hướng dẫn.

## Phạm vi đúng 1 thay đổi

Triển khai một đơn vị tính toán/domain cho capability với OCCT qua bridge của dự án; test geometry/logic, đầu vào xấu và lỗi nguyên tử. Không chứa menu/F6 hay renderer.

**Target invariant:** scalloped edges chỉ cắt phần kim loại yêu cầu, không sinh mặt lỗi.

## Thứ tự làm, từng bước

1. Review reviewed phase-01 schema and prior L4 dependency evidence; verify toolchain/kernel API actually present.
2. Implement only Scallop Bezel computation with typed parameters (scallop cutter type, count, pattern angle, offset theo hướng dẫn).
3. Ensure output: scalloped edges chỉ cắt phần kim loại yêu cầu, không sinh mặt lỗi.
4. Run positive/negative/update fixtures incl. cutter quá sâu; count không khả dụng; boolean fail; zero thickness; commit output atomically and error cleanly.
5. Log OCCT validity/topology/units and golden numeric comparisons using G00-approved thresholds, not screenshot equivalence.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: `OM9-M06-L4-14-006-01`, `OM9-M06-L4-14-004-02`

## Acceptance / tests

- [ ] VALID: BRep difference vs reference, manifold checks, rollback nguyên tử khi boolean fail
- [ ] INVALID: cutter quá sâu; count không khả dụng; boolean fail; zero thickness ⇒ typed failure, no mutation
- [ ] For BRep outputs: IsValid, orientation/open-edge counts; for curve/library outputs: curve validity, continuity and referential integrity; G00 tolerance
- [ ] Rebuild same params twice ⇒ deterministic geometry measurements and stable references
- [ ] Measure no errors/leaks under repeated construct/dispose in existing FFI test harness

## Manual UI

Không nghiệm thu GUI ở task contract/BE này; task FEBE/QA sẽ kiểm tra sau.

## Lỗi phải xử lý

- cutter quá sâu; count không khả dụng; boolean fail; zero thickness.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Unit/geometry tests recorded real PASS on installed OCCT/toolchain
- [ ] Real BRep/typed curves produced (not solely triangulation/display screenshot)
- [ ] No failures leak handles or leave document geometry dirty

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-14-006/geometry-and-unit-test.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
