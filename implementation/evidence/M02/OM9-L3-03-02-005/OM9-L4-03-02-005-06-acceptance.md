# OM9-L4-03-02-005-06 — Zoom Extents And Selected acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Fit 10×20×30 mm box vào viewport với margin, ZoomSelected chỉ fit selected IDs
- **Negative oracle:** Scene trống hoặc bbox zero không sinh scale vô hạn
- **Acceptance:** A 10x20x30 box fits inside the viewport with margin; ZoomSelected ignores far unselected geometry; empty scene is a typed error and a zero-size bound remains finite.
- **Evidence:** om9-app navigation_commands: 6 passed; om9-scene camera_fit: 4 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
