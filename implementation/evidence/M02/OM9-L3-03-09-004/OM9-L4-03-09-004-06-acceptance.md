# OM9-L4-03-09-004-06 — F6 Gem And Report Modes acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** F6 Gem/Report đăng ký menu item có nhãn khi provider có mặt
- **Negative oracle:** Chưa có M05, hành động phải REGISTERED_DISABLED và không tạo gem rỗng
- **Acceptance:** Gem/Report labels appear for a gem selection; absent M05 disables them and never creates an empty gem.
- **Evidence:** om9-app f6 unit: 7 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
