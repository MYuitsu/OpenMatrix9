# OM9-L4-03-03-002-06 — Matrix Left Right Drag Selection acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Kéo trái→phải window và phải→trái crossing vẽ đúng rectangle, trả set tương ứng
- **Negative oracle:** Drag bắt đầu trên UI dock không phát selection marquee
- **Acceptance:** Window selects fully contained objects and crossing selects intersecting objects; a drag starting on the dock produces no marquee.
- **Evidence:** om9-app rubber_band: 2 passed; om9-app selection_overlay: 8 passed; om9-app acceptance_viewport_selection: 8 passed; om9-scene selection_regressions: 7 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
