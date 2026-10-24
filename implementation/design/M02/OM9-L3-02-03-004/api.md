# OM9-L3-02-03-004 — Osnap Surface And Polysurface

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-03` (M02)
- **Sources:** `crates/om9-scene/src/osnap.rs`, `crates/om9-core/src/brep.rs`
- **Positive oracle:** Đối với mặt phẳng BRep, nearest point nằm trên trimmed face và normal đúng
- **Negative oracle:** Điểm gần ngoài trim/face invalid không snap lên phần mặt bị cắt
- **Verification:** om9-app osnap BRep: 1 passed; om9-scene osnap: 11 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
