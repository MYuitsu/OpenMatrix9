# OM9-L4-02-03-004-06 — Osnap Surface And Polysurface acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Đối với mặt phẳng BRep, nearest point nằm trên trimmed face và normal đúng
- **Negative oracle:** Điểm gần ngoài trim/face invalid không snap lên phần mặt bị cắt
- **Acceptance:** A ray through a planar box face returns an on-face point and the face normal; a ray outside the trimmed face produces no surface snap.
- **Evidence:** om9-app osnap BRep: 1 passed; om9-scene osnap: 11 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
