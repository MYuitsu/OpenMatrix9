# Gem Springs And Relaxation (`OM9-L3-04-12-006`)

## Public surface

| Symbol | Contract |
| --- | --- |
| `Controller::dispatch(BuilderGemSprings)` / `gvGemSprings` | opens Gem Springs on a surface + gems |
| `SpringParams` | spacing/min spacing, iterations, strength, resize |
| `spring_relax` | 3D repulsion followed by closest-point projection; X axis kept and normal recomputed |

## Effects

- Preview is session state only; it never inserts a document object.
- Apply is exactly one document transaction (children plus the host feature extension); Undo restores the previous state.
- Refusals are typed `om9_core::Error` values; no clamp and no silent fallback.
- The persisted payload is `om9.gem.pave` (`PaveFeature`, schema version 1).

## Verification

- Positive: overlapping stones are pushed apart while staying on the surface; move/resize use the typed Spacing, Minimum Spacing, Iterations and Strength
- Negative: a spring run without a surface or any gem is refused
- Gate: `OPEN` (M05-UI manual review open).
