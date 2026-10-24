# OM9-L3-03-09-006 — F6 Builder Input Prefill

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-09` (M02)
- **Sources:** `crates/om9-app/src/f6.rs`
- **Positive oracle:** F6 Builder selection đóng gói typed IDs + origin hint đúng theo schema M04
- **Negative oracle:** Chưa có Builder M04 thì chỉ ENABLE sau provider; còn lại disabled tooltip
- **Verification:** om9-app f6 unit: 7 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
