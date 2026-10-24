# OM9-L3-03-05-003 — Curve And Cutter Visibility Presets

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-05` (M02)
- **Sources:** `crates/om9-app/src/display.rs`, `crates/om9-core/src/model.rs`
- **Positive oracle:** Ẩn working curves/cutters theo category tags và giữ mesh khác visible
- **Negative oracle:** Chưa có cutter M07 thì registry category disabled, không bỏ ẩn model
- **Verification:** om9-app display cues: 6 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
