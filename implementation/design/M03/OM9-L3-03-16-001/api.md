# OM9-CURVE-001 — PolyLine menu API

- **L3:** `OM9-L3-03-16-001`
- **Menu icon:** `CurvePolylinePolyline`
- **State:** `IMPLEMENTED_AWAITING_MANUAL_UI` (2026-10-26)
- **Owner candidate:** `OM9-L3-02-04-001`
- **Entry points:** Matrix Curve menu and the command line resolve to this one command id.
- **Transaction:** one command session = one Undo entry; disabled/refused input writes nothing.

## Current standalone scope

Parameter control is exposed through `om9_app::polyline_options::PolylineOptionInterface`, implemented by the existing Controller. All eight options share stable IDs, shortcut/kind metadata and current values; activation remains in the Polyline session. This host API adds no plugin ABI. [Interface verification](../../../../docs/validation/2026-10-25-polyline-parameter-interface/README.md).

The existing enabled Curve menu binding dispatches to the same Rust Polyline session as CMD. Line/Arc options, analytic mixed segments, preview, Enter/Undo/Redo and invalid/cancel behavior are covered by the [2026-10-25 regression](../../../../docs/validation/2026-10-25-command-polyline-shortcuts/README.md). Helpers tracking, continuously closed PersistentClose preview, unrestricted Center arcs and manual M03-UI acceptance remain open.
