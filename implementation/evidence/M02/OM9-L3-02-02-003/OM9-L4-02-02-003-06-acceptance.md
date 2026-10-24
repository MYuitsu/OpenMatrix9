# OM9-L4-02-02-003-06 — CPlane Orientation And Origin Change acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Đổi gốc CPlane sang (10,20,0) và quay 30° giữ world geometry không thay đổi
- **Negative oracle:** Đầu vào 3 điểm thẳng hàng hoặc frame singular trả typed error
- **Acceptance:** Origin (10,20,0) plus a 30-degree rotation leaves every world object's placement/geometry unchanged; collinear three-point input is refused and leaves the old plane in place.
- **Evidence:** om9-scene cplane_presets: 6 passed; om9-app cplane_coordinate_input: 10 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
