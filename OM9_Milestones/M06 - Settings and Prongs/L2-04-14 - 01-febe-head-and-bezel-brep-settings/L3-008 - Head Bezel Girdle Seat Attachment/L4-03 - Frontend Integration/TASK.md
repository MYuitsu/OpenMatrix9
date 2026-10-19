# OM9-M06-L4-14-008-03 — Head Bezel Girdle Seat Attachment: Tích hợp lệnh, preview, giao diện

**Owner:** `OM9-L2-04-14` · **L3:** `OM9-L3-04-14-008` · **capability:** `cap.04-14.head-bezel-girdle-seat-attachment` · **E11/M06**
**Stack:** Rust + OCCT qua FFI đã được audit; đọc `planning/M06/M06_IMPLEMENTATION_CONTRACT.md`.
**State:** `PLANNED_NOT_EXECUTED`; G00/E10 chưa được chứng minh PASS. Không có source implementation được xác nhận từ ZIP.

## Đầu vào và nguồn

- Canonical spec: `specs/06-ms06-settings-and-prongs/01-febe-head-and-bezel-brep-settings/008-febe-head-bezel-girdle-seat-attachment/l3.json`.
- Matrix 8 Book 2: printed [468, 497]/pdf [242, 271] (parent_l2_range); target PDF pages [243, 249, 250].
- Yêu cầu L3 liên ngành, suy ra từ Head/Bezel placement; không có trang manual riêng.
- Input: gem geometry và stable gem ID từ E10; curve girdle; local frames.
- Controls: girdle parameterization, seat frame, normal, offset; tolerance từ G00.

## Phạm vi đúng 1 thay đổi

Gắn command/service vào UI hiện có qua một owner, typed params, responsive preview, Commit/Cancel/Undo, lỗi hiển thị. Không sao chép toán hình học vào UI.

**Target invariant:** transform chuẩn từ gem local → world; attachment không đổi đơn vị/normal khi cập nhật.

## Thứ tự làm, từng bước

1. Use exactly one command owner OM9-L2-04-14; map from Settings/F6/CLI through existing registry.
2. Expose parameters girdle parameterization, seat frame, normal, offset; tolerance từ G00 in typed builder state, preserving the controls described in manual.
3. Perform manual flow: sửa vị trí hoặc xoay gem; head/bezel preview bám girdle, cảnh báo khi detach.
4. No-op actions must be disabled with reason; all errors visible; preview transient only; Enter commits, Escape cancels.
5. Route edits through shared history journal and GPU preview invalidation; do not run OCCT on frame/render loop.

## Dependencies

- Gate ngoài: G00_PASS + E10_PASS + frozen gem input contract.
- L4 nội bộ: `OM9-M06-L4-14-008-02`

## Acceptance / tests

- [ ] Command handler routes to the unique feature owner, not a second implementation
- [ ] Preview/Cancel leaves document checksum unchanged; no lingering preview ID
- [ ] Single Commit ⇒ exactly one undo transaction; Undo/Redo restores same feature parameters/IDs
- [ ] Bad input surfaces typed UI error, no spinner stuck or silent click

## Manual UI

- [ ] sửa vị trí hoặc xoay gem; head/bezel preview bám girdle, cảnh báo khi detach
- [ ] Expected: preview matches selected gem/curve/surface, changes respond to typed input; Cancel leaves original; Commit/Undo/Redo predictable
- [ ] Expected: object not supported ⇒ disabled with tooltip or explanatory error, not silent no-op

## Lỗi phải xử lý

- missing gem ID; nonmanifold gem; singular tangent; sai orientation.
- Panic/kernel failure/Cancel phải rollback, không thay đổi model/ID/scene một phần.

## Bằng chứng và Done

- [ ] Command/F6 builder is callable from real executable without error
- [ ] Preview/Commit/Cancel/Undo shown by real automated/manual UI evidence
- [ ] FEBE not DONE without named human reviewer

**Evidence paths (planned, not created):** `implementation/evidence/M06/OM9-L3-04-14-008/ui-integration-and-transaction.json`

**Khi thiếu điều kiện:** ghi `BLOCKED_SOURCE`, `BLOCKED_GATE`, `BLOCKED_REFERENCE` hoặc `BLOCKED_TOOLCHAIN`, lưu thông tin có thể tái hiện; không viết mock/stub để tuyên bố PASS.

L4 metadata chi tiết: [l4.json](l4.json).
