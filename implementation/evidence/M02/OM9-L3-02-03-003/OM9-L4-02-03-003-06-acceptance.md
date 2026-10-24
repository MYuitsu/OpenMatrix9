# OM9-L4-02-03-003-06 — Osnap Tangent Perpendicular acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Line từ điểm ngoài circle tìm tangent và perpendicular có nghiệm phù hợp
- **Negative oracle:** Điểm nằm trong circle không có tangent thực; không báo snap giả
- **Acceptance:** An external point yields real tangent/perpendicular branches; a point at/inside the circle reports no fake tangent.
- **Evidence:** om9-scene osnap: 11 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
