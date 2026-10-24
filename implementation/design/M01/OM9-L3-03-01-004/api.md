# OM9-L3-03-01-004 — F4 World Origin And Ring Center Display

## Contract

- API: `MatrixApp::world_datum_visible + paint_world_datum`
- Owner: `OM9-L2-03-01`
- Input: typed workspace/view/camera/grid state.
- Output: observable viewport layout state; no document geometry mutation.
- Errors: invalid/duplicate view, non-finite layout values and stale targets are refused explicitly.

## Positive oracle

f4_toggles_world_datum_without_mutating_document

## Negative oracle

F4 toggling must not move any datum coordinate or create Undo entries

## Verification

- `viewport_layout`: 3 passed
- `chrome_actions`: 13 passed
- `viewport_title_menu`: 17 passed
- Manual M01-UI review remains pending.
