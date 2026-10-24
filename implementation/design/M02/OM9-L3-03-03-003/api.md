# OM9-L3-03-03-003 — Matrix Ambiguous Selection Popup

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-03` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-scene/src/picking.rs`
- **Positive oracle:** Nhiều hits hiện popup type+layer swatch; chọn ID cụ thể và đóng popup
- **Negative oracle:** Esc popup không thay selection, stale ID lọc bỏ trước khi chọn
- **Verification:** om9-app pick_disambiguation: 5 passed; om9-app matrix_selection_sequence: 2 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
