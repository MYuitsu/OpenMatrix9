# OM9-L4-03-05-006-06 — Display Settings Preview Presets acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Lưu preset display, reload project và restore giá trị wireframe/edges
- **Negative oracle:** Preset lỗi schema dùng fallback và cảnh báo, không crash
- **Acceptance:** A saved preset round-trips wireframe and edges; malformed/old schema falls back with a warning instead of panicking.
- **Evidence:** om9-app display cues: 6 passed; om9-app display_modes: 3 passed; om9-app display_persistence: 2 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
