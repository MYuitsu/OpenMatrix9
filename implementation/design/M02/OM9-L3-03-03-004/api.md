# OM9-L3-03-03-004 — Matrix Gumball Axis Drag

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-03` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-app/src/session/gumball.rs`, `crates/om9-core/src/gumball.rs`
- **Positive oracle:** Gumball kéo trục X +5mm từ (0,0,0) và Undo/Redo đúng một lần
- **Negative oracle:** Kéo cancel giữ transform, snapshot, journal unchanged
- **Verification:** om9-app gumball_interaction: 6 passed; om9-app gumball_viewport: 3 passed; om9-core gumball: 15 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
