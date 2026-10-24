# OM9-L4-02-02-002-06 — CPlane Preset Worldtop Front Right acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Top/Front/Right preset có origin đúng và basis right-handed
- **Negative oracle:** Preset có norm trục zero/không trực giao phải bị chặn
- **Acceptance:** Every preset has origin (0,0,0) and a right-handed, orthonormal basis; zero/non-orthogonal frames are typed errors; the menu changes one view only.
- **Evidence:** om9-scene cplane_presets: 6 passed; om9-app cplane_menu: 1 passed; om9-app viewport_title_menu: 17 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
