# OM9-L3-03-02-001 — Perspective Right Drag Orbit

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-02` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-app/src/navigation.rs`, `crates/om9-scene/src/camera.rs`
- **Positive oracle:** Right drag 100 px trên Perspective đổi hướng camera nhưng không đổi target world và model
- **Negative oracle:** Right drag trên Orthographic không orbit; pointer ngoài viewport không điều khiển view khác
- **Verification:** om9-app navigation_drag: 3 passed; om9-app navigation_lifecycle: 8 passed; om9-scene camera_orbit: 5 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
