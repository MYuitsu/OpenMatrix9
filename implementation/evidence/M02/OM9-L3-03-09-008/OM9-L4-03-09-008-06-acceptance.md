# OM9-L4-03-09-008-06 — F6 Disabled Unimplemented Actions acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Unavailable command hiển thị disabled cùng lý do; click không dispatch
- **Negative oracle:** Click nút disabled không gọi M01 command engine và không mutate doc
- **Acceptance:** Unavailable actions are disabled with a reason; clicking one does not dispatch or mutate the document.
- **Evidence:** om9-app f6 unit: 7 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
