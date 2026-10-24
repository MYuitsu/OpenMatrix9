# OM9-L4-03-06-002-06 — Pictureframe Position And Scale acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Move/scale quad theo viewport, bounding rect và anchor nhất quán
- **Negative oracle:** Scale âm/zero nếu không hỗ trợ bị từ chối, không phá aspect ratio bất ngờ
- **Acceptance:** Moving changes origin, uniform scaling keeps aspect, and zero/negative widths are refused without changing the frame.
- **Evidence:** om9-app pictureframe unit: 6 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
