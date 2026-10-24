# OM9-L3-03-04-004 — Matrix Smarttrack Toggle And Feedback

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-04` (M02)
- **Sources:** `crates/om9-scene/src/osnap.rs`, `crates/om9-app/src/ui.rs`, `crates/om9-app/src/ui/sidebar_palettes.rs`
- **Positive oracle:** SmartTrack bật/ tắt đồng bộ guides trên tất cả view liên quan
- **Negative oracle:** Không còn anchors thì không còn ghost guides
- **Verification:** om9-app snap_panel_modes: 4 passed; om9-app osnap_ui: 3 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
