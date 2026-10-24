# OM9-L4-03-06-005-06 — Viewport Reference Asset Persistence acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Save/open project bảo toàn ảnh, quad transform, opacity và calibration
- **Negative oracle:** Missing external image hiển thị warning chứ không mất scene; asset hash mismatch bị chặn
- **Acceptance:** Save/reload preserves image bytes/hash, quad transform, opacity and calibration; missing external images warn; hash mismatch blocks the load.
- **Evidence:** om9-app pictureframe unit: 6 passed; om9-app pictureframe persistence: 3 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
