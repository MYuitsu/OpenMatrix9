# OM9-L3-03-09-003 — F6 Curve And Surface Modes

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-09` (M02)
- **Sources:** `crates/om9-app/src/f6.rs`
- **Positive oracle:** F6 với curve/surface có đúng edit command; action available gọi command engine
- **Negative oracle:** Chưa có lệnh M03 thì disabled thay vì gọi stub trả success
- **Verification:** om9-app f6 unit: 7 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
