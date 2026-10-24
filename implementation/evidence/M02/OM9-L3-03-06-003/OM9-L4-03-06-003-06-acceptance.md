# OM9-L4-03-06-003-06 — Reference Image Transparency And Lock acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Opacity 50%, locked prevents edits/selection nhưng vẫn visible
- **Negative oracle:** Unlock/hidden flags không làm hỏng dữ liệu asset hoặc lock state
- **Acceptance:** 50% opacity works, locked prevents edits/selection while visible, and unlock/hidden preserve asset bytes and lock state.
- **Evidence:** om9-app pictureframe unit: 6 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
