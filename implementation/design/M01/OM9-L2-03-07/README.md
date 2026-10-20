# M01 OM9-L2-03-07 — Matrix main menu, icon history and docking

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Target:** standalone Rust OM9
**Owner:** `OM9-L2-03-07` (M01)

## Design direction

The documented Matrix positions and sidebar order are kept:

`ICON HISTORY → MAIN MENU → DISPLAY → SNAPS → INFO & SETTINGS → LAYERS → PROJECTS → BUILDERS`

The visual language is modernized without changing hit targets or layout geometry:

- Near-black left/right rails (`#0A0E13` / `#10171F`) with cool ink text (minimum category contrast 7.5:1).
- Section headers are flat rounded cards with a colored accent rail per section; collapse/expand
  remains the same click/drag target.
- Command icons are rounded tiles with a subtle outline, dark resting fill, lighter hover and muted
  disabled state instead of raised bevel buttons.
- Main Menu categories use the **RGB values sampled from Matrix8 p14** and keep the reference
  colour identity: gold Curve, teal Surface, purple Solid, red Transform, green SubD/TSpline,
  yellow-green Art, red Builder, indigo Tools, blue Gems, magenta Setting, orange Cutters, teal
  Render. Each is a raised dark card showing only the category name in a brightened reference hue (no
  leading square/dot), with a soft drop shadow, a lit top edge and a dark text shadow; the selected
  category uses a dark-green fill (`#123A2A`) and a matching accent border. Categories flow on one horizontal row
  and wrap naturally when they reach the rail edge; the documented category order is preserved.
- Toolbar row uses a soft light gradient and keeps the existing 67-point allocation.
- Icon tooltips keep the command name and append an optional registered right-click command. If no
  alternative is registered, the secondary click is inert and never replays the primary command.
- Command-icon boxes scale with both the dock width and the logical screen size: rail icons follow
  the 240–460 pt rail and toolbar icons follow a 0.85–1.2 screen scale. The base edge is 25% larger
  than the previous 22 pt design (27.5 pt; 23–42 pt on the rail), while the toolbar keeps its 67-pt
  allocation.
- Menu and category labels use 11 pt type for a crisper read at 100% DPI. Font hinting is disabled
  (`Visuals::text_options.font_hinting = false`) so glyph edges use the anti-aliased outline instead
  of grid-fitted, jagged stems.

## Delivered behavior

- Eight L3 capabilities: left sidebar stack/order, category tabs, icon tooltip/right-click slot,
  scrolling icon grid, nine recent command icons, collapse/reorder/float, default reset/reload and
  visibly disabled unavailable actions.
- Existing SVG icon catalog and docking kernel are reused; no new drawing engine.
- Category selection does not mutate the document or Undo.
- Right-click without a registered alternative does not execute the previous command.

## Verification

- `chrome_actions` — **11 passed**: F4 datum, category single-row selection, right-click inert slot, screen
  icon scaling, view tabs/menu, maximized tab switch, disabled snaps.
- `left_stack_layers` — **8 passed**: documented left stack, 300-pt rail, layer placement, reset and
  save/reload.
- `shell_docking` — **10 passed**; `panel_ops` — **33 passed**: collapse/reorder/float/dock.
- `dock_layout_default` — **7**, `dock_layout_persistence` — **6**: default/reset/reload.
- `command_sessions` — **6**, `command_options_and_repeat` — **6**: recent command history.
- `startup_contract` — **3**: failed start leaves layout untouched.
- `om9-app --lib` — **104 passed**.
- Automated UI audit `om9 --ui-audit` at 1280x900 — exit 0, 21 passes, screenshot
  `build/ui-audit-l2-03-07/workspace.png` (SHA-256 `ed6723bd…`). The screenshot shows the modern
  near-black rail, reference-coloured category cards, rounded icon tiles and the unchanged four-viewport
  placement.
  This is automated evidence, not human sign-off.

## Remaining

- Manual M01-UI visual review on the real Windows window is still required; no human sign-off is
  claimed.
- No feature currently registers a right-click alternative; the slot is implemented and inert by
  default.
