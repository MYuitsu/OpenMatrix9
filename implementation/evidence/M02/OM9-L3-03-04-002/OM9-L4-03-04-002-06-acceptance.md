# OM9-L4-03-04-002-06 — Matrix Ortho Planar Project Toggles acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Ortho/Planar/Project toggle thay mode core và ký hiệu bật đúng
- **Negative oracle:** Ba toggle không vi phạm rule tương thích, không làm đổi active CPlane
- **Acceptance:** Each toggle changes its core flag and indicator; Ortho/Planar/Project remain compatible and the active CPlane is unchanged.
- **Evidence:** om9-app sidebar_palettes toggle: 1 passed; om9-app snap_panel_modes: 4 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
