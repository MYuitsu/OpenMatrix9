# OM9-L3-03-01-001 — Matrix Four Viewport Grid Layout

## Contract

- API: `Workspace::new / visible_views / active / view_rects`
- Owner: `OM9-L2-03-01`
- Input: typed workspace/view/camera/grid state.
- Output: observable viewport layout state; no document geometry mutation.
- Errors: invalid/duplicate view, non-finite layout values and stale targets are refused explicitly.

## Positive oracle

matrix_four_view_camera_presets_are_distinct_and_valid + four-view placement

## Negative oracle

invalid/zero viewport allocation is rejected; no partial scene state

## Verification

- `viewport_layout`: 3 passed
- `chrome_actions`: 13 passed
- `viewport_title_menu`: 17 passed
- Manual M01-UI review remains pending.
