# OM9-L3-03-04-001 — Matrix Snap Toggle Panel

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-04` (M02)
- **Sources:** `crates/om9-app/src/ui/sidebar_palettes.rs`, `crates/om9-app/src/ui/chrome.rs`, `crates/om9-app/src/ui.rs`, `crates/om9-scene/src/osnap.rs`
- **Positive oracle:** Bật END ở panel và core snap state nhận enabled=true, biểu tượng đồng bộ
- **Negative oracle:** Tắt all snap thì marker biến mất; panel không tạo solver riêng
- **Verification:** om9-app sidebar_palettes toggle: 1 passed; om9-app osnap_ui: 3 passed; om9-app snap_panel_modes: 4 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
