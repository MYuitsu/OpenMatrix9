# Slab certified periodic NURBS subset — 2026-10-29

Rust admits a full-domain periodic NURBS only when degree-many control points and weights repeat, knot intervals repeat over the period, and seam multiplicity supports positional continuity. The evaluator admits this subset and rejects seam derivative orders beyond certified continuity; other periodic metadata remains unsupported. Scene tessellation uses the certified evaluator. Slab normalizes the certified basis to its equivalent closed nonperiodic representation without modifying the source, then uses the existing sampled fit and BRep publication path.

A degree-two periodic fixture checks matching evaluated points and endpoint tangents, rejection of unsupported seam D2, mismatched repeated poles/knots, measured BRep volume against numerical perimeter, and hidden session preview/commit/Undo in both traversal directions. A simulated viewport fixture selects the periodic source, sets Distance, chooses side and height, checks approximate BRep volume and Undo. This is one repeated-pole representation; arbitrary periodic imports, seam-parameter semantics, global fit/topology and actual desktop remain open. Public Slab stays disabled.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -j 2 -p om9-core --test curve_ops`: 9 passed.
- `cargo test -j 2 -p om9-kernel --features 3dm-brep --test extrusion`: 2 passed, including rejection of an uncertified periodic record. An initial run without `3dm-brep` failed to link the optional 3DM symbols; enabling the required feature passed.
- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 31 passed.
- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_`: 15 passed.
