# Sphere AroundCurve overlapping center hits — 2026-10-29

Once a BRep edge is captured, the viewport filters center-pick hits by the captured source stamp and edge identity before resolving ambiguity. Other objects' overlapping edges no longer block the chosen edge. The initial source-selection stage still rejects ambiguous edges.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_sphere_aroundcurve`: 3 passed. The overlap fixture first captures a box edge through the controller, verifies at least two projected hits from coincident boxes, then uses a simulated viewport click and checks the resulting center. A first attempted one-box fixture produced only one hit and was replaced by the two-box fixture.
- `python tools/public_source_audit.py`: 14723 source files, 594 icon bindings, 593 SVGs, no errors.

This proves the stated simulated center-selection case only. Initial selection of overlapping edges, instances, desktop/GPU and original Matrix behavior remain open.
