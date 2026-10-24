# OM9-SURFACE-133 — Extrude Normal to Surface menu API

- **L3:** `OM9-L3-03-17-033`
- **Menu icon:** `SurfaceExtrudeCurveNormalToSurface`
- **State:** `IMPLEMENTED`
- **Owner candidate:** `OM9-L2-03-17`
- **Selection:** a compatible selected surface/curve profile as declared by the owner; no geometry is produced by the UI
- **Entry points:** Matrix Surface menu and command line resolve to this one command id.
- **Transaction:** one command session = one Undo entry; disabled/refused input writes nothing.

## Disabled policy
