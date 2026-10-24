# OM9-L3-02-02-007 — Cross Viewport Coordinate Consistency

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-02` (M02)
- **Sources:** `crates/om9-scene/src/workspace.rs`, `crates/om9-app/src/session.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Cùng điểm world hiển thị/tái nhập giống nhau khi đổi Top→Front→Perspective
- **Negative oracle:** Thay camera không thay đổi world tọa độ của điểm lưu
- **Verification:** om9-app cplane_coordinate_input: 10 passed; om9-scene coordinate_frame: 4 passed; om9-scene camera_grid: 5 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
