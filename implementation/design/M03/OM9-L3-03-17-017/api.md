# OM9-SURFACE-117 — Surface Extrude All menu API

- **L3:** `OM9-L3-03-17-017`
- **Menu icon:** `SurfaceGVDExtrudeAll`
- **State:** `IMPLEMENTED`
- **Owner candidate:** `OM9-L3-01-09-002`
- **Selection:** a compatible selected surface/curve profile as declared by the owner; no geometry is produced by the UI
- **Entry points:** Matrix Surface menu and command line resolve to this one command id.
- **Transaction:** one command session = one Undo entry; disabled/refused input writes nothing.

## Disabled policy
