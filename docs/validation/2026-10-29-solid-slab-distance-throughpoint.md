# Slab development Distance and ThroughPoint — 2026-10-29

The hidden Slab session now separates a positive numeric Distance from the following side pick. The side pick determines sign while preserving the entered magnitude; ThroughPoint clears the stored magnitude and uses the picked point's signed distance. Undo after side selection returns to the side step, retaining Distance. Preview and commit use the same distance and side. These are partial OM9 session behaviors based on the Slab references; the user-facing command remains disabled.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 8 passed, including three numeric-distance/side and ThroughPoint fixtures, actual BRep volume/bounds, unit conversion and Undo.
- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_`: 3 passed, including numeric Distance followed by simulated side and height picks.
- `python tools/public_source_audit.py`: 14736 source files, 594 icon bindings, 593 SVGs, no errors.

General curves, source selection, InCPlane, complete option interface, default/persistence, Menu/CMD, desktop/GPU and Matrix-original behavior remain open. This supersedes the earlier hidden-session convention of accepting a signed Distance as the side itself.
