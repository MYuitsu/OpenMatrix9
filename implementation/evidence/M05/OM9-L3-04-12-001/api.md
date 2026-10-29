# Auto Pave Builder (`OM9-L3-04-12-001`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderPaveDialog)` / `gvPaveDialog` | opens Auto Pavé on one selected surface |
| `PaveParams` | pattern, spacing/min spacing, base/min/max size, margin/border, angle/flip, taper/equalize/perturb, pre/post correction, relax, seeds, start, attractor/edge curves |
| `pave_surface` | typed UV-lattice plan; local-metric spacing, optional seeds, attractor bias, closed-edge clipping, correction and relaxation |
| `PaveStyle` / `PaveStyleLibrary` | versioned named snapshots; built-ins plus user styles, optional OM9_PAVE_STYLE_LIBRARY file |
| `Controller::gem_pave_apply` | one transaction for every child insert/update/delete plus the om9.gem.pave feature |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.pave` (`PaveFeature`, schema version 1).

## Verification

- Positive: the lattice fills the selected surface region and every stone lies inside it; spacing, margin, border, angle and sizing stay within the typed range
- Negative: a spacing that cannot fit the region is refused (no squeeze)
- Gate: `OPEN` (M05-UI manual review open).
