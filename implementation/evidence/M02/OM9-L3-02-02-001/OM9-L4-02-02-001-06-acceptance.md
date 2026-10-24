# OM9-L4-02-02-001-06 — World And Construction Plane Coordinate Context acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Điểm local (5,0,0) trên CPlane xoay 90° quanh Z thành world (0,5,0)
- **Negative oracle:** World/local point từ viewport khác không dùng nhầm CPlane của viewport hiện tại
- **Acceptance:** Local (5,0,0) on a plane rotated 90 degrees about Z becomes world (0,5,0); a Front local point does not borrow Top's axes; camera pan leaves the stored world point unchanged.
- **Evidence:** om9-core coordinate_input: 4 passed; om9-scene coordinate_frame: 4 passed; om9-app cplane_coordinate_input: 10 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
