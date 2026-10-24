# OM9-L3-03-06-001 — Pictureframe Image Import

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-06` (M02)
- **Sources:** `crates/om9-app/src/pictureframe.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Import PNG 100×100 gắn quad 20×20 mm vào active plane
- **Negative oracle:** Ảnh sai định dạng/giải mã lỗi báo typed error, không tạo quad rỗng
- **Verification:** om9-app pictureframe unit: 6 passed; om9-app pictureframe persistence: 3 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
