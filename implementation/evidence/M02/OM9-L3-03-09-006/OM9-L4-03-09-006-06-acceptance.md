# OM9-L4-03-09-006-06 — F6 Builder Input Prefill acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** F6 Builder selection đóng gói typed IDs + origin hint đúng theo schema M04
- **Negative oracle:** Chưa có Builder M04 thì chỉ ENABLE sau provider; còn lại disabled tooltip
- **Acceptance:** With M04, the prefill carries typed IDs and origin; without M04 the action is disabled with a tooltip.
- **Evidence:** om9-app f6 unit: 7 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
