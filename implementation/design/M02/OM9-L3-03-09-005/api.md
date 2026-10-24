# OM9-L3-03-09-005 — F6 Custom User Mode

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-09` (M02)
- **Sources:** `crates/om9-app/src/f6.rs`, `crates/om9-app/src/session/f6_state.rs`
- **Positive oracle:** User custom menu lưu command IDs và reload với thứ tự giống cũ
- **Negative oracle:** Command ID đã xoá thì menu entry disabled thay vì dispatch sai
- **Verification:** om9-app f6 unit: 7 passed; om9-app f6_context: 2 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
