# OM9-L4-03-05-004-06 — Surface Shading And Backface Cues acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Front/back side mặt BRep có visual cue khác nhau theo normal
- **Negative oracle:** Negative scale hoặc flip surface không đảo cue sai; không phá face IDs
- **Acceptance:** Front and back faces produce different cues; a negative determinant flips the parity correctly without changing an ID.
- **Evidence:** om9-app display cues: 6 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
