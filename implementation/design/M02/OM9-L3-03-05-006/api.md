# OM9-L3-03-05-006 — Display Settings Preview Presets

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-05` (M02)
- **Sources:** `crates/om9-app/src/display.rs`
- **Positive oracle:** Lưu preset display, reload project và restore giá trị wireframe/edges
- **Negative oracle:** Preset lỗi schema dùng fallback và cảnh báo, không crash
- **Verification:** om9-app display cues: 6 passed; om9-app display_modes: 3 passed; om9-app display_persistence: 2 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
