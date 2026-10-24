# OM9-L3-01-07-001 — Screen Ray And Hit Test Query

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-01-07` (M02)
- **Sources:** `crates/om9-scene/src/scene.rs`, `crates/om9-scene/src/picking.rs`, `crates/om9-scene/src/spatial.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Box tại tâm tia, điểm click ở viewport P trúng BodyId của box
- **Negative oracle:** Ray không giao đối tượng trả danh sách rỗng, không chọn
- **Verification:** om9-scene acceptance_l2_01_07: 9 passed; om9-scene world_picking: 7 passed; om9-scene navigation_snapshot: 2 passed; om9-app brep_selection (gpu-sphere): 5 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
