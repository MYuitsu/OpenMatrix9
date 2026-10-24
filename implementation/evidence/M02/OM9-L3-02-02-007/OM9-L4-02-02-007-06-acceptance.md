# OM9-L4-02-02-007-06 — Cross Viewport Coordinate Consistency acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Cùng điểm world hiển thị/tái nhập giống nhau khi đổi Top→Front→Perspective
- **Negative oracle:** Thay camera không thay đổi world tọa độ của điểm lưu
- **Acceptance:** The same world point remains identical through Top, Front and Perspective; changing the camera does not rewrite its coordinates.
- **Evidence:** om9-app cplane_coordinate_input: 10 passed; om9-scene coordinate_frame: 4 passed; om9-scene camera_grid: 5 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
