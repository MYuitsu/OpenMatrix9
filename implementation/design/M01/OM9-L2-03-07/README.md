# M01 OM9-L2-03-07 — Matrix main menu, icon history and docking

**Status:** `AUTOMATED_TARGET_VALIDATED_MANUAL_UI_PENDING` · **Target:** standalone Rust OM9
**Owner:** `OM9-L2-03-07` (M01)

## Design direction

The documented Matrix positions and sidebar order are kept:

`ICON HISTORY → MAIN MENU → DISPLAY → SNAPS → INFO & SETTINGS → LAYERS → PROJECTS → BUILDERS`

The visual language is modernized without changing hit targets or layout geometry:

- Deep slate left/right rails (`#141D28` / `#1B2735`) with cool ink text.
- Section headers are flat rounded cards with a colored accent rail per section; collapse/expand
  remains the same click/drag target.
- Command icons are rounded tiles with a subtle outline, dark resting fill, lighter hover and muted
  disabled state instead of raised bevel buttons.
- Main Menu category chips use a modern six-color palette; the selected category is a filled chip,
  unselected categories are translucent tinted chips. The documented 6-per-row arrangement and all
  twelve category names/order are unchanged.
- Toolbar row uses a soft light gradient and keeps the existing 67-point allocation.
- Icon tooltips keep the command name and append an optional registered right-click command. If no
  alternative is registered, the secondary click is inert and never replays the primary command.

## Delivered behavior

- Eight L3 capabilities: left sidebar stack/order, category tabs, icon tooltip/right-click slot,
  scrolling icon grid, nine recent command icons, collapse/reorder/float, default reset/reload and
  visibly disabled unavailable actions.
- Existing SVG icon catalog and docking kernel are reused; no new drawing engine.
- Category selection does not mutate the document or Undo.
- Right-click without a registered alternative does not execute the previous command.

## Verification

- `chrome_actions` — **9 passed**: F4 datum, category chip selection, right-click inert slot, view
  tabs/menu, maximized tab switch, disabled snaps.
- `left_stack_layers` — **8 passed**: documented left stack, 300-pt rail, layer placement, reset and
  save/reload.
- `shell_docking` — **10 passed**; `panel_ops` — **33 passed**: collapse/reorder/float/dock.
- `dock_layout_default` — **7**, `dock_layout_persistence` — **6**: default/reset/reload.
- `command_sessions` — **6**, `command_options_and_repeat` — **6**: recent command history.
- `startup_contract` — **3**: failed start leaves layout untouched.
- Automated UI audit `om9 --ui-audit` at 1280x900 — exit 0, 21 passes, screenshot
  `build/ui-audit-l2-03-07/workspace.png` (SHA-256 `b7f5ed9d…`). The screenshot shows the modern
  slate rail, colored category chips, rounded icon tiles and the unchanged four-viewport placement.
  This is automated evidence, not human sign-off.

## Remaining

- Manual M01-UI visual review on the real Windows window is still required; no human sign-off is
  claimed.
- No feature currently registers a right-click alternative; the slot is implemented and inert by
  default.
