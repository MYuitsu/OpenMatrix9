# OM9-L4-03-06-004-06 — Known Distance Image Calibration acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Hai pixel cách 100px map thành 25mm, scale quad =0.25mm/pixel
- **Negative oracle:** Khoảng pixel=0 hoặc target length<=0 báo lỗi, không có chia zero
- **Acceptance:** 100 px -> 25 mm yields 0.25 mm/pixel; zero pixel distance or target length returns an error without changing the frame.
- **Evidence:** om9-app pictureframe unit: 6 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
