# OM9-L3-03-02-007 — Viewport Maximize By Title

- **Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING`
- **Owner:** `OM9-L2-03-02` (M02)
- **Sources:** `crates/om9-app/src/ui.rs`, `crates/om9-app/src/viewport_menu.rs`, `crates/om9-scene/src/workspace.rs`
- **Positive oracle:** Double click title maximize active viewport, lần hai restore 2×2 và đúng cameras cũ
- **Negative oracle:** Maximize viewport A không xóa camera B/C/D; double click trên canvas không toggle
- **Verification:** om9-app navigation_maximize: 2 passed; om9-app viewport_layout: 3 passed; om9-app viewport_title_menu: 17 passed; om9-scene camera_grid: 5 passed
- **Manual gate:** M02-UI review pending; no human sign-off is claimed.
