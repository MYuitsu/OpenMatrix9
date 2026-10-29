# Slab degree-one NURBS simulated viewport — 2026-10-29

The simulated egui viewport starts the hidden development session without preselection, selects a rational degree-one L-shaped NURBS, chooses the side on the second segment and the height in a second view. For both forward and reversed traversal it creates a BRep, measures volume against 54 mm³ with 5 mm³ tolerance for simulated pixel picks, and restores the original document with Undo. This supplements the earlier controller fixture; it is not a real desktop or public Menu/CMD test.

With `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_`: 11 passed.
- `cargo check -j 2 -p om9-app --no-default-features --message-format short`: passed with 15 warnings.

Public Slab remains disabled; other source forms, corner cases and Matrix-original behavior still require validation.
