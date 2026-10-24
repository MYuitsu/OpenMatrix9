# OM9-L4-03-02-003-06 — Wheel Zoom To Cursor acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Wheel zoom tại pixel (x,y) giữ điểm world dưới cursor tại cùng pixel sau zoom
- **Negative oracle:** Zoom factor 0/NaN/negative hoặc ở ngoài view được từ chối
- **Acceptance:** A wheel burst keeps the cursor world anchor at the same pixel; zero/NaN/negative zoom factors are typed errors.
- **Evidence:** om9-app navigation_input: 1 passed; om9-app navigation_precision: 3 passed; om9-scene camera_fit: 4 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
