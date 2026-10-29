# Pave Region Controls And Preview (`OM9-L3-04-12-007`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::gem_pave_text` | key=value options, Apply/Enter, Cancel/Esc, Reset |
| `PavePreview` / `pave_framework` | Stones or nearest-neighbour framework segments |
| `ui/gem_pave.rs` | tool-specific sliders and preview painter; no document mutation before Apply |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.pave` (`PaveFeature`, schema version 1).

## Verification

- Positive: margin/border, density and angle change the candidate set interactively; the preview chooses Stones or the nearest-neighbour Framework
- Negative: an unknown option is a typed error and the session stays open
- Gate: `OPEN` (M05-UI manual review open).
