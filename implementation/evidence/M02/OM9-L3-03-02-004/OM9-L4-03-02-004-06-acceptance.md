# OM9-L4-03-02-004-06 — Perspective Shift Right Drag Pan acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Shift+right drag Perspective pan target mà không thay đổi rotation
- **Negative oracle:** Không pan trong khi UI popup chiếm pointer focus
- **Acceptance:** Shift+right Perspective drag changes target but preserves eye-target direction and projection.
- **Evidence:** om9-app navigation_drag: 3 passed; om9-app navigation_lifecycle: 8 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
