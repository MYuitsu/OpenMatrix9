# OM9-L3-02-02-002 — CPlane Preset Worldtop Front Right

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-02` (M02)
- **Sources:** `crates/om9-scene/src/camera.rs`, `crates/om9-scene/src/workspace.rs`, `crates/om9-app/src/viewport_menu.rs`, `crates/om9-app/src/session.rs`
- **Positive oracle:** Top/Front/Right preset có origin đúng và basis right-handed
- **Negative oracle:** Preset có norm trục zero/không trực giao phải bị chặn
- **Verification:** om9-scene cplane_presets: 6 passed; om9-app cplane_menu: 1 passed; om9-app viewport_title_menu: 17 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
