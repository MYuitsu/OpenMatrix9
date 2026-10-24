# OM9-L4-03-04-005-06 — Matrix Snap Shortcut Modes acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Shortcut đổi END/MID theo registry, trạng thái panel cập nhật cùng frame
- **Negative oracle:** Shortcut khi focus text editor không kích hoạt snap và không gõ ký tự lạ
- **Acceptance:** E selects END and M selects MID while a command runs; a focused command editor does not trigger the shortcut.
- **Evidence:** om9-app snap_panel_modes: 4 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
