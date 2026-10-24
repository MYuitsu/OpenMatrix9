# OM9-L3-03-05-004 — Surface Shading And Backface Cues

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-05` (M02)
- **Sources:** `crates/om9-app/src/display.rs`
- **Positive oracle:** Front/back side mặt BRep có visual cue khác nhau theo normal
- **Negative oracle:** Negative scale hoặc flip surface không đảo cue sai; không phá face IDs
- **Verification:** om9-app display cues: 6 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
