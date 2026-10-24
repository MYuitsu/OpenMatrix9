# OM9-L3-02-03-005 — Osnap Priority And One Shot Overrides

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-03` (M02)
- **Sources:** `crates/om9-scene/src/osnap.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Một lần override MID thắng persistent END đúng một pick rồi tự reset
- **Negative oracle:** Cancel cũng tiêu thụ/khôi phục override theo state-machine đã chốt; không leak pick kế
- **Verification:** om9-scene osnap: 11 passed; om9-app osnap UI: 3 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
