# OM9-M06-L4-15-005-02 — Channel Border Curve: Hiện thực lõi không giao diện

**Owner:** `OM9-L2-04-15` · **L3:** `OM9-L3-04-15-005` · **capability:** `cap.04-15.channel-border-curve` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/02-febe-prong-channel-metal-and-bead-settings/005-febe-channel-border-curve/l3.json`.
- Matrix 8 Book 2: printed 496/pdf 270 (section_heading); printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [270, 271].
- Sách 2 trang in 496–497, Channel Border Curve.
- Input: ordered gem line; offset and end parameters.
- Controls: spacing, end offset, vertical offset, corner smoothing, extension Line/Arc/Smooth.

## Phạm vi đúng 1 thay đổi

Triển khai một đơn vị tính toán/domain cho capability với OCCT qua bridge của dự án; test geometry/logic, đầu vào xấu và lỗi nguyên tử. Không chứa menu/F6 hay renderer.

**Target invariant:** guide curve là curve có topology hợp lệ, không giả vờ là solid; respects end and side offsets.

## Thứ tự làm, từng bước

1. Review reviewed phase-01 schema and prior L4 dependency evidence; verify toolchain/kernel API actually present.
2. Implement only Channel Border Curve computation with typed parameters (spacing, end offset, vertical offset, corner smoothing, extension Line/Arc/Smooth).
3. Ensure output: guide curve là curve có topology hợp lệ, không giả vờ là solid; respects end and side offsets.
4. Run positive/negative/update fixtures incl. sharp corner gây self-intersection; gem row không liên tục; tangent undefined; commit output atomically and error cleanly.
5. Log OCCT validity/topology/units and golden numeric comparisons using G00-approved thresholds, not screenshot equivalence.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: `OM9-M06-L4-15-005-01`

## Acceptance / tests

- [ ] VALID: golden curves for straight/curved/closed rows; corners no pinch, stable orientations
- [ ] INVALID: sharp corner gây self-intersection; gem row không liên tục; tangent undefined ⇒ typed failure, no mutation
- [ ] For BRep outputs: IsValid, orientation/open-edge counts; for curve/library outputs: curve validity, continuity and referential integrity; G00 tolerance
- [ ] Rebuild same params twice ⇒ deterministic geometry measurements and stable references
- [ ] Measure no errors/leaks under repeated construct/dispose in existing FFI test harness

## Manual UI

Không nghiệm thu GUI ở task contract/BE này; task FEBE/QA sẽ kiểm tra sau.

## Lỗi phải xử lý

- sharp corner gây self-intersection; gem row không liên tục; tangent undefined.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Unit/geometry tests recorded real PASS on installed OCCT/toolchain
- [ ] Real BRep/typed curves produced (not solely triangulation/display screenshot)
- [ ] No failures leak handles or leave document geometry dirty

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-15-005/geometry-and-unit-test.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
