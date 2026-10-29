# Gem Loader Shape Cut Size Picker (`OM9-L3-04-09-003`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(OM9-GEM-001)` / `gvLoader` | opens the loader preview |
| `GemSession` loader fields | `cut`, `shape`, `size_index`, `size_page`, `keep_original`, `dimensions`, `girdle_mm`, `replace` |
| command options | `Shape=`, `Cut=`, `Size=LxW|D`, `Length=`, `Width=`, `Depth=`, `Girdle=`, `KeepOriginal=` |
| `Controller::gem_apply` | one Insert, or Geometry+Extensions+Attributes when replacing the selected GemId |

## Effects

- A preview is the typed spec plus its computed mesh in session state; it never inserts a document object.
- Apply is exactly one document transaction; Undo restores the previous state and Redo keeps the object identity.
- Refusals are typed `om9_core::Error` values with stable messages (`Invalid`, `Unsupported`, `Stale`, `Busy`); no clamp and no silent fallback.
- The payload is `om9.gem` (`GemRecord`, schema `om9.gem_spec.v1`, version 1) and the mesh owner remains the entity itself.
## Verification

- Positive: one gemstone with the selected shape, cut and size in one Undo step; its preview is painted in the viewport and the committed body is an editable B-Rep
- Negative: invalid sizes/options and a locked layer never mutate the document
- Gate: `OPEN` (M05-UI manual review open).
