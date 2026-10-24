# OM9-L3-02-03-007 — Osnap Debug Marker And Tooltip

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-03` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-scene/src/osnap.rs`
- **Positive oracle:** Marker END hiện đúng khi candidate END thắng và tooltip mô tả chính xác
- **Negative oracle:** Không có candidate thì không còn marker cũ; disabled snap không hiển thị
- **Verification:** om9-app osnap UI: 3 passed; om9-scene osnap: 11 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
