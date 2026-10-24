# OM9-L3-01-07-004 — Selection Window And Crossing

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-01-07` (M02)
- **Sources:** `crates/om9-scene/src/selection.rs`, `crates/om9-scene/src/scene.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Cửa sổ kéo trái→phải chỉ chọn box hoàn toàn nằm trong; phải→trái chọn giao cắt
- **Negative oracle:** Object nằm ngoài không được chọn; locked/hidden đều bỏ qua
- **Verification:** om9-scene selection_regressions: 7 passed; om9-app rubber_band: 2 passed; om9-app acceptance_viewport_selection: 8 passed; om9-app selection_overlay: 8 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
