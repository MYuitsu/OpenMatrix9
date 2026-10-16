# OM9-M06-L4-15-007-03 — Bead On Surface: Tích hợp lệnh, preview, giao diện

**Owner:** `OM9-L2-04-15` · **L3:** `OM9-L3-04-15-007` · **capability:** `cap.04-15.bead-on-surface` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/02-febe-prong-channel-metal-and-bead-settings/007-febe-bead-on-surface/l3.json`.
- Matrix 8 Book 2: printed 489/pdf 263 (section_heading); printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [263, 264].
- Sách 2 trang in 489–490, Bead on Surface.
- Input: surface và click/pick UV position; sphere hoặc reusable object.
- Controls: scale, vertical offset, orientation, flip, reference curve.

## Phạm vi đúng 1 thay đổi

Gắn command/service vào UI hiện có qua một owner, typed params, responsive preview, Commit/Cancel/Undo, lỗi hiển thị. Không sao chép toán hình học vào UI.

**Target invariant:** instance center normal offset measured in mm, coherent orientation over curved faces.

## Thứ tự làm, từng bước

1. Use exactly one command owner OM9-L2-04-15; map from Settings/F6/CLI through existing registry.
2. Expose parameters scale, vertical offset, orientation, flip, reference curve in typed builder state, preserving the controls described in manual.
3. Perform manual flow: F6 từ surface chọn Bead on Surface, click placement, Flip/Scale/Cancel.
4. No-op actions must be disabled with reason; all errors visible; preview transient only; Enter commits, Escape cancels.
5. Route edits through shared history journal and GPU preview invalidation; do not run OCCT on frame/render loop.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: `OM9-M06-L4-15-007-02`

## Acceptance / tests

- [ ] Command handler routes to the unique feature owner, not a second implementation
- [ ] Preview/Cancel leaves document checksum unchanged; no lingering preview ID
- [ ] Single Commit ⇒ exactly one undo transaction; Undo/Redo restores same feature parameters/IDs
- [ ] Bad input surfaces typed UI error, no spinner stuck or silent click

## Manual UI

- [ ] F6 từ surface chọn Bead on Surface, click placement, Flip/Scale/Cancel
- [ ] Expected: preview matches selected gem/curve/surface, changes respond to typed input; Cancel leaves original; Commit/Undo/Redo predictable
- [ ] Expected: object not supported ⇒ disabled with tooltip or explanatory error, not silent no-op

## Lỗi phải xử lý

- hit outside surface; bad normal; unsupported selected object; duplicate bead.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Command/F6 builder is callable from real executable without error
- [ ] Preview/Commit/Cancel/Undo shown by real automated/manual UI evidence
- [ ] FEBE not DONE without named human reviewer

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-15-007/ui-integration-and-transaction.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
