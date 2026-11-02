# Slab development source selection — 2026-10-29

The hidden Slab session can begin with one preselected curve or without selection. In the latter case, a simulated viewport pick captures a validated open line/polyline source; overlapping candidates use the existing popup with hit, viewport, scene epoch and command-run revalidation. Undo from the offset stage can clear the captured source. Cancel/restart invalidates a pending popup, and choosing a source does not change the document or create a transaction. The visible Slab command remains disabled.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_`: 5 passed; includes post-start viewport source selection through measured BRep and Undo, plus overlapping curves and stale popup after Cancel/restart. Popup selection is exercised via its API after simulated viewport input, not by clicking a row.
- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 8 passed.
- `cargo check -j 2 -p om9-app --no-default-features`: passed with 12 warnings.
- `python tools/public_source_audit.py`: 14737 source files, 594 icon bindings, 593 SVGs, no errors.

General curves, source plane vs InCPlane, complete options, actual Menu/CMD command activation, popup row clicks, instance paths, desktop/GPU and original Matrix behavior remain open.
