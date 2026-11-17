# OM9-CURVE-054 — Extract Wire Frame menu API

- **L3:** `OM9-L3-03-16-054`
- **Menu icon:** `CurveCurveFromObjectsExtractWireframe`
- **State:** `IMPLEMENTED_AWAITING_MANUAL_UI` (2026-10-26)
- **Owner candidate:** `OM9-L2-03-16`
- **Entry points:** Matrix Curve menu and the command line resolve to this one command id.
- **Transaction:** one command session = one Undo entry; disabled/refused input writes nothing.

## Current standalone scope

2026-11-10: the implemented source-edge slice accepts one preselected, attested OCCT B-rep with rigid placement and creates one curve per topology edge in one transaction. Isocurves and wider inputs remain unsupported; no interactive selection session or preview is claimed. See `docs/validation/2026-11-10-extractwire-source-first-slice.md`. The generic session description below is historical and does not prove those modes for this command.

The Curve menu row and the typed command resolve to one command id and one Rust
session; the session owns its prompts, options, preview and commit. The icon shown in
the palette is the authored MainMenu.ini icon for this row and maps back to this same
command. Parameter options are parsed by the session before the first mutation, and a
cancelled or invalid session writes nothing. Manual M03-UI click/preview acceptance
remains unsigned.
