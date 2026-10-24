# OM9-L3-01-07-005 — Object Selection Disambiguation

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-01-07` (M02)
- **Sources:** `crates/om9-scene/src/scene.rs`, `crates/om9-scene/src/picking.rs`, `crates/om9-app/src/ui.rs`
- **Positive oracle:** Hai box chồng trên ray tạo menu hai ID phân biệt và chọn theo ý người dùng
- **Negative oracle:** Không có hit không mở popup; popup đóng không đổi selection
- **Verification:** om9-app pick_disambiguation: 5 passed (10/10 consecutive runs); om9-scene acceptance_l2_01_07: 9 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
- **Defect fixed 2026-10-22:** `pick_all` no longer applies the nearest-only depth early-out; both overlapping objects are always candidates. See `2026-10-22-pick-all-fix.md`.
