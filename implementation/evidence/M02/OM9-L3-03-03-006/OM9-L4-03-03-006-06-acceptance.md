# OM9-L4-03-03-006-06 — Matrix Gumball Numeric Edit acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Nhập 12.5 mm vào handle X tạo transform chính xác và một undo entry
- **Negative oracle:** String NaN và trống không commit; parse fail giữ gizmo active
- **Acceptance:** MoveX 12.5 produces exactly +12.5 mm and one undo entry; NaN/empty input commits nothing and keeps the gizmo active.
- **Evidence:** om9-app gumball_numeric: 6 passed; om9-core gumball: 15 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
