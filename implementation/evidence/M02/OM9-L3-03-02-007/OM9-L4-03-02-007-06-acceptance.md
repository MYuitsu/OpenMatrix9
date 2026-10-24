# OM9-L4-03-02-007-06 — Viewport Maximize By Title acceptance

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Positive oracle:** Double click title maximize active viewport, lần hai restore 2×2 và đúng cameras cũ
- **Negative oracle:** Maximize viewport A không xóa camera B/C/D; double click trên canvas không toggle
- **Acceptance:** Double-click title maximizes the active viewport, a second double-click restores 2x2 with all prior cameras; canvas double-click does nothing.
- **Evidence:** om9-app navigation_maximize: 2 passed; om9-app viewport_layout: 3 passed; om9-app viewport_title_menu: 17 passed; om9-scene camera_grid: 5 passed
- **Manual gate:** M02-UI review pending; automated checks are not human approval.
