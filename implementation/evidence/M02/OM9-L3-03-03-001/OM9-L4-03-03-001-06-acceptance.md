# OM9-L4-03-03-001-06 — Matrix Click Shift Ctrl Select acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Shift click thêm, Ctrl click toggle selection, plain click replace theo mapping đã chốt
- **Negative oracle:** Click đối tượng locked không tạo ID vào selection
- **Acceptance:** Plain click replaces, Shift adds and Ctrl toggles; a locked entity never enters the selection set.
- **Evidence:** om9-app matrix_selection_sequence: 2 passed; om9-app acceptance_viewport_selection: 8 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
