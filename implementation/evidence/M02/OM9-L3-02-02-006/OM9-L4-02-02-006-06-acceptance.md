# OM9-L4-02-02-006-06 — Ortho Planar Project Cursor Constrain acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Ortho giữ hướng trục CPlane; Planar project cho điểm hợp lệ lên active plane
- **Negative oracle:** Bật cả chế độ không tạo điểm 3D lệch khỏi CPlane do thứ tự xử lý
- **Acceptance:** Ortho keeps a CPlane axis direction; Planar lands on the active plane; enabling both still yields a point with local z = 0.
- **Evidence:** om9-app cplane_coordinate_input: 10 passed; om9-app chrome_actions: 13 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
