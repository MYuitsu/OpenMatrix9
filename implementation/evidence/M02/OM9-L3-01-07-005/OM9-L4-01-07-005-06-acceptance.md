# OM9-L4-01-07-005-06 — Object Selection Disambiguation acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Hai box chồng trên ray tạo menu hai ID phân biệt và chọn theo ý người dùng
- **Negative oracle:** Không có hit không mở popup; popup đóng không đổi selection
- **Acceptance:** Two overlapping objects create two distinct candidate IDs, nearest first; a click with no hit creates no popup; closing/Escape leaves the prior selection unchanged.
- **Evidence:** om9-app pick_disambiguation: 5 passed (10/10 consecutive runs); om9-scene acceptance_l2_01_07: 9 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.

## Defect fixed 2026-10-22

`Scene::pick_all` used the nearest-only depth early-out unconditionally, so the farther of two overlapping objects could be missing from the candidate list. It is now guarded by `nearest_only`; the regression creates two points on one ray 32 times and asserts both roots nearest-first every time. RED: `2026-10-22-pick-disambiguation-retry.log`; GREEN: `2026-10-22-pick-disambiguation-x10.log` (10/10 runs).
