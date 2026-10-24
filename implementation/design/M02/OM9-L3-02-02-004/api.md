# OM9-L3-02-02-004 — Coordinate Text Point Entry

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-02` (M02)
- **Sources:** `crates/om9-core/src/coords.rs`, `crates/om9-app/src/session.rs`
- **Positive oracle:** Parse W10,20,30 và relative @5,0,0 thành điểm world đúng theo active CPlane
- **Negative oracle:** Chuỗi NaN, dấu phân cách mơ hồ, overflow không được mutate prompt
- **Verification:** om9-core coordinate_input: 4 passed; om9-app cplane_coordinate_input: 10 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
