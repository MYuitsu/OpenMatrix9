# Slab concave closed-polygon options — 2026-10-29

A controller fixture exercises the hidden session with a concave L-shaped closed polycurve. Outward Distance=0.5 and inward ThroughPoint=0.25 produce measured BRep volumes of 18 and 7.5 mm³ at height 2 mm. For both, preview bounds match final BRep bounds; Undo during selection, Undo after commit and Cancel leave the expected document and history.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 25 tests. This fixture covers one concave shape in the controller, not arbitrary polygon topology or actual desktop interactions. Public Slab, periodic/higher-degree closed curves and original Matrix behavior remain open.
