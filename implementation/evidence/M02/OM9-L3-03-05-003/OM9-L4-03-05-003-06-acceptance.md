# OM9-L4-03-05-003-06 — Curve And Cutter Visibility Presets acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Ẩn working curves/cutters theo category tags và giữ mesh khác visible
- **Negative oracle:** Chưa có cutter M07 thì registry category disabled, không bỏ ẩn model
- **Acceptance:** Working curves can be hidden while a mesh stays visible; with no cutter records the cutter category is disabled and the model stays visible.
- **Evidence:** om9-app display cues: 6 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
