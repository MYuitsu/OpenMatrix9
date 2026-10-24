# OM9-L4-03-04-001-06 — Matrix Snap Toggle Panel acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Bật END ở panel và core snap state nhận enabled=true, biểu tượng đồng bộ
- **Negative oracle:** Tắt all snap thì marker biến mất; panel không tạo solver riêng
- **Acceptance:** Clicking END in the palette changes the core enabled bit and icon state; turning snaps off makes the marker disappear.
- **Evidence:** om9-app sidebar_palettes toggle: 1 passed; om9-app osnap_ui: 3 passed; om9-app snap_panel_modes: 4 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
