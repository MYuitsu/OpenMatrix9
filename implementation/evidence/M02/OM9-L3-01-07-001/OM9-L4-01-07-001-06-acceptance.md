# OM9-L4-01-07-001-06 — Screen Ray And Hit Test Query acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Box tại tâm tia, điểm click ở viewport P trúng BodyId của box
- **Negative oracle:** Ray không giao đối tượng trả danh sách rỗng, không chọn
- **Acceptance:** A ray through the box returns its document BodyId; an empty ray returns none; source-bound native BRep faces/edges/vertices pick by UUID topology rather than mesh offsets.
- **Evidence:** om9-scene acceptance_l2_01_07: 9 passed; om9-scene world_picking: 7 passed; om9-scene navigation_snapshot: 2 passed; om9-app brep_selection (gpu-sphere): 5 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
