# OM9-L3-02-03-003 — Osnap Tangent Perpendicular

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-03` (M02)
- **Sources:** `crates/om9-scene/src/osnap.rs`
- **Positive oracle:** Line từ điểm ngoài circle tìm tangent và perpendicular có nghiệm phù hợp
- **Negative oracle:** Điểm nằm trong circle không có tangent thực; không báo snap giả
- **Verification:** om9-scene osnap: 11 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
