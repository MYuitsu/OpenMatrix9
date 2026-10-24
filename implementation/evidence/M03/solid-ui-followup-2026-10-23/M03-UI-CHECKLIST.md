# M03-UI — human review worksheet

Date: 2026-10-23. Status: **PENDING HUMAN REVIEW**. Reviewer: __________.
Build: `L:/om9-target/debug/om9.exe`; SHA256
`74e292a854014d0432a6a4a38dddd7e40e032c98e29a129615227c05011d8195`.
Review inputs/expected outputs are in [review-fixtures](review-fixtures/).
Launch `om9.exe --ui <absolute .om9 path>` with OCCT 8.0.1 DLLs on PATH.
Use a fresh input per row; save results under a new name.

Activate a viewport before selection. Click a body or use a right-to-left crossing
band around the required inputs; both Boolean bodies must be selected. Click the
CMD field to enter each prompt value. Use the original Solid menu label below.
For empty-start primitives use a new document; `*-after.om9` is a comparison output.
For Extract/Fillet/Shell create `Box 10 10 10` via CMD and select that box first.
View geometry in all four panes. Check selection, feedback, invalid input/cancel,
one Undo/Redo and Save/reopen for each row. Numeric volumes below are machine-measured
oracles in mm³; the app need not expose a new measurement UI for this review.

Milestone ID is `OM9-L3-03-18-<row>`. Runtime ID column is `OM9-SOLID-<id>`.

| Row | Runtime ID | Menu label / input | Setup or before fixture | Expected result | Human PASS/FAIL |
|---|---|---|---|---|---|
| 001 | 003 | Boolean Union | OM9-SOLID-003-before | One body, volume 1500 | |
| 002 | 001 | Boolean Difference | OM9-SOLID-001-before | One body, volume 500 | |
| 003 | 002 | Boolean Intersection | OM9-SOLID-002-before | One body, volume 500 | |
| 004 | 004 | Boolean Two Objects → U; repeat fresh with D, I | Boolean2-U/D/I-before | U=1500; D=500; I=500; X retains prompt | |
| 005 | 005 | Cap Planar | Cap-before | Closed box, volume 1000 | |
| 006 | 006 | Extract Surface → 0 | Selected 10×10×10 box | Original stays; extracted face area 100 mm² | |
| 007 | 007 | Fillet Edge → 0 → 1 | Selected 10×10×10 box | Rounded edge, volume 997.853981633975 | |
| 008 | 008 | Text → H → 5 | Empty | Solid H; volume 2.912109375; invalid Height retry and Esc | |
| 009 | 009 | Extrude Curve Straight → 5 | ExtrudeCrv-before | Circle retained; exact cylinder volume 45π | |
| 010 | 010 | Solid Extrude All → 5 | ExtrudeAll-before; select both circles | Two curves + two cylinders; total volume 90π; one Undo removes both solids | |
| 011 | 011 | Pipe → 1 | Pipe-before | Straight capped pipe, volume 10π; spine retained | |
| 012 | 012 | Box → 2 → 3 → 4 | Empty | One solid, volume 24 | |
| 013 | 014 | Sphere → 3 | Empty | One solid, volume 36π | |
| 014 | 013 | Ellipsoid → 2 → 3 → 4 | Empty | One solid, volume 32π | |
| 015 | 015 | Torus → 5 → 1 | Empty | One solid, volume 10π² | |
| 016 | 016 | Cylinder → 3 → 5 | Empty | One solid, volume 45π | |
| 017 | 017 | Tube → 3 → 2 → 5 | Empty | Hollow tube, volume 25π | |
| 018 | 018 | Pyramid → 4 → 3 → 5 | Empty | Four-sided pyramid, volume 30 | |
| 019 | 019 | Cone → 3 → 5 | Empty | One solid, volume 15π | |
| 020 | 020 | Truncated Cone → 3 → 1 → 5 | Empty | One solid, volume 65π/3 | |
| 021 | 029 | Shell → 0 → 1 | Selected 10×10×10 box | Open-top wall solid, volume 424 | |

Agent observations: Text, Boolean Two Objects/U, ExtrudeAll including Undo/Redo
have current-build native screenshots. Ellipsoid has a separate prior-build
[native receipt](../ellipsoid-exact-2026-10-23/native-runtime.json).
All 21 bindings have current measured Controller tests. Neither substitutes for
the human PASS/FAIL column.

References: [report](../../../../docs/validation/2026-10-23-solid-ui-followup.md),
[receipt](receipt.json), [Text screenshot](ui-text-saved.png),
[Boolean screenshot](ui-boolean2-saved.png),
[ExtrudeAll screenshot](ui-extrude-all-saved.png),
[single Undo](ui-extrude-all-one-undo.png), [Redo](ui-extrude-all-redo.png).

Human decision: __________. Date/time: __________. Reviewer identity: __________.
Issues / exact rows needing repair: __________.
This worksheet is unsigned; no automated process may fill the human decision.
