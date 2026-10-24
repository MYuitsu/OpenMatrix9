# OM9-L4-02-03-002-06 — Osnap Near Point Intersection acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Hai đoạn cắt ở (5,0) trả INTERSECTION; NEAR trả closest point có tham số t hợp lệ
- **Negative oracle:** Hai đoạn song song không trả giao điểm giả; point ngoài curve được clamp
- **Acceptance:** Crossing segments return (5,0); parallel segments return no intersection; a point beyond a segment clamps to its endpoint with t=1.
- **Evidence:** om9-scene osnap: 11 passed; om9-app osnap UI: 3 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
