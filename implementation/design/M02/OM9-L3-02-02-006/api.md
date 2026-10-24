# OM9-L3-02-02-006 — Ortho Planar Project Cursor Constrain

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-02` (M02)
- **Sources:** `crates/om9-app/src/session.rs`, `crates/om9-app/src/ui.rs`, `crates/om9-app/src/ui/chrome.rs`, `crates/om9-app/src/ui/sidebar_palettes.rs`
- **Positive oracle:** Ortho giữ hướng trục CPlane; Planar project cho điểm hợp lệ lên active plane
- **Negative oracle:** Bật cả chế độ không tạo điểm 3D lệch khỏi CPlane do thứ tự xử lý
- **Verification:** om9-app cplane_coordinate_input: 10 passed; om9-app chrome_actions: 13 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
