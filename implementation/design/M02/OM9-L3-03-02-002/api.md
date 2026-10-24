# OM9-L3-03-02-002 — Orthographic Right Drag Pan

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-02` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-app/src/navigation.rs`, `crates/om9-scene/src/camera.rs`
- **Positive oracle:** Right drag ở Top dịch camera target, hướng nhìn không đổi
- **Negative oracle:** Drag không làm thay đổi zoom hoặc scene object transform
- **Verification:** om9-app navigation_drag: 3 passed; om9-app viewport_layout: 3 passed; om9-scene camera_grid: 5 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
