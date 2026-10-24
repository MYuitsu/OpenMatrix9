# OM9-L4-02-02-004-06 — Coordinate Text Point Entry acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Parse W10,20,30 và relative @5,0,0 thành điểm world đúng theo active CPlane
- **Negative oracle:** Chuỗi NaN, dấu phân cách mơ hồ, overflow không được mutate prompt
- **Acceptance:** W10,20,30 is world (10,20,30); @5,0,0 is a displacement along the active CPlane; NaN, overflow and ambiguous separators return typed errors with no document write.
- **Evidence:** om9-core coordinate_input: 4 passed; om9-app cplane_coordinate_input: 10 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
