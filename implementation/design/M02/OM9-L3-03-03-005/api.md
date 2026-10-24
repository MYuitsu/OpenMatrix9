# OM9-L3-03-03-005 — Matrix Gumball Alignment And Relocate

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-03` (M02)
- **Sources:** `crates/om9-app/src/session/gumball.rs`, `crates/om9-app/src/ui.rs`, `crates/om9-core/src/gumball.rs`
- **Positive oracle:** World/CPlane/Object alignment chuyển đổi gizmo basis nhưng không sửa BRep
- **Negative oracle:** Gizmo origin relocation ngoài object không làm tự thay đổi object position
- **Verification:** om9-app gumball_placement: 4 passed; om9-app gumball_interaction: 6 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
