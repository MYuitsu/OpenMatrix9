# OM9-L4-03-05-005-06 — Selection Highlight Overrides acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Selected/hover outline vẫn nhìn rõ trong shaded/wireframe
- **Negative oracle:** Object hidden không hiển thị dù selected cũ và mode switch không mất selected IDs
- **Acceptance:** A selected set keeps its outline in both modes and mode switching never drops selected IDs.
- **Evidence:** om9-app selection_overlay: 8 passed; om9-app display_modes: 3 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
