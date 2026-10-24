# OM9-L3-03-05-001 — Wireframe Shaded Mode Switcher

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-05` (M02)
- **Sources:** `crates/om9-app/src/viewport_menu.rs`, `crates/om9-app/src/session.rs`, `crates/om9-scene/src/workspace.rs`
- **Positive oracle:** Box BRep hiển thị wireframe và shaded cùng bbox, không mutate model
- **Negative oracle:** Chuyển mode không thay đổi document dirty/undo journal
- **Verification:** om9-app display_modes: 3 passed; om9-app viewport_title_menu: 18 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
