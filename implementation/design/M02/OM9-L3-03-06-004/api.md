# OM9-L3-03-06-004 — Known Distance Image Calibration

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-06` (M02)
- **Sources:** `crates/om9-app/src/pictureframe.rs`
- **Positive oracle:** Hai pixel cách 100px map thành 25mm, scale quad =0.25mm/pixel
- **Negative oracle:** Khoảng pixel=0 hoặc target length<=0 báo lỗi, không có chia zero
- **Verification:** om9-app pictureframe unit: 6 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
