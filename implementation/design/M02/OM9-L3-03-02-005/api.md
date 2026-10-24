# OM9-L3-03-02-005 — Zoom Extents And Selected

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-02` (M02)
- **Sources:** `crates/om9-app/src/session.rs`, `crates/om9-app/src/ui.rs`, `crates/om9-scene/src/camera.rs`
- **Positive oracle:** Fit 10×20×30 mm box vào viewport với margin, ZoomSelected chỉ fit selected IDs
- **Negative oracle:** Scene trống hoặc bbox zero không sinh scale vô hạn
- **Verification:** om9-app navigation_commands: 6 passed; om9-scene camera_fit: 4 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
