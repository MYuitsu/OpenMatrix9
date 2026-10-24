# OM9-L3-03-03-001 — Matrix Click Shift Ctrl Select

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-03` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-app/src/session/layers.rs`
- **Positive oracle:** Shift click thêm, Ctrl click toggle selection, plain click replace theo mapping đã chốt
- **Negative oracle:** Click đối tượng locked không tạo ID vào selection
- **Verification:** om9-app matrix_selection_sequence: 2 passed; om9-app acceptance_viewport_selection: 8 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
