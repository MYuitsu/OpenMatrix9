# Slab open circular-arc internal publication — 2026-10-29

Safe Rust now constructs a closed exact profile from an open circular arc, its offset arc and the two end connectors, plus a signed extrusion vector and BothSides base shift. The hidden source-checked publication path accepts circularity-preserving placement and calls the existing exact OCCT extrusion adapter. Invalid offset radius is refused before publication. The hidden interactive session does not yet accept partial arcs, and the user-facing Slab command remains disabled.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 12 passed, including a quarter-circle fixture at rotated/translated placement for inward/outward offset, negative height/BothSides, analytic sector volume, valid closed solid and Undo/Redo.
- `cargo test -j 2 -p om9-app --test solid solid_slab_internal_arc_publication_measures_sector_and_undo`: passed independently.
- `cargo check -j 2 -p om9-app --no-default-features`: passed with 13 warnings.
- `python tools/public_source_audit.py`: 14741 source files, 594 icon bindings, 593 SVGs, no errors.

Only one quarter-circle shape and rigid placement were exercised; seam-crossing and reflex arcs, mirrored/nonuniform placement, partial-arc interaction/preview, other curves, desktop and original Matrix behavior remain open. OCCT is required only for exact BRep creation; Rust does not certify native FFI memory safety.
