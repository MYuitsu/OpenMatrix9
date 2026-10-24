# OM9-L3-02-02-003 — CPlane Orientation And Origin Change

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-02` (M02)
- **Sources:** `crates/om9-scene/src/camera.rs`, `crates/om9-app/src/session.rs`, `crates/om9-app/src/viewport_menu.rs`
- **Positive oracle:** Đổi gốc CPlane sang (10,20,0) và quay 30° giữ world geometry không thay đổi
- **Negative oracle:** Đầu vào 3 điểm thẳng hàng hoặc frame singular trả typed error
- **Verification:** om9-scene cplane_presets: 6 passed; om9-app cplane_coordinate_input: 10 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
