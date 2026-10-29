# Slab development open NURBS session — 2026-10-29

The hidden Slab session accepts an open, forward, nonperiodic NURBS under affine placement, derives its plane from sampled world geometry and finds the picked offset side using a bounded nearest-point search and tangent. Wire preview samples the analytical offset for responsiveness; commit separately fits the offset, performs the existing sampled tolerance check and extrudes the fitted boundary. Consequently preview and commit are approximations of the same intended offset, not identical curves. The user-facing Slab command remains disabled.

Standalone OCCT, `CARGO_TARGET_DIR=L:\om9-target`, `-j 2`:

- `cargo test -j 2 -p om9-app --test solid solid_slab_`: 15 passed, including an open rational quadratic quarter-circle NURBS, sampled preview, measured BRep volume against the analytic sector and Undo.
- `cargo test -j 2 -p om9-app --test solid_difference_ui viewport_slab_development_`: 9 passed, including simulated post-start selection, side point and height on the NURBS fixture; measured volume checked.
- `python tools/public_source_audit.py`: 14744 source files, 594 icon bindings, 593 SVGs, no errors.

Plane, nearest-point and offset validation are sampled and do not guarantee global accuracy or topology. Only one NURBS fixture is exercised; periodic/reversed/closed NURBS, arbitrary curves, transformed UI, performance under complex scenes, actual Menu/CMD activation, desktop/GPU and original Matrix behavior remain open.
