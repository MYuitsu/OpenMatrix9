# OM9-CURVE-040 — Duplicate Edge menu API

- **L3:** `OM9-L3-03-16-039`
- **Menu icon:** `CurveCurveFromObjectsDuplicateEdge`
- **State:** `IMPLEMENTED_AWAITING_MANUAL_UI` (2026-10-26)
- **Owner candidate:** `OM9-L3-02-04-010`
- **Entry points:** Matrix Curve menu and the command line resolve to this one command id.
- **Transaction:** one command session = one Undo entry; disabled/refused input writes nothing.

## Current standalone scope

2026-11-11: the scoped source-based route supports edges selected before or after invoking the command, from one root OCCT B-rep with rigid placement, including tested reopened v4 and imported 3DM fixtures. Multiple selected edges commit in one Undo entry in the tested Box fixture. Source/display association uses bounded adaptive numerical verification and refuses the tested mismatch, ambiguity and exhausted-work cases. Menu icon and CMD were exercised with synthetic egui events. Desktop interaction, native peak and general imported topology are not established. See `docs/validation/2026-11-10-dupedge-source-first-slice.md`. The generic session claims below are historical and do not establish other modes for DupEdge.

The Curve menu row and the typed command resolve to one command id and one Rust
session; the session owns its prompt, selection and commit. The palette icon maps
back to this same command. The current DupEdge slice has no command options or
preview geometry; cancel and the tested invalid inputs write nothing. Manual M03-UI
acceptance remains unsigned.
