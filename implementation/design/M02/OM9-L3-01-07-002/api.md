# OM9-L3-01-07-002 — Object Selection Set Core

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-01-07` (M02)
- **Sources:** `crates/om9-app/src/session/layers.rs`, `crates/om9-core/src/selection.rs`, `crates/om9-sdk/src/lib.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Click ID A rồi Ctrl click ID B trả set {A,B}, toggle A trả {B}
- **Negative oracle:** ID đã xóa hoặc layer khóa bị loại và không lưu stale ID
- **Verification:** om9-app acceptance_selection_set: 6 passed; om9-core selection_state: 7 passed; om9-app selection_overlay: 8 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
