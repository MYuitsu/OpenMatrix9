# OM9-CURVE-027 — Extend Curve menu API

- **L3:** `OM9-L3-03-16-027`
- **Menu icon:** `CurveExtendCurveExtendCurve`
- **State:** `IMPLEMENTED_AWAITING_MANUAL_UI` (2026-10-26)
- **Owner candidate:** `OM9-L2-03-16`
- **Entry points:** Matrix Curve menu and the command line resolve to this one command id.
- **Transaction:** one command session = one Undo entry; disabled/refused input writes nothing.

## Current standalone scope

The Curve menu row and the typed command resolve to one command id and one Rust
session; the session owns its prompts, options, preview and commit. The icon shown in
the palette is the authored MainMenu.ini icon for this row and maps back to this same
command. Parameter options are parsed by the session before the first mutation, and a
cancelled or invalid session writes nothing. Manual M03-UI click/preview acceptance
remains unsigned.
