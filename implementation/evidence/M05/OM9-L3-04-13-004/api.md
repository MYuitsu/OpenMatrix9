# Gem Style Save (`OM9-L3-04-13-004`)

## Command

`OthersSaveStyle / gvSaveStyle` — palette icon and typed command line reach the same handler.

## Public surface

| Symbol | Contract |
| --- | --- |
| `gem_edits::GemStyle` | schema `om9.gem_style.v1`, version 1, name and a full attribute `GemEdit`; the gem id, label and placement stay with the instance, not the style |
| `GemStyleStore` | bounded (512) named store with case-insensitive replace-by-name; schema `om9.gem_style_file.v1` |
| `GemStyleStore::{save_to,load_or_empty}` | atomic staging-file + rename (copy fallback on Windows); a missing file is the documented empty store |
| `GemStyle::validate` | reuses `builder_style::validate_style_name`, so the gem store refuses the same names as the shared builder preset store |

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

- Positive: versioned named gem style written atomically to the shared store file
- Negative: invalid style names are typed refusals
- Gate: `M05-UI` — the M05-UI manual gate stays open; no reviewer signature is claimed.
