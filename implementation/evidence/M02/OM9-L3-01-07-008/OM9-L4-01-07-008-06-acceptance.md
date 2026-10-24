# OM9-L4-01-07-008-06 — Drag Transaction And Commit Cancel acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Kéo box +5 mm rồi release tạo đúng một undo; Escape trước release giữ document cũ
- **Negative oracle:** Lỗi kernel, mất selection giữa kéo hoặc Esc không được tạo journal entry
- **Acceptance:** A +5 mm drag and release creates exactly one Undo entry; Escape before release preserves the old document and creates no journal entry.
- **Evidence:** om9-app gumball_interaction: 5 passed; om9-app gumball_numeric: 5 passed; om9-app gumball_cold: 1 passed; om9-app selection_overlay: 8 passed; om9-scene affine_gesture: 5 passed; om9-core gumball: 15 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
