# OM9-L4-03-04-003-06 — Matrix Snap Activation And Indicator acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** One-shot override bật rồi dùng đúng một pick và indicator tự tắt
- **Negative oracle:** Escape prompt phải dọn indicator, không lưu override sang project khác
- **Acceptance:** An END/MID one-shot applies to exactly one pick and its indicator disappears; Escape clears the indicator and does not carry it to the next project.
- **Evidence:** om9-app snap_panel_modes: 4 passed; om9-app osnap_ui: 3 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
