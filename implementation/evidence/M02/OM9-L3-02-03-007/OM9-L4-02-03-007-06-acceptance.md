# OM9-L4-02-03-007-06 — Osnap Debug Marker And Tooltip acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Marker END hiện đúng khi candidate END thắng và tooltip mô tả chính xác
- **Negative oracle:** Không có candidate thì không còn marker cũ; disabled snap không hiển thị
- **Acceptance:** An END winner paints an END marker with the exact 'End point' tooltip; an empty cursor or disabled END leaves no marker.
- **Evidence:** om9-app osnap UI: 3 passed; om9-scene osnap: 11 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
