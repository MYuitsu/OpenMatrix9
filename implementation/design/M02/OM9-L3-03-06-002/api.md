# OM9-L3-03-06-002 — Pictureframe Position And Scale

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-06` (M02)
- **Sources:** `crates/om9-app/src/pictureframe.rs`
- **Positive oracle:** Move/scale quad theo viewport, bounding rect và anchor nhất quán
- **Negative oracle:** Scale âm/zero nếu không hỗ trợ bị từ chối, không phá aspect ratio bất ngờ
- **Verification:** om9-app pictureframe unit: 6 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
