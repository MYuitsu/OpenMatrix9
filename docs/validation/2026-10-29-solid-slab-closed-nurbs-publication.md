# Slab closed NURBS internal publication — 2026-10-29

The hidden source-stamped publication adapter detects a closed degree-two-or-higher NURBS, obtains sample-checked outer/inner curves, extrudes both through OCCT and subtracts them. Input and BRep allocations are budget-accounted; the source remains and one result is inserted in one Undo transaction. BothSides base shift is represented by result placement.

A translated rational ellipse fixture checks outward and inward offsets in both traversal directions, negative height, BothSides placement, closed single BRep volume against an independently integrated ellipse perimeter, Undo/Redo, and rejection of an excessive inward offset without mutation. Volume tolerance is 0.02 mm³. With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`, `cargo test -j 2 -p om9-app --test solid solid_slab_` passed 27 tests and `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_` passed 12 tests.

Offset fit and topology checks remain sampled and not globally certified. This fixture does not exercise closed NURBS selection, preview or desktop, and periodic curves and public Slab remain unsupported.
