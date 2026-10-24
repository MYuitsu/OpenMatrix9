# OM9-L3-03-04-005 — Matrix Snap Shortcut Modes

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-04` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-scene/src/osnap.rs`
- **Positive oracle:** Shortcut đổi END/MID theo registry, trạng thái panel cập nhật cùng frame
- **Negative oracle:** Shortcut khi focus text editor không kích hoạt snap và không gõ ký tự lạ
- **Verification:** om9-app snap_panel_modes: 4 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
