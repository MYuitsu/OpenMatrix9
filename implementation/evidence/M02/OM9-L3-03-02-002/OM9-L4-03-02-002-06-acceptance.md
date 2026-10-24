# OM9-L4-03-02-002-06 — Orthographic Right Drag Pan acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Right drag ở Top dịch camera target, hướng nhìn không đổi
- **Negative oracle:** Drag không làm thay đổi zoom hoặc scene object transform
- **Acceptance:** A Top right drag changes target while preserving view direction and span; model placement is unchanged.
- **Evidence:** om9-app navigation_drag: 3 passed; om9-app viewport_layout: 3 passed; om9-scene camera_grid: 5 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
