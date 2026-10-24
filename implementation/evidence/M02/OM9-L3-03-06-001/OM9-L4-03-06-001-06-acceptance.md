# OM9-L4-03-06-001-06 — Pictureframe Image Import acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Import PNG 100×100 gắn quad 20×20 mm vào active plane
- **Negative oracle:** Ảnh sai định dạng/giải mã lỗi báo typed error, không tạo quad rỗng
- **Acceptance:** A 100x100 PNG produces a 20x20 mm quad on the active plane; malformed bytes return a typed error and create no quad.
- **Evidence:** om9-app pictureframe unit: 6 passed; om9-app pictureframe persistence: 3 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
