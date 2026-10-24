# OM9-L3-02-03-002 — Osnap Near Point Intersection

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-03` (M02)
- **Sources:** `crates/om9-scene/src/osnap.rs`, `crates/om9-scene/src/scene.rs`
- **Positive oracle:** Hai đoạn cắt ở (5,0) trả INTERSECTION; NEAR trả closest point có tham số t hợp lệ
- **Negative oracle:** Hai đoạn song song không trả giao điểm giả; point ngoài curve được clamp
- **Verification:** om9-scene osnap: 11 passed; om9-app osnap UI: 3 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
