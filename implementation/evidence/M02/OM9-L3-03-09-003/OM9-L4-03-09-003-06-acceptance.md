# OM9-L4-03-09-003-06 — F6 Curve And Surface Modes acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** F6 với curve/surface có đúng edit command; action available gọi command engine
- **Negative oracle:** Chưa có lệnh M03 thì disabled thay vì gọi stub trả success
- **Acceptance:** Curve/surface groups show the correct edit entries; without M03 they are disabled rather than calling a success stub.
- **Evidence:** om9-app f6 unit: 7 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
