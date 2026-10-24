# OM9-L4-01-07-007-06 — Generic Translation Rotation Scale Handles acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Gizmo Move/Rotate/Scale xuất hiện theo bounding box, trục đúng world axes
- **Negative oracle:** Object không hợp lệ/locked không tạo handle dùng được
- **Acceptance:** Move/rotate/scale handles follow the selection's bounding frame on the correct world/object axes; an invalid or locked object cannot be selected and therefore produces no usable handle.
- **Evidence:** om9-scene lib gumball_view: 13 passed; om9-app gumball_viewport: 3 passed; om9-app gumball_placement: 4 passed; om9-app gumball_interaction: 5 passed; om9-app gumball_numeric: 5 passed; om9-core gumball: 15 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
