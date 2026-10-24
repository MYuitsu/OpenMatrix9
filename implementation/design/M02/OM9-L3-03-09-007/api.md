# OM9-L3-03-09-007 — F6 Floating Pin And Position

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-09` (M02)
- **Sources:** `crates/om9-app/src/f6.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** F6 hiện tại cursor, pin/move rồi reopen nhớ vị trí khi đủ screen space
- **Negative oracle:** Mở cạnh biên không render ra ngoài monitor hoặc giữ stale screen px DPI
- **Verification:** om9-app f6 unit: 7 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
