# Slab internal publication — 2026-10-29

A hidden controller adapter accepts a captured source stamp and already-resolved world parameters. It resolves a line or connected open polyline, applies the source placement, creates a mitered strip and capped BRep, revalidates the stamp and publishes one insert transaction. Source geometry is retained. Rust owns source/state and budget checks; the existing OCCT extrusion adapter supplies BRep. The actual Slab command is still unavailable and this internal entry point is not a Menu/CMD workflow.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 4 passed. Internal-publication tests measure a translated L-shaped BRep's volume and world bounds, Undo/Redo, stale deleted source, and refusal of a closed curve without commit; the two prior geometry tests also pass.
- `python tools/public_source_audit.py`: 14730 source files, 594 icon bindings, 593 SVGs, no errors.

No UI selection, option handling, input unit conversion, viewport preview, smooth/closed curve support, general plane handling or Matrix-original behavior is claimed. The source conversion accepts only flat connected line polycurves; budget/tolerance and broader transaction races need further validation.
