# OM9-L3-03-02-003 — Wheel Zoom To Cursor

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-02` (M02)
- **Sources:** `crates/om9-app/src/navigation.rs`, `crates/om9-app/src/ui.rs`, `crates/om9-scene/src/camera.rs`
- **Positive oracle:** Wheel zoom tại pixel (x,y) giữ điểm world dưới cursor tại cùng pixel sau zoom
- **Negative oracle:** Zoom factor 0/NaN/negative hoặc ở ngoài view được từ chối
- **Verification:** om9-app navigation_input: 1 passed; om9-app navigation_precision: 3 passed; om9-scene camera_fit: 4 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
