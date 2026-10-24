# OM9-L3-01-07-008 — Drag Transaction And Commit Cancel

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-01-07` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-core/src/gumball.rs`, `crates/om9-core/src/transaction.rs`
- **Positive oracle:** Kéo box +5 mm rồi release tạo đúng một undo; Escape trước release giữ document cũ
- **Negative oracle:** Lỗi kernel, mất selection giữa kéo hoặc Esc không được tạo journal entry
- **Verification:** om9-app gumball_interaction: 5 passed; om9-app gumball_numeric: 5 passed; om9-app gumball_cold: 1 passed; om9-app selection_overlay: 8 passed; om9-scene affine_gesture: 5 passed; om9-core gumball: 15 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
