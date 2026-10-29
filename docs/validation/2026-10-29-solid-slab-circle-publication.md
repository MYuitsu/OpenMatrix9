# Slab circular-source internal publication — 2026-10-29

The safe Rust foundation now derives exact annulus radii and an oriented extrusion frame for a full circle under circularity-preserving placement. The hidden source-checked publication adapter creates outer/inner cylinders and uses the existing OCCT boolean difference; no native handle is retained by Rust. It rejects nonpositive offset radius and nonuniform circle deformation. This is an internal publication path only: the hidden interactive session still admits only open line/polylines and the Slab user command remains disabled.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 10 passed before the final nonuniform rejection assertion was added.
- `cargo test -j 2 -p om9-app --test solid solid_slab_internal_circle_publication_is_exact_and_atomic`: passed after the assertion; covers offset inward/outward, negative height with BothSides, a rotated/translated source, exact analytic volume, closed single solid, invalid radius and Undo/Redo, plus rejection of nonuniform placement.
- `python tools/public_source_audit.py`: 14739 source files, 594 icon bindings, 593 SVGs, no errors.

Other closed curves, partial arcs, circle point/preview interaction, general curve offsets, Matrix-original behavior and desktop remain open. The native exception is for exact BRep construction and boolean; safe Rust does not certify native FFI memory safety.
