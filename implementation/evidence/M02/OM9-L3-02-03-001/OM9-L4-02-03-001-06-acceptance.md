# OM9-L4-02-03-001-06 — Osnap End Mid Center Quadrant acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Polyline 0-10mm snap END (10,0), MID (5,0); Circle tâm (5,5), radius 2 snap QUAD
- **Negative oracle:** Candidate nằm ngoài screen radius không được chọn; radius 0 trả none
- **Acceptance:** A 0-10 line yields END (10,0) and MID (5,0); a circle at (5,5) radius 2 yields QUAD (7,5); candidates outside the radius are omitted.
- **Evidence:** om9-scene osnap: 11 passed; om9-app osnap UI: 3 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
