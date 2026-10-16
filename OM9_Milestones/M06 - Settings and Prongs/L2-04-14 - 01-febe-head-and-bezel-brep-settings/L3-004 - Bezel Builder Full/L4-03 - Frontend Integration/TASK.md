# OM9-M06-L4-14-004-03 — Bezel Builder Full: Tích hợp lệnh, preview, giao diện

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

Gắn command/service vào UI hiện có qua một owner, typed params, responsive preview, Commit/Cancel/Undo, lỗi hiển thị. Không sao chép toán hình học vào UI.

**Target invariant:** bezel/seat NURBS/BRep khép kín, không đâm xuyên quá mức vào gem, theo hình girdle.

## Thứ tự làm, từng bước

1. Use exactly one command owner OM9-L2-04-14; map from Settings/F6/CLI through existing registry.
2. Expose parameters top/base thickness, dome height, bezel angle/height, scale, placement, seat angle/length in typed builder state, preserving the controls described in manual.
3. Perform manual flow: Settings > Bezel Builder; chỉnh thickness, seat, placement; preview/Cancel/Undo.
4. No-op actions must be disabled with reason; all errors visible; preview transient only; Enter commits, Escape cancels.
5. Route edits through shared history journal and GPU preview invalidation; do not run OCCT on frame/render loop.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: `OM9-M06-L4-14-004-02`

## Acceptance / tests

- [ ] Command handler routes to the unique feature owner, not a second implementation
- [ ] Preview/Cancel leaves document checksum unchanged; no lingering preview ID
- [ ] Single Commit ⇒ exactly one undo transaction; Undo/Redo restores same feature parameters/IDs
- [ ] Bad input surfaces typed UI error, no spinner stuck or silent click

## Manual UI

- [ ] Settings > Bezel Builder; chỉnh thickness, seat, placement; preview/Cancel/Undo
- [ ] Expected: preview matches selected gem/curve/surface, changes respond to typed input; Cancel leaves original; Commit/Undo/Redo predictable
- [ ] Expected: object not supported ⇒ disabled with tooltip or explanatory error, not silent no-op

## Lỗi phải xử lý

- top thickness bằng 0; seat vướng pavilion; gem không hỗ trợ; fillet thất bại.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Command/F6 builder is callable from real executable without error
- [ ] Preview/Commit/Cancel/Undo shown by real automated/manual UI evidence
- [ ] FEBE not DONE without named human reviewer

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-14-004/ui-integration-and-transaction.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
