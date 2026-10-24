# OM9-L3-03-04-003 — Matrix Snap Activation And Indicator

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-04` (M02)
- **Sources:** `crates/om9-scene/src/osnap.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** One-shot override bật rồi dùng đúng một pick và indicator tự tắt
- **Negative oracle:** Escape prompt phải dọn indicator, không lưu override sang project khác
- **Verification:** om9-app snap_panel_modes: 4 passed; om9-app osnap_ui: 3 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
