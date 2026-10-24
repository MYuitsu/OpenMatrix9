# OM9-L4-03-04-004-06 — Matrix Smarttrack Toggle And Feedback acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** SmartTrack bật/ tắt đồng bộ guides trên tất cả view liên quan
- **Negative oracle:** Không còn anchors thì không còn ghost guides
- **Acceptance:** SmartTrack toggles on/off, guides paint in every view that can project them, and removing anchors leaves no ghost guide.
- **Evidence:** om9-app snap_panel_modes: 4 passed; om9-app osnap_ui: 3 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
