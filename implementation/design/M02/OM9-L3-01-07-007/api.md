# OM9-L3-01-07-007 — Generic Translation Rotation Scale Handles

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-01-07` (M02)
- **Sources:** `crates/om9-scene/src/gumball_view.rs`, `crates/om9-core/src/gumball.rs`, `crates/om9-app/src/ui.rs`, `crates/om9-app/src/session/gumball.rs`
- **Positive oracle:** Gizmo Move/Rotate/Scale xuất hiện theo bounding box, trục đúng world axes
- **Negative oracle:** Object không hợp lệ/locked không tạo handle dùng được
- **Verification:** om9-scene lib gumball_view: 13 passed; om9-app gumball_viewport: 3 passed; om9-app gumball_placement: 4 passed; om9-app gumball_interaction: 5 passed; om9-app gumball_numeric: 5 passed; om9-core gumball: 15 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
