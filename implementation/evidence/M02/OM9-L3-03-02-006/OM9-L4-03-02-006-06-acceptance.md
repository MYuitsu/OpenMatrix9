# OM9-L4-03-02-006-06 — Viewport Center Reset And Home acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Home phục hồi camera preset rồi lệnh center đưa tâm về world origin
- **Negative oracle:** Không reset CPlane/document layers khi chỉ reset camera
- **Acceptance:** Home restores the preset, Center moves target to world origin preserving rotation, and CPlane/layers/document remain unchanged.
- **Evidence:** om9-app navigation_commands: 6 passed; om9-scene camera_fit: 4 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
