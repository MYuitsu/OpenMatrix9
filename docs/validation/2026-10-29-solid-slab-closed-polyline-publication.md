# Slab closed-polyline internal publication — 2026-10-29

The hidden publication adapter resolves a placed closed line polycurve or continuous nonperiodic degree-one NURBS, obtains nested mitered rings, extrudes both and subtracts the inner BRep. Input and intermediate geometry are budget-accounted; source stamps are checked before publication. The source is retained and the result is inserted as one Undo transaction.

A translated rectangle fixture exercises polycurve and rational degree-one NURBS sources in both directions: the resulting closed single BRep has volume 36 mm³ and expected placed bounds; Undo/Redo works and an invalid collapsing offset does not mutate the document or Undo history. The earlier core fixture separately checks inward, BothSides, concave and vertical-plane rings. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 23 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 11 tests.

The hidden interactive session still rejects closed polygon sources; no closed-polygon viewport or real desktop test has been performed. Arbitrary topology, multiple loops and Matrix-original behavior remain unverified; public Slab remains disabled.
