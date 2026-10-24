# OM9-L3-03-04-002 — Matrix Ortho Planar Project Toggles

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-04` (M02)
- **Sources:** `crates/om9-app/src/session.rs`, `crates/om9-app/src/ui/sidebar_palettes.rs`, `crates/om9-app/src/ui/chrome.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Ortho/Planar/Project toggle thay mode core và ký hiệu bật đúng
- **Negative oracle:** Ba toggle không vi phạm rule tương thích, không làm đổi active CPlane
- **Verification:** om9-app sidebar_palettes toggle: 1 passed; om9-app snap_panel_modes: 4 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
