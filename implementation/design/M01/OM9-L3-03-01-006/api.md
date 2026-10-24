# OM9-L3-03-01-006 — Maximize And Restore Viewport Layout

## Contract

- API: `Workspace::toggle_maximize + visible_views`
- Owner: `OM9-L2-03-01`
- Input: typed workspace/view/camera/grid state.
- Output: observable viewport layout state; no document geometry mutation.
- Errors: invalid/duplicate view, non-finite layout values and stale targets are refused explicitly.

## Positive oracle

maximize_is_per_view_and_restores_all_four_views

## Negative oracle

maximizing a foreign/stale view is rejected; other views remain unchanged

## Verification

- `viewport_layout`: 3 passed
- `chrome_actions`: 13 passed
- `viewport_title_menu`: 17 passed
- Manual M01-UI review remains pending.
