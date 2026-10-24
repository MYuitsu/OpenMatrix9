# OM9-L3-03-05-002 — Gem View Material Color Cues

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-05` (M02)
- **Sources:** `crates/om9-app/src/display.rs`, `crates/om9-core/src/model.rs`
- **Positive oracle:** Gem-like semantic material cue có token màu nếu semantic provider tồn tại
- **Negative oracle:** Không có M05 Gem records thì hiển thị neutral placeholder và ghi DEFERRED_WAIT_M05
- **Verification:** om9-app display cues: 6 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
