# OM9-L3-03-03-002 — Matrix Left Right Drag Selection

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-03` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-scene/src/selection.rs`
- **Positive oracle:** Kéo trái→phải window và phải→trái crossing vẽ đúng rectangle, trả set tương ứng
- **Negative oracle:** Drag bắt đầu trên UI dock không phát selection marquee
- **Verification:** om9-app rubber_band: 2 passed; om9-app selection_overlay: 8 passed; om9-app acceptance_viewport_selection: 8 passed; om9-scene selection_regressions: 7 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
