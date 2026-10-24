# OM9-L4-03-09-002-06 — F6 Default General Mode acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** F6 khi empty selection hiển thị General + hành động khả dụng
- **Negative oracle:** No document hoặc command disabled phải được mô tả lý do
- **Acceptance:** Empty selection shows General with available commands; a missing document/command is represented as a disabled action with a reason.
- **Evidence:** om9-app f6 unit: 7 passed; om9-app f6_context: 2 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
