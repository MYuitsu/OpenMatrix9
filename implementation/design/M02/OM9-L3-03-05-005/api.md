# OM9-L3-03-05-005 — Selection Highlight Overrides

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-05` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-app/tests/selection_overlay.rs`
- **Positive oracle:** Selected/hover outline vẫn nhìn rõ trong shaded/wireframe
- **Negative oracle:** Object hidden không hiển thị dù selected cũ và mode switch không mất selected IDs
- **Verification:** om9-app selection_overlay: 8 passed; om9-app display_modes: 3 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
