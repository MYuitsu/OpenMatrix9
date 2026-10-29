# Manual Pave Builder (`OM9-L3-04-12-002`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderPaveBuilder)` / `gvPaveBuilder` | opens Manual Pavé on one selected surface |
| `PaveParams` + `GemPaveSession::quantity` | exact stone number, size, spacing and angle |
| `pave_surface` | the same lattice owner; the session truncates to the typed quantity |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.pave` (`PaveFeature`, schema version 1).

## Verification

- Positive: an explicit count is honoured exactly; hexagonal spacing and Angle are typed and recompute the preview
- Negative: a zero count falls back to the region fill instead of inserting an empty layout
- Gate: `OPEN` (M05-UI manual review open).
