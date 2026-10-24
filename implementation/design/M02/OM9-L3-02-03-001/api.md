# OM9-L3-02-03-001 — Osnap End Mid Center Quadrant

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-02-03` (M02)
- **Sources:** `crates/om9-scene/src/osnap.rs`, `crates/om9-scene/src/scene.rs`, `crates/om9-scene/src/camera.rs`
- **Positive oracle:** Polyline 0-10mm snap END (10,0), MID (5,0); Circle tâm (5,5), radius 2 snap QUAD
- **Negative oracle:** Candidate nằm ngoài screen radius không được chọn; radius 0 trả none
- **Verification:** om9-scene osnap: 11 passed; om9-app osnap UI: 3 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
