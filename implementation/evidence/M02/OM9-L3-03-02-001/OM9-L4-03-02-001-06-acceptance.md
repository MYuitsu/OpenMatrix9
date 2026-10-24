# OM9-L4-03-02-001-06 — Perspective Right Drag Orbit acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Right drag 100 px trên Perspective đổi hướng camera nhưng không đổi target world và model
- **Negative oracle:** Right drag trên Orthographic không orbit; pointer ngoài viewport không điều khiển view khác
- **Acceptance:** A Perspective right drag changes eye direction but keeps the target and document unchanged; orthographic right drag does not orbit.
- **Evidence:** om9-app navigation_drag: 3 passed; om9-app navigation_lifecycle: 8 passed; om9-scene camera_orbit: 5 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
