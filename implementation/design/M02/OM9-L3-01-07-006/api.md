# OM9-L3-01-07-006 — Selection Highlighting And Hover

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-01-07` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-render/src/scene.rs`, `crates/om9-scene/src/selection.rs`
- **Positive oracle:** Hover cạnh face rồi select box hiển thị màu highlight đồng bộ cả 4 view
- **Negative oracle:** Object hidden hoặc đã xóa không còn outline sau scene refresh
- **Verification:** om9-app selection_overlay: 8 passed; om9-app subobject_occurrences: 5 passed; om9-app acceptance_viewport_selection: 8 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
