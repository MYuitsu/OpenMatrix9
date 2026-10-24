# OM9-L4-03-03-004-06 — Matrix Gumball Axis Drag acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Gumball kéo trục X +5mm từ (0,0,0) và Undo/Redo đúng một lần
- **Negative oracle:** Kéo cancel giữ transform, snapshot, journal unchanged
- **Acceptance:** X axis +5 mm from (0,0,0) produces exactly +5 mm and one Undo/Redo entry; cancel leaves transform and journal unchanged.
- **Evidence:** om9-app gumball_interaction: 6 passed; om9-app gumball_viewport: 3 passed; om9-core gumball: 15 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
