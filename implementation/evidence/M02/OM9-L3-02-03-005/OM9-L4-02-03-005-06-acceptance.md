# OM9-L4-02-03-005-06 — Osnap Priority And One Shot Overrides acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Một lần override MID thắng persistent END đúng một pick rồi tự reset
- **Negative oracle:** Cancel cũng tiêu thụ/khôi phục override theo state-machine đã chốt; không leak pick kế
- **Acceptance:** A one-shot MID beats persistent END for one pick, then END returns; Escape consumes the override without leaking it.
- **Evidence:** om9-scene osnap: 11 passed; om9-app osnap UI: 3 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
