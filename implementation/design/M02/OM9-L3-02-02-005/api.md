# OM9-L3-02-02-005 — Matrix F4 Origin Shortcut

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-02` (M02)
- **Sources:** `crates/om9-app/src/session.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** F4 khi đang hỏi point chèn world origin (0,0,0) vào current prompt
- **Negative oracle:** F4 ngoài point prompt không tạo object hoặc insert chuỗi sai
- **Verification:** om9-app cplane_f4_ui: 1 passed; om9-app cplane_coordinate_input: 10 passed; om9-app chrome_actions: 13 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
