# OM9-L4-03-05-002-06 — Gem View Material Color Cues acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Gem-like semantic material cue có token màu nếu semantic provider tồn tại
- **Negative oracle:** Không có M05 Gem records thì hiển thị neutral placeholder và ghi DEFERRED_WAIT_M05
- **Acceptance:** A semantic gem with a colour shows that token; a gem without M05 material shows neutral and the explicit deferred token.
- **Evidence:** om9-app display cues: 6 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
