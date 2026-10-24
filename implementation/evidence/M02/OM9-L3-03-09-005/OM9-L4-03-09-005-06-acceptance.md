# OM9-L4-03-09-005-06 — F6 Custom User Mode acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** User custom menu lưu command IDs và reload với thứ tự giống cũ
- **Negative oracle:** Command ID đã xoá thì menu entry disabled thay vì dispatch sai
- **Acceptance:** Custom menu reload preserves order; a deleted command ID is disabled instead of dispatched.
- **Evidence:** om9-app f6 unit: 7 passed; om9-app f6_context: 2 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
