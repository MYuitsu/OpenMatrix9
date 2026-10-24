# OM9-L4-03-05-001-06 — Wireframe Shaded Mode Switcher acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Box BRep hiển thị wireframe và shaded cùng bbox, không mutate model
- **Negative oracle:** Chuyển mode không thay đổi document dirty/undo journal
- **Acceptance:** A 10x20x30 box keeps the same scene bbox in both modes; document and journal are unchanged.
- **Evidence:** om9-app display_modes: 3 passed; om9-app viewport_title_menu: 18 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
