# OM9-L3-03-06-005 — Viewport Reference Asset Persistence

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-06` (M02)
- **Sources:** `crates/om9-app/src/pictureframe.rs`, `crates/om9-app/src/session/pictureframe_state.rs`
- **Positive oracle:** Save/open project bảo toàn ảnh, quad transform, opacity và calibration
- **Negative oracle:** Missing external image hiển thị warning chứ không mất scene; asset hash mismatch bị chặn
- **Verification:** om9-app pictureframe unit: 6 passed; om9-app pictureframe persistence: 3 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
