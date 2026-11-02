# Slab development nearest-segment side — 2026-10-29

The hidden Slab session computes signed offset from the closest segment of an open polyline instead of always using its first segment. Both preview and point acceptance call the same safe Rust routine. Equally close segments yielding different offsets are rejected as ambiguous; no source or document mutation occurs on rejection. Ties discovered before a later, closer segment do not prematurely reject it.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 7 passed, including an L-shaped polyline picked near its second segment, a rejected ambiguous corner, preview, measured volume/bounds and Undo.
- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_`: 2 passed, including simulated point input near the second segment and a picked height in Side view.
- `python tools/public_source_audit.py`: 14735 source files, 594 icon bindings, 593 SVGs, no errors.

This remains a hidden development session. General curves, complete Slab options, source selection, Menu/CMD, broader tie/tolerance cases, desktop/GPU and Matrix-original behavior remain open.
