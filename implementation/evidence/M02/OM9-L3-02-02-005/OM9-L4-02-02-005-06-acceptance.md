# OM9-L4-02-02-005-06 — Matrix F4 Origin Shortcut acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** F4 khi đang hỏi point chèn world origin (0,0,0) vào current prompt
- **Negative oracle:** F4 ngoài point prompt không tạo object hoặc insert chuỗi sai
- **Acceptance:** F4 in a line/point prompt commits world (0,0,0); F4 outside a prompt creates no object and does not insert a malformed string.
- **Evidence:** om9-app cplane_f4_ui: 1 passed; om9-app cplane_coordinate_input: 10 passed; om9-app chrome_actions: 13 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
