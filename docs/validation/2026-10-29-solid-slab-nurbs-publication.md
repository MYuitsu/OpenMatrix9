# Slab open NURBS internal publication — 2026-10-29

The hidden source-checked publication path accepts an open, nonperiodic, forward planar NURBS. Rust fits an offset with 128 stations and compares 33 independently evaluated expected offset points against the fitted curve, rejecting any measured deviation above 0.0001 mm. It closes the profile with two lines and the reversed offset, then the existing OCCT extrusion creates a capped BRep. This check is sampled and one-sided; it does not prove a global deviation bound or absence of all self-intersections. The interactive Slab session and user command still reject NURBS.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 14 passed, including a translated rational quadratic quarter-circle NURBS, BRep volume versus analytic sector within 0.01 mm³, Undo/Redo and incompatible plane refusal without transaction.
- `python tools/public_source_audit.py`: 14743 source files, 594 icon bindings, 593 SVGs, no errors.

One source curve is tested. Closed, periodic or reversed NURBS, arbitrary curvature and topology, global fit error, interaction/preview, desktop and Matrix-original behavior remain open. OCCT supplies exact extrusion of the fitted profile; the offset itself is approximate. Rust's safe interface does not certify native FFI memory safety.
