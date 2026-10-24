# OM9-L3-02-02-001 — World And Construction Plane Coordinate Context

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-02` (M02)
- **Sources:** `crates/om9-scene/src/camera.rs`, `crates/om9-scene/src/workspace.rs`, `crates/om9-core/src/coords.rs`, `crates/om9-app/src/session.rs`
- **Positive oracle:** Điểm local (5,0,0) trên CPlane xoay 90° quanh Z thành world (0,5,0)
- **Negative oracle:** World/local point từ viewport khác không dùng nhầm CPlane của viewport hiện tại
- **Verification:** om9-core coordinate_input: 4 passed; om9-scene coordinate_frame: 4 passed; om9-app cplane_coordinate_input: 10 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
