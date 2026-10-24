# OM9-L3-03-01-005 — Matrix Viewport Grid And Axes

## Contract

- API: `Grid::new + grid/axis strokes per CPlane`
- Owner: `OM9-L2-03-01`
- Input: typed workspace/view/camera/grid state.
- Output: observable viewport layout state; no document geometry mutation.
- Errors: invalid/duplicate view, non-finite layout values and stale targets are refused explicitly.

## Positive oracle

jewelry_grid_axes_follow_each_view_cplane

## Negative oracle

non-finite/negative spacing is refused; grid cannot draw outside its viewport

## Verification

- `viewport_layout`: 3 passed
- `chrome_actions`: 13 passed
- `viewport_title_menu`: 17 passed
- Manual M01-UI review remains pending.
