# Gem Map Viewport Overlay (`OM9-L3-04-13-007`)

## Command

`BuilderGemMap / gvGemMap` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_edits::gem_map(entries)` | numbered rows (`GemId`, number, shape, cut, girdle, world point); numbers start at one and never skip |
| `gem_edits::map_line(entry)` | stable text `#n · Shape Cut · L × W mm @ (x, y, z)` for the printed key |
| `ui::gem_edits::gem_map_labels(rows, camera, viewport, rect)` | projects every visible row into a numbered caption `#n  L × W`; a stone behind the camera or outside the rectangle is skipped, never pinned to the edge, and the caption is clamped inside the viewport |
| `MatrixApp::gem_map_overlay_ui` / `gem_map_window` | paints the numbered marker and girdle size over the committed viewport and opens a key window that lists the same numbers; both call the same typed rows |
| `BuilderGemMap` session | an empty selection maps the whole document; `File=` exports the same rows |

## Effects

- A preview is pure session data: planned records and, for a report, computed rows. Nothing is
  written to the document before Apply.
- Apply is exactly one document transaction for every mutating command; a style save writes the
  versioned store atomically. Undo restores the previous records, geometry and label.
- Refusals are typed `om9_core::Error` values (`Invalid`, `Missing`, `Stale`, `Busy`,
  `Budget`); an attribute, stone or row is never silently dropped or replaced.
- Every edit keeps the `GemId`, the object id and the `om9.gem.placement` link; only the
  `om9.gem` record and the regenerated geometry change.

## Verification

- Positive: numbered map rows from the live world placement, painted as numbered captions with girdle sizes in the viewport plus a matching key window, read-only
- Negative: a non-finite point or an oversized map is a typed refusal; an off-screen stone is omitted from the overlay instead of being mis-pinned
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
