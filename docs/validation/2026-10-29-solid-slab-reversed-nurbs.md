# Slab reversed NURBS development validation — 2026-10-29

The internal Slab development session previously rejected reversed NURBS. It now accepts a nonperiodic open planar NURBS in either traversal direction. For a reversed source, the offset is fitted with the corresponding opposite sign on the forward curve and then reversed, preserving the side chosen relative to the original curve direction. The existing 33-point tolerance check compares against the directed source.

A reversed rational quarter-circle of radius 5, with an interior radius 3 pick and height 3, produces volume 12π within 0.01; the controller test also checks Undo. A simulated viewport test selects the curve, picks the side and height, and checks the same volume.

Validation with `CARGO_TARGET_DIR=L:\om9-target` and `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 16 passed.
- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_`: 10 passed.

The offset check samples 33 points and is not a global accuracy or topology proof. The preview is sampled; periodic and closed NURBS, other curve forms, and desktop behavior remain unverified. Slab remains disabled in public menu and command invocation; this does not establish complete Matrix behavior.
