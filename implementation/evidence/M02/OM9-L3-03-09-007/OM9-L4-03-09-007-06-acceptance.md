# OM9-L4-03-09-007-06 — F6 Floating Pin And Position acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** F6 hiện tại cursor, pin/move rồi reopen nhớ vị trí khi đủ screen space
- **Negative oracle:** Mở cạnh biên không render ra ngoài monitor hoặc giữ stale screen px DPI
- **Acceptance:** A stored position is restored when it fits; an off-screen position is clamped and never rendered outside the monitor.
- **Evidence:** om9-app f6 unit: 7 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
