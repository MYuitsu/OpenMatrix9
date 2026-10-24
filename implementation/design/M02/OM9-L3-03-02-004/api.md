# OM9-L3-03-02-004 — Perspective Shift Right Drag Pan

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-02` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-app/src/navigation.rs`
- **Positive oracle:** Shift+right drag Perspective pan target mà không thay đổi rotation
- **Negative oracle:** Không pan trong khi UI popup chiếm pointer focus
- **Verification:** om9-app navigation_drag: 3 passed; om9-app navigation_lifecycle: 8 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
